use arboard::Clipboard;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{assistive, llm_cleanup, storage, AppRuntime, AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransformPreset {
    id: String,
    label: String,
    instruction: String,
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
