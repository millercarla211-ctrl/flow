use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{llm_cleanup, AppRuntime, AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransformPreset {
    id: String,
    label: String,
    instruction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransformResult {
    preset_id: Option<String>,
    label: String,
    original: String,
    transformed: String,
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

    Ok(TransformResult {
        preset_id,
        label,
        original: text,
        transformed,
    })
}
