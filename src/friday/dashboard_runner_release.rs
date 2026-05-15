use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    FridayDashboardReleaseReviewHandoff, FridayTrustedHostLiveRunnerState,
    FridayTrustedHostRunnerCancellationUxReport, FridayTrustedHostRunnerOperatorReviewFilter,
    FridayTrustedHostRunnerOperatorReviewReport, friday_dashboard_release_review_from_export,
    friday_trusted_host_runner_cancellation_ux_report,
    friday_trusted_host_runner_operator_review_report, read_friday_trusted_host_live_runner_state,
    read_friday_trusted_host_runner_history,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedRunnerReleaseEvidenceFile {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub path: String,
    pub required: bool,
    pub present: bool,
    pub bytes: u64,
    pub sha256: Option<String>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedRunnerReleasePackageManifest {
    pub package_id: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub package_json: String,
    pub dashboard_export_dir: String,
    pub history_json: String,
    pub live_state_json: String,
    pub release_review_json: String,
    pub dashboard_index_json: String,
    pub evidence_count: usize,
    pub missing_count: usize,
    pub warning_count: usize,
    pub package_signature: String,
    pub commands: Vec<String>,
    pub files: Vec<FridayTrustedRunnerReleaseEvidenceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedRunnerReleasePackageReport {
    pub summary: String,
    pub ready_to_ship: bool,
    pub warnings: Vec<String>,
    pub manifest: FridayTrustedRunnerReleasePackageManifest,
    pub operator_review: Option<FridayTrustedHostRunnerOperatorReviewReport>,
    pub cancellation_ux: Option<FridayTrustedHostRunnerCancellationUxReport>,
    pub live_state: Option<FridayTrustedHostLiveRunnerState>,
    pub release_review: Option<FridayDashboardReleaseReviewHandoff>,
    pub incident_markdown: String,
}

impl FridayTrustedRunnerReleasePackageReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_trusted_runner_release_package_report(
    dashboard_export_dir: impl AsRef<Path>,
    history_path: impl AsRef<Path>,
    live_state_path: impl AsRef<Path>,
    package_path: impl AsRef<Path>,
) -> FridayTrustedRunnerReleasePackageReport {
    let dashboard_export_dir = dashboard_export_dir.as_ref();
    let history_path = history_path.as_ref();
    let live_state_path = live_state_path.as_ref();
    let package_path = package_path.as_ref();
    let generated_at_unix_ms = unix_ms();

    let history = read_friday_trusted_host_runner_history(history_path).ok();
    let operator_review = history
        .as_ref()
        .map(|history| friday_trusted_host_runner_operator_review_report(
            history,
            FridayTrustedHostRunnerOperatorReviewFilter::default(),
        ));
    let live_state = read_friday_trusted_host_live_runner_state(live_state_path).ok();
    let cancellation_ux = live_state
        .as_ref()
        .map(friday_trusted_host_runner_cancellation_ux_report);
    let release_review = friday_dashboard_release_review_from_export(dashboard_export_dir).ok();
    let incident_markdown = operator_review
        .as_ref()
        .map(incident_markdown)
        .unwrap_or_default();

    let dashboard_manifest_path = dashboard_export_dir.join("manifest.json");
    let release_review_path = dashboard_export_dir.join("release-review.json");
    let dashboard_index_path = dashboard_export_dir.join("dashboard-index.json");
    let mut files = vec![
        evidence_file(
            "runner-history",
            "Trusted runner history",
            "runner-history-json",
            history_path,
            true,
        ),
        evidence_file(
            "runner-live-state",
            "Trusted runner live state",
            "runner-live-state-json",
            live_state_path,
            true,
        ),
        evidence_file(
            "release-review",
            "Friday release review",
            "release-review-json",
            &release_review_path,
            true,
        ),
        evidence_file(
            "dashboard-index",
            "Dashboard index",
            "dashboard-json",
            &dashboard_index_path,
            true,
        ),
        evidence_file(
            "dashboard-manifest",
            "Dashboard manifest",
            "manifest-json",
            &dashboard_manifest_path,
            true,
        ),
        evidence_virtual(
            "incident-notes",
            "Trusted runner incident notes",
            "incident-markdown",
            "trusted-runner-incidents.md",
            &incident_markdown,
            false,
        ),
    ];
    let mut warnings = files
        .iter()
        .filter_map(|file| file.warning.clone())
        .collect::<Vec<_>>();
    if let Some(review) = &operator_review {
        if review.blocked_count > 0 {
            warnings.push(format!(
                "{} trusted runner record(s) still block release review.",
                review.blocked_count
            ));
        }
    } else {
        warnings.push("Trusted runner history could not be read for release packaging.".to_string());
    }
    if let Some(live_state) = &live_state {
        if live_state.stale_count > 0 {
            warnings.push(format!(
                "{} stale live runner record(s) require cleanup before release.",
                live_state.stale_count
            ));
        }
        if live_state.pending_count + live_state.running_count > 0 {
            warnings.push("Live trusted runner work is still pending or running.".to_string());
        }
    } else {
        warnings.push("Trusted runner live-state JSON is missing from the package.".to_string());
    }
    if release_review.is_none() {
        warnings.push("Friday release-review JSON is missing from the package.".to_string());
    }

    let missing_count = files.iter().filter(|file| file.required && !file.present).count();
    let warning_count = warnings.len();
    let package_signature = package_signature(&files, &warnings);
    let ready_to_ship = missing_count == 0 && warning_count == 0;
    let package_json = path_string(package_path);
    files.push(evidence_virtual(
        "package-signature",
        "Release package signature",
        "signature",
        &package_json,
        &package_signature,
        true,
    ));

    let manifest = FridayTrustedRunnerReleasePackageManifest {
        package_id: format!("trusted-runner-release-{generated_at_unix_ms}"),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        package_json,
        dashboard_export_dir: path_string(dashboard_export_dir),
        history_json: path_string(history_path),
        live_state_json: path_string(live_state_path),
        release_review_json: path_string(&release_review_path),
        dashboard_index_json: path_string(&dashboard_index_path),
        evidence_count: files.len(),
        missing_count,
        warning_count,
        package_signature,
        commands: vec![
            format!(
                "flow --friday-trusted-host-runner-release-package {} --history {} --state {} --output {}",
                path_string(dashboard_export_dir),
                path_string(history_path),
                path_string(live_state_path),
                path_string(package_path)
            ),
            format!(
                "flow --friday-trusted-host-runner-review-json {}",
                path_string(history_path)
            ),
        ],
        files,
    };

    FridayTrustedRunnerReleasePackageReport {
        summary: if ready_to_ship {
            "Trusted runner release package is complete and ready for local release review."
                .to_string()
        } else {
            format!(
                "Trusted runner release package needs review: {missing_count} missing evidence item(s), {warning_count} warning(s)."
            )
        },
        ready_to_ship,
        warnings,
        manifest,
        operator_review,
        cancellation_ux,
        live_state,
        release_review,
        incident_markdown,
    }
}

pub fn write_friday_trusted_runner_release_package(
    package_path: impl AsRef<Path>,
    report: &FridayTrustedRunnerReleasePackageReport,
) -> Result<()> {
    let package_path = package_path.as_ref();
    if let Some(parent) = package_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create trusted runner release package directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(package_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write trusted runner release package {}",
            package_path.display()
        )
    })
}

