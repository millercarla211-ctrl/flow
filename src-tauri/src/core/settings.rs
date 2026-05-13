use serde::Deserialize;
use tauri::{AppHandle, Emitter};

use super::hotkeys;
use crate::settings::{
    canonicalize_app_locale, canonicalize_app_locale_or_default, LlmProvider,
    LocalDataStoragePolicy, RecordingPrunePolicy, ThemeMode, TranscriptionMode, TtsVoiceMode,
    UserSettings,
};

use crate::{analytics, model_manager, pill, tray, AppRuntime, AppState, EVENT_SETTINGS_CHANGED};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateSettingsArgs {
    pub smart_shortcut: String,
    #[serde(default)]
    pub smart_shortcuts: Vec<String>,
    pub smart_enabled: bool,
    pub hold_shortcut: String,
    #[serde(default)]
    pub hold_shortcuts: Vec<String>,
    pub hold_enabled: bool,
    pub toggle_shortcut: String,
    #[serde(default)]
    pub toggle_shortcuts: Vec<String>,
    pub toggle_enabled: bool,
    pub command_shortcut: String,
    #[serde(default)]
    pub command_shortcuts: Vec<String>,
    pub command_enabled: bool,
    pub paste_last_transcript_shortcut: String,
    #[serde(default)]
    pub paste_last_transcript_shortcuts: Vec<String>,
    pub paste_last_transcript_enabled: bool,
    pub cancel_shortcut: String,
    #[serde(default)]
    pub cancel_shortcuts: Vec<String>,
    pub cancel_enabled: bool,
    #[serde(default)]
    pub wake_listening_enabled: bool,
    #[serde(default)]
    pub wake_phrases: Vec<String>,
    #[serde(default)]
    pub wake_speaker_verification_enabled: bool,
    pub transcription_mode: TranscriptionMode,
    pub local_model: String,
    #[serde(default)]
    pub tts_enabled: bool,
    #[serde(default)]
    pub tts_auto_after_stt: bool,
    #[serde(default)]
    pub tts_auto_play: bool,
    #[serde(default = "crate::settings::default_tts_volume")]
    pub tts_volume: f32,
    #[serde(default = "crate::tts::default_tts_model")]
    pub tts_model: String,
    #[serde(default)]
    pub tts_voice_mode: TtsVoiceMode,
    #[serde(default)]
    pub tts_speaker: String,
    #[serde(default)]
    pub tts_instruction: String,
    pub microphone_device: Option<String>,
    pub language: String,
    pub app_locale: String,
    pub theme_mode: ThemeMode,
    pub llm_enabled: bool,

    pub cleanup_enabled: bool,
    pub llm_provider: LlmProvider,
    pub llm_endpoint: String,
    pub llm_api_key: String,
    pub llm_model: String,
    pub edit_mode_enabled: bool,
    pub auto_transform_enabled: bool,
    pub auto_transform_preset_id: String,
    pub vibe_coding_enabled: bool,
    pub vibe_coding_variable_recognition: bool,
    pub vibe_coding_file_tagging: bool,
    pub vibe_coding_include_window_context: bool,
    pub media_control_enabled: bool,
    pub auto_update_enabled: bool,
    pub auto_launch_enabled: bool,
    pub recording_prune_policy: RecordingPrunePolicy,
    pub local_data_storage_policy: LocalDataStoragePolicy,
    pub context_awareness_enabled: bool,
    pub analytics_enabled: bool,
}

fn canonicalize_shortcut_for_storage(shortcut: &str) -> Result<String, String> {
    hotkeys::normalize_recording_shortcut(shortcut)
        .map(|shortcut| shortcut.trim().to_string())
        .map_err(|err| err.to_string())
}

