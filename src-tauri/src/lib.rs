mod accessibility_context;
mod analytics;
mod app_paths;
mod assistive;
mod audio;
mod context_formatting;
mod core;
mod crypto;
mod dictionary;
mod downloader;
mod flow_fetch;
mod friday_chat;
mod insights;
mod library;
mod llm_cleanup;
mod local_text_model;
mod local_transcription;
mod mode_context;
mod model_language_table;
mod model_manager;
mod music;
mod ocr;
mod permissions;
mod personalization;
mod pill;
mod platform;
mod recent_transcriptions;
mod recorder;
mod scratchpad;
mod settings;
mod snippets;
mod storage;
mod streaming_transcription;
mod toast;
mod transcribe;
mod transcription_api;
mod transforms;
mod tray;
mod tts;
mod update_checker;
mod vibe_coding;
mod voice_commands;
mod wake;
mod wake_speaker;

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use anyhow::{Context, Result};
use chrono::{DateTime, Days, Local, Months};
use pill::PillController;
use recorder::{
    validate_recording, CompletedRecording, RecorderManager, RecordingRejectionReason,
    RecordingSaved,
};
use reqwest::Client;
use serde::Serialize;
use settings::{
    default_local_model, LlmProvider, LocalDataStoragePolicy, RecordingPrunePolicy, SettingsStore,
    TranscriptionMode, UserSettings,
};
use tauri::async_runtime;
use tauri::tray::TrayIcon;
use tauri::Emitter;
#[cfg(target_os = "macos")]
use tauri::Listener;
use tauri::{AppHandle, Manager, Wry};

#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as AutostartManagerExt};
use tauri_plugin_opener::OpenerExt;

pub(crate) const MAIN_WINDOW_LABEL: &str = "main";
pub(crate) const SETTINGS_WINDOW_LABEL: &str = "settings";
pub(crate) const EVENT_RECORDING_START: &str = "recording:start";
pub(crate) const EVENT_AUDIO_SPECTRUM: &str = "audio:spectrum";
pub(crate) const EVENT_TRANSCRIPTION_COMPLETE: &str = "transcription:complete";
pub(crate) const EVENT_TRANSCRIPTION_ERROR: &str = "transcription:error";
pub(crate) const EVENT_SCRATCHPAD_CHANGED: &str = "scratchpad:changed";
pub(crate) const EVENT_SCRATCHPAD_ENTRY_CREATED: &str = "scratchpad:entry-created";
pub(crate) const EVENT_NAVIGATE_SCRATCHPAD: &str = "navigate:scratchpad";
pub(crate) const EVENT_SNIPPETS_CHANGED: &str = "snippets:changed";
pub(crate) const EVENT_FLOW_FETCH_CHANGED: &str = "flow-fetch:changed";
pub(crate) const EVENT_FLOW_FETCH_LINK_CAPTURED: &str = "flow-fetch:link-captured";
pub(crate) const EVENT_SETTINGS_CHANGED: &str = "settings:changed";
pub(crate) const FEEDBACK_URL: &str =
    "https://github.com/essencefromexistence/flow/issues/new/choose";
#[cfg(target_os = "windows")]
pub(crate) const FFMPEG_HELP_URL: &str =
    "https://github.com/essencefromexistence/flow/wiki/ffmpeg-windows";
#[cfg(not(target_os = "windows"))]
pub(crate) const FFMPEG_HELP_URL: &str =
    "https://github.com/essencefromexistence/flow/wiki/ffmpeg-mac";

