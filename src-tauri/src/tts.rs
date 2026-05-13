use std::{
    collections::HashSet,
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, OnceLock,
    },
    time::Instant,
};

use anyhow::{anyhow, Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::{async_runtime, AppHandle, Emitter, Manager, Runtime};

use crate::{
    downloader::{download_model_files, ModelFileDescriptor},
    model_language_table::SupportedLanguageInfo,
    settings::{TtsVoiceMode, UserSettings},
    toast, AppRuntime, AppState,
};

const TTS_MODELS_ROOT: &str = "models/tts";
const TTS_OUTPUT_ROOT: &str = "tts";
const TTS_EVENT_COMPLETE: &str = "tts:complete";
const TTS_EVENT_ERROR: &str = "tts:error";
const RUNNER_SCRIPT_NAME: &str = "qwen3_tts_runner.py";
const RUNNER_SCRIPT: &str = include_str!("../../scripts/qwen3_tts_runner.py");
static KOKORO_WORKER: OnceLock<Mutex<Option<KokoroWorker>>> = OnceLock::new();
static KOKORO_PREWARMING: AtomicBool = AtomicBool::new(false);

pub const TTS_CAPABILITY_VOICE_CLONE: &str = "voice_clone";
pub const TTS_CAPABILITY_CUSTOM_VOICE: &str = "custom_voice";
pub const TTS_CAPABILITY_FAST_LOCAL: &str = "fast_local";

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;
#[cfg(target_os = "windows")]
const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x00004000;

const TTS_CPU_THREADS: &str = "4";

#[derive(Debug, Clone)]
pub struct TtsModelDefinition {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub repository: &'static str,
    pub size_mb: f32,
    pub files: &'static [ModelFileDescriptor],
    pub variant: &'static str,
    pub tags: &'static [&'static str],
    pub capabilities: &'static [&'static str],
}

