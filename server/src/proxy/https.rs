use super::vhost::{build_domains, normalize_host};
use crate::access::{prepare_visitor, AccessPolicy};
use crate::control::Control;
use crate::metrics;
use anyhow::{anyhow, Result};
use orbien_core::limit::{maybe_limit, BandwidthLimiter};
use orbien_core::msg::NewProxy;
use orbien_core::tls::{peek_client_hello_sni, PrefixedStream};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Notify};

#[derive(Clone)]
pub struct HttpsRoute {
    pub proxy_name: String,
    pub run_id: String,
    pub control: Weak<Control>,
    pub limiter: Option<Arc<BandwidthLimiter>>,
}

pub struct HttpsVhost {
    routes: Mutex<HashMap<String, HttpsRoute>>,
    pub listen_port: u16,
}

impl HttpsVhost {
    pub fn new(listen_port: u16) -> Self {
        Self {
            routes: Mutex::new(HashMap::new()),
            listen_port,
        }
    }

    pub async fn register(&self, domain: &str, route: HttpsRoute) -> Result<()> {
        let key = normalize_host(domain);
        if key.is_empty() {
            return Err(anyhow!("empty https domain"));
        }
        let mut map = self.routes.lock().await;
        if let Some(existing) = map.get(&key) {
            let live = existing.control.upgrade().is_some();
            if live && (existing.run_id != route.run_id || existing.proxy_name != route.proxy_name)
            {
                return Err(anyhow!("https domain already in use: {key}"));
            }
        }
        map.insert(key.clone(), route);
        tracing::info!(domain = %key, "https route registered");
        Ok(())
    }

    pub async fn unregister_proxy(&self, proxy_name: &str, run_id: &str) {
        let mut map = self.routes.lock().await;
        map.retain(|_, r| !(r.proxy_name == proxy_name && r.run_id == run_id));
    }

    pub async fn lookup(&self, sni: &str) -> Option<HttpsRoute> {
        let key = normalize_host(sni);
        let map = self.routes.lock().await;
        map.get(&key).cloned()
    }
}

pub struct HttpsProxy {
    pub name: String,
    pub domains: Vec<String>,
    run_id: String,
    vhost: Arc<HttpsVhost>,
    closed: AtomicBool,
}

impl HttpsProxy {
    pub async fn register(
        np: &NewProxy,
        control: Arc<Control>,
        vhost: Arc<HttpsVhost>,
        sub_domain_host: &str,
        limiter: Option<Arc<BandwidthLimiter>>,
    ) -> Result<Self> {
        let domains = build_domains(&np.custom_domains, &np.subdomain, sub_domain_host)?;
        let name = np.proxy_name.clone();

        for domain in &domains {
            if let Err(e) = vhost
                .register(
                    domain,
                    HttpsRoute {
                        proxy_name: name.clone(),
                        run_id: control.run_id.clone(),
                        control: Arc::downgrade(&control),
                        limiter: limiter.clone(),
                    },
                )
                .await
            {
                vhost.unregister_proxy(&name, &control.run_id).await;
                return Err(e);
            }
        }

        tracing::info!(
            proxy = %name,
            domains = ?domains,
            "https proxy registered (SNI passthrough)"
        );

        Ok(Self {
            name,
            domains,
            run_id: control.run_id.clone(),
            vhost,
            closed: AtomicBool::new(false),
        })
    }

    pub async fn close(&self) {
        if self
            .closed
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.vhost.unregister_proxy(&self.name, &self.run_id).await;
        }
    }
}

pub async fn run_vhost_https_listener(
    bind_addr: String,
    port: u16,
    vhost: Arc<HttpsVhost>,
    access: Arc<AccessPolicy>,
    shutdown: Arc<Notify>,
) -> Result<()> {
    let addr = format!("{bind_addr}:{port}");
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "https vhost listener ready (SNI mux, no TLS terminate)");

    loop {
        tokio::select! {
            _ = shutdown.notified() => break,
            accepted = listener.accept() => {
                match accepted {
                    Ok((stream, peer)) => {
                        let vhost = Arc::clone(&vhost);
                        let access = Arc::clone(&access);
                        tokio::spawn(async move {
                            if let Err(e) = handle_https_visitor(vhost, stream, peer, access).await {
                                tracing::debug!(%peer, error = %e, "https visitor ended");
                            }
                        });
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "https vhost accept failed");
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_https_visitor(
    vhost: Arc<HttpsVhost>,
    stream: TcpStream,
    peer: std::net::SocketAddr,
    access: Arc<AccessPolicy>,
) -> Result<()> {
    let mut visitor = prepare_visitor(stream, peer, &access).await?;
    let (sni, prefix) = peek_client_hello_sni(&mut visitor.stream).await?;
    let Some(route) = vhost.lookup(&sni).await else {
        tracing::debug!(
            peer = %visitor.peer,
            visitor = %visitor.visitor,
            %sni,
            "https no route for SNI"
        );

        return Ok(());
    };

    let Some(control) = route.control.upgrade() else {
        return Err(anyhow!("https proxy client gone: {}", route.proxy_name));
    };

    let work = control.get_work_conn().await?;
    let work = control
        .start_work_conn(
            work,
            &route.proxy_name,
            visitor.visitor.ip().to_string(),
            visitor.visitor.port(),
            visitor
                .local
                .map(|a| a.ip().to_string())
                .unwrap_or_default(),
            visitor.local.map(|a| a.port()).unwrap_or(0),
        )
        .await?;

    let work = maybe_limit(work, route.limiter.clone());
    let user = PrefixedStream::new(prefix, visitor.stream);

    tracing::debug!(
        proxy = %route.proxy_name,
        %sni,
        peer = %visitor.peer,
        visitor = %visitor.visitor,
        "https joining visitor <-> work (passthrough)"
    );
    let _ =
        metrics::join_and_record(&control.metrics, &route.proxy_name, "https", user, work).await;
    Ok(())
}