fn launched_via_autostart() -> bool {
    std::env::args_os().any(|arg| arg == "--autostart")
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn sync_launch_at_login(
    app: &AppHandle<AppRuntime>,
    enabled: bool,
) -> Result<(), String> {
    let autostart = app.autolaunch();
    let currently_enabled = autostart
        .is_enabled()
        .map_err(|err| format!("Failed to read launch at login status: {err}"))?;

    if currently_enabled == enabled {
        return Ok(());
    }

    if enabled {
        autostart
            .enable()
            .map_err(|err| format!("Failed to enable launch at login: {err}"))?;
    } else {
        autostart
            .disable()
            .map_err(|err| format!("Failed to disable launch at login: {err}"))?;
    }

    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn sync_launch_at_login(
    _app: &AppHandle<AppRuntime>,
    _enabled: bool,
) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn handle_app_menu_event(app: &AppHandle<AppRuntime>, id: &str) {
    use crate::recent_transcriptions::{
        copy_transcription_to_clipboard, paste_latest_transcription_from_menu,
        MENU_ID_RECENT_TRANSCRIPTION_PREFIX,
    };
    use platform::macos::menu::{
        MENU_ID_CHECK_UPDATES, MENU_ID_MIC_DEFAULT, MENU_ID_MIC_PREFIX, MENU_ID_MODEL_PREFIX,
        MENU_ID_MODE_LOCAL, MENU_ID_PASTE_LAST_TRANSCRIPT, MENU_ID_REPORT_ISSUE, MENU_ID_WEBSITE,
    };
    use tauri_plugin_opener::OpenerExt;

    match id {
        MENU_ID_CHECK_UPDATES => {
            let _ = app.emit("navigate:about", ());
        }
        MENU_ID_WEBSITE => {
            let _ = app
                .opener()
                .open_url("https://github.com/essencefromexistence/flow", None::<&str>);
        }
        MENU_ID_REPORT_ISSUE => {
            let _ = app.opener().open_url(FEEDBACK_URL, None::<&str>);
        }
        MENU_ID_PASTE_LAST_TRANSCRIPT => paste_latest_transcription_from_menu(app),
        MENU_ID_MODE_LOCAL => {
            set_transcription_mode(app, settings::TranscriptionMode::Local);
        }
        MENU_ID_MIC_DEFAULT => {
            set_microphone(app, None);
        }
        _ => {
            if let Some(transcription_id) = id.strip_prefix(MENU_ID_RECENT_TRANSCRIPTION_PREFIX) {
                copy_transcription_to_clipboard(app, transcription_id);
            } else if let Some(model_key) = id.strip_prefix(MENU_ID_MODEL_PREFIX) {
                set_local_model(app, model_key);
            } else if let Some(device_id_raw) = id.strip_prefix(MENU_ID_MIC_PREFIX) {
                let device_id = device_id_raw.strip_prefix("dev:").unwrap_or(device_id_raw);
                set_microphone(app, Some(device_id));
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn set_transcription_mode(app: &AppHandle<AppRuntime>, mode: settings::TranscriptionMode) {
    let state = app.state::<AppState>();
    let mut current = state.current_settings();
    if current.transcription_mode == mode {
        return;
    }
    current.transcription_mode = mode;
    match state.persist_settings(current.clone()) {
        Ok(saved) => {
            state.request_preflight_refresh();
            if let Err(err) = set_app_menu(app, &saved) {
                eprintln!("Failed to refresh app menu: {err}");
            }
            if let Err(err) = tray::refresh_tray_menu(app, &saved) {
                eprintln!("Failed to refresh tray menu: {err}");
            }
            if let Err(err) = app.emit(EVENT_SETTINGS_CHANGED, &saved) {
                eprintln!("Failed to emit settings change: {err}");
            }
        }
        Err(err) => eprintln!("Failed to update transcription mode: {err}"),
    }
}

#[cfg(target_os = "macos")]
fn set_local_model(app: &AppHandle<AppRuntime>, model_key: &str) {
    if model_manager::definition(model_key).is_none() {
        eprintln!("Ignoring unknown model selection: {model_key}");
        return;
    }

    match model_manager::check_model_status(app.clone(), model_key.to_string()) {
        Ok(status) if status.installed => {}
        Ok(_) => {
            eprintln!("Model not installed: {model_key}");
            return;
        }
        Err(err) => {
            eprintln!("Failed to check model status for {model_key}: {err}");
            return;
        }
    }

    let state = app.state::<AppState>();
    let mut current = state.current_settings();
    if current.local_model == model_key {
        return;
    }
    current.local_model = model_key.to_string();
    match state.persist_settings(current.clone()) {
        Ok(saved) => {
            if let Err(err) = set_app_menu(app, &saved) {
                eprintln!("Failed to refresh app menu: {err}");
            }
            if let Err(err) = tray::refresh_tray_menu(app, &saved) {
                eprintln!("Failed to refresh tray menu: {err}");
            }
            if let Err(err) = app.emit(EVENT_SETTINGS_CHANGED, &saved) {
                eprintln!("Failed to emit settings change: {err}");
            }
        }
        Err(err) => eprintln!("Failed to update model selection: {err}"),
    }
}

#[cfg(target_os = "macos")]
fn set_microphone(app: &AppHandle<AppRuntime>, device_id: Option<&str>) {
    let state = app.state::<AppState>();
    let mut current = state.current_settings();
    if current.microphone_device.as_deref() == device_id {
        return;
    }
    current.microphone_device = device_id.map(|id| id.to_string());
    match state.persist_settings(current.clone()) {
        Ok(saved) => {
            if let Err(err) = set_app_menu(app, &saved) {
                eprintln!("Failed to refresh app menu: {err}");
            }
            if let Err(err) = tray::refresh_tray_menu(app, &saved) {
                eprintln!("Failed to refresh tray menu: {err}");
            }
            if let Err(err) = app.emit(EVENT_SETTINGS_CHANGED, &saved) {
                eprintln!("Failed to emit settings change: {err}");
            }
        }
        Err(err) => eprintln!("Failed to update microphone selection: {err}"),
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn set_app_menu(
    app: &AppHandle<AppRuntime>,
    settings: &settings::UserSettings,
) -> tauri::Result<()> {
    let menu = platform::macos::menu::build_app_menu(app, settings)?;
    app.set_menu(menu)?;
    Ok(())
}

pub fn run() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let _guard = rt.enter();
    tauri::async_runtime::set(rt.handle().clone());

    let builder = tauri::Builder::default();

    #[cfg(target_os = "windows")]
    let builder = builder.device_event_filter(tauri::DeviceEventFilter::Always);

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|_, _, _| {}));

    let builder = builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let builder = builder.plugin(tauri_plugin_autostart::init(
        MacosLauncher::LaunchAgent,
        Some(vec!["--autostart"]),
    ));

    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_plugin_macos_permissions::init());

    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    #[cfg(target_os = "macos")]
    let builder = builder.on_menu_event(|app, event| {
        let id = event.id().as_ref();
        handle_app_menu_event(app, id);
    });

    builder
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            let handle = app.handle();
            let settings_store = Arc::new(SettingsStore::new(handle)?);
            let mut settings = settings_store.load().unwrap_or_default();
            if model_manager::definition(&settings.local_model).is_none() {
                settings.local_model = default_local_model();
                if let Err(err) = settings_store.save(&settings) {
                    eprintln!("Failed to persist default local model: {err}");
                }
            }

            app.manage(AppState::new(Arc::clone(&settings_store), settings, handle));
            {
                let state = handle.state::<AppState>();
                let settings = state.current_settings();
                tts::prewarm_if_needed(handle, &settings);
                state.download_default_local_model_if_missing(handle, &settings, "startup");
                state.preload_local_model_if_needed(handle, &settings, "startup");
                llm_cleanup::prewarm_if_needed(handle, &settings);
                schedule_local_data_prune(handle.clone(), settings);
            }
            library::commands::recover_interrupted_library_items(handle);

            #[cfg(target_os = "macos")]
            {
                let h = handle.clone();
                handle.listen(library::EVENT_LIBRARY_RENDERER_READY, move |_| {
                    library::commands::mark_library_import_renderer_ready(&h);
                });
            }

            {
                let handle = app.handle();
                let settings = handle.state::<AppState>().current_settings();
                if let Err(err) = sync_launch_at_login(handle, settings.auto_launch_enabled) {
                    eprintln!("Failed to sync launch at login state: {err}");
                }
            }

            #[cfg(target_os = "macos")]
            {
                let handle = app.handle();
                let settings = handle.state::<AppState>().current_settings();
                if let Err(err) = set_app_menu(handle, &settings) {
                    eprintln!("Failed to set app menu: {err}");
                }
                if let Err(err) = platform::macos::audio_devices::init(handle) {
                    eprintln!("Failed to initialize input device watcher: {err}");
                }
            }

            if let Some(window) = handle.get_webview_window(MAIN_WINDOW_LABEL) {
                let _ = window.hide();
                platform::overlay::init(handle, &window);
            }

            if let Some(toast_window) = handle.get_webview_window(toast::WINDOW_LABEL) {
                let _ = toast_window.hide();
                platform::toast::init(handle, &toast_window);
            }

            if let Some(settings_window) = handle.get_webview_window(SETTINGS_WINDOW_LABEL) {
                platform::settings_window::init(&settings_window);
            }

            if let Ok(tray) = tray::build_tray(handle) {
                handle.state::<AppState>().store_tray(tray);
            }

            flow_fetch::start_monitor(handle.clone());

            if let Err(err) = pill::register_shortcuts(handle) {
                eprintln!("Failed to register shortcuts: {err}");
            }
            {
                let state = handle.state::<AppState>();
                let settings = state.current_settings();
                if let Err(err) = state.wake.sync(handle, &settings) {
                    eprintln!("Failed to sync wake listener: {err}");
                }
            }
            pill::show_overlay(handle);

            if !launched_via_autostart() {
                let h = handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(300));
                    let _ = tray::toggle_settings_window(&h);
                });
            }

            update_checker::check_post_auto_update(handle);

            let update_handle = handle.clone();
            let update_state = handle.state::<AppState>().update_state().clone();
            update_checker::start_background_checker(update_handle, update_state);

            handle
                .state::<AppState>()
                .start_preflight_loop(handle.clone());

            {
                let h = handle.clone();
                tauri::async_runtime::spawn(async move {
                    analytics::init(&h).await;
                    analytics::track_app_started(&h);
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_shortcut_capture_active,
            update_settings,
            set_auto_transform_setting,
            preview_recording_prune,
            preview_local_data_prune,
            set_user_name,
            dictionary::set_dictionary,
            dictionary::add_dictionary_entries,
            dictionary::get_replacements,
            dictionary::set_replacements,
            personalization::get_personalities,
            personalization::set_personalities,
            personalization::list_installed_apps,
            personalization::list_website_icons,
            personalization::get_active_style_preview,
            get_app_info,
            open_data_dir,
            get_transcriptions,
            set_transcription_pinned,
            export_transcriptions_to_path,
            delete_transcription,
            delete_all_transcriptions,
            retry_transcription,
            retry_llm_cleanup,
            undo_llm_cleanup,
            cancel_retry_transcription,
            scratchpad::list_scratchpad_entries,
            scratchpad::list_scratchpad_versions,
            scratchpad::create_scratchpad_entry,
            scratchpad::update_scratchpad_entry,
            scratchpad::delete_scratchpad_entry,
            snippets::list_snippets,
            snippets::create_snippet,
            snippets::update_snippet,
            snippets::delete_snippet,
            snippets::open_snippets_view,
            flow_fetch::list_flow_fetch_links,
            flow_fetch::delete_flow_fetch_link,
            flow_fetch::copy_flow_fetch_link,
            insights::get_insights,
            transforms::get_transform_presets,
            transforms::get_transform_source,
            transforms::list_transform_history,
            transforms::delete_transform_history_entry,
            transforms::open_transforms_view,
            transforms::paste_transform_result,
            transforms::transform_text,
            library::commands::create_library_item,
            library::commands::get_library_items_page,
            library::commands::update_library_item,
            library::commands::delete_library_item,
            library::commands::cancel_library_transcription,
            library::commands::retry_library_transcription,
            library::commands::export_library_item_to_path,
            library::commands::get_library_tags,
            model_manager::list_models,
            model_manager::check_model_status,
            get_local_model_runtime_status,
            friday_chat::friday_local_chat,
            friday_chat::friday_local_agent_run,
            friday_chat::friday_local_research,
            model_manager::download_model,
            model_manager::delete_model,
            model_manager::cancel_download,
            ocr::get_ocr_status,
            ocr::run_ocr_image,
            tts::list_tts_models,
            tts::check_tts_model_status,
            tts::download_tts_model,
            tts::delete_tts_model,
            tts::synthesize_tts,
            wake_speaker::enroll_wake_speaker_profile,
            wake_speaker::clear_wake_speaker_profile,
            audio::list_input_devices,
            toast::toast_dismissed,
            open_accessibility_settings,
            check_accessibility_permission,
            get_auto_paste_status,
            paste_text_to_focused_app,
            paste_last_transcript,
            copy_last_transcript,
            check_microphone_permission,
            request_microphone_permission,
            open_microphone_settings,
            open_input_monitoring_settings,
            open_llm_cleanup_settings,
            open_app_settings,
            open_ffmpeg_install,
            complete_onboarding,
            cancel_recording,
            pause_flow_temporarily,
            toggle_recording,
            pill::set_pill_expanded,
            pill::set_pill_overlay_tip_frame,
            reset_onboarding,
            toast::debug_show_toast,
            fetch_llm_models,
            open_whats_new,
            open_about_page,
            update_checker::get_update_status,
            update_checker::check_for_updates,
            update_checker::download_and_install_update
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|handler, event| match event {
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Opened { urls } => {
                let paths = urls
                    .into_iter()
                    .filter_map(|url| url.to_file_path().ok())
                    .collect();
                if let Err(err) = library::handle_opened_paths(handler, paths) {
                    eprintln!("Failed to handle opened files: {err}");
                }
            }
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen {
                has_visible_windows,
                ..
            } => {
                if !has_visible_windows {
                    let _ = tray::toggle_settings_window(handler);
                }
            }
            tauri::RunEvent::Exit => {
                let state = handler.state::<AppState>();
                state.local_transcriber.unload();
                state.stop_preflight_loop();
                let now = Instant::now();
                let counters = state.session_counters.lock();
                analytics::track_app_exited(
                    handler,
                    (now - state.session_started_at).as_secs_f64(),
                    counters.transcription_count,
                );
            }
            _ => {}
        });
}

pub(crate) type AppRuntime = Wry;

type FlowResult<T> = Result<T>;

#[derive(Clone)]
pub struct LibraryJob {
    pub id: String,
    pub kind: LibraryJobKind,
}

#[derive(Clone)]
pub enum LibraryJobKind {
    Import {
        source_path: PathBuf,
        store_original: bool,
    },
    TranscribeExisting,
}

pub struct AppState {
    pill: Arc<PillController>,
    http: Client,
    pub(crate) local_transcriber: Arc<local_transcription::LocalTranscriber>,
    storage: Arc<storage::StorageManager>,
    settings_store: Arc<SettingsStore>,
    settings: parking_lot::Mutex<UserSettings>,
    hotkeys: core::hotkeys::HotkeyCoordinator,
    wake: wake::WakeCoordinator,
    shortcut_capture_active: AtomicBool,
    pub(crate) tray: parking_lot::Mutex<Option<TrayIcon<AppRuntime>>>,
    pub(crate) settings_close_handler_registered: AtomicBool,
    transcription_cancelled: AtomicBool,
    transcription_token: parking_lot::Mutex<Option<CancellationToken>>,
    ffmpeg_toast_shown: AtomicBool,
    pending_recording_path: parking_lot::Mutex<Option<PathBuf>>,
    pending_selected_text: parking_lot::Mutex<Option<String>>,
    pending_command_mode: AtomicBool,
    download_tokens: parking_lot::Mutex<HashMap<String, CancellationToken>>,
    library_tokens: parking_lot::Mutex<HashMap<String, CancellationToken>>,
    library_queue: parking_lot::Mutex<VecDeque<LibraryJob>>,
    library_active: parking_lot::Mutex<Option<String>>,
    retry_tokens: parking_lot::Mutex<HashMap<String, CancellationToken>>,
    preload_models: parking_lot::Mutex<HashSet<String>>,
    update_state: update_checker::SharedUpdateState,
    auto_update_completed: AtomicBool,
    preflight_cancel: CancellationToken,
    preflight_started: AtomicBool,
    preflight_notify: Arc<Notify>,
    session_started_at: Instant,
    session_counters: parking_lot::Mutex<SessionCounters>,
    streaming_session: parking_lot::Mutex<Option<streaming_transcription::StreamingSession>>,
}

#[derive(Clone, Copy)]
struct SessionCounters {
    transcription_count: u32,
}

impl AppState {
    pub fn new(
        settings_store: Arc<SettingsStore>,
        settings: UserSettings,
        app_handle: &AppHandle<AppRuntime>,
    ) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client");

        let storage_path = app_paths::app_data_dir(app_handle)
            .expect("Failed to resolve app data directory")
            .join("transcriptions.db");

        let storage = storage::StorageManager::new(storage_path)
            .expect("Failed to initialize transcription storage");

        let recorder = Arc::new(RecorderManager::new());

        let local_transcriber = Arc::new(local_transcription::LocalTranscriber::new());
        local_transcriber.start_idle_monitor();

        Self {
            pill: Arc::new(PillController::new(Arc::clone(&recorder))),
            http,
            local_transcriber,
            storage: Arc::new(storage),
            settings_store,
            settings: parking_lot::Mutex::new(settings),
            hotkeys: core::hotkeys::HotkeyCoordinator::default(),
            wake: wake::WakeCoordinator::default(),
            shortcut_capture_active: AtomicBool::new(false),
            tray: parking_lot::Mutex::new(None),
            settings_close_handler_registered: AtomicBool::new(false),
            transcription_cancelled: AtomicBool::new(false),
            transcription_token: parking_lot::Mutex::new(None),
            ffmpeg_toast_shown: AtomicBool::new(false),
            pending_recording_path: parking_lot::Mutex::new(None),
            pending_selected_text: parking_lot::Mutex::new(None),
            pending_command_mode: AtomicBool::new(false),
            download_tokens: parking_lot::Mutex::new(HashMap::new()),
            library_tokens: parking_lot::Mutex::new(HashMap::new()),
            library_queue: parking_lot::Mutex::new(VecDeque::new()),
            library_active: parking_lot::Mutex::new(None),
            retry_tokens: parking_lot::Mutex::new(HashMap::new()),
            preload_models: parking_lot::Mutex::new(HashSet::new()),
            update_state: update_checker::create_state(),
            auto_update_completed: AtomicBool::new(false),
            preflight_cancel: CancellationToken::new(),
            preflight_started: AtomicBool::new(false),
            preflight_notify: Arc::new(Notify::new()),
            session_started_at: Instant::now(),
            session_counters: parking_lot::Mutex::new(SessionCounters {
                transcription_count: 0,
            }),
            streaming_session: parking_lot::Mutex::new(None),
        }
    }

    pub fn start_streaming_session(
        &self,
        app: &AppHandle<AppRuntime>,
        model: &model_manager::ReadyModel,
    ) {
        let _ = self.stop_streaming_session(app);
        let session = streaming_transcription::StreamingSession::start(app, model);
        *self.streaming_session.lock() = Some(session);
    }

    pub fn stop_streaming_session(&self, app: &AppHandle<AppRuntime>) -> Option<String> {
        let session = self.streaming_session.lock().take()?;
        Some(session.stop(app))
    }

    pub fn has_streaming_session(&self) -> bool {
        self.streaming_session.lock().is_some()
    }

    /// Read analytics state from the in-memory cache (single lock acquisition).
    pub fn analytics_state(&self) -> (bool, String) {
        let s = self.settings.lock();
        (s.analytics_enabled, s.analytics_install_id.clone())
    }

    /// Read auto-update setting from the in-memory cache (no DB hit).
    pub fn is_auto_update_enabled(&self) -> bool {
        self.settings.lock().auto_update_enabled
    }

    pub fn is_backend_idle(&self) -> bool {
        self.download_tokens.lock().is_empty()
            && self.library_active.lock().is_none()
            && self.library_queue.lock().is_empty()
            && self.retry_tokens.lock().is_empty()
    }

    pub fn set_auto_update_completed(&self) {
        self.auto_update_completed.store(true, Ordering::SeqCst);
    }

    pub fn take_auto_update_completed(&self) -> bool {
        self.auto_update_completed.swap(false, Ordering::SeqCst)
    }

    pub fn current_settings(&self) -> UserSettings {
        match self.settings_store.load() {
            Ok(latest) => {
                *self.settings.lock() = latest.clone();
                latest
            }
            Err(err) => {
                eprintln!("Failed to load settings from DB, using cache: {err}");
                self.settings.lock().clone()
            }
        }
    }

    pub fn persist_settings(&self, mut next: UserSettings) -> FlowResult<UserSettings> {
        if matches!(next.transcription_mode, TranscriptionMode::Cloud) {
            next.transcription_mode = TranscriptionMode::Local;
        }

        self.settings_store.save(&next)?;
        *self.settings.lock() = next.clone();
        Ok(next)
    }

    pub fn pill(&self) -> &PillController {
        &self.pill
    }

    pub(crate) fn wake(&self) -> &wake::WakeCoordinator {
        &self.wake
    }

    pub fn download_default_local_model_if_missing(
        &self,
        app: &AppHandle<AppRuntime>,
        settings: &UserSettings,
        reason: &'static str,
    ) {
        if !matches!(settings.transcription_mode, TranscriptionMode::Local) {
            return;
        }

        let model_key = settings.local_model.clone();
        if model_key != default_local_model() {
            return;
        }

        match model_manager::check_model_status(app.clone(), model_key.clone()) {
            Ok(status) if status.installed => return,
            Ok(_) => {}
            Err(err) => {
                eprintln!("[ModelManager] Could not check {reason} status for {model_key}: {err}");
            }
        }

        if self.download_tokens.lock().contains_key(&model_key) {
            return;
        }

        let app_handle = app.clone();
        async_runtime::spawn(async move {
            eprintln!("[ModelManager] Downloading default STT model {model_key} for {reason}");
            match model_manager::download_model_internal(app_handle.clone(), model_key.clone())
                .await
            {
                Ok(status) if status.installed => {
                    eprintln!("[ModelManager] Default STT model {model_key} is ready");
                    let state = app_handle.state::<AppState>();
                    let settings = state.current_settings();
                    state.preload_local_model_if_needed(&app_handle, &settings, "download");
                }
                Ok(status) => {
                    eprintln!(
                        "[ModelManager] Default STT model {model_key} is still incomplete: {}",
                        status.missing_files.join(", ")
                    );
                }
                Err(err) => {
                    eprintln!(
                        "[ModelManager] Failed to download default STT model {model_key}: {err}"
                    );
                }
            }
        });
    }

    pub fn preload_local_model_if_needed(
        &self,
        app: &AppHandle<AppRuntime>,
        settings: &UserSettings,
        reason: &'static str,
    ) {
        if !matches!(settings.transcription_mode, TranscriptionMode::Local) {
            return;
        }

        let model_key = settings.local_model.clone();
        if model_key.trim().is_empty() {
            return;
        }

        {
            let mut preload_models = self.preload_models.lock();
            if !preload_models.insert(model_key.clone()) {
                return;
            }
        }

        let app_handle = app.clone();
        let transcriber = self.local_transcriber();
        let preload_key = model_key.clone();
        if let Err(err) = std::thread::Builder::new()
            .name(format!("flow-stt-preload-{model_key}"))
            .spawn(move || {
                let started = Instant::now();
                let ready_model = match model_manager::ensure_model_ready(&app_handle, &preload_key)
                {
                    Ok(model) => model,
                    Err(err) => {
                        eprintln!(
                            "[LocalTranscriber] Skipping {reason} preload for {preload_key}: {err}"
                        );
                        app_handle
                            .state::<AppState>()
                            .preload_models
                            .lock()
                            .remove(&preload_key);
                        return;
                    }
                };

                match transcriber.preload_and_warm(&ready_model) {
                    Ok(()) => eprintln!(
                        "[LocalTranscriber] Warmed {preload_key} for {reason} in {:.2}s",
                        started.elapsed().as_secs_f32()
                    ),
                    Err(err) => eprintln!(
                        "[LocalTranscriber] {reason} preload warmup failed for {preload_key}: {err}"
                    ),
                }

                app_handle
                    .state::<AppState>()
                    .preload_models
                    .lock()
                    .remove(&preload_key);
            })
        {
            eprintln!("[LocalTranscriber] Failed to start {reason} preload thread: {err}");
            self.preload_models.lock().remove(&model_key);
        }
    }

    pub fn set_shortcut_capture_active(&self, active: bool) {
        self.shortcut_capture_active.store(active, Ordering::SeqCst);
    }

    pub fn is_shortcut_capture_active(&self) -> bool {
        self.shortcut_capture_active.load(Ordering::SeqCst)
    }

    pub fn record_transcription_completed(&self) {
        self.session_counters.lock().transcription_count += 1;
    }

    fn http(&self) -> Client {
        self.http.clone()
    }

    fn local_transcriber(&self) -> Arc<local_transcription::LocalTranscriber> {
        Arc::clone(&self.local_transcriber)
    }

    fn storage(&self) -> Arc<storage::StorageManager> {
        Arc::clone(&self.storage)
    }

    pub fn store_tray(&self, tray: TrayIcon<AppRuntime>) {
        *self.tray.lock() = Some(tray);
    }

    pub fn request_cancellation(&self) {
        self.transcription_cancelled.store(true, Ordering::SeqCst);
        if let Some(token) = self.transcription_token.lock().as_ref() {
            token.cancel();
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.transcription_cancelled.load(Ordering::SeqCst)
    }

    pub fn clear_cancellation(&self) {
        self.transcription_cancelled.store(false, Ordering::SeqCst);
        *self.transcription_token.lock() = None;
    }

    pub fn create_transcription_token(&self) -> CancellationToken {
        let token = CancellationToken::new();
        *self.transcription_token.lock() = Some(token.clone());
        token
    }

    pub fn should_show_ffmpeg_toast(&self) -> bool {
        self.ffmpeg_toast_shown
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    pub fn set_pending_path(&self, path: Option<PathBuf>) {
        *self.pending_recording_path.lock() = path;
    }

    pub fn take_pending_path(&self) -> Option<PathBuf> {
        self.pending_recording_path.lock().take()
    }

    pub fn set_pending_selected_text(&self, text: Option<String>) {
        *self.pending_selected_text.lock() = text;
    }

    pub fn take_pending_selected_text(&self) -> Option<String> {
        self.pending_selected_text.lock().take()
    }

    pub fn set_pending_command_mode(&self, active: bool) {
        self.pending_command_mode.store(active, Ordering::SeqCst);
    }

    pub fn take_pending_command_mode(&self) -> bool {
        self.pending_command_mode.swap(false, Ordering::SeqCst)
    }

    pub fn create_download_token(&self, model: &str) -> CancellationToken {
        let token = CancellationToken::new();
        self.download_tokens
            .lock()
            .insert(model.to_string(), token.clone());
        token
    }

    pub fn cancel_download(&self, model: &str) -> bool {
        if let Some(token) = self.download_tokens.lock().remove(model) {
            token.cancel();
            true
        } else {
            false
        }
    }

    pub fn clear_download_token(&self, model: &str) {
        self.download_tokens.lock().remove(model);
    }

    pub fn register_library_transcription(&self, id: String) -> CancellationToken {
        let mut tokens = self.library_tokens.lock();
        if let Some(token) = tokens.get(&id) {
            return token.clone();
        }
        let token = CancellationToken::new();
        tokens.insert(id, token.clone());
        token
    }

    pub fn enqueue_library_job(&self, job: LibraryJob) -> bool {
        if self.library_tokens.lock().contains_key(&job.id) {
            return false;
        }
        if self.library_active.lock().as_deref() == Some(&job.id) {
            return false;
        }
        let mut queue = self.library_queue.lock();
        if queue.iter().any(|queued| queued.id == job.id) {
            return false;
        }
        queue.push_back(job);
        true
    }

    pub fn claim_next_library_job(&self) -> Option<LibraryJob> {
        let mut active = self.library_active.lock();
        if active.is_some() {
            return None;
        }
        let mut queue = self.library_queue.lock();
        let next = queue.pop_front()?;
        *active = Some(next.id.clone());
        Some(next)
    }

    pub fn clear_active_library_job(&self, id: &str) {
        let mut active = self.library_active.lock();
        if active.as_deref() == Some(id) {
            *active = None;
        }
    }

    pub fn remove_library_job(&self, id: &str) -> bool {
        let mut queue = self.library_queue.lock();
        let before = queue.len();
        queue.retain(|queued| queued.id != id);
        before != queue.len()
    }

    pub fn cancel_library_transcription(&self, id: &str) {
        if let Some(token) = self.library_tokens.lock().get(id) {
            token.cancel();
        }
    }

    pub fn clear_library_transcription(&self, id: &str) {
        self.library_tokens.lock().remove(id);
    }

    pub fn register_retry_transcription(&self, id: String) -> CancellationToken {
        let token = CancellationToken::new();
        self.retry_tokens.lock().insert(id, token.clone());
        token
    }

    pub fn cancel_retry_transcription(&self, id: &str) -> bool {
        if let Some(token) = self.retry_tokens.lock().remove(id) {
            token.cancel();
            true
        } else {
            false
        }
    }

    pub fn clear_retry_transcription(&self, id: &str) {
        self.retry_tokens.lock().remove(id);
    }

    pub fn start_preflight_loop(&self, app: AppHandle<AppRuntime>) {
        if self.preflight_started.swap(true, Ordering::SeqCst) {
            return;
        }

        let token = self.preflight_cancel.clone();
        let notify = Arc::clone(&self.preflight_notify);

        async_runtime::spawn(async move {
            let mut ticker = tokio::time::interval(llm_cleanup::PREFLIGHT_TTL);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let mut refresh_pending = false;

            loop {
                tokio::select! {
                    _ = token.cancelled() => break,
                    _ = notify.notified() => {
                        refresh_pending = true;
                    }
                    _ = ticker.tick() => {}
                }

                if token.is_cancelled() {
                    break;
                }

                if !refresh_pending {
                    // If woken by ticker, check if cache is still fresh (< TTL).
                    // Returns None when cache is expired, triggering refresh below.
                    if llm_cleanup::cached_preflight_available().is_some() {
                        continue;
                    }
                }

                let state = app.state::<AppState>();
                let settings = state.current_settings();
                llm_cleanup::run_preflight(state.http(), settings).await;
                refresh_pending = false;
            }
        });
    }

    pub fn stop_preflight_loop(&self) {
        self.preflight_cancel.cancel();
    }

    pub fn request_preflight_refresh(&self) {
        llm_cleanup::clear_preflight_cache();
        self.preflight_notify.notify_one();
    }

    pub fn update_state(&self) -> &update_checker::SharedUpdateState {
        &self.update_state
    }
}

#[tauri::command]
fn get_settings(app: AppHandle<AppRuntime>) -> Result<UserSettings, String> {
    if let Some(state) = app.try_state::<AppState>() {
        return Ok(state.current_settings());
    }

    let settings_store = SettingsStore::new(&app).map_err(|err| err.to_string())?;
    let mut settings = settings_store.load().unwrap_or_default();
    if model_manager::definition(&settings.local_model).is_none() {
        settings.local_model = default_local_model();
    }
    Ok(settings)
}

#[tauri::command]
fn set_shortcut_capture_active(active: bool, app: AppHandle<AppRuntime>) -> Result<(), String> {
    let state = app.state::<AppState>();
    state.set_shortcut_capture_active(active);

    if active {
        state.hotkeys.stop_registration();
        if let Err(err) = state.hotkeys.start_capture(&app) {
            state.set_shortcut_capture_active(false);
            if let Err(register_err) = pill::register_shortcuts(&app) {
                eprintln!("Failed to restore shortcuts after capture start error: {register_err}");
            }
            return Err(err.to_string());
        }
        return Ok(());
    }

    state.hotkeys.stop_capture();
    pill::register_shortcuts(&app).map_err(|err| err.to_string())
}

#[tauri::command]
fn open_accessibility_settings() -> Result<(), String> {
    permissions::open_accessibility_settings()
}

#[tauri::command]
fn check_accessibility_permission() -> bool {
    permissions::check_accessibility_permission()
}

#[tauri::command]
fn get_auto_paste_status() -> AutoPasteStatus {
    AutoPasteStatus {
        enabled: transcription_api::auto_paste_enabled(),
        accessibility_granted: permissions::check_accessibility_permission(),
    }
}

#[tauri::command]
fn paste_text_to_focused_app(
    app: AppHandle<AppRuntime>,
    text: String,
) -> Result<PasteTextResult, String> {
    paste_text_into_focused_app(
        &app,
        text,
        "Copied text",
        "Paste was blocked, so Flow copied the text to your clipboard.",
    )
}

#[tauri::command]
fn paste_last_transcript(app: AppHandle<AppRuntime>) -> Result<PasteTextResult, String> {
    recent_transcriptions::paste_latest_transcription(&app)
}

#[tauri::command]
fn copy_last_transcript(app: AppHandle<AppRuntime>) -> Result<(), String> {
    recent_transcriptions::copy_latest_transcription_to_clipboard(&app)
}

pub(crate) fn paste_text_into_focused_app(
    app: &AppHandle<AppRuntime>,
    text: String,
    fallback_title: &str,
    fallback_message: &str,
) -> Result<PasteTextResult, String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("Text is required".to_string());
    }

    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        let _ = window.hide();
        std::thread::sleep(std::time::Duration::from_millis(140));
    }

    match assistive::paste_text(&text) {
        Ok(()) => Ok(PasteTextResult {
            pasted: true,
            copied: false,
            message: "Pasted text".to_string(),
        }),
        Err(err) => {
            assistive::copy_text_to_clipboard(&text).map_err(|copy_err| {
                format!("Paste failed: {err}. Clipboard fallback failed: {copy_err}")
            })?;
            toast::show(app, "info", Some(fallback_title), fallback_message);
            Ok(PasteTextResult {
                pasted: false,
                copied: true,
                message: format!("Paste failed; copied to clipboard. Reason: {err}"),
            })
        }
    }
}

#[tauri::command]
fn check_microphone_permission() -> bool {
    permissions::check_microphone_permission()
}

#[tauri::command]
fn request_microphone_permission() -> Result<(), String> {
    permissions::request_microphone_permission()
}

#[tauri::command]
fn open_microphone_settings() -> Result<(), String> {
    permissions::open_microphone_settings()
}

#[tauri::command]
fn open_input_monitoring_settings() -> Result<(), String> {
    permissions::open_input_monitoring_settings()
}

#[tauri::command]
fn open_llm_cleanup_settings(app: AppHandle<AppRuntime>) -> Result<(), String> {
    if let Err(err) = tray::toggle_settings_window(&app) {
        eprintln!("Failed to open settings window: {err}");
        return Err(err.to_string());
    }

    let app_clone = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));
        if let Err(err) = app_clone.emit("navigate:models", ()) {
            eprintln!("Failed to emit navigate:models: {err}");
        }
    });

    Ok(())
}