#[derive(Debug, Serialize, Clone)]
pub struct TtsModelInfo {
    pub key: String,
    pub label: String,
    pub description: String,
    pub repository: String,
    pub size_mb: f32,
    pub file_count: usize,
    pub engine_id: String,
    pub engine: String,
    pub variant: String,
    pub tags: Vec<String>,
    pub capabilities: Vec<String>,
    pub supported_languages: Vec<SupportedLanguageInfo>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TtsModelStatus {
    pub key: String,
    pub installed: bool,
    pub bytes_on_disk: u64,
    pub missing_files: Vec<String>,
    pub directory: String,
}

#[derive(Debug, Clone)]
pub struct ReadyTtsModel {
    pub key: String,
    pub path: PathBuf,
    pub voice_mode: TtsVoiceMode,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynthesizeTtsArgs {
    pub text: String,
    pub reference_audio_path: Option<String>,
    pub reference_text: Option<String>,
    pub model: Option<String>,
    pub voice_mode: Option<TtsVoiceMode>,
    pub speaker: Option<String>,
    pub instruction: Option<String>,
    pub auto_play: Option<bool>,
    pub volume: Option<f32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TtsCompletePayload {
    pub path: String,
    pub transcript: String,
    pub model: String,
    pub elapsed_ms: u128,
    pub auto_play: bool,
    pub volume: f32,
    pub audio_data_url: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TtsErrorPayload {
    pub message: String,
    pub model: String,
}

struct KokoroWorker {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

#[derive(Debug, Deserialize)]
struct KokoroWorkerMessage {
    ok: bool,
    #[serde(default)]
    ready: bool,
    #[serde(default)]
    error: Option<String>,
}

const QWEN3_TTS_BASE_FILES: [ModelFileDescriptor; 11] = [
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/resolve/main/config.json",
        name: "config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/resolve/main/generation_config.json",
        name: "generation_config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/resolve/main/merges.txt",
        name: "merges.txt",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/resolve/main/model.safetensors",
        name: "model.safetensors",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/resolve/main/preprocessor_config.json",
        name: "preprocessor_config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/resolve/main/speech_tokenizer/config.json",
        name: "speech_tokenizer/config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/resolve/main/speech_tokenizer/configuration.json",
        name: "speech_tokenizer/configuration.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/resolve/main/speech_tokenizer/model.safetensors",
        name: "speech_tokenizer/model.safetensors",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/resolve/main/speech_tokenizer/preprocessor_config.json",
        name: "speech_tokenizer/preprocessor_config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/resolve/main/tokenizer_config.json",
        name: "tokenizer_config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/resolve/main/vocab.json",
        name: "vocab.json",
    },
];

const QWEN3_TTS_CUSTOM_VOICE_FILES: [ModelFileDescriptor; 11] = [
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/resolve/main/config.json",
        name: "config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/resolve/main/generation_config.json",
        name: "generation_config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/resolve/main/merges.txt",
        name: "merges.txt",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/resolve/main/model.safetensors",
        name: "model.safetensors",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/resolve/main/preprocessor_config.json",
        name: "preprocessor_config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/resolve/main/speech_tokenizer/config.json",
        name: "speech_tokenizer/config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/resolve/main/speech_tokenizer/configuration.json",
        name: "speech_tokenizer/configuration.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/resolve/main/speech_tokenizer/model.safetensors",
        name: "speech_tokenizer/model.safetensors",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/resolve/main/speech_tokenizer/preprocessor_config.json",
        name: "speech_tokenizer/preprocessor_config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/resolve/main/tokenizer_config.json",
        name: "tokenizer_config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice/resolve/main/vocab.json",
        name: "vocab.json",
    },
];

const DEFAULT_KOKORO_VOICE: &str = "af_bella";

const KOKORO_82M_FILES: [ModelFileDescriptor; 4] = [
    ModelFileDescriptor {
        url: "https://huggingface.co/hexgrad/Kokoro-82M/resolve/main/config.json",
        name: "config.json",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/hexgrad/Kokoro-82M/resolve/main/kokoro-v1_0.pth",
        name: "kokoro-v1_0.pth",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/hexgrad/Kokoro-82M/resolve/main/voices/af_heart.pt",
        name: "voices/af_heart.pt",
    },
    ModelFileDescriptor {
        url: "https://huggingface.co/hexgrad/Kokoro-82M/resolve/main/voices/af_bella.pt",
        name: "voices/af_bella.pt",
    },
];

pub const TTS_MODEL_DEFINITIONS: &[TtsModelDefinition] = &[
    TtsModelDefinition {
        key: "kokoro_82m",
        label: "Kokoro 82M",
        description: "Tiny, fast local TTS tuned for low-latency everyday voice output.",
        repository: "hexgrad/Kokoro-82M",
        size_mb: 328.0,
        files: &KOKORO_82M_FILES,
        variant: "82M",
        tags: &["Fast", "Local", "CPU-friendly"],
        capabilities: &[TTS_CAPABILITY_FAST_LOCAL, TTS_CAPABILITY_CUSTOM_VOICE],
    },
    TtsModelDefinition {
        key: "qwen3_tts_0_6b_base",
        label: "Qwen3-TTS 0.6B Base",
        description: "Local voice-clone TTS from a short reference audio clip.",
        repository: "Qwen/Qwen3-TTS-12Hz-0.6B-Base",
        size_mb: 2402.0,
        files: &QWEN3_TTS_BASE_FILES,
        variant: "Base",
        tags: &["Voice clone", "Local", "10 languages"],
        capabilities: &[TTS_CAPABILITY_VOICE_CLONE],
    },
    TtsModelDefinition {
        key: "qwen3_tts_0_6b_custom_voice",
        label: "Qwen3-TTS 0.6B CustomVoice",
        description: "Local preset-speaker TTS with Qwen3 custom voice controls.",
        repository: "Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice",
        size_mb: 2385.0,
        files: &QWEN3_TTS_CUSTOM_VOICE_FILES,
        variant: "CustomVoice",
        tags: &["Preset voices", "Local", "10 languages"],
        capabilities: &[TTS_CAPABILITY_CUSTOM_VOICE],
    },
];

pub fn default_tts_model() -> String {
    "kokoro_82m".to_string()
}

pub fn definition(key: &str) -> Option<&'static TtsModelDefinition> {
    TTS_MODEL_DEFINITIONS.iter().find(|def| def.key == key)
}

pub fn model_supports_voice_mode(model_key: &str, mode: TtsVoiceMode) -> bool {
    definition(model_key)
        .map(|def| match mode {
            TtsVoiceMode::SourceAudio => def.capabilities.contains(&TTS_CAPABILITY_VOICE_CLONE),
            TtsVoiceMode::Preset => def.capabilities.contains(&TTS_CAPABILITY_CUSTOM_VOICE),
        })
        .unwrap_or(false)
}

pub fn get_tts_model_dir<R: Runtime>(app: &AppHandle<R>, key: &str) -> Result<PathBuf> {
    let mut dir = crate::app_paths::app_data_dir(app)?;
    dir.push(TTS_MODELS_ROOT);
    dir.push(key);
    Ok(dir)
}

fn push_unique_dir(seen: &mut HashSet<PathBuf>, dirs: &mut Vec<PathBuf>, dir: PathBuf) {
    if seen.insert(dir.clone()) {
        dirs.push(dir);
    }
}

fn push_data_root_candidate(
    seen: &mut HashSet<PathBuf>,
    dirs: &mut Vec<PathBuf>,
    root: PathBuf,
    key: &str,
) {
    push_unique_dir(seen, dirs, root.join(TTS_MODELS_ROOT).join(key));
}

fn tts_model_dir_candidates<R: Runtime>(app: &AppHandle<R>, key: &str) -> Result<Vec<PathBuf>> {
    let primary = get_tts_model_dir(app, key)?;
    let mut dirs = Vec::new();
    let mut seen = HashSet::new();

    push_unique_dir(&mut seen, &mut dirs, primary);

    if let Some(root) = std::env::var_os("FLOW_DATA_DIR").map(PathBuf::from) {
        push_data_root_candidate(&mut seen, &mut dirs, root, key);
    }

    #[cfg(target_os = "windows")]
    {
        for letter in b'D'..=b'Z' {
            let root = PathBuf::from(format!("{}:\\", letter as char));
            if root.exists() {
                push_data_root_candidate(&mut seen, &mut dirs, root.join("Flow").join("data"), key);
            }
        }
    }

    Ok(dirs)
}

fn ensure_tts_models_root<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
    let mut dir = crate::app_paths::app_data_dir(app)?;
    dir.push(TTS_MODELS_ROOT);
    fs::create_dir_all(&dir).context("Failed to prepare TTS models directory")?;
    Ok(dir)
}

fn qwen_languages() -> Vec<SupportedLanguageInfo> {
    [
        ("zh", "Chinese"),
        ("en", "English"),
        ("ja", "Japanese"),
        ("ko", "Korean"),
        ("de", "German"),
        ("fr", "French"),
        ("ru", "Russian"),
        ("pt", "Portuguese"),
        ("es", "Spanish"),
        ("it", "Italian"),
    ]
    .into_iter()
    .map(|(code, name)| SupportedLanguageInfo {
        code: code.to_string(),
        name: name.to_string(),
    })
    .collect()
}

#[tauri::command]
pub fn list_tts_models() -> Vec<TtsModelInfo> {
    TTS_MODEL_DEFINITIONS
        .iter()
        .map(|def| TtsModelInfo {
            key: def.key.to_string(),
            label: def.label.to_string(),
            description: def.description.to_string(),
            repository: def.repository.to_string(),
            size_mb: def.size_mb,
            file_count: def.files.len(),
            engine_id: if def.key == "kokoro_82m" {
                "kokoro".to_string()
            } else {
                "qwen_tts".to_string()
            },
            engine: if def.key == "kokoro_82m" {
                "Kokoro".to_string()
            } else {
                "Qwen3-TTS".to_string()
            },
            variant: def.variant.to_string(),
            tags: def.tags.iter().map(|s| s.to_string()).collect(),
            capabilities: def.capabilities.iter().map(|s| s.to_string()).collect(),
            supported_languages: qwen_languages(),
        })
        .collect()
}

#[tauri::command]
pub fn check_tts_model_status<R: Runtime>(
    app: AppHandle<R>,
    model: String,
) -> Result<TtsModelStatus, String> {
    let def = definition(&model).ok_or_else(|| "Unknown TTS model".to_string())?;
    let (_, status) =
        tts_status_from_candidates(&app, &model, def).map_err(|err| err.to_string())?;
    Ok(status)
}

#[tauri::command]
pub async fn download_tts_model(
    app: AppHandle<AppRuntime>,
    model: String,
) -> Result<TtsModelStatus, String> {
    download_tts_model_internal(app, model).await
}

pub async fn download_tts_model_internal(
    app: AppHandle<AppRuntime>,
    model: String,
) -> Result<TtsModelStatus, String> {
    let def = definition(&model).ok_or_else(|| "Unknown TTS model".to_string())?;
    ensure_tts_models_root(&app).map_err(|err| err.to_string())?;
    let dir = get_tts_model_dir(&app, &model).map_err(|err| err.to_string())?;
    if let Ok((_, status)) = tts_status_from_candidates(&app, &model, def) {
        if status.installed {
            return Ok(status);
        }
    }

    let client = app.state::<AppState>().http();
    let cancel_token = app.state::<AppState>().create_download_token(&model);
    let result = download_model_files(&app, &client, &model, def.files, &dir, &cancel_token).await;
    app.state::<AppState>().clear_download_token(&model);
    result.map_err(|err| err.to_string())?;

    Ok(TtsModelStatus::from_definition(&dir, def))
}

#[tauri::command]
pub fn delete_tts_model(
    app: AppHandle<AppRuntime>,
    model: String,
) -> Result<TtsModelStatus, String> {
    let def = definition(&model).ok_or_else(|| "Unknown TTS model".to_string())?;
    let dirs = tts_model_dir_candidates(&app, &model).map_err(|err| err.to_string())?;
    for dir in dirs {
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|err| err.to_string())?;
        }
    }
    let dir = get_tts_model_dir(&app, &model).map_err(|err| err.to_string())?;
    Ok(TtsModelStatus::from_definition(&dir, def))
}

