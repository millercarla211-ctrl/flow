use std::path::PathBuf;

use anyhow::{Context, Result};
use tauri::{AppHandle, Manager, Runtime};

fn env_dir(name: &str) -> Option<PathBuf> {
    let value = std::env::var_os(name)?;
    let path = PathBuf::from(value);
    if path.as_os_str().is_empty() {
        return None;
    }
    Some(path)
}

fn ensure_dir(path: PathBuf) -> Result<PathBuf> {
    std::fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create {}", path.display()))?;
    Ok(path)
}

#[cfg(target_os = "windows")]
fn best_drive_dir(kind: &str) -> Option<PathBuf> {
    let mut best: Option<(u64, PathBuf)> = None;

    for letter in b'D'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\", letter as char));
        if !root.exists() {
            continue;
        }

        let Ok(free_space) = fs2::available_space(&root) else {
            continue;
        };

        if match best.as_ref() {
            Some((best_free_space, _)) => free_space > *best_free_space,
            None => true,
        } {
            best = Some((free_space, root.join("Flow").join(kind)));
        }
    }

    best.map(|(_, path)| path)
}

#[cfg(not(target_os = "windows"))]
fn best_drive_dir(_kind: &str) -> Option<PathBuf> {
    None
}

pub fn app_data_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
    if let Some(path) = env_dir("FLOW_DATA_DIR") {
        return ensure_dir(path);
    }

    if let Some(path) = best_drive_dir("data") {
        return ensure_dir(path);
    }

    let path = app
        .path()
        .app_data_dir()
        .context("Unable to resolve app data directory")?;
    ensure_dir(path)
}

pub fn app_config_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
    if let Some(path) = env_dir("FLOW_CONFIG_DIR") {
        return ensure_dir(path);
    }

    if let Some(data_path) = env_dir("FLOW_DATA_DIR") {
        return ensure_dir(data_path.join("config"));
    }

    if let Some(path) = best_drive_dir("config") {
        return ensure_dir(path);
    }

    let resolver = app.path();
    let path = resolver
        .app_config_dir()
        .or_else(|_| resolver.app_data_dir())
        .context("Unable to resolve app config directory")?;
    ensure_dir(path)
}
