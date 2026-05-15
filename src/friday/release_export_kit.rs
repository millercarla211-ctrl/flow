use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    FridayDashboardPanelStatus, read_friday_release_operator_checklist,
    read_friday_release_operator_signoffs, read_friday_release_qa_command_center_report,
    read_friday_trusted_runner_release_package, read_friday_trusted_runner_release_timeline,
};

const STALE_AFTER_MS: u128 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseEvidenceExportKitFile {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub path: String,
    pub required: bool,
    pub present: bool,
    pub stale: bool,
    pub bytes: u64,
    pub sha256: Option<String>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseEvidenceExportKitManifest {
    pub kit_id: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub kit_json: String,
    pub export_dir: String,
    pub file_count: usize,
    pub required_count: usize,
    pub missing_count: usize,
    pub stale_count: usize,
    pub warning_count: usize,
    pub manifest_sha256: String,
    pub commands: Vec<String>,
    pub files: Vec<FridayReleaseEvidenceExportKitFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseEvidenceExportKitReport {
    pub summary: String,
    pub ready_to_attach: bool,
    pub status: FridayDashboardPanelStatus,
    pub checklist_ready: Option<bool>,
    pub qa_score_out_of_100: Option<u8>,
    pub qa_ready_to_ship: Option<bool>,
    pub package_ready_to_ship: Option<bool>,
    pub timeline_package_count: Option<usize>,
    pub signoff_count: usize,
    pub warnings: Vec<String>,
    pub operator_copy: String,
    pub manifest: FridayReleaseEvidenceExportKitManifest,
}

impl FridayReleaseEvidenceExportKitReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn friday_release_evidence_export_kit_report(
    kit_path: impl AsRef<Path>,
    export_dir: impl AsRef<Path>,
    checklist_path: impl AsRef<Path>,
    qa_path: impl AsRef<Path>,
    package_path: impl AsRef<Path>,
    timeline_path: impl AsRef<Path>,
    signoff_path: impl AsRef<Path>,
    cargo_check_result_path: impl AsRef<Path>,
    extension_typecheck_result_path: impl AsRef<Path>,
    dashboard_smoke_result_path: impl AsRef<Path>,
) -> FridayReleaseEvidenceExportKitReport {
    let kit_path = kit_path.as_ref();
    let export_dir = export_dir.as_ref();
    let checklist_path = checklist_path.as_ref();
    let qa_path = qa_path.as_ref();
    let package_path = package_path.as_ref();
    let timeline_path = timeline_path.as_ref();
    let signoff_path = signoff_path.as_ref();
    let cargo_check_result_path = cargo_check_result_path.as_ref();
    let extension_typecheck_result_path = extension_typecheck_result_path.as_ref();
    let dashboard_smoke_result_path = dashboard_smoke_result_path.as_ref();
    let generated_at_unix_ms = unix_ms();

    let checklist = read_friday_release_operator_checklist(checklist_path).ok();
    let qa = read_friday_release_qa_command_center_report(qa_path).ok();
    let package = read_friday_trusted_runner_release_package(package_path).ok();
    let timeline = read_friday_trusted_runner_release_timeline(timeline_path).ok();
    let signoffs = read_friday_release_operator_signoffs(signoff_path).unwrap_or_default();

    let files = vec![
        evidence_file(
            "release-checklist",
            "Release operator checklist",
            "release-checklist-json",
            checklist_path,
            true,
            false,
        ),
        evidence_file(
            "release-qa",
            "Release QA command center",
            "release-qa-json",
            qa_path,
            true,
            false,
        ),
        evidence_file(
            "release-package",
            "Trusted runner release package",
            "release-package-json",
            package_path,
            true,
            false,
        ),
        evidence_file(
            "release-timeline",
            "Trusted runner evidence timeline",
            "release-timeline-json",
            timeline_path,
            true,
            false,
        ),
        evidence_file(
            "release-signoffs",
            "Release signoffs",
            "release-signoffs-json",
            signoff_path,
            true,
            false,
        ),
        evidence_file(
            "rust-cargo-check-result",
            "Rust cargo check result",
            "check-result",
            cargo_check_result_path,
            true,
            true,
        ),
        evidence_file(
            "extension-typecheck-result",
            "Extension TypeScript typecheck result",
            "check-result",
            extension_typecheck_result_path,
            true,
            true,
        ),
        evidence_file(
            "dashboard-smoke-result",
            "Dashboard smoke result",
            "check-result",
            dashboard_smoke_result_path,
            true,
            true,
        ),
    ];

    let required_count = files.iter().filter(|file| file.required).count();
    let missing_count = files
        .iter()
        .filter(|file| file.required && !file.present)
        .count();
    let stale_count = files.iter().filter(|file| file.stale).count();
    let mut warnings = files
        .iter()
        .filter_map(|file| file.warning.clone())
        .collect::<Vec<_>>();

    if checklist.is_none() {
        warnings.push("Release checklist could not be parsed.".to_string());
    }
    if qa.is_none() {
        warnings.push("Release QA command-center report could not be parsed.".to_string());
    }
    if package.is_none() {
        warnings.push("Trusted runner release package could not be parsed.".to_string());
    }
    if timeline.is_none() {
        warnings.push("Trusted runner evidence timeline could not be parsed.".to_string());
    }

    if let Some(checklist) = &checklist {
        if !checklist.ready_to_ship {
            warnings.push(format!(
                "Release checklist is not ready: {} blocking issue(s), {} warning(s).",
                checklist.blocking_count, checklist.warning_count
            ));
        }
    }
    if let Some(qa) = &qa {
        if !qa.ready_to_ship {
            warnings.push(format!(
                "Release QA is not ready: score {} / 100, {} blocking issue(s), {} stale result(s).",
                qa.score_out_of_100, qa.blocking_count, qa.stale_count
            ));
        }
    }
    if let Some(package) = &package {
        if !package.ready_to_ship {
            warnings.push(format!(
                "Release package is not ready: {} missing item(s), {} warning(s).",
                package.manifest.missing_count, package.manifest.warning_count
            ));
        }
    }
    if let Some(timeline) = &timeline {
        if timeline.missing_evidence_regressions > 0 || timeline.warning_regressions > 0 {
            warnings.push(format!(
                "Release timeline has {} missing-evidence regression(s) and {} warning regression(s).",
                timeline.missing_evidence_regressions, timeline.warning_regressions
            ));
        }
    }
    if signoffs.is_empty() {
        warnings.push("No release signoff is attached to the export kit.".to_string());
    }

    warnings.sort();
    warnings.dedup();

    let warning_count = warnings.len();
    let manifest_sha256 = manifest_signature(&files, &warnings);
    let kit_json = path_string(kit_path);
    let commands = vec![
        "cargo check > tmp/friday-dashboard/cargo-check.txt".to_string(),
        "cd extensions/flow-webext && npm run typecheck > ../../tmp/friday-dashboard/extension-typecheck.txt".to_string(),
        "cd extensions/flow-webext && npm run smoke:dashboard > ../../tmp/friday-dashboard/dashboard-smoke.txt".to_string(),
        format!(
            "flow --friday-release-export-kit --export-dir {} --output {}",
            path_string(export_dir),
            kit_json
        ),
    ];
    let ready_to_attach = missing_count == 0
        && stale_count == 0
        && warning_count == 0
        && checklist
            .as_ref()
            .is_some_and(|checklist| checklist.ready_to_ship)
        && qa.as_ref().is_some_and(|qa| qa.ready_to_ship)
        && package
            .as_ref()
            .is_some_and(|package| package.ready_to_ship)
        && timeline.as_ref().is_some_and(|timeline| {
            timeline.missing_evidence_regressions == 0 && timeline.warning_regressions == 0
        })
        && !signoffs.is_empty();
    let status = if missing_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if stale_count > 0 || warning_count > 0 {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };

    let manifest = FridayReleaseEvidenceExportKitManifest {
        kit_id: format!("friday-release-export-kit-{generated_at_unix_ms}"),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        kit_json: kit_json.clone(),
        export_dir: path_string(export_dir),
        file_count: files.len(),
        required_count,
        missing_count,
        stale_count,
        warning_count,
        manifest_sha256: manifest_sha256.clone(),
        commands,
        files,
    };
    let operator_copy = operator_copy(&manifest, ready_to_attach);

    FridayReleaseEvidenceExportKitReport {
        summary: format!(
            "Friday release evidence kit has {} file(s), {} missing, {} stale, and {} warning(s).",
            manifest.file_count,
            manifest.missing_count,
            manifest.stale_count,
            manifest.warning_count
        ),
        ready_to_attach,
        status,
        checklist_ready: checklist.as_ref().map(|checklist| checklist.ready_to_ship),
        qa_score_out_of_100: qa.as_ref().map(|qa| qa.score_out_of_100),
        qa_ready_to_ship: qa.as_ref().map(|qa| qa.ready_to_ship),
        package_ready_to_ship: package.as_ref().map(|package| package.ready_to_ship),
        timeline_package_count: timeline.as_ref().map(|timeline| timeline.package_count),
        signoff_count: signoffs.len(),
        warnings,
        operator_copy,
        manifest,
    }
}

pub fn write_friday_release_evidence_export_kit(
    kit_path: impl AsRef<Path>,
    report: &FridayReleaseEvidenceExportKitReport,
) -> Result<()> {
    let kit_path = kit_path.as_ref();
    if let Some(parent) = kit_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release evidence export-kit directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(kit_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release evidence export kit {}",
            kit_path.display()
        )
    })
}