fn tts_status_from_candidates<R: Runtime>(
    app: &AppHandle<R>,
    model: &str,
    def: &TtsModelDefinition,
) -> Result<(PathBuf, TtsModelStatus)> {
    let candidates = tts_model_dir_candidates(app, model)?;
    let primary = candidates
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("No TTS model directory candidates"))?;
    let primary_status = TtsModelStatus::from_definition(&primary, def);

    if primary_status.installed {
        return Ok((primary, primary_status));
    }

    for candidate in candidates.into_iter().skip(1) {
        let status = TtsModelStatus::from_definition(&candidate, def);
        if status.installed {
            return Ok((candidate, status));
        }
    }

    Ok((primary, primary_status))
}

impl TtsModelStatus {
    fn from_definition(dir: &Path, def: &TtsModelDefinition) -> Self {
        let missing_files = missing_files(dir, def);
        let installed = missing_files.is_empty() && dir.exists();
        let bytes_on_disk = if dir.exists() {
            calculate_dir_size(dir).unwrap_or(0)
        } else {
            0
        };

        Self {
            key: def.key.to_string(),
            installed,
            bytes_on_disk,
            missing_files,
            directory: dir.display().to_string(),
        }
    }
}

fn missing_files(dir: &Path, def: &TtsModelDefinition) -> Vec<String> {
    def.files
        .iter()
        .filter_map(|descriptor| {
            let file_path = dir.join(descriptor.name);
            let is_ready = file_path
                .metadata()
                .map(|metadata| metadata.is_file() && metadata.len() > 0)
                .unwrap_or(false);

            if is_ready {
                None
            } else {
                Some(descriptor.name.to_string())
            }
        })
        .collect()
}

