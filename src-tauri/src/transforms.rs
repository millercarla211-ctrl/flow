use arboard::Clipboard;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::{assistive, llm_cleanup, storage, toast, tray, AppRuntime, AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransformPreset {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) instruction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransformResult {
    history_id: Option<String>,
    preset_id: Option<String>,
    label: String,
    original: String,
    transformed: String,
    instruction: Option<String>,
    created_at: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransformSource {
    source: String,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransformPasteResult {
    pasted: bool,
    copied: bool,
    message: String,
}

pub(crate) fn default_transform_presets() -> Vec<TransformPreset> {
    vec![
        preset(
            "polish",
            "Polish",
            "Polish this text for clarity while preserving the original meaning and structure.",
        ),
        preset(
            "professional",
            "Professional",
            "Rewrite this in a crisp professional tone while preserving all facts.",
        ),
        preset(
            "fix_grammar",
            "Fix grammar",
            "Fix grammar, spelling, punctuation, and obvious wording issues without changing the meaning.",
        ),
        preset(
            "shorter",
            "Shorter",
            "Make this shorter and easier to scan while preserving the important facts.",
        ),
        preset(
            "longer",
            "More detailed",
            "Expand this with useful detail and clearer transitions without adding unsupported claims.",
        ),
        preset(
            "summarize",
            "Summarize",
            "Summarize this into the clearest useful version. Preserve names, numbers, and decisions.",
        ),
        preset(
            "casual",
            "Casual",
            "Rewrite this in a natural casual tone while preserving all facts.",
        ),
        preset(
            "gen_z",
            "Gen Z",
            "Rewrite this in a concise Gen Z tone without adding new facts.",
        ),
        preset(
            "partner",
            "Partner",
            "Rewrite this in a warm collaborative partner tone while preserving all facts.",
        ),
        preset(
            "turn_to_list",
            "Turn to list",
            "Convert this into a clean bullet list. Preserve every important detail.",
        ),
        preset(
            "turn_to_table",
            "Turn to table",
            "Convert this into a compact markdown table when the content supports it. Preserve facts.",
        ),
        preset(
            "prompt_engineer",
            "Prompt engineer",
            "Rewrite this as a precise prompt for an AI assistant. Keep the user's intent intact.",
        ),
        preset(
            "code_review",
            "Code review note",
            "Rewrite this as a concise engineering review comment with clear action, risk, and context.",
        ),
        preset(
            "terminal_command",
            "Terminal command",
            "Convert this spoken request into the safest executable shell command or short command sequence. Output only the command text, no markdown fences and no explanation. Preserve paths, package names, flags, branches, ports, and environment variable names exactly when they are spoken.",
        ),
        preset(
            "vibe_coding",
            "Vibe coding",
            "Turn this into a clear implementation request for a coding assistant, including expected behavior and constraints.",
        ),
    ]
}

fn preset(id: &str, label: &str, instruction: &str) -> TransformPreset {
    TransformPreset {
        id: id.to_string(),
        label: label.to_string(),
        instruction: instruction.to_string(),
    }
}

#[tauri::command]
pub(crate) fn get_transform_presets() -> Vec<TransformPreset> {
    default_transform_presets()
}

pub(crate) fn transform_preset_exists(id: &str) -> bool {
    default_transform_presets()
        .iter()
        .any(|preset| preset.id == id)
}

#[tauri::command]
pub(crate) fn get_transform_source() -> Result<TransformSource, String> {
    if let Some(text) = assistive::get_selected_text_ax() {
        return Ok(TransformSource {
            source: "selection".to_string(),
            text,
        });
    }

    let mut clipboard =
        Clipboard::new().map_err(|err| format!("Failed to access clipboard: {err}"))?;
    let text = clipboard
        .get_text()
        .map_err(|err| format!("No selected or clipboard text available: {err}"))?;

    if text.trim().is_empty() {
        return Err("No selected or clipboard text available.".to_string());
    }

    Ok(TransformSource {
        source: "clipboard".to_string(),
        text,
    })
}

#[tauri::command]
pub(crate) fn list_transform_history(
    state: tauri::State<AppState>,
    limit: Option<usize>,
) -> Result<Vec<storage::TransformHistoryEntry>, String> {
    state
        .storage()
        .get_transform_history(limit.unwrap_or(20))
        .map_err(|err| format!("Failed to list transform history: {err}"))
}

#[tauri::command]
pub(crate) fn delete_transform_history_entry(
    state: tauri::State<AppState>,
    id: String,
) -> Result<bool, String> {
    state
        .storage()
        .delete_transform_history_entry(&id)
        .map_err(|err| format!("Failed to delete transform history item: {err}"))
}

#[tauri::command]
pub(crate) fn open_transforms_view(
    app: AppHandle<AppRuntime>,
    text: Option<String>,
) -> Result<(), String> {
    tray::toggle_settings_window(&app).map_err(|err| format!("Failed to open Flow: {err}"))?;

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(180));
        if let Err(err) = app.emit("navigate:transforms", ()) {
            eprintln!("Failed to emit navigate:transforms: {err}");
        }
        if let Some(text) = text
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            std::thread::sleep(std::time::Duration::from_millis(80));
            if let Err(err) = app.emit(
                "transforms:load_text",
                serde_json::json!({ "text": text, "source": "overlay" }),
            ) {
                eprintln!("Failed to emit transforms:load_text: {err}");
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub(crate) fn paste_transform_result(
    app: AppHandle<AppRuntime>,
    text: String,
) -> Result<TransformPasteResult, String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("Text is required".to_string());
    }

    if let Some(window) = app.get_webview_window(crate::SETTINGS_WINDOW_LABEL) {
        let _ = window.hide();
        std::thread::sleep(std::time::Duration::from_millis(140));
    }

    match assistive::paste_text(&text) {
        Ok(()) => Ok(TransformPasteResult {
            pasted: true,
            copied: false,
            message: "Pasted transformed text".to_string(),
        }),
        Err(err) => {
            assistive::copy_text_to_clipboard(&text).map_err(|copy_err| {
                format!("Paste failed: {err}. Clipboard fallback failed: {copy_err}")
            })?;
            toast::show(
                &app,
                "info",
                Some("Copied transformed text"),
                "Paste was blocked, so Flow copied the result to your clipboard.",
            );
            Ok(TransformPasteResult {
                pasted: false,
                copied: true,
                message: format!("Paste failed; copied to clipboard. Reason: {err}"),
            })
        }
    }
}

#[tauri::command]
pub(crate) async fn transform_text(
    app: AppHandle<AppRuntime>,
    text: String,
    preset_id: Option<String>,
    instruction: Option<String>,
) -> Result<TransformResult, String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("Text is required".to_string());
    }

    let presets = default_transform_presets();
    let preset = preset_id
        .as_ref()
        .and_then(|id| presets.iter().find(|preset| &preset.id == id))
        .cloned();

    let instruction = instruction
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| preset.as_ref().map(|preset| preset.instruction.clone()))
        .ok_or_else(|| "Transform instruction is required".to_string())?;

    let label = preset
        .as_ref()
        .map(|preset| preset.label.clone())
        .unwrap_or_else(|| "Custom".to_string());

    let state = app.state::<AppState>();
    let settings = state.current_settings();
    if !llm_cleanup::is_llm_available(&settings) {
        return Err(
            "Configure a local or remote AI provider in Settings -> Models first.".to_string(),
        );
    }

    let transformed =
        llm_cleanup::edit_transcription(&state.http(), &text, &instruction, &settings)
            .await
            .map_err(|err| format!("Transform failed: {err}"))?;

    let history_entry = state
        .storage()
        .save_transform_history(
            label.clone(),
            preset_id.clone(),
            Some(instruction.clone()),
            text.clone(),
            transformed.clone(),
        )
        .map_err(|err| format!("Transform succeeded but history could not be saved: {err}"))?;

    Ok(TransformResult {
        history_id: Some(history_entry.id),
        preset_id,
        label,
        original: text,
        transformed,
        instruction: Some(instruction),
        created_at: Some(history_entry.created_at),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_presets_include_terminal_command_transform() {
        let presets = default_transform_presets();
        let terminal = presets
            .iter()
            .find(|preset| preset.id == "terminal_command")
            .expect("terminal command preset");

        assert_eq!(terminal.label, "Terminal command");
        assert!(terminal
            .instruction
            .contains("Output only the command text"));
    }
}
