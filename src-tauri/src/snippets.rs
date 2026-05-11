use tauri::AppHandle;

use crate::{storage, AppRuntime, AppState, EVENT_SNIPPETS_CHANGED};

#[tauri::command]
pub(crate) fn list_snippets(
    state: tauri::State<AppState>,
) -> Result<Vec<storage::Snippet>, String> {
    state
        .storage()
        .get_snippets()
        .map_err(|err| format!("Failed to list snippets: {err}"))
}

#[tauri::command]
pub(crate) fn create_snippet(
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
    trigger: String,
    expansion: String,
) -> Result<storage::Snippet, String> {
    let snippet = state
        .storage()
        .create_snippet(trigger, expansion)
        .map_err(|err| format!("Failed to create snippet: {err}"))?;

    crate::emit_event(&app, EVENT_SNIPPETS_CHANGED, ());
    Ok(snippet)
}

#[tauri::command]
pub(crate) fn update_snippet(
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
    id: String,
    trigger: String,
    expansion: String,
) -> Result<Option<storage::Snippet>, String> {
    let snippet = state
        .storage()
        .update_snippet(id, trigger, expansion)
        .map_err(|err| format!("Failed to update snippet: {err}"))?;

    crate::emit_event(&app, EVENT_SNIPPETS_CHANGED, ());
    Ok(snippet)
}

#[tauri::command]
pub(crate) fn delete_snippet(
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
    id: String,
) -> Result<bool, String> {
    let deleted = state
        .storage()
        .delete_snippet(&id)
        .map_err(|err| format!("Failed to delete snippet: {err}"))?;

    crate::emit_event(&app, EVENT_SNIPPETS_CHANGED, ());
    Ok(deleted)
}
