use tauri::{AppHandle, Emitter};

use crate::{storage, tray, AppRuntime, AppState, EVENT_SNIPPETS_CHANGED};

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

#[tauri::command]
pub(crate) fn open_snippets_view(
    app: AppHandle<AppRuntime>,
    expansion: Option<String>,
) -> Result<(), String> {
    tray::toggle_settings_window(&app).map_err(|err| format!("Failed to open Flow: {err}"))?;

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(180));
        if let Err(err) = app.emit("navigate:snippets", ()) {
            eprintln!("Failed to emit navigate:snippets: {err}");
        }

        if let Some(expansion) = expansion
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            std::thread::sleep(std::time::Duration::from_millis(80));
            if let Err(err) = app.emit(
                "snippets:load_expansion",
                serde_json::json!({ "expansion": expansion }),
            ) {
                eprintln!("Failed to emit snippets:load_expansion: {err}");
            }
        }
    });

    Ok(())
}
