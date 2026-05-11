use tauri::{AppHandle, Manager};

use crate::{
    assistive, storage, AppRuntime, AppState, EVENT_SCRATCHPAD_CHANGED,
    EVENT_SCRATCHPAD_ENTRY_CREATED,
};

pub(crate) fn save_paste_fallback(
    app: &AppHandle<AppRuntime>,
    text: &str,
    source: &str,
) -> Result<storage::ScratchpadEntry, String> {
    if let Err(err) = assistive::copy_text_to_clipboard(text) {
        eprintln!("Failed to copy paste fallback transcript to clipboard: {err}");
    }

    let state = app.state::<AppState>();
    let entry = state
        .storage()
        .create_scratchpad_entry(text.to_string(), source.to_string())
        .map_err(|err| format!("Failed to save transcript to Scratchpad: {err}"))?;

    crate::emit_event(app, EVENT_SCRATCHPAD_ENTRY_CREATED, entry.clone());
    crate::emit_event(app, EVENT_SCRATCHPAD_CHANGED, ());
    Ok(entry)
}

#[tauri::command]
pub(crate) fn list_scratchpad_entries(
    state: tauri::State<AppState>,
    search_query: Option<String>,
) -> Result<Vec<storage::ScratchpadEntry>, String> {
    state
        .storage()
        .get_scratchpad_entries(search_query.as_deref())
        .map_err(|err| format!("Failed to list Scratchpad entries: {err}"))
}

#[tauri::command]
pub(crate) fn create_scratchpad_entry(
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
    body: String,
    source: Option<String>,
) -> Result<storage::ScratchpadEntry, String> {
    let entry = state
        .storage()
        .create_scratchpad_entry(body, source.unwrap_or_else(|| "manual".to_string()))
        .map_err(|err| format!("Failed to create Scratchpad entry: {err}"))?;

    crate::emit_event(&app, EVENT_SCRATCHPAD_ENTRY_CREATED, entry.clone());
    crate::emit_event(&app, EVENT_SCRATCHPAD_CHANGED, ());
    Ok(entry)
}

#[tauri::command]
pub(crate) fn update_scratchpad_entry(
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
    id: String,
    title: Option<String>,
    body: String,
) -> Result<Option<storage::ScratchpadEntry>, String> {
    let entry = state
        .storage()
        .update_scratchpad_entry(id, title, body)
        .map_err(|err| format!("Failed to update Scratchpad entry: {err}"))?;

    crate::emit_event(&app, EVENT_SCRATCHPAD_CHANGED, ());
    Ok(entry)
}

#[tauri::command]
pub(crate) fn delete_scratchpad_entry(
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
    id: String,
) -> Result<bool, String> {
    let deleted = state
        .storage()
        .delete_scratchpad_entry(&id)
        .map_err(|err| format!("Failed to delete Scratchpad entry: {err}"))?;

    crate::emit_event(&app, EVENT_SCRATCHPAD_CHANGED, ());
    Ok(deleted)
}
