use crate::control::{Control, SessionEnd};
use crate::run_id;
use crate::sanitize::sanitize_for_logging;
use anyhow::Result;
use orbien_core::config::ClientConfig;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

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

    pub async fn run(self) -> Result<()> {
        let mut run_id = run_id::load(&self.config_path);
        if !run_id.is_empty() {
            tracing::info!(%run_id, "restored persisted run_id");
        }

        loop {
            match Control::start(&self.cfg, run_id.clone(), &self.config_path).await {
                Ok(SessionEnd::Kicked {
                    run_id: rid,
                    reason,
                }) => {
                    let safe_reason = sanitize_for_logging(&reason);
                    tracing::error!(
                        run_id = %rid,
                        reason = %safe_reason,
                        "kicked by server — process will exit (no reconnect)"
                    );
                    return Ok(());
                }
                Ok(SessionEnd::Disconnected { run_id: rid }) => {
                    run_id = rid;
                    tracing::warn!("control session ended, reconnecting in 3s...");
                }
                Err(e) => {
                    tracing::error!(error = %e, "failed to establish control, retry in 3s");
                }
            }
            sleep(Duration::from_secs(3)).await;
        }
    }
}