fn calculate_dir_size(dir: &Path) -> Result<u64> {
    let mut total = 0u64;
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                total += calculate_dir_size(&entry.path())?;
            } else {
                total += metadata.len();
            }
        }
    }
    Ok(total)
}

pub fn ensure_tts_model_ready<R: Runtime>(
    app: &AppHandle<R>,
    model: &str,
    voice_mode: TtsVoiceMode,
) -> Result<ReadyTtsModel> {
    let def = definition(model).ok_or_else(|| anyhow!("Unknown TTS model"))?;
    if !model_supports_voice_mode(model, voice_mode) {
        return Err(anyhow!(
            "{} does not support the selected TTS voice mode",
            def.label
        ));
    }
    let (dir, status) = tts_status_from_candidates(app, model, def)?;
    if !status.installed {
        return Err(anyhow!(
            "{} is not fully installed. Missing: {}",
            def.label,
            status.missing_files.join(", ")
        ));
    }
    Ok(ReadyTtsModel {
        key: def.key.to_string(),
        path: dir,
        voice_mode,
    })
}

pub fn queue_after_transcription(
    app: &AppHandle<AppRuntime>,
    transcript: String,
    reference_audio_path: Option<PathBuf>,
    reference_text: Option<String>,
    settings: UserSettings,
) {
    if !settings.tts_enabled || !settings.tts_auto_after_stt || transcript.trim().is_empty() {
        return;
    }
    if matches!(settings.tts_voice_mode, TtsVoiceMode::SourceAudio)
        && reference_audio_path.is_none()
    {
        eprintln!("[TTS] Skipping source-audio voice clone because no reference audio was saved");
        return;
    }

    let app_handle = app.clone();
    async_runtime::spawn(async move {
        match synthesize_with_settings(
            &app_handle,
            transcript,
            reference_audio_path,
            reference_text,
            &settings,
        )
        .await
        {
            Ok(payload) => {
                let _ = app_handle.emit(TTS_EVENT_COMPLETE, payload.clone());
                toast::show(
                    &app_handle,
                    "success",
                    Some("Voice output ready"),
                    &format!("Generated with {}.", payload.model),
                );
            }
            Err(err) => {
                let _ = app_handle.emit(
                    TTS_EVENT_ERROR,
                    TtsErrorPayload {
                        message: err.to_string(),
                        model: settings.tts_model.clone(),
                    },
                );
                toast::show(&app_handle, "error", Some("TTS failed"), &format!("{err}"));
            }
        }
    });
}