#[tauri::command]
fn open_app_settings(app: AppHandle<AppRuntime>) -> Result<(), String> {
    if let Err(err) = tray::toggle_settings_window(&app) {
        eprintln!("Failed to open settings window: {err}");
        return Err(err.to_string());
    }

    let app_clone = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));
        if let Err(err) = app_clone.emit("navigate:app", ()) {
            eprintln!("Failed to emit navigate:app: {err}");
        }
    });

    Ok(())
}

#[tauri::command]
fn open_ffmpeg_install(app: AppHandle<AppRuntime>) -> Result<(), String> {
    app.opener()
        .open_url(FFMPEG_HELP_URL, None::<&str>)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn complete_onboarding(
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    core::settings::complete_onboarding(&app, &state)
}

#[tauri::command]
fn reset_onboarding(
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    core::settings::reset_onboarding(&app, &state)
}

#[tauri::command]
fn update_settings(
    args: core::settings::UpdateSettingsArgs,
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
) -> Result<UserSettings, String> {
    core::settings::update_settings(args, &app, &state)
}

#[tauri::command]
fn set_auto_transform_setting(
    enabled: bool,
    preset_id: Option<String>,
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
) -> Result<UserSettings, String> {
    let mut next = state.current_settings();
    let resolved_preset_id = preset_id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| next.auto_transform_preset_id.clone());

    if !transforms::transform_preset_exists(&resolved_preset_id) {
        return Err("Unknown auto transform preset".to_string());
    }

    if enabled && !llm_cleanup::is_llm_available(&next) {
        return Err(
            "Configure a language model in Settings -> Models before enabling Auto Transform."
                .to_string(),
        );
    }

    next.auto_transform_enabled = enabled;
    next.auto_transform_preset_id = resolved_preset_id;
    let saved = state
        .persist_settings(next)
        .map_err(|err| err.to_string())?;

    state.request_preflight_refresh();
    if let Err(err) = app.emit(EVENT_SETTINGS_CHANGED, &saved) {
        eprintln!("Failed to emit settings change: {err}");
    }

    Ok(saved)
}

