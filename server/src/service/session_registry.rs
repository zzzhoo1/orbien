use super::{OfflineClientRecord, Service};
use crate::control::Control;
use crate::metrics::ServerMetrics;
use anyhow::{anyhow, Result};
use orbien_core::auth;
use orbien_core::msg::{self, Login, LoginResp, Message, NewDataConn};
use orbien_core::transport::DynStream;
use orbien_core::VERSION;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

impl Service {
    pub(crate) async fn register_control(
        self: Arc<Self>,
        stream: DynStream,
        login: Login,
        peer: SocketAddr,
    ) -> Result<()> {
        if !auth::verify_login(&self.cfg.auth.token, &login.auth_digest, login.timestamp) {
            let mut stream = stream;
            let _ = msg::write_msg(
                &mut stream,
                &Message::LoginResp(LoginResp {
                    version: VERSION.into(),
                    session_id: String::new(),
                    error: "authorization failed".into(),
                }),
            )
            .await;
            return Err(anyhow!("authorization failed"));
        }

        let session_id = if login.session_id.is_empty() {
            short_session_id()
        } else {
            login.session_id.clone()
        };

        let mut stream = stream;
        msg::write_msg(
            &mut stream,
            &Message::LoginResp(LoginResp {
                version: VERSION.into(),
                session_id: session_id.clone(),
                error: String::new(),
            }),
        )
        .await?;

        tracing::info!(%session_id, %peer, pool = login.pool_count, "client logged in");

        let max_pool = self.cfg.transport.max_conn_pool.max(0) as usize;
        let pool_count = (login.pool_count.max(0) as usize).min(max_pool);

        let client_ip = peer.ip().to_string();

        let control = Control::new(
            session_id.clone(),
            stream,
            self.cfg.clone(),
            pool_count,
            self.http_gw.clone(),
            self.https_gw.clone(),
            Arc::clone(&self.access),
            login.user.clone(),
            login.hostname.clone(),
            login.os.clone(),
            login.arch.clone(),
            login.version.clone(),
            client_ip,
            Arc::clone(&self.metrics),
        );
        let control = Arc::new(control);

        {
            let mut offline = self.offline_clients.lock().await;
            offline.remove(&session_id);
        }

        let old = {
            let mut map = self.controls.lock().await;
            map.insert(session_id.clone(), Arc::clone(&control))
        };

        if let Some(old) = old {
            old.shutdown().await;
        }

        self.metrics.new_client(&session_id);

        let controls = Arc::clone(&self.controls);
        let offline_clients = Arc::clone(&self.offline_clients);
        let metrics = Arc::clone(&self.metrics);
        let rid = session_id.clone();
        let result = Arc::clone(&control).run().await;
        control.shutdown().await;
        metrics.close_client();

        let tunnel_count = control.tunnel_count().await;
        let mut map = controls.lock().await;
        if map
            .get(&rid)
            .map(|c| Arc::ptr_eq(c, &control))
            .unwrap_or(false)
        {
            map.remove(&rid);
        }
        if !map.contains_key(&rid) {
            drop(map);
            let mut offline = offline_clients.lock().await;
            offline.insert(
                rid.clone(),
                OfflineClientRecord {
                    session_id: rid,
                    user: control.user.clone(),
                    hostname: control.hostname.clone(),
                    os: control.os.clone(),
                    arch: control.arch.clone(),
                    client_ip: control.client_ip.clone(),
                    version: control.version.clone(),
                    tunnel_count,
                    disconnected_at: Instant::now(),
                },
            );
        }

        result
    }

    pub(crate) async fn register_data_conn(
        self: Arc<Self>,
        stream: DynStream,
        nw: NewDataConn,
    ) -> Result<()> {
        if nw.session_id.trim().is_empty() {
            return Err(anyhow!("empty session_id for data conn"));
        }
        if !auth::verify_auth_digest(&self.cfg.auth.token, &nw.auth_digest, nw.timestamp) {
            return Err(anyhow!(
                "data conn auth failed for session_id={}",
                nw.session_id
            ));
        }
        let control = {
            let map = self.controls.lock().await;
            map.get(&nw.session_id).cloned()
        };
        match control {
            Some(c) => {
                c.push_data_conn(stream).await;
                Ok(())
            }
            None => Err(anyhow!(
                "unknown session_id for data conn: {}",
                nw.session_id
            )),
        }
    }
}

fn short_session_id() -> String {
    let hex = Uuid::new_v4().simple().to_string();
    hex[..16].to_owned()
}
