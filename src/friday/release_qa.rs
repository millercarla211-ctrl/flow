use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseOperatorChecklistReport,
    FridayTrustedRunnerReleasePackageReport, FridayTrustedRunnerReleaseTimeline,
    read_friday_release_operator_checklist, read_friday_trusted_runner_release_package,
    read_friday_trusted_runner_release_timeline,
};

const STALE_AFTER_MS: u128 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseQaCheckStatus {
    Passed,
    Warning,
    Failed,
    Missing,
    Stale,
}

impl FridayReleaseQaCheckStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
            Self::Missing => "missing",
            Self::Stale => "stale",
        }
    }

    fn score_multiplier(self) -> f32 {
        match self {
            Self::Passed => 1.0,
            Self::Warning | Self::Stale => 0.5,
            Self::Failed | Self::Missing => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseQaCheck {
    pub id: String,
    pub label: String,
    pub command: String,
    pub result_path: String,
    pub required: bool,
    pub present: bool,
    pub stale: bool,
    pub bytes: u64,
    pub status: FridayReleaseQaCheckStatus,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseQaCommandCenterReport {
    pub report_id: String,
    pub report_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub ready_to_ship: bool,
    pub summary: String,
    pub checklist_json: String,
    pub package_json: String,
    pub timeline_json: String,
    pub warning_count: usize,
    pub blocking_count: usize,
    pub stale_count: usize,
    pub missing_count: usize,
    pub checks: Vec<FridayReleaseQaCheck>,
    pub commands: Vec<String>,
}

impl FridayReleaseQaCommandCenterReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_qa_command_center_report(
    report_path: impl AsRef<Path>,
    checklist_path: impl AsRef<Path>,
    package_path: impl AsRef<Path>,
    timeline_path: impl AsRef<Path>,
    cargo_check_result_path: impl AsRef<Path>,
    extension_typecheck_result_path: impl AsRef<Path>,
    dashboard_smoke_result_path: impl AsRef<Path>,
) -> FridayReleaseQaCommandCenterReport {
    let report_path = report_path.as_ref();
    let checklist_path = checklist_path.as_ref();
    let package_path = package_path.as_ref();
    let timeline_path = timeline_path.as_ref();
    let cargo_check_result_path = cargo_check_result_path.as_ref();
    let extension_typecheck_result_path = extension_typecheck_result_path.as_ref();
    let dashboard_smoke_result_path = dashboard_smoke_result_path.as_ref();
    let checklist = read_friday_release_operator_checklist(checklist_path).ok();
    let package = read_friday_trusted_runner_release_package(package_path).ok();
    let timeline = read_friday_trusted_runner_release_timeline(timeline_path).ok();
    let checks = vec![
        checklist_check(checklist_path, checklist.as_ref()),
        package_check(package_path, package.as_ref()),
        timeline_check(timeline_path, timeline.as_ref()),
        result_file_check(
            "rust-cargo-check",
            "Rust cargo check",
            "cargo check",
            cargo_check_result_path,
            true,
        ),
        result_file_check(
            "extension-typecheck",
            "Extension TypeScript typecheck",
            "npm run typecheck",
            extension_typecheck_result_path,
            true,
        ),
        result_file_check(
            "dashboard-smoke",
            "Dashboard smoke",
            "npm run smoke:dashboard",
            dashboard_smoke_result_path,
            true,
        ),
    ];
    let warning_count = checks
        .iter()
        .filter(|check| {
            matches!(
                check.status,
                FridayReleaseQaCheckStatus::Warning | FridayReleaseQaCheckStatus::Stale
            )
        })
        .count();
    let blocking_count = checks
        .iter()
        .filter(|check| {
            matches!(
                check.status,
                FridayReleaseQaCheckStatus::Failed | FridayReleaseQaCheckStatus::Missing
            )
        })
        .count();
    let stale_count = checks.iter().filter(|check| check.stale).count();
    let missing_count = checks.iter().filter(|check| !check.present).count();
    let score_out_of_100 = score_checks(&checks);
    let ready_to_ship = blocking_count == 0
        && warning_count == 0
        && checklist
            .as_ref()
            .is_some_and(|checklist| checklist.ready_to_ship);
    let status = if blocking_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if warning_count > 0 || !ready_to_ship {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };

    FridayReleaseQaCommandCenterReport {
        report_id: format!("friday-release-qa-{}", unix_ms()),
        report_json: path_string(report_path),
        generated_at_unix_ms: unix_ms(),
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        score_out_of_100,
        ready_to_ship,
        summary: format!(
            "Friday release QA is {score_out_of_100}/100 with {warning_count} warning(s), {blocking_count} blocking issue(s), and {stale_count} stale result(s)."
        ),
        checklist_json: path_string(checklist_path),
        package_json: path_string(package_path),
        timeline_json: path_string(timeline_path),
        warning_count,
        blocking_count,
        stale_count,
        missing_count,
        checks,
        commands: vec![
            "cargo check > tmp/friday-dashboard/cargo-check.txt".to_string(),
            "cd extensions/flow-webext && npm run typecheck > ../../tmp/friday-dashboard/extension-typecheck.txt".to_string(),
            "cd extensions/flow-webext && npm run smoke:dashboard > ../../tmp/friday-dashboard/dashboard-smoke.txt".to_string(),
            format!(
                "flow --friday-release-qa --checklist {} --package {} --timeline {} --output {}",
                path_string(checklist_path),
                path_string(package_path),
                path_string(timeline_path),
                path_string(report_path)
            ),
        ],
    }
}

pub fn write_friday_release_qa_command_center_report(
    report_path: impl AsRef<Path>,
    report: &FridayReleaseQaCommandCenterReport,
) -> anyhow::Result<()> {
    let report_path = report_path.as_ref();
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(report_path, report.to_pretty_json()?)?;
    Ok(())
}

fn checklist_check(
    path: &Path,
    checklist: Option<&FridayReleaseOperatorChecklistReport>,
) -> FridayReleaseQaCheck {
    match checklist {
        Some(checklist) => {
            let status = if checklist.ready_to_ship {
                FridayReleaseQaCheckStatus::Passed
            } else if checklist.blocking_count > 0 {
                FridayReleaseQaCheckStatus::Failed
            } else {
                FridayReleaseQaCheckStatus::Warning
            };
            source_check(
                "release-checklist",
                "Release operator checklist",
                "flow --friday-release-checklist",
                path,
                true,
                status,
                format!(
                    "{} / {} checklist item(s) ready; {} blocking.",
                    checklist.ready_count, checklist.total_count, checklist.blocking_count
                ),
                "Resolve checklist blockers and record operator signoff before shipping.",
            )
        }
        None => source_check(
            "release-checklist",
            "Release operator checklist",
            "flow --friday-release-checklist",
            path,
            true,
            FridayReleaseQaCheckStatus::Missing,
            "Release checklist JSON is missing or unreadable.",
            "Generate the release checklist before running the QA command center.",
        ),
    }
}

fn package_check(
    path: &Path,
    package: Option<&FridayTrustedRunnerReleasePackageReport>,
) -> FridayReleaseQaCheck {
    match package {
        Some(package) => source_check(
            "release-package",
            "Trusted runner release package",
            "flow --friday-trusted-host-runner-release-package",
            path,
            true,
            if package.ready_to_ship {
                FridayReleaseQaCheckStatus::Passed
            } else if package.manifest.missing_count > 0 {
                FridayReleaseQaCheckStatus::Failed
            } else {
                FridayReleaseQaCheckStatus::Warning
            },
            format!(
                "{} evidence item(s), {} missing, {} warning(s).",
                package.manifest.evidence_count,
                package.manifest.missing_count,
                package.manifest.warning_count
            ),
            "Refresh the release package after runner evidence changes.",
        ),
        None => source_check(
            "release-package",
            "Trusted runner release package",
            "flow --friday-trusted-host-runner-release-package",
            path,
            true,
            FridayReleaseQaCheckStatus::Missing,
            "Release package JSON is missing or unreadable.",
            "Generate the trusted runner release package before QA signoff.",
        ),
    }
}

fn timeline_check(
    path: &Path,
    timeline: Option<&FridayTrustedRunnerReleaseTimeline>,
) -> FridayReleaseQaCheck {
    match timeline {
        Some(timeline) => source_check(
            "release-timeline",
            "Trusted runner evidence timeline",
            "flow --friday-trusted-runner-release-timeline",
            path,
            true,
            if timeline.missing_evidence_regressions > 0 {
                FridayReleaseQaCheckStatus::Failed
            } else if timeline.warning_regressions > 0 {
                FridayReleaseQaCheckStatus::Warning
            } else {
                FridayReleaseQaCheckStatus::Passed
            },
            format!(
                "{} package(s), {} missing-evidence regression(s), {} warning regression(s).",
                timeline.package_count,
                timeline.missing_evidence_regressions,
                timeline.warning_regressions
            ),
            "Archive the latest package and resolve regressions before release signoff.",
        ),
        None => source_check(
            "release-timeline",
            "Trusted runner evidence timeline",
            "flow --friday-trusted-runner-release-timeline",
            path,
            true,
            FridayReleaseQaCheckStatus::Missing,
            "Release timeline JSON is missing or unreadable.",
            "Create the release evidence timeline before QA signoff.",
        ),
    }
}

fn result_file_check(
    id: &str,
    label: &str,
    command: &str,
    result_path: &Path,
    required: bool,
) -> FridayReleaseQaCheck {
    match fs::metadata(result_path) {
        Ok(metadata) => {
            let modified_ms = metadata
                .modified()
                .ok()
                .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_millis())
                .unwrap_or_default();
            let stale = unix_ms().saturating_sub(modified_ms) > STALE_AFTER_MS;
            source_check(
                id,
                label,
                command,
                result_path,
                required,
                if stale {
                    FridayReleaseQaCheckStatus::Stale
                } else {
                    FridayReleaseQaCheckStatus::Passed
                },
                format!("{} byte(s) captured from the latest lightweight check.", metadata.len()),
                "Refresh this check result after meaningful code changes.",
            )
        }
        Err(_) => source_check(
            id,
            label,
            command,
            result_path,
            required,
            if required {
                FridayReleaseQaCheckStatus::Missing
            } else {
                FridayReleaseQaCheckStatus::Warning
            },
            "Check result file is missing.",
            "Run the command and save its output before release QA signoff.",
        ),
    }
}

fn source_check(
    id: &str,
    label: &str,
    command: &str,
    result_path: &Path,
    required: bool,
    status: FridayReleaseQaCheckStatus,
    summary: impl Into<String>,
    next_action: &str,
) -> FridayReleaseQaCheck {
    let metadata = fs::metadata(result_path).ok();
    FridayReleaseQaCheck {
        id: id.to_string(),
        label: label.to_string(),
        command: command.to_string(),
        result_path: path_string(result_path),
        required,
        present: metadata.is_some(),
        stale: status == FridayReleaseQaCheckStatus::Stale,
        bytes: metadata.map(|metadata| metadata.len()).unwrap_or_default(),
        status,
        summary: summary.into(),
        next_action: next_action.to_string(),
    }
}

fn score_checks(checks: &[FridayReleaseQaCheck]) -> u8 {
    let possible = checks.len() as f32;
    if possible <= f32::EPSILON {
        return 0;
    }
    let earned = checks
        .iter()
        .map(|check| check.status.score_multiplier())
        .sum::<f32>();
    ((earned / possible) * 100.0).round().clamp(0.0, 100.0) as u8
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