#[derive(Serialize)]
struct RecordingPrunePreview {
    candidate_count: u32,
}

#[derive(Serialize)]
struct LocalDataPrunePreview {
    transcription_count: u32,
    transform_history_count: u32,
}

#[tauri::command]
async fn preview_recording_prune(
    policy: RecordingPrunePolicy,
    app: AppHandle<AppRuntime>,
) -> Result<RecordingPrunePreview, String> {
    let candidate_count =
        async_runtime::spawn_blocking(move || preview_recording_prune_for_policy(&app, policy))
            .await
            .map_err(|err| err.to_string())?
            .map_err(|err| err.to_string())?;

    Ok(RecordingPrunePreview { candidate_count })
}

#[tauri::command]
async fn preview_local_data_prune(
    policy: LocalDataStoragePolicy,
    state: tauri::State<'_, AppState>,
) -> Result<LocalDataPrunePreview, String> {
    let storage = state.storage();
    let cutoff_ms =
        local_data_prune_cutoff(policy, Local::now()).map(|cutoff| cutoff.timestamp_millis());

    let counts = async_runtime::spawn_blocking(move || storage.preview_local_data_prune(cutoff_ms))
        .await
        .map_err(|err| err.to_string())?
        .map_err(|err| err.to_string())?;

    Ok(LocalDataPrunePreview {
        transcription_count: counts.transcription_count,
        transform_history_count: counts.transform_history_count,
    })
}