fn shortcut_inputs(shortcuts: &[String], fallback: &str) -> Vec<String> {
    let mut sources = shortcuts.to_vec();
    if !fallback.trim().is_empty() {
        if sources.is_empty() {
            sources.push(fallback.to_string());
        } else {
            sources[0] = fallback.to_string();
        }
    }

    sources
        .iter()
        .map(String::as_str)
        .map(|shortcut| shortcut.trim())
        .filter(|shortcut| !shortcut.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn canonicalize_shortcut_list_for_storage(
    label: &str,
    enabled: bool,
    shortcuts: &[String],
    fallback: &str,
) -> Result<Vec<String>, String> {
    let inputs = shortcut_inputs(shortcuts, fallback);
    let mut seen_inputs = std::collections::HashSet::new();
    let inputs: Vec<String> = inputs
        .into_iter()
        .filter(|input| seen_inputs.insert(input.to_ascii_lowercase()))
        .collect();

    if enabled && inputs.is_empty() {
        return Err(format!("{label} shortcut cannot be empty when enabled"));
    }

    if inputs.len() > 4 {
        return Err(format!("{label} supports up to 4 shortcuts"));
    }

    let mut canonical = Vec::new();
    for input in inputs {
        let shortcut = canonicalize_shortcut_for_storage(&input)
            .map_err(|err| format!("{label} shortcut is invalid: {err}"))?;
        if !canonical
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&shortcut))
        {
            canonical.push(shortcut);
        }
    }

    Ok(canonical)
}

fn primary_shortcut(shortcuts: &[String], fallback: String) -> String {
    shortcuts.first().cloned().unwrap_or(fallback)
}

fn validate_update_settings_args(args: &UpdateSettingsArgs) -> Result<(), String> {
    if !args.smart_enabled && !args.hold_enabled && !args.toggle_enabled {
        return Err("At least one recording mode must be enabled".into());
    }

    let mut enabled_shortcuts: Vec<(&str, hotkeys::ShortcutBinding)> = vec![];

    let groups = [
        (
            "Smart",
            args.smart_enabled,
            args.smart_shortcuts.as_slice(),
            args.smart_shortcut.as_str(),
        ),
        (
            "Hold",
            args.hold_enabled,
            args.hold_shortcuts.as_slice(),
            args.hold_shortcut.as_str(),
        ),
        (
            "Toggle",
            args.toggle_enabled,
            args.toggle_shortcuts.as_slice(),
            args.toggle_shortcut.as_str(),
        ),
        (
            "Command Mode",
            args.command_enabled,
            args.command_shortcuts.as_slice(),
            args.command_shortcut.as_str(),
        ),
        (
            "Paste Last Transcript",
            args.paste_last_transcript_enabled,
            args.paste_last_transcript_shortcuts.as_slice(),
            args.paste_last_transcript_shortcut.as_str(),
        ),
        (
            "Cancel",
            args.cancel_enabled,
            args.cancel_shortcuts.as_slice(),
            args.cancel_shortcut.as_str(),
        ),
    ];

    for (label, enabled, shortcuts, fallback) in groups {
        let canonical =
            canonicalize_shortcut_list_for_storage(label, enabled, shortcuts, fallback)?;
        if enabled {
            for shortcut in canonical {
                let binding = hotkeys::parse_shortcut_binding(&shortcut)
                    .map_err(|err| format!("{label} shortcut is invalid: {err}"))?;
                enabled_shortcuts.push((label, binding));
            }
        }
    }

    for i in 0..enabled_shortcuts.len() {
        for j in (i + 1)..enabled_shortcuts.len() {
            let (name1, normalized1) = &enabled_shortcuts[i];
            let (name2, normalized2) = &enabled_shortcuts[j];
            if normalized1 == normalized2 {
                return Err(format!(
                    "{} and {} shortcuts cannot be the same",
                    name1, name2
                ));
            }

            if hotkeys::shortcut_bindings_conflict(normalized1, normalized2) {
                return Err(format!(
                    "{} shortcut overlaps {} shortcut. Choose a more specific combination.",
                    name1, name2
                ));
            }
        }
    }

    if model_manager::definition(&args.local_model).is_none() {
        return Err("Unknown model selection".into());
    }

    if crate::tts::definition(&args.tts_model).is_none() {
        return Err("Unknown TTS model selection".into());
    }

    if !crate::tts::model_supports_voice_mode(&args.tts_model, args.tts_voice_mode) {
        return Err("Selected TTS model does not support that voice mode".into());
    }

    if canonicalize_app_locale(&args.app_locale).is_none() {
        return Err("Unknown app language selection".into());
    }

    if args.llm_enabled && matches!(args.llm_provider, LlmProvider::None) {
        return Err("LLM cannot be enabled when provider is None".into());
    }

    if args.cleanup_enabled && !args.llm_enabled {
        return Err("AI Cleanup cannot be enabled without an active language model".into());
    }

    if args.auto_transform_enabled && !args.llm_enabled {
        return Err("Auto Transform cannot be enabled without an active language model".into());
    }

    if !crate::transforms::transform_preset_exists(args.auto_transform_preset_id.trim()) {
        return Err("Unknown auto transform preset".into());
    }

    if args.llm_enabled {
        if matches!(args.llm_provider, LlmProvider::Local)
            && !crate::local_text_model::is_model_available(args.llm_model.trim())
        {
            return Err("Selected local text model is not installed".into());
        }
        if matches!(args.llm_provider, LlmProvider::Custom) && args.llm_endpoint.trim().is_empty() {
            return Err("Custom LLM endpoint cannot be empty".into());
        }
        if matches!(args.llm_provider, LlmProvider::OpenAI) && args.llm_api_key.trim().is_empty() {
            return Err("OpenAI API key is required".into());
        }
        if args.llm_model.trim().is_empty() {
            return Err("Choose a language model before enabling AI features".into());
        }
    }

    Ok(())
}

