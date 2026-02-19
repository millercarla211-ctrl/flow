use std::fs;
use std::path::{Component, PathBuf};

use anyhow::{Context, Result};
use tauri::{AppHandle, Emitter};

use crate::{AppRuntime, AppState, LibraryJob, LibraryJobKind};

use super::processing::{
    build_export_content, create_item_from_path, library_root, stored_original_path,
};
use super::queue::{release_library_slot, schedule_library_job};
use super::types::{
    ExportFormat, LibraryErrorPayload, LibraryFilter, LibraryImportOptions, LibraryItem,
    LibraryItemPatch, LibraryItemStatus, LibraryItemsPage, EVENT_LIBRARY_ERROR,
    EVENT_LIBRARY_OPEN_IMPORT,
};

#[tauri::command]
pub fn create_library_item(
    path: String,
    options: LibraryImportOptions,
    app: AppHandle<AppRuntime>,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryItem, String> {
    let source_path = PathBuf::from(path);
    let storage = state.storage();
    let item = create_item_from_path(&app, storage, &source_path, &options)
        .map_err(|err| err.to_string())?;
    schedule_library_job(
        &app,
        &state,
        LibraryJob {
            id: item.id.clone(),
            kind: LibraryJobKind::Import {
                source_path,
                store_original: options.store_original,
            },
        },
    );
    Ok(item)
}

#[tauri::command]
pub fn get_library_items_page(
    filter: Option<LibraryFilter>,
    limit: u32,
    offset: u32,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryItemsPage, String> {
    let filter = filter.unwrap_or_default();
    let limit = limit.clamp(1, 200) as usize;
    let offset = offset as usize;
    state
        .storage()
        .get_library_items_page(filter, limit, offset)
        .map(|(items, has_more)| LibraryItemsPage { items, has_more })
        .map_err(|err| format!("Failed to load library items page: {err}"))
}

#[tauri::command]
pub fn update_library_item(
    id: String,
    patch: LibraryItemPatch,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryItem, String> {
    let storage = state.storage();
    let updated = storage
        .update_library_item(&id, patch.clone())
        .map_err(|err| format!("Failed to update library item: {err}"))?
        .ok_or_else(|| "Library item not found".to_string())?;

    Ok(updated)
}

#[tauri::command]
pub fn delete_library_item(
    id: String,
    app: AppHandle<AppRuntime>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.remove_library_job(&id);
    state.cancel_library_transcription(&id);
    release_library_slot(&app, &state, &id);

    let storage = state.storage();
    let item = storage
        .get_library_item(&id)
        .map_err(|err| format!("Failed to load library item: {err}"))?;
    let Some(item) = item else {
        return Ok(());
    };

    let path = PathBuf::from(&item.audio_path);
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("Invalid library file path; delete aborted.".to_string());
    }

    let root = match library_root(&app) {
        Ok(root) => root,
        Err(err) => {
            eprintln!("Failed to resolve library root for delete: {err}");
            return Err("Failed to resolve library storage location.".to_string());
        }
    };
    let root = root
        .canonicalize()
        .map_err(|_| "Failed to resolve library storage location.".to_string())?;
    let safe_path = if path.exists() {
        path.canonicalize()
            .map_err(|_| "Library item path could not be resolved; delete aborted.".to_string())?
    } else if let Some(parent) = path.parent() {
        if parent.exists() {
            let parent = parent.canonicalize().map_err(|_| {
                "Library item folder could not be resolved; delete aborted.".to_string()
            })?;
            parent.join(path.file_name().unwrap_or_default())
        } else {
            path.clone()
        }
    } else {
        path.clone()
    };
    if !safe_path.starts_with(&root) {
        return Err(
            "Library item is stored outside the library folder; delete aborted.".to_string(),
        );
    }

    if let Some(parent) = safe_path.parent() {
        if parent == root {
            if safe_path.exists() {
                fs::remove_file(&safe_path)
                    .map_err(|err| format!("Failed to delete library file: {err}"))?;
            }
        } else if parent.exists() {
            fs::remove_dir_all(parent)
                .map_err(|err| format!("Failed to delete library files: {err}"))?;
        } else if safe_path.exists() {
            fs::remove_file(&safe_path)
                .map_err(|err| format!("Failed to delete library file: {err}"))?;
        }
    } else if safe_path.exists() {
        fs::remove_file(&safe_path)
            .map_err(|err| format!("Failed to delete library file: {err}"))?;
    }

    storage
        .delete_library_item(&id)
        .map_err(|err| format!("Failed to delete library item: {err}"))?;
    Ok(())
}

#[tauri::command]
pub fn cancel_library_transcription(
    id: String,
    app: AppHandle<AppRuntime>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if state.remove_library_job(&id) {
        let _ = state.storage().update_library_item(
            &id,
            LibraryItemPatch {
                status: Some(LibraryItemStatus::Cancelled),
                ..Default::default()
            },
        );
        let _ = app.emit(
            EVENT_LIBRARY_ERROR,
            LibraryErrorPayload {
                id,
                message: "Transcription cancelled".to_string(),
            },
        );
        return Ok(());
    }
    state.cancel_library_transcription(&id);
    let _ = state.storage().update_library_item(
        &id,
        LibraryItemPatch {
            status: Some(LibraryItemStatus::Cancelling),
            ..Default::default()
        },
    );
    Ok(())
}

#[tauri::command]
pub fn retry_library_transcription(
    id: String,
    app: AppHandle<AppRuntime>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let storage = state.storage();
    let item = storage
        .get_library_item(&id)
        .map_err(|err| format!("Failed to load library item: {err}"))?
        .ok_or_else(|| "Library item not found".to_string())?;

    let audio_exists = PathBuf::from(&item.audio_path).exists();
    let mut job = LibraryJobKind::TranscribeExisting;

    if !audio_exists {
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(path) = stored_original_path(&item) {
            candidates.push(path);
        }
        let source = item.source_path.trim();
        if !source.is_empty() {
            candidates.push(PathBuf::from(source));
        }

        if let Some(source_path) = candidates.into_iter().find(|path| path.exists()) {
            job = LibraryJobKind::Import {
                source_path,
                store_original: item.store_original,
            };
        } else {
            let message = "Original file not found. Re-import the file to try again.".to_string();
            let _ = storage.update_library_item(
                &id,
                LibraryItemPatch {
                    status: Some(LibraryItemStatus::Error {
                        message: message.clone(),
                    }),
                    ..Default::default()
                },
            );
            let _ = app.emit(
                EVENT_LIBRARY_ERROR,
                LibraryErrorPayload {
                    id,
                    message: message.clone(),
                },
            );
            return Err(message);
        }
    }

    let _ = storage.update_library_item(
        &id,
        LibraryItemPatch {
            status: Some(LibraryItemStatus::Pending),
            ..Default::default()
        },
    );
    schedule_library_job(&app, &state, LibraryJob { id, kind: job });
    Ok(())
}

#[tauri::command]
pub fn export_library_item_to_path(
    id: String,
    format: ExportFormat,
    output_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let item = state
        .storage()
        .get_library_item(&id)
        .map_err(|err| format!("Failed to load library item: {err}"))?
        .ok_or_else(|| "Library item not found".to_string())?;

    let content = build_export_content(&item, format.clone())
        .map_err(|err| format!("Failed to build export: {err}"))?;

    let output_path = PathBuf::from(&output_path);

    // Validate output path is absolute and doesn't contain path traversal
    if !output_path.is_absolute() {
        return Err("Export path must be absolute".to_string());
    }
    if output_path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("Export path contains invalid components".to_string());
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create export directory")
            .map_err(|err| err.to_string())?;
    }

    fs::write(&output_path, content.as_bytes())
        .with_context(|| "Failed to write export file".to_string())
        .map_err(|err| err.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_library_tags(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    state
        .storage()
        .get_library_tags()
        .map_err(|err| format!("Failed to load tags: {err}"))
}

pub fn handle_opened_paths(app: &AppHandle<AppRuntime>, urls: Vec<PathBuf>) -> Result<()> {
    let paths: Vec<String> = urls
        .into_iter()
        .filter(|path| path.is_file())
        .map(|path| path.display().to_string())
        .collect();
    if paths.is_empty() {
        return Ok(());
    }
    if let Err(err) = crate::tray::toggle_settings_window(app) {
        eprintln!("Failed to open settings window: {err}");
    }
    let _ = app.emit(EVENT_LIBRARY_OPEN_IMPORT, paths);
    Ok(())
}