#[tauri::command]
fn set_user_name(
    name: String,
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
) -> Result<UserSettings, String> {
    core::settings::set_user_name(name, &app, &state)
}

#[derive(Serialize)]
struct AppInfo {
    version: String,
    data_dir_size_bytes: u64,
    data_dir_path: String,
}

#[derive(Serialize)]
struct LocalModelRuntimeStatus {
    selected_model: String,
    loaded_model: Option<String>,
    warming: bool,
}

#[tauri::command]
fn get_local_model_runtime_status(state: tauri::State<AppState>) -> LocalModelRuntimeStatus {
    let settings = state.current_settings();
    let selected_model = settings.local_model;
    let loaded_model = state.local_transcriber.loaded_model_key();
    let warming = state.preload_models.lock().contains(&selected_model);

    LocalModelRuntimeStatus {
        selected_model,
        loaded_model,
        warming,
    }
}

#[tauri::command]
fn get_app_info(app: AppHandle<AppRuntime>) -> Result<AppInfo, String> {
    let version = env!("CARGO_PKG_VERSION").to_string();

    let data_dir =
        app_paths::app_data_dir(&app).map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let data_dir_path = data_dir.display().to_string();

    let data_dir_size_bytes = calculate_dir_size(&data_dir).unwrap_or(0);

    Ok(AppInfo {
        version,
        data_dir_size_bytes,
        data_dir_path,
    })
}

