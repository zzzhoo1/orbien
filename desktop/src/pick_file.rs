use slint::{ComponentHandle, Weak};
use std::path::{Path, PathBuf};
use std::thread;

use crate::config_bridge;
use crate::{apply_config_to_ui, toast_err, AppWindow};

#[derive(Clone, Copy)]
enum FilterKind {
    Config,
    CertKey,
}

pub fn pick_into(ui: &AppWindow, target: &str) {
    let (kind, title, current) = match target {
        "config-file" => (
            FilterKind::Config,
            ui.global::<crate::Tr>().get_pick_config_title().to_string(),
            ui.get_config_file_path().to_string(),
        ),
        "tls-ca" => (
            FilterKind::CertKey,
            ui.global::<crate::Tr>().get_pick_cert_title().to_string(),
            ui.get_config_tls_ca().to_string(),
        ),
        "tls-cert" => (
            FilterKind::CertKey,
            ui.global::<crate::Tr>().get_pick_cert_title().to_string(),
            ui.get_config_tls_cert().to_string(),
        ),
        "tls-key" => (
            FilterKind::CertKey,
            ui.global::<crate::Tr>().get_pick_cert_title().to_string(),
            ui.get_config_tls_key().to_string(),
        ),
        "plugin-cert" => (
            FilterKind::CertKey,
            ui.global::<crate::Tr>().get_pick_cert_title().to_string(),
            ui.get_tunnel_edit_plugin_cert_file().to_string(),
        ),
        "plugin-key" => (
            FilterKind::CertKey,
            ui.global::<crate::Tr>().get_pick_cert_title().to_string(),
            ui.get_tunnel_edit_plugin_key_file().to_string(),
        ),
        other => {
            tracing::warn!(target = %other, "unknown pick-file target");
            return;
        }
    };

    let target = target.to_string();
    let weak = ui.as_weak();
    thread::spawn(move || {
        let path = open_dialog(&title, kind, &current);
        let _ = slint::invoke_from_event_loop(move || {
            apply_path(&weak, &target, path);
        });
    });
}

fn open_dialog(title: &str, kind: FilterKind, current: &str) -> Option<PathBuf> {
    let mut dlg = rfd::FileDialog::new().set_title(title);
    if let Some(dir) = start_directory(current) {
        dlg = dlg.set_directory(dir);
    }
    match kind {
        FilterKind::Config => {
            dlg = dlg
                .add_filter("TOML", &["toml"])
                .add_filter("All files", &["*"]);
        }
        FilterKind::CertKey => {
            dlg = dlg
                .add_filter("Certificate / Key", &["pem", "crt", "cer", "key", "pub"])
                .add_filter("All files", &["*"]);
        }
    }
    dlg.pick_file()
}

fn start_directory(current: &str) -> Option<PathBuf> {
    let p = Path::new(current.trim());
    if current.trim().is_empty() {
        return None;
    }
    if p.is_dir() {
        return Some(p.to_path_buf());
    }
    p.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.to_path_buf())
}

fn apply_path(weak: &Weak<AppWindow>, target: &str, path: Option<PathBuf>) {
    let Some(path) = path else {
        return;
    };
    let Some(ui) = weak.upgrade() else {
        return;
    };
    let s: slint::SharedString = path.to_string_lossy().as_ref().into();
    match target {
        "config-file" => {
            ui.set_config_file_path(s.clone());
            match config_bridge::load_config(s.as_ref()) {
                Ok((cfg, resolved)) => {
                    ui.set_config_file_path(config_bridge::path_display(&resolved).into());
                    apply_config_to_ui(&ui, &cfg);
                }
                Err(e) => {
                    toast_err(&ui, format!("load config failed: {e}"));
                }
            }
        }
        "tls-ca" => ui.set_config_tls_ca(s),
        "tls-cert" => ui.set_config_tls_cert(s),
        "tls-key" => ui.set_config_tls_key(s),
        "plugin-cert" => ui.set_tunnel_edit_plugin_cert_file(s),
        "plugin-key" => ui.set_tunnel_edit_plugin_key_file(s),
        _ => {}
    }
}
