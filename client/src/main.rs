mod connector;
mod control;
mod plugin;
mod proxy;
mod run_id;
mod sanitize;
mod service;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "orbien",
    about = "orbien client — TCP tunnel over TCP/QUIC",
    after_help = "Config:\n  \
        orbien                         # try ./orbien.toml, then ./conf/orbien.toml\n  \
        orbien -c conf/orbien.toml     # explicit path"
)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let args = Args::parse();
    let config_path = orbien_core::config::resolve_client_config_path(args.config.as_deref())?;
    tracing::info!(config = %config_path.display(), "loading config");

    let cfg = orbien_core::config::ClientConfig::load(&config_path)?;
    tracing::info!(
        server = %cfg.server_endpoint(),
        protocol = %cfg.transport.protocol,
        proxies = cfg.proxies.len(),
        "starting orbien"
    );

    service::Service::new(cfg, &config_path).run().await
}
