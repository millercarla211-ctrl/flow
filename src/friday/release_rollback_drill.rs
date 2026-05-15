use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseCandidateArchive, FridayReleaseDeploymentGateDecision,
    FridayReleaseDeploymentGateReport, FridayReleasePostPromotionMonitorReport,
    FridayReleasePromotionDecision, FridayReleasePromotionLedger,
    read_friday_release_candidate_archive, read_friday_release_deployment_gate,
    read_friday_release_post_promotion_monitor_report, read_friday_release_promotion_ledger,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseRollbackDrillCheckStatus {
    Passed,
    Warning,
    Failed,
    Missing,
    Stale,
}

impl FridayReleaseRollbackDrillCheckStatus {
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
pub struct FridayReleaseRollbackDrillCheck {
    pub id: String,
    pub label: String,
    pub source_path: String,
    pub required: bool,
    pub present: bool,
    pub stale: bool,
    pub bytes: u64,
    pub status: FridayReleaseRollbackDrillCheckStatus,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseRollbackDrillReport {
    pub drill_id: String,
    pub drill_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub ready_to_rollback: bool,
    pub ready_for_stable: bool,
    pub active_candidate_id: Option<String>,
    pub active_promotion_id: Option<String>,
    pub active_rollback_reference: Option<String>,
    pub latest_promotion_decision: Option<FridayReleasePromotionDecision>,
    pub deployment_gate_decision: Option<FridayReleaseDeploymentGateDecision>,
    pub post_promotion_monitor_json: String,
    pub promotion_ledger_json: String,
    pub candidate_archive_json: String,
    pub deployment_gate_json: String,
    pub rollback_command: String,
    pub dry_run_command: String,
    pub operator: String,
    pub reason: String,
    pub blocking_count: usize,
    pub warning_count: usize,
    pub stale_count: usize,
    pub missing_evidence_count: usize,
    pub checks: Vec<FridayReleaseRollbackDrillCheck>,
    pub blocked_reasons: Vec<String>,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseRollbackDrillReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_rollback_drill_report(
    drill_path: impl AsRef<Path>,
    post_promotion_monitor_path: impl AsRef<Path>,
    promotion_ledger_path: impl AsRef<Path>,
    candidate_archive_path: impl AsRef<Path>,
    deployment_gate_path: impl AsRef<Path>,
    rollback_command: impl Into<String>,
    operator: impl Into<String>,
    reason: impl Into<String>,
) -> FridayReleaseRollbackDrillReport {
    let drill_path = drill_path.as_ref();
    let post_promotion_monitor_path = post_promotion_monitor_path.as_ref();
    let promotion_ledger_path = promotion_ledger_path.as_ref();
    let candidate_archive_path = candidate_archive_path.as_ref();
    let deployment_gate_path = deployment_gate_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let rollback_command = rollback_command.into();
    let dry_run_command = dry_run_command(&rollback_command);
    let operator = operator.into();
    let reason = reason.into();

    let monitor =
        read_friday_release_post_promotion_monitor_report(post_promotion_monitor_path).ok();
    let ledger = read_friday_release_promotion_ledger(promotion_ledger_path).ok();
    let archive = read_friday_release_candidate_archive(candidate_archive_path).ok();
    let gate = read_friday_release_deployment_gate(deployment_gate_path).ok();

    let active_rollback_reference = monitor
        .as_ref()
        .and_then(|monitor| monitor.active_rollback_reference.clone())
        .or_else(|| {
            ledger
                .as_ref()
                .and_then(|ledger| ledger.active_rollback_reference.clone())
        });
    let active_candidate_id = monitor
        .as_ref()
        .and_then(|monitor| monitor.active_candidate_id.clone())
        .or_else(|| {
            ledger
                .as_ref()
                .and_then(|ledger| ledger.active_candidate_id.clone())
        })
        .or_else(|| {
            archive
                .as_ref()
                .and_then(|archive| archive.latest_candidate_id.clone())
        });
    let active_promotion_id = monitor
        .as_ref()
        .and_then(|monitor| monitor.active_promotion_id.clone())
        .or_else(|| {
            ledger
                .as_ref()
                .and_then(|ledger| ledger.active_promotion_id.clone())
        });
    let latest_promotion_decision = monitor
        .as_ref()
        .and_then(|monitor| monitor.latest_decision)
        .or_else(|| ledger.as_ref().and_then(|ledger| ledger.latest_decision));
    let deployment_gate_decision = gate
        .as_ref()
        .map(|gate| gate.decision)
        .or_else(|| archive.as_ref().and_then(|archive| archive.latest_decision));

    let mut checks = vec![
        post_promotion_monitor_check(post_promotion_monitor_path, monitor.as_ref()),
        promotion_ledger_check(promotion_ledger_path, ledger.as_ref()),
        candidate_archive_check(candidate_archive_path, archive.as_ref()),
        deployment_gate_check(deployment_gate_path, gate.as_ref()),
        rollback_reference_check(active_rollback_reference.as_deref()),
        dry_run_command_check(&dry_run_command),
    ];

    checks.sort_by(|left, right| left.id.cmp(&right.id));

    let blocking_count = checks
        .iter()
        .filter(|check| {
            matches!(
                check.status,
                FridayReleaseRollbackDrillCheckStatus::Failed
                    | FridayReleaseRollbackDrillCheckStatus::Missing
            )
        })
        .count();
    let warning_count = checks
        .iter()
        .filter(|check| check.status == FridayReleaseRollbackDrillCheckStatus::Warning)
        .count();
    let stale_count = checks
        .iter()
        .filter(|check| check.status == FridayReleaseRollbackDrillCheckStatus::Stale)
        .count();
    let missing_evidence_count = checks.iter().filter(|check| !check.present).count();
    let score_out_of_100 = score_checks(&checks);
    let ready_to_rollback = active_rollback_reference
        .as_ref()
        .is_some_and(|reference| !reference.trim().is_empty())
        && !dry_run_command.trim().is_empty()
        && blocking_count == 0
        && stale_count == 0;
    let ready_for_stable = ready_to_rollback
        && monitor
            .as_ref()
            .is_some_and(|monitor| monitor.ready_for_stable)
        && gate.as_ref().is_some_and(|gate| gate.ready_to_deploy);
    let status = if blocking_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if warning_count > 0 || stale_count > 0 {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };
    let blocked_reasons = checks
        .iter()
        .filter(|check| {
            matches!(
                check.status,
                FridayReleaseRollbackDrillCheckStatus::Failed
                    | FridayReleaseRollbackDrillCheckStatus::Missing
                    | FridayReleaseRollbackDrillCheckStatus::Stale
            )
        })
        .map(|check| format!("{}: {}", check.label, check.summary))
        .collect::<Vec<_>>();
    let drill_json = path_string(drill_path);
    let post_promotion_monitor_json = path_string(post_promotion_monitor_path);
    let promotion_ledger_json = path_string(promotion_ledger_path);
    let candidate_archive_json = path_string(candidate_archive_path);
    let deployment_gate_json = path_string(deployment_gate_path);

    FridayReleaseRollbackDrillReport {
        drill_id: format!("friday-release-rollback-drill-{generated_at_unix_ms}"),
        drill_json: drill_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        score_out_of_100,
        ready_to_rollback,
        ready_for_stable,
        active_candidate_id,
        active_promotion_id,
        active_rollback_reference,
        latest_promotion_decision,
        deployment_gate_decision,
        post_promotion_monitor_json: post_promotion_monitor_json.clone(),
        promotion_ledger_json: promotion_ledger_json.clone(),
        candidate_archive_json: candidate_archive_json.clone(),
        deployment_gate_json: deployment_gate_json.clone(),
        rollback_command: rollback_command.clone(),
        dry_run_command: dry_run_command.clone(),
        operator,
        reason,
        blocking_count,
        warning_count,
        stale_count,
        missing_evidence_count,
        checks,
        blocked_reasons,
        summary: format!(
            "Friday rollback drill is {score_out_of_100}/100 with {blocking_count} blocking issue(s), {warning_count} warning(s), and {stale_count} stale check(s)."
        ),
        commands: vec![
            format!(
                "flow --friday-release-rollback-drill --output {} --post-promotion-monitor {} --promotion-ledger {} --candidate-archive {} --deployment-gate {} --rollback-command {}",
                drill_json,
                post_promotion_monitor_json,
                promotion_ledger_json,
                candidate_archive_json,
                deployment_gate_json,
                shell_quote(&rollback_command)
            ),
            format!(
                "flow --friday-release-rollback-drill-json --output {} --post-promotion-monitor {} --promotion-ledger {} --candidate-archive {} --deployment-gate {}",
                drill_json,
                post_promotion_monitor_json,
                promotion_ledger_json,
                candidate_archive_json,
                deployment_gate_json
            ),
            dry_run_command,
        ],
    }
}

pub fn write_friday_release_rollback_drill_report(
    drill_path: impl AsRef<Path>,
    report: &FridayReleaseRollbackDrillReport,
) -> Result<()> {
    let drill_path = drill_path.as_ref();
    if let Some(parent) = drill_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday rollback drill directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(drill_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday rollback drill {}",
            drill_path.display()
        )
    })
}

pub fn read_friday_release_rollback_drill_report(
    drill_path: impl AsRef<Path>,
) -> Result<FridayReleaseRollbackDrillReport> {
    let drill_path = drill_path.as_ref();
    let bytes = fs::read(drill_path).with_context(|| {
        format!(
            "Could not read Friday rollback drill {}",
            drill_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday rollback drill {}",
            drill_path.display()
        )
    })
}

fn post_promotion_monitor_check(
    path: &Path,
    monitor: Option<&FridayReleasePostPromotionMonitorReport>,
) -> FridayReleaseRollbackDrillCheck {
    match monitor {
        Some(monitor) => {
            let status = if monitor.blocking_count > 0 {
                FridayReleaseRollbackDrillCheckStatus::Failed
            } else if monitor.stale_count > 0 {
                FridayReleaseRollbackDrillCheckStatus::Stale
            } else if monitor.warning_count > 0 || !monitor.ready_for_stable {
                FridayReleaseRollbackDrillCheckStatus::Warning
            } else {
                FridayReleaseRollbackDrillCheckStatus::Passed
            };
            source_check(
                "post-promotion-monitor",
                "Post-promotion monitor",
                path,
                true,
                status,
                &format!(
                    "Monitor has {} blocking issue(s), {} warning(s), and {} stale check(s).",
                    monitor.blocking_count, monitor.warning_count, monitor.stale_count
                ),
                "Resolve post-promotion blockers and stale evidence before treating the release as stable.",
            )
        }
        None => source_check(
            "post-promotion-monitor",
            "Post-promotion monitor",
            path,
            true,
            FridayReleaseRollbackDrillCheckStatus::Missing,
            "Post-promotion monitor JSON is missing.",
            "Generate the post-promotion monitor before running a rollback drill.",
        ),
    }
}

fn promotion_ledger_check(
    path: &Path,
    ledger: Option<&FridayReleasePromotionLedger>,
) -> FridayReleaseRollbackDrillCheck {
    match ledger {
        Some(ledger) => {
            let status = if ledger.record_count == 0 {
                FridayReleaseRollbackDrillCheckStatus::Failed
            } else if ledger.active_rollback_reference.is_none() {
                FridayReleaseRollbackDrillCheckStatus::Warning
            } else {
                FridayReleaseRollbackDrillCheckStatus::Passed
            };
            source_check(
                "promotion-ledger",
                "Promotion ledger",
                path,
                true,
                status,
                &format!(
                    "Ledger has {} record(s), {} promoted candidate(s), and rollback reference {}.",
                    ledger.record_count,
                    ledger.promoted_count,
                    ledger
                        .active_rollback_reference
                        .as_deref()
                        .unwrap_or("not recorded")
                ),
                "Record the active promotion and rollback reference before rollback drill signoff.",
            )
        }
        None => source_check(
            "promotion-ledger",
            "Promotion ledger",
            path,
            true,
            FridayReleaseRollbackDrillCheckStatus::Missing,
            "Promotion ledger JSON is missing.",
            "Generate the release promotion ledger before a rollback drill.",
        ),
    }
}

fn candidate_archive_check(
    path: &Path,
    archive: Option<&FridayReleaseCandidateArchive>,
) -> FridayReleaseRollbackDrillCheck {
    match archive {
        Some(archive) => {
            let status = if archive.candidate_count == 0 {
                FridayReleaseRollbackDrillCheckStatus::Failed
            } else if archive.regression_count > 0 {
                FridayReleaseRollbackDrillCheckStatus::Warning
            } else {
                FridayReleaseRollbackDrillCheckStatus::Passed
            };
            source_check(
                "candidate-archive",
                "Candidate archive",
                path,
                true,
                status,
                &format!(
                    "Archive has {} candidate(s), {} regression warning(s), and latest candidate {}.",
                    archive.candidate_count,
                    archive.regression_count,
                    archive.latest_candidate_id.as_deref().unwrap_or("none")
                ),
                "Keep the promoted candidate and previous stable candidate evidence in the archive.",
            )
        }
        None => source_check(
            "candidate-archive",
            "Candidate archive",
            path,
            true,
            FridayReleaseRollbackDrillCheckStatus::Missing,
            "Candidate archive JSON is missing.",
            "Archive the active release candidate before a rollback drill.",
        ),
    }
}

fn deployment_gate_check(
    path: &Path,
    gate: Option<&FridayReleaseDeploymentGateReport>,
) -> FridayReleaseRollbackDrillCheck {
    match gate {
        Some(gate) => {
            let status = match gate.decision {
                FridayReleaseDeploymentGateDecision::Go => {
                    FridayReleaseRollbackDrillCheckStatus::Passed
                }
                FridayReleaseDeploymentGateDecision::Draft
                | FridayReleaseDeploymentGateDecision::NoGo => {
                    FridayReleaseRollbackDrillCheckStatus::Warning
                }
            };
            source_check(
                "deployment-gate",
                "Deployment gate",
                path,
                true,
                status,
                &format!(
                    "Deployment gate decision is {} with {} no-deploy reason(s).",
                    gate.decision.label(),
                    gate.no_deploy_reason_count
                ),
                "Keep the deployment gate attached so rollback drill context matches the promoted candidate.",
            )
        }
        None => source_check(
            "deployment-gate",
            "Deployment gate",
            path,
            true,
            FridayReleaseRollbackDrillCheckStatus::Missing,
            "Deployment gate JSON is missing.",
            "Generate the deployment gate before a rollback drill.",
        ),
    }
}

fn rollback_reference_check(reference: Option<&str>) -> FridayReleaseRollbackDrillCheck {
    match reference.filter(|value| !value.trim().is_empty()) {
        Some(reference) => inline_check(
            "rollback-reference",
            "Rollback reference",
            FridayReleaseRollbackDrillCheckStatus::Passed,
            true,
            &format!("Active rollback reference is {reference}."),
            "Keep this reference in the promotion ledger and deployment notes.",
        ),
        None => inline_check(
            "rollback-reference",
            "Rollback reference",
            FridayReleaseRollbackDrillCheckStatus::Failed,
            true,
            "No active rollback reference is recorded.",
            "Record the previous stable Friday candidate, tag, package, or restore point.",
        ),
    }
}

fn dry_run_command_check(command: &str) -> FridayReleaseRollbackDrillCheck {
    if command.trim().is_empty() {
        inline_check(
            "dry-run-command",
            "Dry-run command",
            FridayReleaseRollbackDrillCheckStatus::Failed,
            true,
            "No rollback dry-run command is available.",
            "Pass --rollback-command with the safe command operators should rehearse.",
        )
    } else {
        inline_check(
            "dry-run-command",
            "Dry-run command",
            FridayReleaseRollbackDrillCheckStatus::Passed,
            true,
            "A local-only dry-run command is recorded.",
            "Copy the command from this report only after reviewing every blocked reason.",
        )
    }
}

fn source_check(
    id: &str,
    label: &str,
    path: &Path,
    required: bool,
    status: FridayReleaseRollbackDrillCheckStatus,
    summary: &str,
    next_action: &str,
) -> FridayReleaseRollbackDrillCheck {
    let (present, bytes) = file_info(path);
    FridayReleaseRollbackDrillCheck {
        id: id.to_string(),
        label: label.to_string(),
        source_path: path_string(path),
        required,
        present,
        stale: status == FridayReleaseRollbackDrillCheckStatus::Stale,
        bytes,
        status,
        summary: summary.to_string(),
        next_action: next_action.to_string(),
    }
}

fn inline_check(
    id: &str,
    label: &str,
    status: FridayReleaseRollbackDrillCheckStatus,
    required: bool,
    summary: &str,
    next_action: &str,
) -> FridayReleaseRollbackDrillCheck {
    FridayReleaseRollbackDrillCheck {
        id: id.to_string(),
        label: label.to_string(),
        source_path: "inline".to_string(),
        required,
        present: !matches!(status, FridayReleaseRollbackDrillCheckStatus::Missing),
        stale: status == FridayReleaseRollbackDrillCheckStatus::Stale,
        bytes: 0,
        status,
        summary: summary.to_string(),
        next_action: next_action.to_string(),
    }
}

fn score_checks(checks: &[FridayReleaseRollbackDrillCheck]) -> u8 {
    if checks.is_empty() {
        return 0;
    }

    let earned = checks
        .iter()
        .map(|check| check.status.score_multiplier())
        .sum::<f32>();

    ((earned / checks.len() as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn dry_run_command(command: &str) -> String {
    let command = command.trim();
    if command.is_empty() {
        return String::new();
    }

    let lower = command.to_ascii_lowercase();
    if lower.contains("--dry-run") || lower.contains("rollback-drill") {
        command.to_string()
    } else {
        format!("{command} --dry-run")
    }
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "-_./:".contains(character))
    {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('"', "\\\""))
    }
}

fn file_info(path: &Path) -> (bool, u64) {
    match fs::metadata(path) {
        Ok(metadata) => (metadata.is_file(), metadata.len()),
        Err(_) => (false, 0),
    }
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
