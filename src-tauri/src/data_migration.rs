use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use tauri::{AppHandle, Manager};

use crate::AppRuntime;

const LEGACY_APP_IDENTIFIER: &str = "dev.glimpse.glimpse";
const MIGRATION_MARKER_FILE: &str =
    ".migration-complete-dev.glimpse.glimpse-to-com.glimpse.data-v1";

#[derive(Default)]
struct CopyStats {
    copied_files: u64,
    skipped_files: u64,
    skipped_symlinks: u64,
}

pub(crate) fn migrate_legacy_app_dirs(app: &AppHandle<AppRuntime>) -> Result<()> {
    let current_identifier = app.config().identifier.as_str();
    if current_identifier == LEGACY_APP_IDENTIFIER {
        return Ok(());
    }

    let resolver = app.path();
    let current_data_dir = resolver
        .app_data_dir()
        .context("Failed to resolve app data directory")?;
    let marker_path = migration_marker_path(&current_data_dir);
    if is_migration_marked_complete(&marker_path) {
        return Ok(());
    }

    if let Some(legacy_data_dir) =
        legacy_sibling_dir(&current_data_dir, current_identifier, LEGACY_APP_IDENTIFIER)
    {
        migrate_dir(&legacy_data_dir, &current_data_dir)
            .context("Failed to migrate app data directory")?;
    }

    let current_config_dir = resolver
        .app_config_dir()
        .or_else(|_| resolver.app_data_dir())
        .context("Failed to resolve app config directory")?;

    if current_config_dir != current_data_dir {
        if let Some(legacy_config_dir) = legacy_sibling_dir(
            &current_config_dir,
            current_identifier,
            LEGACY_APP_IDENTIFIER,
        ) {
            let legacy_settings_dir = legacy_config_dir.join("Glimpse");
            let current_settings_dir = current_config_dir.join("Glimpse");
            migrate_dir(&legacy_settings_dir, &current_settings_dir)
                .context("Failed to migrate settings directory")?;
        }
    }

    write_migration_marker(&current_data_dir, &marker_path)
        .context("Failed to persist migration marker")?;

    Ok(())
}

fn migration_marker_path(current_data_dir: &Path) -> PathBuf {
    current_data_dir.join(MIGRATION_MARKER_FILE)
}

fn is_migration_marked_complete(marker_path: &Path) -> bool {
    marker_path.is_file()
}

fn write_migration_marker(current_data_dir: &Path, marker_path: &Path) -> Result<()> {
    fs::create_dir_all(current_data_dir).with_context(|| {
        format!(
            "Failed to create app data directory {}",
            current_data_dir.display()
        )
    })?;
    fs::write(
        marker_path,
        "legacy=dev.glimpse.glimpse\ncurrent=com.glimpse.data\nstatus=complete\n",
    )
    .with_context(|| format!("Failed to write migration marker {}", marker_path.display()))?;
    Ok(())
}

fn legacy_sibling_dir(
    current_dir: &Path,
    current_identifier: &str,
    legacy_identifier: &str,
) -> Option<PathBuf> {
    let current_leaf = current_dir.file_name()?.to_str()?;
    if current_leaf != current_identifier {
        return None;
    }

    Some(current_dir.parent()?.join(legacy_identifier))
}

fn migrate_dir(legacy_dir: &Path, current_dir: &Path) -> Result<()> {
    if !legacy_dir.exists() {
        return Ok(());
    }

    if !legacy_dir.is_dir() {
        return Err(anyhow!(
            "Legacy path is not a directory: {}",
            legacy_dir.display()
        ));
    }

    if current_dir.exists() {
        if !current_dir.is_dir() {
            return Err(anyhow!(
                "Migration target is not a directory: {}",
                current_dir.display()
            ));
        }

        let mut stats = CopyStats::default();
        copy_missing_recursive(legacy_dir, current_dir, &mut stats).with_context(|| {
            format!(
                "Failed to merge {} into {}",
                legacy_dir.display(),
                current_dir.display()
            )
        })?;

        if stats.skipped_files == 0 && stats.skipped_symlinks == 0 {
            if let Err(err) = fs::remove_dir_all(legacy_dir) {
                eprintln!(
                    "Merged legacy directory but failed to remove {}: {err}",
                    legacy_dir.display()
                );
            }
        }

        if stats.copied_files > 0 || stats.skipped_files > 0 || stats.skipped_symlinks > 0 {
            eprintln!(
                "Merged legacy directory {} into {} (copied: {}, skipped_existing: {}, skipped_symlinks: {})",
                legacy_dir.display(),
                current_dir.display(),
                stats.copied_files,
                stats.skipped_files,
                stats.skipped_symlinks
            );
        }
        return Ok(());
    }

    if let Some(parent) = current_dir.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create migration target parent {}",
                parent.display()
            )
        })?;
    }

    match fs::rename(legacy_dir, current_dir) {
        Ok(_) => {
            eprintln!(
                "Migrated {} -> {}",
                legacy_dir.display(),
                current_dir.display()
            );
            Ok(())
        }
        Err(rename_err) => {
            let mut stats = CopyStats::default();
            copy_missing_recursive(legacy_dir, current_dir, &mut stats).with_context(|| {
                format!(
                    "Failed to copy {} into {} after move failure",
                    legacy_dir.display(),
                    current_dir.display()
                )
            })?;

            if let Err(err) = fs::remove_dir_all(legacy_dir) {
                eprintln!(
                    "Copied legacy directory but failed to remove {}: {err}",
                    legacy_dir.display()
                );
            }

            eprintln!(
                "Copied {} into {} after move failed ({rename_err}); copied {} files",
                legacy_dir.display(),
                current_dir.display(),
                stats.copied_files
            );
            Ok(())
        }
    }
}

fn copy_missing_recursive(from: &Path, to: &Path, stats: &mut CopyStats) -> Result<()> {
    fs::create_dir_all(to).with_context(|| format!("Failed to create {}", to.display()))?;

    for entry in fs::read_dir(from).with_context(|| format!("Failed to read {}", from.display()))? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source = entry.path();
        let destination = to.join(entry.file_name());

        if file_type.is_dir() {
            copy_missing_recursive(&source, &destination, stats)?;
            continue;
        }

        if file_type.is_file() {
            if destination.exists() {
                stats.skipped_files += 1;
                continue;
            }
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create {}", parent.display()))?;
            }
            fs::copy(&source, &destination).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    source.display(),
                    destination.display()
                )
            })?;
            stats.copied_files += 1;
            continue;
        }

        if file_type.is_symlink() {
            stats.skipped_symlinks += 1;
        }
    }

    Ok(())
}