fn incident_markdown(review: &FridayTrustedHostRunnerOperatorReviewReport) -> String {
    review
        .incident_notes
        .iter()
        .map(|note| note.export_markdown.trim())
        .filter(|note| !note.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn evidence_file(
    id: &str,
    label: &str,
    kind: &str,
    path: &Path,
    required: bool,
) -> FridayTrustedRunnerReleaseEvidenceFile {
    match fs::read(path) {
        Ok(bytes) => FridayTrustedRunnerReleaseEvidenceFile {
            id: id.to_string(),
            label: label.to_string(),
            kind: kind.to_string(),
            path: path_string(path),
            required,
            present: true,
            bytes: bytes.len() as u64,
            sha256: Some(sha256_hex(&bytes)),
            warning: None,
        },
        Err(_) => FridayTrustedRunnerReleaseEvidenceFile {
            id: id.to_string(),
            label: label.to_string(),
            kind: kind.to_string(),
            path: path_string(path),
            required,
            present: false,
            bytes: 0,
            sha256: None,
            warning: required.then(|| format!("Required evidence `{label}` is missing.")),
        },
    }
}

fn evidence_virtual(
    id: &str,
    label: &str,
    kind: &str,
    path: &str,
    content: &str,
    required: bool,
) -> FridayTrustedRunnerReleaseEvidenceFile {
    let present = !content.trim().is_empty();
    FridayTrustedRunnerReleaseEvidenceFile {
        id: id.to_string(),
        label: label.to_string(),
        kind: kind.to_string(),
        path: path.to_string(),
        required,
        present,
        bytes: content.len() as u64,
        sha256: present.then(|| sha256_hex(content.as_bytes())),
        warning: (required && !present).then(|| format!("Required evidence `{label}` is empty.")),
    }
}

fn package_signature(files: &[FridayTrustedRunnerReleaseEvidenceFile], warnings: &[String]) -> String {
    let mut input = files
        .iter()
        .map(|file| {
            format!(
                "{}:{}:{}:{}",
                file.id,
                file.path,
                file.bytes,
                file.sha256.as_deref().unwrap_or("missing")
            )
        })
        .collect::<Vec<_>>();
    input.extend(warnings.iter().map(|warning| format!("warning:{warning}")));
    input.sort();
    sha256_hex(input.join("\n").as_bytes())
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