pub fn prewarm_if_needed(app: &AppHandle<AppRuntime>, settings: &UserSettings) {
    if settings.tts_model != "kokoro_82m" {
        return;
    }

    let def = match definition("kokoro_82m") {
        Some(def) => def,
        None => return,
    };

    match tts_status_from_candidates(app, "kokoro_82m", def) {
        Ok((_, status)) if status.installed => {}
        _ => return,
    }

    if KOKORO_PREWARMING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    let app_handle = app.clone();
    async_runtime::spawn_blocking(move || {
        let started = Instant::now();
        if let Err(err) = ensure_kokoro_worker(&app_handle) {
            eprintln!("[TTS] Kokoro prewarm failed: {err}");
        } else {
            let elapsed_ms = started.elapsed().as_millis();
            eprintln!("[TTS] Kokoro worker ready in {elapsed_ms}ms");
            let _ = app_handle.emit(
                "tts:warmed",
                serde_json::json!({ "model": "kokoro_82m", "elapsed_ms": elapsed_ms }),
            );
        }
        KOKORO_PREWARMING.store(false, Ordering::SeqCst);
    });
}

#[tauri::command]
pub async fn synthesize_tts(
    app: AppHandle<AppRuntime>,
    args: SynthesizeTtsArgs,
) -> Result<TtsCompletePayload, String> {
    let settings = app.state::<AppState>().current_settings();
    let mut settings = settings;
    if let Some(model) = args.model {
        settings.tts_model = model;
    }
    if let Some(voice_mode) = args.voice_mode {
        settings.tts_voice_mode = voice_mode;
    }
    if let Some(speaker) = args.speaker {
        settings.tts_speaker = speaker;
    }
    if let Some(instruction) = args.instruction {
        settings.tts_instruction = instruction;
    }
    if let Some(auto_play) = args.auto_play {
        settings.tts_auto_play = auto_play;
    }
    if let Some(volume) = args.volume {
        settings.tts_volume = crate::settings::clamp_tts_volume(volume);
    }
    let reference_audio_path = args.reference_audio_path.map(PathBuf::from);
    synthesize_with_settings(
        &app,
        args.text,
        reference_audio_path,
        args.reference_text,
        &settings,
    )
    .await
    .map_err(|err| err.to_string())
}

