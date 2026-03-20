use serde_json::json;
use tauri::Manager;

use crate::{AppRuntime, AppState};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const POSTHOG_API_KEY: Option<&str> = option_env!("POSTHOG_API_KEY");
const POSTHOG_HOST: Option<&str> = option_env!("POSTHOG_HOST");

/// Initialize the global PostHog client and identify the anonymous install.
/// Respects `analytics_enabled`; skips entirely when opted out or unconfigured.
pub async fn init(app: &tauri::AppHandle<AppRuntime>) {
    let (api_key, host) = match (POSTHOG_API_KEY, POSTHOG_HOST) {
        (Some(k), Some(h)) if !k.is_empty() && !h.is_empty() => (k, h),
        _ => return,
    };

    let (enabled, distinct_id) = app.state::<AppState>().analytics_state();
    if !enabled || distinct_id.is_empty() {
        return;
    }

    let options = match posthog_rs::ClientOptionsBuilder::default()
        .api_key(api_key.to_string())
        .host(host)
        .build()
    {
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("Failed to build PostHog client options: {err}");
            return;
        }
    };

    if let Err(err) = posthog_rs::init_global(options).await {
        eprintln!("Failed to init PostHog: {err}");
        return;
    }

    let mut identify = posthog_rs::Event::new("$identify", &distinct_id);
    let _ = identify.insert_prop(
        "$set",
        json!({
            "app_version": APP_VERSION,
            "platform": std::env::consts::OS,
        }),
    );
    let _ = posthog_rs::capture(identify).await;
}

fn build_event(
    app: &tauri::AppHandle<AppRuntime>,
    event_name: &str,
    props: serde_json::Value,
) -> Option<posthog_rs::Event> {
    if POSTHOG_API_KEY.is_none_or(|k| k.is_empty()) || POSTHOG_HOST.is_none_or(|h| h.is_empty()) {
        return None;
    }

    let (enabled, distinct_id) = app.state::<AppState>().analytics_state();
    if !enabled || distinct_id.is_empty() {
        return None;
    }

    let mut event = posthog_rs::Event::new(event_name, &distinct_id);
    if let Some(obj) = props.as_object() {
        for (key, value) in obj {
            let _ = event.insert_prop(key.as_str(), value.clone());
        }
    }
    Some(event)
}

fn capture_event(app: &tauri::AppHandle<AppRuntime>, event_name: &str, props: serde_json::Value) {
    if let Some(event) = build_event(app, event_name, props) {
        tauri::async_runtime::spawn(async move {
            let _ = posthog_rs::capture(event).await;
        });
    }
}

/// Best-effort blocking capture for use during app exit.
/// SAFETY: Must be called from a synchronous context (e.g. Tauri window event handler).
/// Calling from within an async Tokio task will panic.
fn capture_event_blocking(
    app: &tauri::AppHandle<AppRuntime>,
    event_name: &str,
    props: serde_json::Value,
) {
    if let Some(event) = build_event(app, event_name, props) {
        let _ = tauri::async_runtime::block_on(async {
            tokio::time::timeout(
                std::time::Duration::from_secs(2),
                posthog_rs::capture(event),
            )
            .await
        });
    }
}

pub fn track_app_started(app: &tauri::AppHandle<AppRuntime>) {
    capture_event(app, "app_started", json!({}));
}

pub fn track_transcription_completed(
    app: &tauri::AppHandle<AppRuntime>,
    mode: &str,
    model: Option<&str>,
    llm_cleaned: bool,
) {
    capture_event(
        app,
        "transcription_completed",
        json!({
            "mode": mode,
            "model": model.unwrap_or("unknown"),
            "llm_cleaned": llm_cleaned,
        }),
    );
}

pub fn track_transcription_failed(app: &tauri::AppHandle<AppRuntime>, stage: &str, reason: &str) {
    capture_event(
        app,
        "transcription_failed",
        json!({ "stage": stage, "reason": reason }),
    );
}

pub fn track_model_downloaded(app: &tauri::AppHandle<AppRuntime>, model: &str) {
    capture_event(app, "model_downloaded", json!({ "model": model }));
}

pub fn track_onboarding_completed(app: &tauri::AppHandle<AppRuntime>) {
    capture_event(app, "onboarding_completed", json!({}));
}

pub fn track_app_exited(
    app: &tauri::AppHandle<AppRuntime>,
    uptime_seconds: f64,
    transcription_count: u32,
) {
    capture_event_blocking(
        app,
        "app_exited",
        json!({
            "uptime_seconds": uptime_seconds,
            "transcription_count": transcription_count,
        }),
    );
}