#[tauri::command]
async fn fetch_llm_models(
    endpoint: String,
    provider: LlmProvider,
    api_key: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    llm_cleanup::fetch_available_models(&state.http(), &endpoint, &provider, &api_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_whats_new(app: AppHandle<AppRuntime>) {
    if let Err(err) = tray::toggle_settings_window(&app) {
        eprintln!("Failed to open settings window: {err}");
        return;
    }

    let app_clone = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if let Err(e) = app_clone.emit("navigate:about", ()) {
            eprintln!("Failed to emit navigate:about: {e}");
        }
        std::thread::sleep(std::time::Duration::from_millis(400));
        if let Err(e) = app_clone.emit("open_whats_new", ()) {
            eprintln!("Failed to emit open_whats_new: {e}");
        }
    });
}

#[tauri::command]
fn open_about_page(app: AppHandle<AppRuntime>) {
    if let Err(err) = tray::toggle_settings_window(&app) {
        eprintln!("Failed to open settings window: {err}");
        return;
    }

    let app_clone = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));
        if let Err(e) = app_clone.emit("navigate:about", ()) {
            eprintln!("Failed to emit navigate:about: {e}");
        }
    });
}

#[tauri::command]
fn open_data_dir(path: Option<String>, app: AppHandle<AppRuntime>) -> Result<(), String> {
    let path = path.ok_or_else(|| "Path is empty".to_string())?;
    let path = PathBuf::from(&path);

    let data_dir =
        app_paths::app_data_dir(&app).map_err(|e| format!("Failed to get app data dir: {e}"))?;

    let canonical_path = path
        .canonicalize()
        .map_err(|_| "Path does not exist".to_string())?;
    let canonical_data_dir = data_dir
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize data dir: {e}"))?;

    if !canonical_path.starts_with(&canonical_data_dir) {
        return Err("Path is outside app data directory".to_string());
    }

    app.opener()
        .reveal_item_in_dir(&canonical_path)
        .map_err(|err| format!("Failed to open path: {err}"))
}

fn calculate_dir_size(path: &std::path::Path) -> Result<u64> {
    let mut total_size = 0u64;

    if !path.exists() {
        return Ok(0);
    }

    if path.is_file() {
        return Ok(path.metadata()?.len());
    }

    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += calculate_dir_size(&entry.path())?;
            }
        }
    }

    Ok(total_size)
}

#[tauri::command]
fn get_transcriptions(
    state: tauri::State<AppState>,
    search_query: Option<String>,
) -> Result<Vec<storage::TranscriptionRecord>, String> {
    state
        .storage()
        .get_all_filtered(search_query.as_deref())
        .map_err(|err| format!("Failed to get transcriptions: {err}"))
}

#[tauri::command]
fn set_transcription_pinned(
    id: String,
    pinned: bool,
    state: tauri::State<AppState>,
) -> Result<storage::TranscriptionRecord, String> {
    state
        .storage()
        .set_pinned(&id, pinned)
        .map_err(|err| format!("Failed to update pinned transcription: {err}"))?
        .ok_or_else(|| "Transcription not found".to_string())
}

#[tauri::command]
fn export_transcriptions_to_path(
    ids: Vec<String>,
    output_path: String,
    state: tauri::State<AppState>,
) -> Result<u32, String> {
    if ids.is_empty() {
        return Err("Choose at least one transcription to export.".to_string());
    }

    let mut records = Vec::new();
    for id in ids {
        if let Some(record) = state.storage().get_by_id(&id) {
            records.push(record);
        }
    }

    if records.is_empty() {
        return Err("No selected transcriptions were found.".to_string());
    }

    let output_path = PathBuf::from(output_path);
    if let Some(parent) = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create export folder: {err}"))?;
    }

    let content = build_transcriptions_markdown_export(&records);
    fs::write(&output_path, content.as_bytes())
        .map_err(|err| format!("Failed to write export file: {err}"))?;

    Ok(records.len() as u32)
}