pub fn read_friday_release_evidence_export_kit(
    kit_path: impl AsRef<Path>,
) -> Result<FridayReleaseEvidenceExportKitReport> {
    let kit_path = kit_path.as_ref();
    let bytes = fs::read(kit_path).with_context(|| {
        format!(
            "Could not read Friday release evidence export kit {}",
            kit_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release evidence export kit {}",
            kit_path.display()
        )
    })
}

fn evidence_file(
    id: &str,
    label: &str,
    kind: &str,
    path: &Path,
    required: bool,
    stale_checked: bool,
) -> FridayReleaseEvidenceExportKitFile {
    match fs::read(path) {
        Ok(bytes) => {
            let stale = stale_checked && file_is_stale(path);
            FridayReleaseEvidenceExportKitFile {
                id: id.to_string(),
                label: label.to_string(),
                kind: kind.to_string(),
                path: path_string(path),
                required,
                present: true,
                stale,
                bytes: bytes.len() as u64,
                sha256: Some(sha256_hex(&bytes)),
                warning: stale.then(|| {
                    format!("{label} is older than 24 hours; refresh it before a major checkpoint.")
                }),
            }
        }
        Err(_) => FridayReleaseEvidenceExportKitFile {
            id: id.to_string(),
            label: label.to_string(),
            kind: kind.to_string(),
            path: path_string(path),
            required,
            present: false,
            stale: false,
            bytes: 0,
            sha256: None,
            warning: required.then(|| format!("Required evidence is missing: {label}.")),
        },
    }
}