async fn synthesize_with_settings(
    app: &AppHandle<AppRuntime>,
    transcript: String,
    reference_audio_path: Option<PathBuf>,
    reference_text: Option<String>,
    settings: &UserSettings,
) -> Result<TtsCompletePayload> {
    let ready = ensure_tts_model_ready(app, &settings.tts_model, settings.tts_voice_mode)?;
    let started = Instant::now();
    let output_path = next_output_path(app)?;
    let output_path_for_runner = output_path.clone();
    let runner = ensure_runner_script(app)?;
    let python = resolve_python(app)?;
    let ready_key = ready.key.clone();
    let ready_path = ready.path.clone();
    let language = qwen_language_name(&settings.language);
    let speaker = settings.tts_speaker.trim().to_string();
    let instruct = settings.tts_instruction.trim().to_string();
    let auto_play = settings.tts_auto_play;
    let volume = crate::settings::clamp_tts_volume(settings.tts_volume);

    let text = transcript.trim().to_string();
    if text.is_empty() {
        return Err(anyhow!("No transcript text to synthesize"));
    }

    let reference_audio = reference_audio_path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let reference_text = reference_text.unwrap_or_default();
    let model_kind = if ready.key == "kokoro_82m" {
        "kokoro"
    } else {
        match ready.voice_mode {
            TtsVoiceMode::SourceAudio => "voice_clone",
            TtsVoiceMode::Preset => "custom_voice",
        }
    };

    let app_data = crate::app_paths::app_data_dir(app)?;
    let hf_home = app_data.join("huggingface");
    let torch_home = app_data.join("torch");
    fs::create_dir_all(&hf_home)?;
    fs::create_dir_all(&torch_home)?;

    if ready_key == "kokoro_82m" {
        let app_for_worker = app.clone();
        let output_for_worker = output_path_for_runner.clone();
        async_runtime::spawn_blocking(move || {
            ensure_kokoro_worker(&app_for_worker)?;
            synthesize_kokoro_with_worker(&text, &output_for_worker, &speaker)
        })
        .await
        .map_err(|err| anyhow!("TTS task failed: {err}"))??;
    } else {
        let runner_result = async_runtime::spawn_blocking(move || {
        let mut command = Command::new(python);
        command
            .arg(runner)
            .arg("--model-dir")
            .arg(&ready_path)
            .arg("--model-kind")
            .arg(model_kind)
            .arg("--text")
            .arg(&text)
            .arg("--output")
            .arg(&output_path_for_runner)
            .arg("--language")
            .arg(language);
        apply_tts_process_env(&mut command, &hf_home, &torch_home);

        if !reference_audio.is_empty() {
            command.arg("--reference-audio").arg(reference_audio);
        }
        if !reference_text.trim().is_empty() {
            command.arg("--reference-text").arg(reference_text);
        }
        if !speaker.is_empty() {
            command.arg("--speaker").arg(speaker);
        }
        if !instruct.is_empty() {
            command.arg("--instruct").arg(instruct);
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS);
        }

        let output = command.output().context("Failed to start local TTS runner")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() {
                stdout.trim().to_string()
            } else {
                stderr.trim().to_string()
            };
            return Err(anyhow!(
                "Local TTS runner failed. Install Kokoro with scripts/setup-kokoro-tts-runtime.ps1 or Qwen with scripts/setup-qwen3-tts-runtime.ps1. {detail}"
            ));
        }
        Ok::<_, anyhow::Error>(())
        })
        .await
        .map_err(|err| anyhow!("TTS task failed: {err}"))?;
        runner_result?;
    }

    let path = output_path.display().to_string();
    let audio_data_url = audio_data_url(&output_path).ok();
    Ok(TtsCompletePayload {
        path,
        transcript,
        model: ready_key,
        elapsed_ms: started.elapsed().as_millis(),
        auto_play,
        volume,
        audio_data_url,
    })
}

