use super::gw::{build_domains, normalize_host};
use crate::access::{prepare_ingress, AccessPolicy};
use crate::control::Control;
use crate::metrics;
use anyhow::{anyhow, Result};
use orbien_core::limit::{maybe_limit, BandwidthLimiter};
use orbien_core::msg::NewTunnel;
use orbien_core::tls::{peek_client_hello_sni, PrefixedStream};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Notify};

#[derive(Clone)]
pub struct HttpsRoute {
    pub tunnel_name: String,
    pub control: Weak<Control>,
    pub limiter: Option<Arc<BandwidthLimiter>>,
}

pub struct HttpsGw {
    routes: Mutex<HashMap<String, HttpsRoute>>,
    pub listen_port: u16,
}

impl HttpsGw {
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
            if existing.tunnel_name != route.tunnel_name {
                return Err(anyhow!("router config conflict: domain={key} (https)"));
            }
        }
        map.insert(key.clone(), route);
        tracing::info!(domain = %key, "https route registered");
        Ok(())
    }

    pub async fn unregister_tunnel(&self, tunnel_name: &str) {
        let mut map = self.routes.lock().await;
        map.retain(|_, r| r.tunnel_name != tunnel_name);
    }

    pub async fn lookup(&self, sni: &str) -> Option<HttpsRoute> {
        let key = normalize_host(sni);
        let map = self.routes.lock().await;
        map.get(&key).cloned()
    }
}

pub struct HttpsTunnel {
    pub name: String,
    pub domains: Vec<String>,
    gw: Arc<HttpsGw>,
    closed: AtomicBool,
}

impl HttpsTunnel {
    pub async fn register(
        np: &NewTunnel,
        control: Arc<Control>,
        gw: Arc<HttpsGw>,
        sub_domain_host: &str,
        limiter: Option<Arc<BandwidthLimiter>>,
    ) -> Result<Self> {
        let domains = build_domains(&np.domains, sub_domain_host)?;
        let name = np.tunnel_name.clone();

        gw.unregister_tunnel(&name).await;

        for domain in &domains {
            gw.register(
                domain,
                HttpsRoute {
                    tunnel_name: name.clone(),
                    control: Arc::downgrade(&control),
                    limiter: limiter.clone(),
                },
            )
            .await?;
        }

        tracing::info!(
            tunnel = %name,
            domains = ?domains,
            "https tunnel registered (SNI passthrough)"
        );

        Ok(Self {
            name,
            domains,
            gw,
            closed: AtomicBool::new(false),
        })
    }

    pub async fn close(&self) {
        if self
            .closed
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.gw.unregister_tunnel(&self.name).await;
        }
    }
}

pub async fn run_https_gw_listener(
    bind_addr: String,
    port: u16,
    gw: Arc<HttpsGw>,
    access: Arc<AccessPolicy>,
    shutdown: Arc<Notify>,
) -> Result<()> {
    let addr = format!("{bind_addr}:{port}");
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "https gateway listener ready (SNI mux, no TLS terminate)");

    loop {
        tokio::select! {
            _ = shutdown.notified() => break,
            accepted = listener.accept() => {
                match accepted {
                    Ok((stream, peer)) => {
                        orbien_core::net::enable_nodelay(&stream);
                        let gw = Arc::clone(&gw);
                        let access = Arc::clone(&access);
                        tokio::spawn(async move {
                            if let Err(e) = handle_https_ingress(gw, stream, peer, access).await {
                                tracing::debug!(%peer, error = %e, "https ingress ended");
                            }
                        });
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "https gateway accept failed");
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_https_ingress(
    gw: Arc<HttpsGw>,
    stream: TcpStream,
    peer: std::net::SocketAddr,
    access: Arc<AccessPolicy>,
) -> Result<()> {
    let mut ingress = prepare_ingress(stream, peer, &access).await?;
    let (sni, prefix) = peek_client_hello_sni(&mut ingress.stream).await?;
    let Some(route) = gw.lookup(&sni).await else {
        tracing::debug!(
            peer = %ingress.peer,
            source = %ingress.source,
            %sni,
            "https no route for SNI"
        );

        return Ok(());
    };

    let Some(control) = route.control.upgrade() else {
        return Err(anyhow!("https tunnel client gone: {}", route.tunnel_name));
    };

    let data = control.get_data_conn().await?;
    let data = control
        .start_data_conn(
            data,
            &route.tunnel_name,
            ingress.source.ip().to_string(),
            ingress.source.port(),
            ingress
                .local
                .map(|a| a.ip().to_string())
                .unwrap_or_default(),
            ingress.local.map(|a| a.port()).unwrap_or(0),
        )
        .await?;

    let data = maybe_limit(data, route.limiter.clone());
    let user = PrefixedStream::new(prefix, ingress.stream);

    tracing::debug!(
        tunnel = %route.tunnel_name,
        %sni,
        peer = %ingress.peer,
        source = %ingress.source,
        "https joining ingress <-> data (passthrough)"
    );
    let _ =
        metrics::join_and_record(&control.metrics, &route.tunnel_name, "https", user, data).await;
    Ok(())
}
