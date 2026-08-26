use crate::control::{Control, SessionEnd};
use crate::handle::ClientStatus;
use crate::session_id;
use anyhow::Result;
use orbien_core::config::ClientConfig;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

const RECONNECT_BASE_SECS: u64 = 1;
const RECONNECT_MAX_SECS: u64 = 60;

pub struct Service {
    cfg: ClientConfig,
    config_path: PathBuf,
}

impl Service {
    pub fn new(cfg: ClientConfig, config_path: impl Into<PathBuf>) -> Self {
        Self {
            cfg,
            config_path: config_path.into(),
        }
    }

    pub async fn run(
        self,
        cancel: CancellationToken,
        mut on_status: impl FnMut(ClientStatus),
        mut on_log: impl FnMut(String),
        on_tunnel_remote: Arc<dyn Fn(String, String) + Send + Sync>,
        on_remotes_clear: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<()> {
        let mut session_id = session_id::load(&self.config_path);
        if !session_id.is_empty() {
            tracing::info!(%session_id, "restored persisted session_id");
        }

        let mut first_attempt = true;
        let mut backoff_secs = RECONNECT_BASE_SECS;
        loop {
            if cancel.is_cancelled() {
                tracing::info!("client service cancelled");
                return Ok(());
            }

            on_remotes_clear();

            if first_attempt {
                on_status(ClientStatus::Starting);
                on_log("INFO  connecting to server".into());
            } else {
                on_status(ClientStatus::Reconnecting);
            }

            let end = Control::start(
                &self.cfg,
                session_id.clone(),
                &self.config_path,
                cancel.clone(),
                || {
                    on_status(ClientStatus::Running);
                    on_log("INFO  connected to server".into());
                },
                Arc::clone(&on_tunnel_remote),
            )
            .await;

            on_remotes_clear();

            match end {
                Ok(SessionEnd::Kicked {
                    session_id: rid,
                    reason,
                }) => {
                    tracing::warn!(
                        session_id = %rid,
                        %reason,
                        "kicked by server — stopping (no reconnect)"
                    );
                    on_log(format!("WARN  kicked by server: {reason}"));
                    return Ok(());
                }
                Ok(SessionEnd::Disconnected { session_id: rid }) => {
                    if cancel.is_cancelled() {
                        tracing::info!(session_id = %rid, "session ended after cancel");
                        return Ok(());
                    }
                    session_id = rid;
                    on_log("WARN  disconnected from server".into());
                    on_status(ClientStatus::Reconnecting);
                    backoff_secs = RECONNECT_BASE_SECS;
                }
                Err(e) => {
                    if cancel.is_cancelled() {
                        tracing::info!("session error after cancel: {e}");
                        return Ok(());
                    }
                    on_log(format!("ERROR failed to connect: {e}"));
                    on_status(ClientStatus::Reconnecting);
                }
            }

            first_attempt = false;
            let delay = backoff_secs;
            on_log(format!("INFO  retrying in {delay}s"));
            tracing::info!(delay_secs = delay, "reconnect backoff");

            tokio::select! {
                _ = cancel.cancelled() => {
                    tracing::info!("client service cancelled during backoff");
                    return Ok(());
                }
                _ = sleep(Duration::from_secs(delay)) => {}
            }

            backoff_secs = backoff_secs
                .saturating_mul(2)
                .clamp(RECONNECT_BASE_SECS, RECONNECT_MAX_SECS);
        }
    }
}