fn audio_data_url(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:audio/wav;base64,{encoded}"))
}

fn apply_tts_process_env(command: &mut Command, hf_home: &Path, torch_home: &Path) {
    command
        .env("PYTHONUTF8", "1")
        .env("PYTHONNOUSERSITE", "1")
        .env("HF_HOME", hf_home)
        .env("HF_HUB_DISABLE_TELEMETRY", "1")
        .env("TRANSFORMERS_CACHE", hf_home.join("transformers"))
        .env("TORCH_HOME", torch_home)
        .env("TOKENIZERS_PARALLELISM", "false")
        .env("OMP_NUM_THREADS", TTS_CPU_THREADS)
        .env("MKL_NUM_THREADS", TTS_CPU_THREADS)
        .env("NUMEXPR_NUM_THREADS", TTS_CPU_THREADS)
        .env("FLOW_TTS_TORCH_THREADS", TTS_CPU_THREADS);
}

fn ensure_runner_script<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
    let mut dir = crate::app_paths::app_data_dir(app)?;
    dir.push("tools");
    fs::create_dir_all(&dir)?;
    let path = dir.join(RUNNER_SCRIPT_NAME);
    let should_write = fs::read_to_string(&path)
        .map(|existing| existing != RUNNER_SCRIPT)
        .unwrap_or(true);
    if should_write {
        fs::write(&path, RUNNER_SCRIPT)?;
    }
    Ok(path)
}

fn kokoro_worker_cell() -> &'static Mutex<Option<KokoroWorker>> {
    KOKORO_WORKER.get_or_init(|| Mutex::new(None))
}

fn ensure_kokoro_worker(app: &AppHandle<AppRuntime>) -> Result<()> {
    let mut guard = kokoro_worker_cell()
        .lock()
        .map_err(|_| anyhow!("Kokoro worker lock was poisoned"))?;
    if let Some(worker) = guard.as_mut() {
        if worker.child.try_wait()?.is_none() {
            return Ok(());
        }
        *guard = None;
    }

    let worker = start_kokoro_worker(app)?;
    *guard = Some(worker);
    Ok(())
}

fn start_kokoro_worker(app: &AppHandle<AppRuntime>) -> Result<KokoroWorker> {
    let runner = ensure_runner_script(app)?;
    let python = resolve_python(app)?;
    let def = definition("kokoro_82m").ok_or_else(|| anyhow!("Kokoro TTS is not registered"))?;
    let (model_dir, status) = tts_status_from_candidates(app, "kokoro_82m", def)?;
    if !status.installed {
        return Err(anyhow!(
            "Kokoro is not fully installed. Missing: {}",
            status.missing_files.join(", ")
        ));
    }
    let app_data = crate::app_paths::app_data_dir(app)?;
    let hf_home = app_data.join("huggingface");
    let torch_home = app_data.join("torch");
    fs::create_dir_all(&hf_home)?;
    fs::create_dir_all(&torch_home)?;

    let mut command = Command::new(python);
    command
        .arg(runner)
        .arg("--model-kind")
        .arg("kokoro")
        .arg("--model-dir")
        .arg(&model_dir)
        .arg("--server")
        .arg("--language")
        .arg("English")
        .arg("--speaker")
        .arg(DEFAULT_KOKORO_VOICE)
        .arg("--device")
        .arg("cpu")
        .env("HF_HUB_OFFLINE", "1")
        .env("TRANSFORMERS_OFFLINE", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    apply_tts_process_env(&mut command, &hf_home, &torch_home);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS);
    }

    let mut child = command.spawn().context("Failed to start Kokoro worker")?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("Kokoro worker stdin was unavailable"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Kokoro worker stdout was unavailable"))?;
    let mut worker = KokoroWorker {
        child,
        stdin,
        stdout: BufReader::new(stdout),
    };

    loop {
        let message = read_kokoro_message(&mut worker.stdout)?;
        if message.ready && message.ok {
            return Ok(worker);
        }
        if !message.ok {
            return Err(anyhow!(
                "Kokoro worker failed to start: {}",
                message.error.unwrap_or_else(|| "unknown error".to_string())
            ));
        }
    }
}

