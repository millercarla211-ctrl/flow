use std::{
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

use arboard::Clipboard;
use regex::Regex;
use tauri::{AppHandle, Manager};

use crate::{
    assistive, storage, AppRuntime, AppState, EVENT_FLOW_FETCH_CHANGED,
    EVENT_FLOW_FETCH_LINK_CAPTURED,
};

static STARTED: OnceLock<()> = OnceLock::new();

pub(crate) fn start_monitor(app: AppHandle<AppRuntime>) {
    if STARTED.set(()).is_err() {
        return;
    }

    if let Err(err) = thread::Builder::new()
        .name("flow-fetch-monitor".into())
        .spawn(move || {
            let mut last_seen = String::new();
            let mut last_prune = Instant::now();

            loop {
                thread::sleep(Duration::from_secs(2));
                let Some(url) = clipboard_url() else {
                    continue;
                };
                if url == last_seen {
                    continue;
                }
                last_seen = url.clone();

                let state = app.state::<AppState>();
                match state.storage().upsert_flow_fetch_link(url) {
                    Ok(link) => {
                        crate::emit_event(&app, EVENT_FLOW_FETCH_LINK_CAPTURED, link);
                        crate::emit_event(&app, EVENT_FLOW_FETCH_CHANGED, ());
                    }
                    Err(err) => eprintln!("Failed to capture Flow Fetch link: {err}"),
                }

                if last_prune.elapsed() > Duration::from_secs(60 * 60) {
                    if let Err(err) = state.storage().prune_flow_fetch_links() {
                        eprintln!("Failed to prune Flow Fetch links: {err}");
                    }
                    last_prune = Instant::now();
                }
            }
        })
    {
        eprintln!("Failed to start Flow Fetch monitor: {err}");
    }
}

fn clipboard_url() -> Option<String> {
    let mut clipboard = Clipboard::new().ok()?;
    let text = clipboard.get_text().ok()?;
    first_url_candidate(&text)
}

fn first_url_candidate(text: &str) -> Option<String> {
    static URL_RE: OnceLock<Regex> = OnceLock::new();
    let regex = URL_RE.get_or_init(|| Regex::new(r#"https?://[^\s<>"']+"#).expect("url regex"));
    let candidate = regex.find(text)?.as_str().to_string();
    storage::normalize_flow_fetch_url(candidate).ok()
}

#[tauri::command]
pub(crate) fn list_flow_fetch_links(
    state: tauri::State<AppState>,
    limit: Option<usize>,
) -> Result<Vec<storage::FlowFetchLink>, String> {
    state
        .storage()
        .get_flow_fetch_links(limit.unwrap_or(30))
        .map_err(|err| format!("Failed to list Flow Fetch links: {err}"))
}

#[tauri::command]
pub(crate) fn delete_flow_fetch_link(
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
    id: String,
) -> Result<bool, String> {
    let deleted = state
        .storage()
        .delete_flow_fetch_link(&id)
        .map_err(|err| format!("Failed to delete Flow Fetch link: {err}"))?;

    crate::emit_event(&app, EVENT_FLOW_FETCH_CHANGED, ());
    Ok(deleted)
}

#[tauri::command]
pub(crate) fn copy_flow_fetch_link(url: String) -> Result<(), String> {
    assistive::copy_text_to_clipboard(&url)
        .map_err(|err| format!("Failed to copy Flow Fetch link: {err}"))
}

#[cfg(test)]
mod tests {
    use super::first_url_candidate;

    #[test]
    fn finds_a_url_inside_clipboard_text() {
        assert_eq!(
            first_url_candidate("read https://example.com/hello-world now").as_deref(),
            Some("https://example.com/hello-world")
        );
    }
}
