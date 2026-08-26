use orbien_client::{ClientHandle, ClientStatus};
use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("orbien-client")
            .build()
            .expect("failed to create orbien client runtime")
    })
}

pub fn handle() -> ClientHandle {
    static HANDLE: OnceLock<ClientHandle> = OnceLock::new();
    HANDLE.get_or_init(ClientHandle::new).clone()
}

pub fn status() -> ClientStatus {
    handle().status()
}

pub fn drain_logs() -> Vec<String> {
    handle().drain_logs()
}

pub fn tunnel_remotes_if_changed(
    since_gen: u64,
) -> Option<(u64, std::collections::HashMap<String, String>)> {
    handle().tunnel_remotes_if_changed(since_gen)
}

pub fn start(cfg: orbien_client::ClientConfig, path: std::path::PathBuf) -> anyhow::Result<()> {
    let h = handle();
    if h.status().is_active() {
        anyhow::bail!("client already running");
    }
    let _guard = runtime().enter();
    h.start(cfg, path)
}

pub fn restart(cfg: orbien_client::ClientConfig, path: std::path::PathBuf) -> anyhow::Result<()> {
    let h = handle();
    let rt = runtime();
    rt.block_on(async {
        if h.status().is_active() {
            h.stop().await;
        }
    });
    let _guard = rt.enter();
    h.start(cfg, path)
}

pub fn stop_async(on_done: impl FnOnce() + Send + 'static) {
    let h = handle();
    runtime().spawn(async move {
        h.stop().await;
        let _ = slint::invoke_from_event_loop(move || on_done());
    });
}