fn synthesize_kokoro_with_worker(text: &str, output_path: &Path, speaker: &str) -> Result<()> {
    let request = serde_json::json!({
        "text": text,
        "output": output_path,
        "speaker": if speaker.trim().is_empty() { DEFAULT_KOKORO_VOICE } else { speaker.trim() },
    });

    let mut guard = kokoro_worker_cell()
        .lock()
        .map_err(|_| anyhow!("Kokoro worker lock was poisoned"))?;
    let worker = guard
        .as_mut()
        .ok_or_else(|| anyhow!("Kokoro worker is not ready"))?;

    writeln!(worker.stdin, "{request}")?;
    worker.stdin.flush()?;

    loop {
        let message = read_kokoro_message(&mut worker.stdout)?;
        if message.ok {
            return Ok(());
        }
        return Err(anyhow!(
            "Kokoro generation failed: {}",
            message.error.unwrap_or_else(|| "unknown error".to_string())
        ));
    }
}

fn read_kokoro_message(stdout: &mut BufReader<ChildStdout>) -> Result<KokoroWorkerMessage> {
    let mut line = String::new();
    loop {
        line.clear();
        let bytes = stdout.read_line(&mut line)?;
        if bytes == 0 {
            return Err(anyhow!("Kokoro worker closed stdout"));
        }
        if let Ok(message) = serde_json::from_str::<KokoroWorkerMessage>(line.trim()) {
            return Ok(message);
        }
    }
}

fn resolve_python<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("FLOW_TTS_PYTHON").map(PathBuf::from) {
        if path.exists() {
            return Ok(path);
        }
    }

    let app_data = crate::app_paths::app_data_dir(app)?;
    let flow_root = app_data.parent().map(Path::to_path_buf);
    let mut runtime_roots = Vec::new();
    if let Some(root) = flow_root {
        runtime_roots.push(root.join("runtime").join("kokoro-tts").join(".venv"));
        runtime_roots.push(root.join("runtime").join("tts-bench").join(".venv"));
        runtime_roots.push(root.join("runtime").join("qwen-tts").join(".venv"));
    }
    runtime_roots.push(app_data.join("runtime").join("kokoro-tts").join(".venv"));
    runtime_roots.push(app_data.join("runtime").join("qwen-tts").join(".venv"));

    for root in runtime_roots {
        let python = root.join(if cfg!(target_os = "windows") {
            "Scripts/python.exe"
        } else {
            "bin/python"
        });
        if python.exists() {
            return Ok(python);
        }
    }

    Ok(PathBuf::from("python"))
}

fn next_output_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
    let mut dir = crate::app_paths::app_data_dir(app)?;
    dir.push(TTS_OUTPUT_ROOT);
    fs::create_dir_all(&dir)?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S%.3f");
    Ok(dir.join(format!("flow-tts-{stamp}.wav")))
}

fn qwen_language_name(language: &str) -> String {
    match language.to_ascii_lowercase().as_str() {
        "zh" | "zh-cn" | "zh-hans" => "Chinese",
        "ja" => "Japanese",
        "ko" => "Korean",
        "de" => "German",
        "fr" => "French",
        "ru" => "Russian",
        "pt" | "pt-br" | "pt-pt" => "Portuguese",
        "es" => "Spanish",
        "it" => "Italian",
        _ => "English",
    }
    .to_string()
}