pub(crate) fn complete_onboarding(
    app: &AppHandle<AppRuntime>,
    state: &AppState,
) -> Result<(), String> {
    let mut settings = state.current_settings();
    settings.onboarding_completed = true;
    let next = state
        .persist_settings(settings)
        .map_err(|err| err.to_string())?;

    if let Err(err) = app.emit(EVENT_SETTINGS_CHANGED, &next) {
        eprintln!("Failed to emit settings change: {err}");
    }

    analytics::track_onboarding_completed(app);
    Ok(())
}

pub(crate) fn reset_onboarding(
    app: &AppHandle<AppRuntime>,
    state: &AppState,
) -> Result<(), String> {
    let mut settings = state.current_settings();
    settings.onboarding_completed = false;
    let next = state
        .persist_settings(settings)
        .map_err(|err| err.to_string())?;

    if let Err(err) = app.emit(EVENT_SETTINGS_CHANGED, &next) {
        eprintln!("Failed to emit settings change: {err}");
    }

    Ok(())
}

pub(crate) fn set_user_name(
    name: String,
    app: &AppHandle<AppRuntime>,
    state: &AppState,
) -> Result<UserSettings, String> {
    let mut settings = state.current_settings();
    settings.user_name = name.trim().to_string();
    let next = state
        .persist_settings(settings)
        .map_err(|err| err.to_string())?;

    if let Err(err) = app.emit(EVENT_SETTINGS_CHANGED, &next) {
        eprintln!("Failed to emit settings change: {err}");
    }

    Ok(next)
}

