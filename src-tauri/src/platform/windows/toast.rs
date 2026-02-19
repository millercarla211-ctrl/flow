use crate::AppRuntime;
use anyhow::Result;
use tauri::{AppHandle, WebviewWindow};

pub fn init(_app: &AppHandle<AppRuntime>, _toast_window: &WebviewWindow<AppRuntime>) -> Result<()> {
    Ok(())
}

pub fn show(_app: &AppHandle<AppRuntime>, _toast_window: &WebviewWindow<AppRuntime>) -> Result<()> {
    Ok(())
}

pub fn hide(_app: &AppHandle<AppRuntime>, _toast_window: &WebviewWindow<AppRuntime>) -> Result<()> {
    Ok(())
}
