use std::path::{Path, PathBuf};

pub fn path_for(config_path: &Path) -> PathBuf {
    let mut p = config_path.to_path_buf();
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!("{e}.session_id"))
        .unwrap_or_else(|| "session_id".into());
    p.set_extension(ext);
    p
}

fn read_valid_id(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| {
            !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
        })
}

pub fn load(config_path: &Path) -> String {
    read_valid_id(&path_for(config_path)).unwrap_or_default()
}

pub fn save(config_path: &Path, session_id: &str) -> std::io::Result<()> {
    let path = path_for(config_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_for_replaces_extension() {
        let p = path_for(Path::new("/tmp/foo.toml"));
        assert_eq!(p, PathBuf::from("/tmp/foo.toml.session_id"));
    }

    #[test]
    fn path_for_without_extension() {
        let p = path_for(Path::new("/tmp/foo"));
        assert_eq!(p, PathBuf::from("/tmp/foo.session_id"));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir();
        let cfg = dir.join("orbien_runid_test.toml");
        save(&cfg, "ebe14e38ff3f4073").unwrap();
        assert_eq!(load(&cfg), "ebe14e38ff3f4073");
        std::fs::remove_file(path_for(&cfg)).ok();
    }

    #[test]
    fn load_rejects_invalid() {
        let dir = std::env::temp_dir();
        let cfg = dir.join("orbien_runid_invalid.toml");
        // Write a value with invalid chars (space) directly to the run_id file.
        let p = path_for(&cfg);
        std::fs::write(&p, "has space here").unwrap();
        assert_eq!(load(&cfg), "");
        std::fs::remove_file(p).ok();
    }

    #[test]
    fn load_missing_returns_empty() {
        let cfg = Path::new("/nonexistent/dir/config.toml");
        assert_eq!(load(cfg), "");
    }
}
