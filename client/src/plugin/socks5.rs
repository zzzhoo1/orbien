use super::{ConnectionInfo, Plugin, PluginContext};
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use fast_socks5::{
    server::{run_tcp_proxy, DnsResolveHelper, Socks5ServerProtocol, SocksServerError},
    ReplyError, Socks5Command,
};
use orbien_core::config::PluginConfig;
use orbien_core::transport::DynStream;
use std::time::Duration;

const RELAY_TIMEOUT_SECS: u64 = 30;

pub struct Socks5Plugin {
    tunnel_name: String,
    username: String,
    password: String,
}

impl Socks5Plugin {
    pub fn new(ctx: PluginContext, cfg: &PluginConfig) -> Result<Self> {
        let (username, password) = required_credentials(cfg)?;
        tracing::info!(
            tunnel = %ctx.name,
            username = %username,
            "plugin socks5 ready"
        );
        Ok(Self {
            tunnel_name: ctx.name,
            username,
            password,
        })
    }

    async fn serve(&self, stream: DynStream) -> Result<()> {
        let timeout = Duration::from_secs(RELAY_TIMEOUT_SECS);
        let expected_user = self.username.clone();
        let expected_pass = self.password.clone();
        let (proto, cmd, target_addr) =
            Socks5ServerProtocol::accept_password_auth(stream, move |user, pass| {
                user == expected_user && pass == expected_pass
            })
            .await
            .map_err(map_server_err)?
            .0
            .read_command()
            .await
            .map_err(map_server_err)?
            .resolve_dns()
            .await
            .map_err(map_server_err)?;

        match cmd {
            Socks5Command::TCPConnect => {
                tracing::debug!(
                    tunnel = %self.tunnel_name,
                    target = %target_addr,
                    "socks5 relay starting"
                );
                if let Err(e) = run_tcp_proxy(proto, &target_addr, timeout, false).await {
                    tracing::debug!(
                        tunnel = %self.tunnel_name,
                        error = %e,
                        "socks5 relay ended"
                    );
                }
            }
            _ => {
                let _ = proto.reply_error(&ReplyError::CommandNotSupported).await;
                return Err(anyhow!("socks5 unsupported command {:?}", cmd));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Plugin for Socks5Plugin {
    fn name(&self) -> &str {
        "socks5"
    }

    async fn handle(&self, conn: ConnectionInfo) -> Result<()> {
        self.serve(conn.stream).await
    }
}

fn required_credentials(cfg: &PluginConfig) -> Result<(String, String)> {
    let user = cfg.username.trim();
    let pass = cfg.password.trim();
    if user.is_empty() || pass.is_empty() {
        bail!("socks5 plugin requires username and password");
    }
    Ok((user.to_string(), pass.to_string()))
}

fn map_server_err(err: SocksServerError) -> anyhow::Error {
    anyhow::Error::new(err)
}
