use crate::control::Control;
use std::collections::HashMap;
use std::sync::Weak;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct HttpRoute {
    pub proxy_name: String,
    pub run_id: String,
    pub control: Weak<Control>,

    pub locations: Vec<String>,
    pub host_header_rewrite: String,

    pub limiter: Option<std::sync::Arc<orbien_core::limit::BandwidthLimiter>>,
}

pub struct HttpVhost {
    routes: Mutex<HashMap<String, Vec<HttpRoute>>>,
    pub listen_port: u16,
}

impl HttpVhost {
    pub fn new(listen_port: u16) -> Self {
        Self {
            routes: Mutex::new(HashMap::new()),
            listen_port,
        }
    }

    pub async fn register(&self, domain: &str, route: HttpRoute) -> anyhow::Result<()> {
        let key = normalize_host(domain);
        if key.is_empty() {
            return Err(anyhow::anyhow!("empty http domain"));
        }
        let mut map = self.routes.lock().await;
        let list = map.entry(key.clone()).or_default();

        list.retain(|r| !(r.proxy_name == route.proxy_name && r.run_id == route.run_id));
        list.push(route);
        tracing::info!(domain = %key, "http route registered");
        Ok(())
    }

    pub async fn unregister_proxy(&self, proxy_name: &str, run_id: &str) {
        let mut map = self.routes.lock().await;
        map.retain(|_, list| {
            list.retain(|r| !(r.proxy_name == proxy_name && r.run_id == run_id));
            !list.is_empty()
        });
    }

    pub async fn lookup(&self, host: &str, path: &str) -> Option<HttpRoute> {
        let key = normalize_host(host);
        let map = self.routes.lock().await;
        let list = map.get(&key)?;
        pick_by_location(list, path).cloned()
    }
}

fn pick_by_location<'a>(list: &'a [HttpRoute], path: &str) -> Option<&'a HttpRoute> {
    let mut best: Option<(&HttpRoute, usize)> = None;
    for r in list {
        let locs = if r.locations.is_empty() {
            vec![String::new()]
        } else {
            r.locations.clone()
        };
        for loc in locs {
            if loc.is_empty() || path.starts_with(&loc) {
                let score = loc.len();
                if best.map(|(_, s)| score >= s).unwrap_or(true) {
                    best = Some((r, score));
                }
            }
        }
    }
    best.map(|(r, _)| r)
}

pub fn normalize_host(host: &str) -> String {
    let host = host.trim();
    let without_port = if let Some(h) = host.strip_prefix('[') {
        if let Some(end) = h.find(']') {
            &h[..end]
        } else {
            host
        }
    } else {
        host.split(':').next().unwrap_or(host)
    };
    without_port.trim().to_ascii_lowercase()
}

pub fn build_domains(
    custom_domains: &[String],
    subdomain: &str,
    sub_domain_host: &str,
) -> anyhow::Result<Vec<String>> {
    let mut out = Vec::new();
    for d in custom_domains {
        let d = d.trim();
        if !d.is_empty() {
            out.push(normalize_host(d));
        }
    }
    let sub = subdomain.trim();
    if !sub.is_empty() {
        let base = sub_domain_host.trim();
        if base.is_empty() {
            if out.is_empty() {
                return Err(anyhow::anyhow!(
                    "subdomain set but server subDomainHost is empty"
                ));
            }
            tracing::warn!(
                subdomain = %sub,
                "subdomain ignored: server subDomainHost is empty; using customDomains only"
            );
        } else if sub.contains('.') || sub.contains('*') {
            return Err(anyhow::anyhow!("subdomain must not contain '.' or '*'"));
        } else {
            out.push(normalize_host(&format!("{sub}.{base}")));
        }
    }
    if out.is_empty() {
        return Err(anyhow::anyhow!(
            "http/https proxy requires customDomains and/or subdomain"
        ));
    }
    Ok(out)
}
