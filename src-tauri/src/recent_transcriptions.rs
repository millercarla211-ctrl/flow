use crate::{assistive, toast, AppRuntime, AppState, PasteTextResult};
use tauri::menu::{MenuItem, SubmenuBuilder};
use tauri::{AppHandle, Manager};

pub const MENU_ID_RECENT_TRANSCRIPTION_PREFIX: &str = "menu_recent_transcription_";
const MENU_ID_RECENT_TRANSCRIPTION_EMPTY: &str = "menu_recent_transcription_empty";
const MENU_ID_RECENT_TRANSCRIPTION_ERROR: &str = "menu_recent_transcription_error";
const RECENT_TRANSCRIPTIONS_LIMIT: usize = 5;
const RECENT_TRANSCRIPTIONS_PREVIEW_LEN: usize = 60;

pub fn build_recent_transcriptions_menu(
    app: &AppHandle<AppRuntime>,
    label: &str,
) -> tauri::Result<tauri::menu::Submenu<AppRuntime>> {
    let mut submenu = SubmenuBuilder::new(app, label);
    if let Some(state) = app.try_state::<AppState>() {
        match state
            .storage()
            .get_recent_transcriptions(RECENT_TRANSCRIPTIONS_LIMIT)
        {
            Ok(records) if !records.is_empty() => {
                for record in records {
                    let preview = format_transcription_preview(
                        &record.text,
                        RECENT_TRANSCRIPTIONS_PREVIEW_LEN,
                    );
                    let item = MenuItem::with_id(
                        app,
                        format!("{MENU_ID_RECENT_TRANSCRIPTION_PREFIX}{}", record.id),
                        preview,
                        true,
                        None::<&str>,
                    )?;
                    submenu = submenu.item(&item);
                }
            }
            Ok(_) => {
                let item = MenuItem::with_id(
                    app,
                    MENU_ID_RECENT_TRANSCRIPTION_EMPTY,
                    "No transcriptions yet",
                    false,
                    None::<&str>,
                )?;
                submenu = submenu.item(&item);
            }
            Err(err) => {
                eprintln!("Failed to load recent transcriptions for menu: {err}");
                let item = MenuItem::with_id(
                    app,
                    MENU_ID_RECENT_TRANSCRIPTION_ERROR,
                    "Unable to load transcriptions",
                    false,
                    None::<&str>,
                )?;
                submenu = submenu.item(&item);
            }
        }
    } else {
        let item = MenuItem::with_id(
            app,
            MENU_ID_RECENT_TRANSCRIPTION_EMPTY,
            "No transcriptions yet",
            false,
            None::<&str>,
        )?;
        submenu = submenu.item(&item);
    }

    submenu.build()
}

pub fn copy_transcription_to_clipboard(app: &AppHandle<AppRuntime>, transcription_id: &str) {
    let record = app
        .state::<AppState>()
        .storage()
        .get_by_id(transcription_id);
    let Some(record) = record else {
        emit_copy_error_toast(app, "Transcription no longer available");
        refresh_recent_menus(app);
        return;
    };
    let text = record.text.trim();
    if text.is_empty() {
        emit_copy_error_toast(app, "Transcription is empty");
        refresh_recent_menus(app);
        return;
    }
    if let Err(err) = assistive::copy_text_to_clipboard(text) {
        eprintln!("Failed to copy transcription to clipboard: {err}");
        emit_copy_error_toast(app, "Unable to copy to clipboard");
        return;
    }

    toast::emit_toast(
        app,
        toast::Payload {
            toast_type: "success".to_string(),
            title: None,
            message: "Copied to clipboard".to_string(),
            auto_dismiss: Some(true),
            duration: Some(1200),
            retry_id: None,
            mode: None,
            action: None,
            action_label: None,
        },
    );
}

pub fn paste_latest_transcription(app: &AppHandle<AppRuntime>) -> Result<PasteTextResult, String> {
    let record = app
        .state::<AppState>()
        .storage()
        .get_recent_transcriptions(1)
        .map_err(|err| format!("Failed to load last transcript: {err}"))?
        .into_iter()
        .next()
        .ok_or_else(|| "No transcript available to paste".to_string())?;

    let text = record.text.trim();
    if text.is_empty() {
        return Err("Last transcript is empty".to_string());
    }

    crate::paste_text_into_focused_app(
        app,
        text.to_string(),
        "Copied last transcript",
        "Paste was blocked, so Flow copied the last transcript to your clipboard.",
    )
}

pub fn paste_latest_transcription_from_menu(app: &AppHandle<AppRuntime>) {
    match paste_latest_transcription(app) {
        Ok(result) if result.pasted => {
            toast::emit_toast(
                app,
                toast::Payload {
                    toast_type: "success".to_string(),
                    title: None,
                    message: "Pasted last transcript".to_string(),
                    auto_dismiss: Some(true),
                    duration: Some(1200),
                    retry_id: None,
                    mode: None,
                    action: None,
                    action_label: None,
                },
            );
        }
        Ok(_) => {}
        Err(err) => emit_copy_error_toast(app, &err),
    }
}

fn emit_copy_error_toast(app: &AppHandle<AppRuntime>, message: &str) {
    toast::emit_toast(
        app,
        toast::Payload {
            toast_type: "error".to_string(),
            title: None,
            message: message.to_string(),
            auto_dismiss: Some(true),
            duration: Some(1600),
            retry_id: None,
            mode: None,
            action: None,
            action_label: None,
        },
    );
}

fn refresh_recent_menus(app: &AppHandle<AppRuntime>) {
    let settings = app.state::<AppState>().current_settings();
    if let Err(err) = crate::tray::refresh_tray_menu(app, &settings) {
        eprintln!("Failed to refresh tray menu: {err}");
    }
    #[cfg(target_os = "macos")]
    if let Err(err) = crate::set_app_menu(app, &settings) {
        eprintln!("Failed to refresh app menu: {err}");
    }
}

fn format_transcription_preview(text: &str, max_len: usize) -> String {
    let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() {
        return "Empty transcription".to_string();
    }

    let mut chars = cleaned.chars();
    let preview: String = chars.by_ref().take(max_len).collect();
    if chars.next().is_some() {
        let trim_len = max_len.saturating_sub(3);
        let mut shortened: String = preview.chars().take(trim_len).collect();
        shortened.push_str("...");
        shortened
    } else {
        preview
    }
}