fn file_is_stale(path: &Path) -> bool {
    fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| unix_ms().saturating_sub(duration.as_millis()) > STALE_AFTER_MS)
        .unwrap_or(false)
}

fn manifest_signature(files: &[FridayReleaseEvidenceExportKitFile], warnings: &[String]) -> String {
    let mut input = files
        .iter()
        .map(|file| {
            format!(
                "{}:{}:{}:{}:{}",
                file.id,
                file.path,
                file.present,
                file.bytes,
                file.sha256.as_deref().unwrap_or("missing")
            )
        })
        .collect::<Vec<_>>();
    input.extend(warnings.iter().map(|warning| format!("warning:{warning}")));
    input.sort();
    sha256_hex(input.join("\n").as_bytes())
}

fn operator_copy(
    manifest: &FridayReleaseEvidenceExportKitManifest,
    ready_to_attach: bool,
) -> String {
    format!(
        "Friday release evidence kit: {}\nStatus: {}\nManifest checksum: {}\nFiles: {} total, {} missing, {} stale, {} warning(s).\nAttach this JSON plus the referenced local evidence files to the checkpoint or deployment note.",
        manifest.kit_json,
        if ready_to_attach {
            "ready to attach"
        } else {
            "needs review"
        },
        manifest.manifest_sha256,
        manifest.file_count,
        manifest.missing_count,
        manifest.stale_count,
        manifest.warning_count
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