pub(crate) fn update_settings(
    args: UpdateSettingsArgs,
    app: &AppHandle<AppRuntime>,
    state: &AppState,
) -> Result<UserSettings, String> {
    validate_update_settings_args(&args)?;

    let mut next = state.current_settings();
    let prev = next.clone();
    next.smart_shortcuts = canonicalize_shortcut_list_for_storage(
        "Smart",
        args.smart_enabled,
        &args.smart_shortcuts,
        &args.smart_shortcut,
    )?;
    next.smart_shortcut = primary_shortcut(&next.smart_shortcuts, args.smart_shortcut);
    next.smart_enabled = args.smart_enabled;
    next.hold_shortcuts = canonicalize_shortcut_list_for_storage(
        "Hold",
        args.hold_enabled,
        &args.hold_shortcuts,
        &args.hold_shortcut,
    )?;
    next.hold_shortcut = primary_shortcut(&next.hold_shortcuts, args.hold_shortcut);
    next.hold_enabled = args.hold_enabled;
    next.toggle_shortcuts = canonicalize_shortcut_list_for_storage(
        "Toggle",
        args.toggle_enabled,
        &args.toggle_shortcuts,
        &args.toggle_shortcut,
    )?;
    next.toggle_shortcut = primary_shortcut(&next.toggle_shortcuts, args.toggle_shortcut);
    next.toggle_enabled = args.toggle_enabled;
    next.command_shortcuts = canonicalize_shortcut_list_for_storage(
        "Command Mode",
        args.command_enabled,
        &args.command_shortcuts,
        &args.command_shortcut,
    )?;
    next.command_shortcut = primary_shortcut(&next.command_shortcuts, args.command_shortcut);
    next.command_enabled = args.command_enabled;
    next.paste_last_transcript_shortcuts = canonicalize_shortcut_list_for_storage(
        "Paste Last Transcript",
        args.paste_last_transcript_enabled,
        &args.paste_last_transcript_shortcuts,
        &args.paste_last_transcript_shortcut,
    )?;
    next.paste_last_transcript_shortcut = primary_shortcut(
        &next.paste_last_transcript_shortcuts,
        args.paste_last_transcript_shortcut,
    );
    next.paste_last_transcript_enabled = args.paste_last_transcript_enabled;
    next.cancel_shortcuts = canonicalize_shortcut_list_for_storage(
        "Cancel",
        args.cancel_enabled,
        &args.cancel_shortcuts,
        &args.cancel_shortcut,
    )?;
    next.cancel_shortcut = primary_shortcut(&next.cancel_shortcuts, args.cancel_shortcut);
    next.cancel_enabled = args.cancel_enabled;
    next.wake_listening_enabled = args.wake_listening_enabled;
    next.wake_phrases = args
        .wake_phrases
        .into_iter()
        .map(|phrase| phrase.trim().to_ascii_lowercase())
        .filter(|phrase| !phrase.is_empty())
        .take(8)
        .collect();
    if next.wake_phrases.is_empty() {
        next.wake_phrases = crate::settings::default_wake_phrases();
    }
    next.wake_speaker_verification_enabled = args.wake_speaker_verification_enabled;
    next.transcription_mode = args.transcription_mode;
    next.local_model = args.local_model;
    next.tts_enabled = args.tts_enabled;
    next.tts_auto_after_stt = args.tts_auto_after_stt;
    next.tts_auto_play = args.tts_auto_play;
    next.tts_volume = crate::settings::clamp_tts_volume(args.tts_volume);
    next.tts_model = args.tts_model;
    next.tts_voice_mode = args.tts_voice_mode;
    next.tts_speaker = args.tts_speaker.trim().to_string();
    next.tts_instruction = args.tts_instruction.trim().to_string();
    next.microphone_device = args.microphone_device;
    next.language = args.language;
    next.app_locale = canonicalize_app_locale_or_default(&args.app_locale);
    next.theme_mode = args.theme_mode;
    next.llm_enabled = args.llm_enabled;

    next.cleanup_enabled = args.cleanup_enabled;
    next.llm_provider = args.llm_provider;
    next.llm_endpoint = args.llm_endpoint;
    next.llm_api_key = args.llm_api_key;
    next.llm_model = args.llm_model.trim().to_string();
    next.edit_mode_enabled = args.edit_mode_enabled;
    next.auto_transform_enabled = args.auto_transform_enabled;
    next.auto_transform_preset_id = args.auto_transform_preset_id.trim().to_string();
    next.vibe_coding_enabled = args.vibe_coding_enabled;
    next.vibe_coding_variable_recognition = args.vibe_coding_variable_recognition;
    next.vibe_coding_file_tagging = args.vibe_coding_file_tagging;
    next.vibe_coding_include_window_context = args.vibe_coding_include_window_context;
    next.media_control_enabled = args.media_control_enabled;
    next.auto_update_enabled = args.auto_update_enabled;
    next.auto_launch_enabled = args.auto_launch_enabled;
    next.recording_prune_policy = args.recording_prune_policy;
    next.local_data_storage_policy = args.local_data_storage_policy;
    next.context_awareness_enabled = args.context_awareness_enabled;
    next.analytics_enabled = args.analytics_enabled;

    let launch_changed = prev.auto_launch_enabled != next.auto_launch_enabled;
    if launch_changed {
        crate::sync_launch_at_login(app, next.auto_launch_enabled)?;
    }
    let requested_auto_launch_enabled = next.auto_launch_enabled;

    let next = match state.persist_settings(next) {
        Ok(next) => next,
        Err(err) => {
            if launch_changed {
                if let Err(rollback_err) =
                    crate::sync_launch_at_login(app, prev.auto_launch_enabled)
                {
                    return Err(format!(
                        "{} (also failed to roll back launch at login from {} back to {}: {})",
                        err, requested_auto_launch_enabled, prev.auto_launch_enabled, rollback_err
                    ));
                }
            }
            return Err(err.to_string());
        }
    };

    state.request_preflight_refresh();
    if prev.tts_enabled != next.tts_enabled || prev.tts_model != next.tts_model {
        crate::tts::prewarm_if_needed(app, &next);
    }

    pill::register_shortcuts(app).map_err(|err| err.to_string())?;
    state.wake.sync(app, &next).map_err(|err| err.to_string())?;

    if prev.transcription_mode != next.transcription_mode
        || prev.local_model != next.local_model
        || prev.microphone_device != next.microphone_device
    {
        if prev.transcription_mode != next.transcription_mode
            || prev.local_model != next.local_model
        {
            state.download_default_local_model_if_missing(app, &next, "settings");
            state.preload_local_model_if_needed(app, &next, "settings");
        }

        if let Err(err) = tray::refresh_tray_menu(app, &next) {
            eprintln!("Failed to refresh tray menu: {err}");
        }
        #[cfg(target_os = "macos")]
        if let Err(err) = crate::set_app_menu(app, &next) {
            eprintln!("Failed to refresh app menu: {err}");
        }
    }

    if let Err(err) = app.emit(EVENT_SETTINGS_CHANGED, &next) {
        eprintln!("Failed to emit settings change: {err}");
    }

    if prev.recording_prune_policy != next.recording_prune_policy {
        crate::schedule_recording_prune(app.clone(), next.clone());
    }
    if prev.local_data_storage_policy != next.local_data_storage_policy {
        crate::schedule_local_data_prune(app.clone(), next.clone());
    }

    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::default_local_model;

    fn base_args() -> UpdateSettingsArgs {
        UpdateSettingsArgs {
            smart_shortcut: "Control+Space".to_string(),
            smart_shortcuts: vec!["Control+Space".to_string(), "Win+Alt+1".to_string()],
            smart_enabled: true,
            hold_shortcut: "Control+Shift+Space".to_string(),
            hold_shortcuts: vec!["Control+Shift+Space".to_string()],
            hold_enabled: false,
            toggle_shortcut: "Control+Alt+Space".to_string(),
            toggle_shortcuts: vec!["Control+Alt+Space".to_string()],
            toggle_enabled: false,
            command_shortcut: "Control+Alt+E".to_string(),
            command_shortcuts: vec!["Control+Alt+E".to_string()],
            command_enabled: false,
            paste_last_transcript_shortcut: "Shift+Alt+Z".to_string(),
            paste_last_transcript_shortcuts: vec!["Shift+Alt+Z".to_string()],
            paste_last_transcript_enabled: true,
            cancel_shortcut: "Control+Alt+Escape".to_string(),
            cancel_shortcuts: vec!["Control+Alt+Escape".to_string()],
            cancel_enabled: false,
            wake_listening_enabled: true,
            wake_phrases: crate::settings::default_wake_phrases(),
            wake_speaker_verification_enabled: false,
            transcription_mode: TranscriptionMode::Local,
            local_model: default_local_model(),
            tts_enabled: true,
            tts_auto_after_stt: true,
            tts_auto_play: true,
            tts_volume: crate::settings::default_tts_volume(),
            tts_model: crate::tts::default_tts_model(),
            tts_voice_mode: TtsVoiceMode::SourceAudio,
            tts_speaker: String::new(),
            tts_instruction: String::new(),
            microphone_device: None,
            language: "en".to_string(),
            app_locale: "system".to_string(),
            theme_mode: ThemeMode::default(),
            llm_enabled: false,

            cleanup_enabled: false,
            llm_provider: LlmProvider::None,
            llm_endpoint: String::new(),
            llm_api_key: String::new(),
            llm_model: String::new(),
            edit_mode_enabled: false,
            auto_transform_enabled: false,
            auto_transform_preset_id: "polish".to_string(),
            vibe_coding_enabled: true,
            vibe_coding_variable_recognition: true,
            vibe_coding_file_tagging: true,
            vibe_coding_include_window_context: true,
            media_control_enabled: true,
            auto_update_enabled: true,
            auto_launch_enabled: false,
            recording_prune_policy: RecordingPrunePolicy::Never,
            local_data_storage_policy: LocalDataStoragePolicy::Store,
            context_awareness_enabled: true,
            analytics_enabled: true,
        }
    }

    #[test]
    fn rejects_enabling_llm_without_provider() {
        let mut args = base_args();
        args.llm_enabled = true;

        let err = validate_update_settings_args(&args).unwrap_err();

        assert_eq!(err, "LLM cannot be enabled when provider is None");
    }

    #[test]
    fn rejects_enabling_cleanup_without_llm() {
        let mut args = base_args();
        args.cleanup_enabled = true;

        let err = validate_update_settings_args(&args).unwrap_err();

        assert_eq!(
            err,
            "AI Cleanup cannot be enabled without an active language model"
        );
    }

    #[test]
    fn rejects_enabling_auto_transform_without_llm() {
        let mut args = base_args();
        args.auto_transform_enabled = true;

        let err = validate_update_settings_args(&args).unwrap_err();

        assert_eq!(
            err,
            "Auto Transform cannot be enabled without an active language model"
        );
    }

    #[test]
    fn rejects_unknown_auto_transform_preset() {
        let mut args = base_args();
        args.auto_transform_preset_id = "unknown".to_string();

        let err = validate_update_settings_args(&args).unwrap_err();

        assert_eq!(err, "Unknown auto transform preset");
    }

    #[test]
    fn rejects_shortcut_collisions_after_normalization() {
        let mut args = base_args();
        args.hold_enabled = true;
        args.hold_shortcut = "Ctrl+Space".to_string();

        let err = validate_update_settings_args(&args).unwrap_err();

        assert_eq!(err, "Smart and Hold shortcuts cannot be the same");
    }

    #[test]
    fn rejects_command_shortcut_overlap() {
        let mut args = base_args();
        args.command_enabled = true;
        args.command_shortcut = "Ctrl+Space".to_string();

        let err = validate_update_settings_args(&args).unwrap_err();

        assert_eq!(err, "Smart and Command Mode shortcuts cannot be the same");
    }

    #[test]
    fn rejects_paste_last_transcript_shortcut_overlap() {
        let mut args = base_args();
        args.paste_last_transcript_shortcut = "Ctrl+Space".to_string();

        let err = validate_update_settings_args(&args).unwrap_err();

        assert_eq!(
            err,
            "Smart and Paste Last Transcript shortcuts cannot be the same"
        );
    }

    #[test]
    fn rejects_cancel_shortcut_overlap() {
        let mut args = base_args();
        args.cancel_enabled = true;
        args.cancel_shortcut = "Ctrl+Space".to_string();

        let err = validate_update_settings_args(&args).unwrap_err();

        assert_eq!(err, "Smart and Cancel shortcuts cannot be the same");
    }

    #[test]
    fn accepts_mouse_shortcut_bindings() {
        let mut args = base_args();
        args.toggle_enabled = true;
        args.toggle_shortcuts = vec!["MouseBack".to_string()];

        validate_update_settings_args(&args).unwrap();
    }

    #[test]
    fn rejects_more_than_four_shortcuts_per_action() {
        let mut args = base_args();
        args.smart_shortcuts = vec![
            "Ctrl+Space".to_string(),
            "Win+Alt+1".to_string(),
            "MouseBack".to_string(),
            "MouseForward".to_string(),
            "MouseMiddle".to_string(),
        ];

        let err = validate_update_settings_args(&args).unwrap_err();

        assert_eq!(err, "Smart supports up to 4 shortcuts");
    }

    #[test]
    fn accepts_modifier_only_recording_shortcut() {
        let mut args = base_args();
        args.smart_shortcut = "Ctrl".to_string();

        validate_update_settings_args(&args).unwrap();
    }

    #[test]
    fn rejects_enabling_llm_without_explicit_model_selection() {
        let mut args = base_args();
        args.llm_enabled = true;
        args.llm_provider = LlmProvider::Ollama;

        let err = validate_update_settings_args(&args).unwrap_err();

        assert_eq!(err, "Choose a language model before enabling AI features");
    }
}