fn build_transcriptions_markdown_export(records: &[storage::TranscriptionRecord]) -> String {
    let mut output = String::new();
    output.push_str("# Flow Transcripts\n\n");
    output.push_str(&format!("Exported: {}\n\n", Local::now().to_rfc3339()));
    output.push_str(&format!("Items: {}\n\n", records.len()));

    for (index, record) in records.iter().enumerate() {
        output.push_str(&format!(
            "## {}. {}\n\n",
            index + 1,
            record.timestamp.format("%Y-%m-%d %H:%M")
        ));
        output.push_str(&format!("- Status: {:?}\n", record.status));
        output.push_str(&format!(
            "- Words: {}\n",
            if record.word_count > 0 {
                record.word_count
            } else {
                count_export_words(&record.text)
            }
        ));
        if record.pinned {
            output.push_str("- Pinned: yes\n");
        }
        if !record.speech_model.trim().is_empty() {
            output.push_str(&format!("- Speech model: {}\n", record.speech_model.trim()));
        }
        if let Some(mode_name) = record
            .mode_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            output.push_str(&format!("- Mode: {}\n", mode_name.trim()));
        }
        if let Some(transform_label) = record
            .auto_transform_label
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            output.push_str(&format!("- Auto Transform: {}\n", transform_label.trim()));
        }
        output.push('\n');

        let body = if record.text.trim().is_empty() {
            record
                .error_message
                .as_deref()
                .unwrap_or("No transcript text.")
        } else {
            record.text.trim()
        };
        output.push_str(body);
        output.push_str("\n\n");

        if let Some(raw_text) = record
            .raw_text
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            output.push_str("<details>\n<summary>Original transcript</summary>\n\n");
            output.push_str(raw_text.trim());
            output.push_str("\n\n</details>\n\n");
        }
    }

    output
}

fn count_export_words(text: &str) -> u32 {
    text.split_whitespace().count().min(u32::MAX as usize) as u32
}

#[tauri::command]
fn delete_transcription(
    id: String,
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
) -> Result<bool, String> {
    let result = match state.storage().delete(&id) {
        Ok(Some(audio_path)) => {
            let path = PathBuf::from(audio_path);
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
            Ok(true)
        }
        Ok(None) => Ok(false),
        Err(err) => Err(format!("Failed to delete transcription: {err}")),
    }?;

    let settings = state.current_settings();
    if let Err(err) = tray::refresh_tray_menu(&app, &settings) {
        eprintln!("Failed to refresh tray menu: {err}");
    }
    #[cfg(target_os = "macos")]
    if let Err(err) = set_app_menu(&app, &settings) {
        eprintln!("Failed to refresh app menu: {err}");
    }

    Ok(result)
}

#[tauri::command]
fn delete_all_transcriptions(
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
) -> Result<u32, String> {
    let audio_paths = state
        .storage()
        .delete_all()
        .map_err(|err| format!("Failed to delete all transcriptions: {err}"))?;

    let deleted_count = audio_paths.len() as u32;
    for audio_path in audio_paths {
        let _ = std::fs::remove_file(audio_path);
    }

    let settings = state.current_settings();
    if let Err(err) = tray::refresh_tray_menu(&app, &settings) {
        eprintln!("Failed to refresh tray menu: {err}");
    }
    #[cfg(target_os = "macos")]
    if let Err(err) = set_app_menu(&app, &settings) {
        eprintln!("Failed to refresh app menu: {err}");
    }

    Ok(deleted_count)
}

#[tauri::command]
async fn retry_transcription(
    id: String,
    app: AppHandle<AppRuntime>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    core::transcriptions::retry_transcription(id, &app, &state)
}

#[tauri::command]
fn cancel_retry_transcription(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    Ok(state.cancel_retry_transcription(&id))
}

#[tauri::command]
async fn retry_llm_cleanup(
    id: String,
    app: AppHandle<AppRuntime>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    core::transcriptions::retry_llm_cleanup(id, &app, &state)
}

#[tauri::command]
async fn undo_llm_cleanup(
    id: String,
    app: AppHandle<AppRuntime>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    core::transcriptions::undo_llm_cleanup(id, &app, &state)
}

pub(crate) fn hide_overlay(app: &AppHandle<AppRuntime>) {
    app.state::<AppState>().pill().reset(app);
}

pub(crate) fn stop_active_recording(app: &AppHandle<AppRuntime>) {
    app.state::<AppState>().pill().cancel(app);
}

#[tauri::command]
fn cancel_recording(app: AppHandle<AppRuntime>) {
    let state = app.state::<AppState>();
    if state.pill().status() == pill::PillStatus::Processing {
        state.pill().cancel_processing(&app);
    } else {
        stop_active_recording(&app);
        hide_overlay(&app);
    }
}

#[tauri::command]
fn pause_flow_temporarily(app: AppHandle<AppRuntime>, seconds: Option<u64>) {
    let state = app.state::<AppState>();
    if state.pill().status() == pill::PillStatus::Processing {
        state.pill().cancel_processing(&app);
    } else if state.pill().status() != pill::PillStatus::Idle {
        state.pill().cancel(&app);
    }

    let hide_duration = Duration::from_secs(seconds.unwrap_or(300).max(1));
    state.pill().hide_overlay_for(hide_duration);
    pill::hide_overlay(&app);

    let settings = state.current_settings();
    if let Err(err) = state.wake.sync(&app, &settings) {
        eprintln!("Failed to keep wake listener armed after hiding overlay: {err}");
    }

    let app_handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(hide_duration);
        if app_handle.state::<AppState>().pill().status() == pill::PillStatus::Idle {
            pill::show_overlay(&app_handle);
        }
    });
}

#[tauri::command]
fn toggle_recording(app: AppHandle<AppRuntime>) {
    app.state::<AppState>().pill().toggle_from_overlay(&app);
}

pub(crate) fn persist_recording_async(app: AppHandle<AppRuntime>, recording: CompletedRecording) {
    let base_dir = match recordings_root(&app) {
        Ok(path) => path,
        Err(err) => {
            emit_error(
                &app,
                format!("Failed to resolve recordings directory: {err}"),
            );
            return;
        }
    };

    let recording_for_transcription = recording.clone();

    async_runtime::spawn(async move {
        let task =
            async_runtime::spawn_blocking(move || recorder::persist_recording(base_dir, recording));
        match task.await {
            Ok(Ok(saved)) => emit_complete(&app, saved, recording_for_transcription),
            Ok(Err(err)) => emit_error(&app, format!("Unable to save recording: {err}")),
            Err(err) => emit_error(&app, format!("Recording task failed: {err}")),
        }
    });
}

fn emit_complete(
    app: &AppHandle<AppRuntime>,
    saved: RecordingSaved,
    recording: CompletedRecording,
) {
    if let Err(rejection) = validate_recording(&recording) {
        let reason = match rejection {
            RecordingRejectionReason::TooShort {
                duration_ms,
                min_ms,
            } => {
                format!("Recording too short ({duration_ms}ms < {min_ms}ms minimum)")
            }
            RecordingRejectionReason::TooQuiet { rms, threshold } => {
                format!("Recording too quiet (energy {rms:.4} < {threshold} threshold)")
            }
            RecordingRejectionReason::NoSpeechDetected => {
                "No speech detected in recording".to_string()
            }
            RecordingRejectionReason::EmptyBuffer => "Recording buffer is empty".to_string(),
        };
        eprintln!("Recording rejected: {reason}");

        if let Err(err) = std::fs::remove_file(&saved.path) {
            eprintln!("Failed to remove rejected recording file: {err}");
        }

        app.state::<AppState>().pill().reset(app);
        return;
    }

    transcribe::queue_transcription(app, saved, recording);
}

pub(crate) fn emit_error(app: &AppHandle<AppRuntime>, message: String) {
    app.state::<AppState>()
        .pill()
        .transition_to_error(app, &message);
}

pub(crate) fn emit_event<T: Serialize + Clone>(
    app: &AppHandle<AppRuntime>,
    event: &str,
    payload: T,
) {
    if let Err(err) = app.emit(event, payload) {
        eprintln!("Failed to emit {event}: {err}");
    }
}

fn recordings_root(app: &AppHandle<AppRuntime>) -> FlowResult<PathBuf> {
    let mut data_dir = app_paths::app_data_dir(app).context("App data directory not found")?;
    data_dir.push("recordings");
    Ok(data_dir)
}

