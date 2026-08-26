use anyhow::Result;
use clap::Parser;
use orbien_client::ClientHandle;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "orbien",
    about = "orbien client",
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
    let config_path = orbien_client::resolve_client_config_path(args.config.as_deref())?;
    tracing::info!(config = %config_path.display(), "loading config");

    let cfg = orbien_client::ClientConfig::load(&config_path)?;
    tracing::info!(
        server = %cfg.server_endpoint(),
        protocol = %cfg.transport.protocol,
        tunnels = cfg.tunnels.len(),
        "starting orbien"
    );

    ClientHandle::new().run_foreground(cfg, config_path).await
}