pub(crate) fn schedule_recording_prune(app: AppHandle<AppRuntime>, settings: UserSettings) {
    if matches!(settings.recording_prune_policy, RecordingPrunePolicy::Never) {
        return;
    }

    async_runtime::spawn(async move {
        let app_handle = app.clone();
        match async_runtime::spawn_blocking(move || {
            prune_recordings_for_settings(&app_handle, &settings)
        })
        .await
        {
            Ok(Ok(count)) => {
                if count > 0 {
                    let _ = app.emit(
                        EVENT_TRANSCRIPTION_COMPLETE,
                        TranscriptionCompletePayload {
                            transcript: String::new(),
                            auto_paste: false,
                        },
                    );
                }
            }
            Ok(Err(err)) => eprintln!("Failed to prune recordings: {err}"),
            Err(err) => eprintln!("Recording prune task failed: {err}"),
        }
    });
}

pub(crate) fn schedule_local_data_prune(app: AppHandle<AppRuntime>, settings: UserSettings) {
    if matches!(
        settings.local_data_storage_policy,
        LocalDataStoragePolicy::Store
    ) {
        return;
    }

    async_runtime::spawn(async move {
        let app_handle = app.clone();
        match async_runtime::spawn_blocking(move || {
            prune_local_data_for_settings(&app_handle, &settings)
        })
        .await
        {
            Ok(Ok(counts)) => {
                if counts.transcription_count > 0 || counts.transform_history_count > 0 {
                    let _ = app.emit(
                        EVENT_TRANSCRIPTION_COMPLETE,
                        TranscriptionCompletePayload {
                            transcript: String::new(),
                            auto_paste: false,
                        },
                    );
                }
            }
            Ok(Err(err)) => eprintln!("Failed to prune local data: {err}"),
            Err(err) => eprintln!("Local data prune task failed: {err}"),
        }
    });
}

fn prune_local_data_for_settings(
    app: &AppHandle<AppRuntime>,
    settings: &UserSettings,
) -> FlowResult<storage::LocalDataPruneCounts> {
    let cutoff_ms = local_data_prune_cutoff(settings.local_data_storage_policy, Local::now())
        .map(|cutoff| cutoff.timestamp_millis());
    let result = app
        .state::<AppState>()
        .storage()
        .prune_local_data(cutoff_ms)?;

    for audio_path in &result.audio_paths {
        if audio_path.trim().is_empty() {
            continue;
        }

        if let Err(err) = std::fs::remove_file(audio_path) {
            if Path::new(audio_path).exists() {
                eprintln!("Failed to remove local data audio {}: {err}", audio_path);
            }
        }
    }

    Ok(result.counts)
}

fn local_data_prune_cutoff(
    policy: LocalDataStoragePolicy,
    now: DateTime<Local>,
) -> Option<DateTime<Local>> {
    match policy {
        LocalDataStoragePolicy::Store => None,
        LocalDataStoragePolicy::Day => now.checked_sub_days(Days::new(1)),
        LocalDataStoragePolicy::Never => None,
    }
}

fn prune_recordings_for_settings(
    app: &AppHandle<AppRuntime>,
    settings: &UserSettings,
) -> FlowResult<u32> {
    count_or_prune_recordings(
        app,
        settings.recording_prune_policy,
        Local::now(),
        RecordingPruneAction::Delete,
    )
}

fn preview_recording_prune_for_policy(
    app: &AppHandle<AppRuntime>,
    policy: RecordingPrunePolicy,
) -> FlowResult<u32> {
    count_or_prune_recordings(app, policy, Local::now(), RecordingPruneAction::Count)
}

enum RecordingPruneAction {
    Count,
    Delete,
}

fn count_or_prune_recordings(
    app: &AppHandle<AppRuntime>,
    policy: RecordingPrunePolicy,
    now: DateTime<Local>,
    action: RecordingPruneAction,
) -> FlowResult<u32> {
    let root = recordings_root(app)?;
    if !root.exists() || matches!(policy, RecordingPrunePolicy::Never) {
        return Ok(0);
    }

    let cutoff = recording_prune_cutoff(policy, now);
    let (count, _) = match action {
        RecordingPruneAction::Count => count_prunable_recording_tree(&root, policy, cutoff)?,
        RecordingPruneAction::Delete => prune_recording_tree(&root, policy, cutoff)?,
    };
    Ok(count)
}

fn recording_prune_cutoff(
    policy: RecordingPrunePolicy,
    now: DateTime<Local>,
) -> Option<DateTime<Local>> {
    match policy {
        RecordingPrunePolicy::Never => None,
        RecordingPrunePolicy::Immediately => Some(now),
        RecordingPrunePolicy::Day => now.checked_sub_days(Days::new(1)),
        RecordingPrunePolicy::Week => now.checked_sub_days(Days::new(7)),
        RecordingPrunePolicy::Month => now.checked_sub_months(Months::new(1)),
        RecordingPrunePolicy::ThreeMonths => now.checked_sub_months(Months::new(3)),
        RecordingPrunePolicy::Year => now.checked_sub_months(Months::new(12)),
    }
}

fn prune_recording_tree(
    path: &Path,
    policy: RecordingPrunePolicy,
    cutoff: Option<DateTime<Local>>,
) -> FlowResult<(u32, bool)> {
    let mut deleted_count = 0;
    let mut is_empty = true;

    for entry in fs::read_dir(path)
        .with_context(|| format!("Failed to read recordings directory {}", path.display()))?
    {
        let entry = entry?;
        let child_path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            let (child_deleted, child_empty) = prune_recording_tree(&child_path, policy, cutoff)?;
            deleted_count += child_deleted;
            if child_empty {
                fs::remove_dir(&child_path).with_context(|| {
                    format!(
                        "Failed to remove empty recordings directory {}",
                        child_path.display()
                    )
                })?;
            } else {
                is_empty = false;
            }
            continue;
        }

        if should_prune_recording_file(&child_path, &metadata, policy, cutoff) {
            fs::remove_file(&child_path)
                .with_context(|| format!("Failed to remove recording {}", child_path.display()))?;
            deleted_count += 1;
        } else {
            is_empty = false;
        }
    }

    Ok((deleted_count, is_empty))
}

fn count_prunable_recording_tree(
    path: &Path,
    policy: RecordingPrunePolicy,
    cutoff: Option<DateTime<Local>>,
) -> FlowResult<(u32, bool)> {
    let mut candidate_count = 0;
    let mut is_empty = true;

    for entry in fs::read_dir(path)
        .with_context(|| format!("Failed to read recordings directory {}", path.display()))?
    {
        let entry = entry?;
        let child_path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            let (child_count, child_empty) =
                count_prunable_recording_tree(&child_path, policy, cutoff)?;
            candidate_count += child_count;
            if !child_empty {
                is_empty = false;
            }
            continue;
        }

        if should_prune_recording_file(&child_path, &metadata, policy, cutoff) {
            candidate_count += 1;
        } else {
            is_empty = false;
        }
    }

    Ok((candidate_count, is_empty))
}

fn should_prune_recording_file(
    path: &Path,
    metadata: &fs::Metadata,
    policy: RecordingPrunePolicy,
    cutoff: Option<DateTime<Local>>,
) -> bool {
    if !metadata.is_file() {
        return false;
    }

    let is_wav = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("wav"));
    if !is_wav {
        return false;
    }

    if matches!(policy, RecordingPrunePolicy::Immediately) {
        return true;
    }

    let Some(cutoff) = cutoff else {
        return false;
    };

    metadata
        .modified()
        .ok()
        .map(|modified| DateTime::<Local>::from(modified) <= cutoff)
        .unwrap_or(false)
}

#[derive(Serialize, Clone)]
pub(crate) struct RecordingStartPayload {
    pub(crate) started_at: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct AudioSpectrumPayload {
    pub(crate) bins: Vec<u8>,
}

#[derive(Serialize, Clone)]
pub(crate) struct TranscriptionCompletePayload {
    pub(crate) transcript: String,
    pub(crate) auto_paste: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct TranscriptionErrorPayload {
    pub(crate) message: String,
    pub(crate) stage: String,
}

#[derive(Serialize, Clone)]
struct AutoPasteStatus {
    enabled: bool,
    accessibility_granted: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct PasteTextResult {
    pasted: bool,
    copied: bool,
    message: String,
}
