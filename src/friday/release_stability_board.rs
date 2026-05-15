use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseCandidateArchive, FridayReleaseDeploymentGateDecision,
    FridayReleaseDeploymentGateReport, FridayReleasePostPromotionMonitorReport,
    FridayReleasePromotionDecision, FridayReleasePromotionLedger,
    FridayReleaseQaCommandCenterReport, FridayReleaseRollbackDrillReport,
    read_friday_release_candidate_archive, read_friday_release_deployment_gate,
    read_friday_release_post_promotion_monitor_report, read_friday_release_promotion_ledger,
    read_friday_release_qa_command_center_report, read_friday_release_rollback_drill_report,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseStabilityBoardCategory {
    DeploymentReadiness,
    QaHealth,
    CandidateRegression,
    PromotionState,
    PostPromotionFreshness,
    RollbackRecovery,
}

impl FridayReleaseStabilityBoardCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::DeploymentReadiness => "deployment-readiness",
            Self::QaHealth => "qa-health",
            Self::CandidateRegression => "candidate-regression",
            Self::PromotionState => "promotion-state",
            Self::PostPromotionFreshness => "post-promotion-freshness",
            Self::RollbackRecovery => "rollback-recovery",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseStabilityBoardCheckStatus {
    Passed,
    Warning,
    Failed,
    Missing,
    Stale,
}

impl FridayReleaseStabilityBoardCheckStatus {
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
pub struct FridayReleaseStabilityBoardCheck {
    pub id: String,
    pub label: String,
    pub category: FridayReleaseStabilityBoardCategory,
    pub source_path: String,
    pub required: bool,
    pub present: bool,
    pub stale: bool,
    pub bytes: u64,
    pub status: FridayReleaseStabilityBoardCheckStatus,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseStabilityBoardEvidenceLink {
    pub id: String,
    pub label: String,
    pub path: String,
    pub present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseStabilityBoardReport {
    pub board_id: String,
    pub board_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub ready_for_checkpoint: bool,
    pub ready_to_deploy: bool,
    pub stable_after_promotion: bool,
    pub recoverable: bool,
    pub active_candidate_id: Option<String>,
    pub active_promotion_id: Option<String>,
    pub active_rollback_reference: Option<String>,
    pub latest_promotion_decision: Option<FridayReleasePromotionDecision>,
    pub deployment_gate_decision: Option<FridayReleaseDeploymentGateDecision>,
    pub qa_json: String,
    pub candidate_archive_json: String,
    pub promotion_ledger_json: String,
    pub post_promotion_monitor_json: String,
    pub rollback_drill_json: String,
    pub deployment_gate_json: String,
    pub blocking_count: usize,
    pub warning_count: usize,
    pub stale_count: usize,
    pub missing_evidence_count: usize,
    pub checks: Vec<FridayReleaseStabilityBoardCheck>,
    pub evidence_links: Vec<FridayReleaseStabilityBoardEvidenceLink>,
    pub active_risks: Vec<String>,
    pub next_actions: Vec<String>,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseStabilityBoardReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_stability_board_report(
    board_path: impl AsRef<Path>,
    qa_path: impl AsRef<Path>,
    candidate_archive_path: impl AsRef<Path>,
    promotion_ledger_path: impl AsRef<Path>,
    post_promotion_monitor_path: impl AsRef<Path>,
    rollback_drill_path: impl AsRef<Path>,
    deployment_gate_path: impl AsRef<Path>,
) -> FridayReleaseStabilityBoardReport {
    let board_path = board_path.as_ref();
    let qa_path = qa_path.as_ref();
    let candidate_archive_path = candidate_archive_path.as_ref();
    let promotion_ledger_path = promotion_ledger_path.as_ref();
    let post_promotion_monitor_path = post_promotion_monitor_path.as_ref();
    let rollback_drill_path = rollback_drill_path.as_ref();
    let deployment_gate_path = deployment_gate_path.as_ref();
    let generated_at_unix_ms = unix_ms();

    let qa = read_friday_release_qa_command_center_report(qa_path).ok();
    let archive = read_friday_release_candidate_archive(candidate_archive_path).ok();
    let ledger = read_friday_release_promotion_ledger(promotion_ledger_path).ok();
    let monitor =
        read_friday_release_post_promotion_monitor_report(post_promotion_monitor_path).ok();
    let rollback_drill = read_friday_release_rollback_drill_report(rollback_drill_path).ok();
    let gate = read_friday_release_deployment_gate(deployment_gate_path).ok();

    let active_candidate_id = monitor
        .as_ref()
        .and_then(|monitor| monitor.active_candidate_id.clone())
        .or_else(|| {
            rollback_drill
                .as_ref()
                .and_then(|drill| drill.active_candidate_id.clone())
        })
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
            rollback_drill
                .as_ref()
                .and_then(|drill| drill.active_promotion_id.clone())
        })
        .or_else(|| {
            ledger
                .as_ref()
                .and_then(|ledger| ledger.active_promotion_id.clone())
        });
    let active_rollback_reference = rollback_drill
        .as_ref()
        .and_then(|drill| drill.active_rollback_reference.clone())
        .or_else(|| {
            monitor
                .as_ref()
                .and_then(|monitor| monitor.active_rollback_reference.clone())
        })
        .or_else(|| {
            ledger
                .as_ref()
                .and_then(|ledger| ledger.active_rollback_reference.clone())
        });
    let latest_promotion_decision = monitor
        .as_ref()
        .and_then(|monitor| monitor.latest_decision)
        .or_else(|| {
            rollback_drill
                .as_ref()
                .and_then(|drill| drill.latest_promotion_decision)
        })
        .or_else(|| ledger.as_ref().and_then(|ledger| ledger.latest_decision));
    let deployment_gate_decision = gate
        .as_ref()
        .map(|gate| gate.decision)
        .or_else(|| {
            rollback_drill
                .as_ref()
                .and_then(|drill| drill.deployment_gate_decision)
        })
        .or_else(|| archive.as_ref().and_then(|archive| archive.latest_decision));

    let mut checks = vec![
        deployment_check(deployment_gate_path, gate.as_ref()),
        qa_check(qa_path, qa.as_ref()),
        candidate_regression_check(candidate_archive_path, archive.as_ref()),
        promotion_state_check(promotion_ledger_path, ledger.as_ref()),
        post_promotion_check(post_promotion_monitor_path, monitor.as_ref()),
        rollback_recovery_check(rollback_drill_path, rollback_drill.as_ref()),
    ];
    checks.sort_by(|left, right| left.category.label().cmp(right.category.label()));

    let blocking_count = checks
        .iter()
        .filter(|check| {
            matches!(
                check.status,
                FridayReleaseStabilityBoardCheckStatus::Failed
                    | FridayReleaseStabilityBoardCheckStatus::Missing
            )
        })
        .count();
    let warning_count = checks
        .iter()
        .filter(|check| check.status == FridayReleaseStabilityBoardCheckStatus::Warning)
        .count();
    let stale_count = checks
        .iter()
        .filter(|check| check.status == FridayReleaseStabilityBoardCheckStatus::Stale)
        .count();
    let missing_evidence_count = checks.iter().filter(|check| !check.present).count();
    let score_out_of_100 = score_checks(&checks);
    let ready_to_deploy = gate.as_ref().is_some_and(|gate| gate.ready_to_deploy)
        && qa.as_ref().is_some_and(|qa| qa.ready_to_ship);
    let stable_after_promotion = monitor
        .as_ref()
        .is_some_and(|monitor| monitor.ready_for_stable);
    let recoverable = rollback_drill
        .as_ref()
        .is_some_and(|drill| drill.ready_to_rollback)
        && active_rollback_reference
            .as_ref()
            .is_some_and(|reference| !reference.trim().is_empty());
    let ready_for_checkpoint = ready_to_deploy
        && stable_after_promotion
        && recoverable
        && blocking_count == 0
        && warning_count == 0
        && stale_count == 0;
    let status = if blocking_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if warning_count > 0 || stale_count > 0 || !ready_for_checkpoint {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };
    let evidence_links = vec![
        evidence_link("release-qa", "Release QA", qa_path),
        evidence_link(
            "candidate-archive",
            "Candidate archive",
            candidate_archive_path,
        ),
        evidence_link(
            "promotion-ledger",
            "Promotion ledger",
            promotion_ledger_path,
        ),
        evidence_link(
            "post-promotion-monitor",
            "Post-promotion monitor",
            post_promotion_monitor_path,
        ),
        evidence_link("rollback-drill", "Rollback drill", rollback_drill_path),
        evidence_link("deployment-gate", "Deployment gate", deployment_gate_path),
    ];
    let active_risks = checks
        .iter()
        .filter(|check| check.status != FridayReleaseStabilityBoardCheckStatus::Passed)
        .map(|check| format!("{}: {}", check.label, check.summary))
        .collect::<Vec<_>>();
    let next_actions = unique_next_actions(&checks);
    let board_json = path_string(board_path);
    let qa_json = path_string(qa_path);
    let candidate_archive_json = path_string(candidate_archive_path);
    let promotion_ledger_json = path_string(promotion_ledger_path);
    let post_promotion_monitor_json = path_string(post_promotion_monitor_path);
    let rollback_drill_json = path_string(rollback_drill_path);
    let deployment_gate_json = path_string(deployment_gate_path);

    FridayReleaseStabilityBoardReport {
        board_id: format!("friday-release-stability-board-{generated_at_unix_ms}"),
        board_json: board_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        score_out_of_100,
        ready_for_checkpoint,
        ready_to_deploy,
        stable_after_promotion,
        recoverable,
        active_candidate_id,
        active_promotion_id,
        active_rollback_reference,
        latest_promotion_decision,
        deployment_gate_decision,
        qa_json: qa_json.clone(),
        candidate_archive_json: candidate_archive_json.clone(),
        promotion_ledger_json: promotion_ledger_json.clone(),
        post_promotion_monitor_json: post_promotion_monitor_json.clone(),
        rollback_drill_json: rollback_drill_json.clone(),
        deployment_gate_json: deployment_gate_json.clone(),
        blocking_count,
        warning_count,
        stale_count,
        missing_evidence_count,
        checks,
        evidence_links,
        active_risks,
        next_actions,
        summary: format!(
            "Friday stability board is {score_out_of_100}/100 with {blocking_count} blocking issue(s), {warning_count} warning(s), and {stale_count} stale check(s)."
        ),
        commands: vec![
            format!(
                "flow --friday-release-stability-board --output {} --qa {} --candidate-archive {} --promotion-ledger {} --post-promotion-monitor {} --rollback-drill {} --deployment-gate {}",
                board_json,
                qa_json,
                candidate_archive_json,
                promotion_ledger_json,
                post_promotion_monitor_json,
                rollback_drill_json,
                deployment_gate_json
            ),
            format!(
                "flow --friday-release-stability-board-json --output {} --qa {} --candidate-archive {} --promotion-ledger {} --post-promotion-monitor {} --rollback-drill {} --deployment-gate {}",
                board_json,
                qa_json,
                candidate_archive_json,
                promotion_ledger_json,
                post_promotion_monitor_json,
                rollback_drill_json,
                deployment_gate_json
            ),
        ],
    }
}

pub fn write_friday_release_stability_board_report(
    board_path: impl AsRef<Path>,
    report: &FridayReleaseStabilityBoardReport,
) -> Result<()> {
    let board_path = board_path.as_ref();
    if let Some(parent) = board_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday stability board directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(board_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday stability board {}",
            board_path.display()
        )
    })
}

pub fn read_friday_release_stability_board_report(
    board_path: impl AsRef<Path>,
) -> Result<FridayReleaseStabilityBoardReport> {
    let board_path = board_path.as_ref();
    let bytes = fs::read(board_path).with_context(|| {
        format!(
            "Could not read Friday stability board {}",
            board_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday stability board {}",
            board_path.display()
        )
    })
}

fn deployment_check(
    path: &Path,
    gate: Option<&FridayReleaseDeploymentGateReport>,
) -> FridayReleaseStabilityBoardCheck {
    match gate {
        Some(gate) => {
            let status = if gate.ready_to_deploy {
                FridayReleaseStabilityBoardCheckStatus::Passed
            } else if gate.decision == FridayReleaseDeploymentGateDecision::Draft {
                FridayReleaseStabilityBoardCheckStatus::Warning
            } else {
                FridayReleaseStabilityBoardCheckStatus::Failed
            };
            source_check(
                "deployment-gate",
                "Deployment readiness",
                FridayReleaseStabilityBoardCategory::DeploymentReadiness,
                path,
                true,
                status,
                &format!(
                    "Deployment gate is {} with {} no-deploy reason(s).",
                    gate.decision.label(),
                    gate.no_deploy_reason_count
                ),
                "Resolve deployment gate blockers before a major checkpoint.",
            )
        }
        None => source_check(
            "deployment-gate",
            "Deployment readiness",
            FridayReleaseStabilityBoardCategory::DeploymentReadiness,
            path,
            true,
            FridayReleaseStabilityBoardCheckStatus::Missing,
            "Deployment gate JSON is missing.",
            "Generate the deployment gate before reviewing release stability.",
        ),
    }
}

fn qa_check(
    path: &Path,
    qa: Option<&FridayReleaseQaCommandCenterReport>,
) -> FridayReleaseStabilityBoardCheck {
    match qa {
        Some(qa) => {
            let status = if qa.blocking_count > 0 {
                FridayReleaseStabilityBoardCheckStatus::Failed
            } else if qa.stale_count > 0 {
                FridayReleaseStabilityBoardCheckStatus::Stale
            } else if qa.warning_count > 0 || !qa.ready_to_ship {
                FridayReleaseStabilityBoardCheckStatus::Warning
            } else {
                FridayReleaseStabilityBoardCheckStatus::Passed
            };
            source_check(
                "release-qa",
                "QA health",
                FridayReleaseStabilityBoardCategory::QaHealth,
                path,
                true,
                status,
                &format!(
                    "Release QA has {} blocking issue(s), {} warning(s), and {} stale check(s).",
                    qa.blocking_count, qa.warning_count, qa.stale_count
                ),
                "Refresh lightweight checks and resolve QA blockers before release signoff.",
            )
        }
        None => source_check(
            "release-qa",
            "QA health",
            FridayReleaseStabilityBoardCategory::QaHealth,
            path,
            true,
            FridayReleaseStabilityBoardCheckStatus::Missing,
            "Release QA JSON is missing.",
            "Generate the release QA command center before reviewing stability.",
        ),
    }
}

fn candidate_regression_check(
    path: &Path,
    archive: Option<&FridayReleaseCandidateArchive>,
) -> FridayReleaseStabilityBoardCheck {
    match archive {
        Some(archive) => {
            let status = if archive.candidate_count == 0 {
                FridayReleaseStabilityBoardCheckStatus::Failed
            } else if archive.regression_count > 0 {
                FridayReleaseStabilityBoardCheckStatus::Warning
            } else {
                FridayReleaseStabilityBoardCheckStatus::Passed
            };
            source_check(
                "candidate-regression",
                "Candidate regression",
                FridayReleaseStabilityBoardCategory::CandidateRegression,
                path,
                true,
                status,
                &format!(
                    "Candidate archive has {} candidate(s) and {} regression warning(s).",
                    archive.candidate_count, archive.regression_count
                ),
                "Compare the latest candidate against the previous checkpoint before promoting.",
            )
        }
        None => source_check(
            "candidate-regression",
            "Candidate regression",
            FridayReleaseStabilityBoardCategory::CandidateRegression,
            path,
            true,
            FridayReleaseStabilityBoardCheckStatus::Missing,
            "Candidate archive JSON is missing.",
            "Archive the active release candidate before reviewing stability.",
        ),
    }
}

fn promotion_state_check(
    path: &Path,
    ledger: Option<&FridayReleasePromotionLedger>,
) -> FridayReleaseStabilityBoardCheck {
    match ledger {
        Some(ledger) => {
            let status = if ledger.record_count == 0 {
                FridayReleaseStabilityBoardCheckStatus::Failed
            } else if ledger.promoted_count == 0 || ledger.active_rollback_reference.is_none() {
                FridayReleaseStabilityBoardCheckStatus::Warning
            } else {
                FridayReleaseStabilityBoardCheckStatus::Passed
            };
            source_check(
                "promotion-state",
                "Promotion state",
                FridayReleaseStabilityBoardCategory::PromotionState,
                path,
                true,
                status,
                &format!(
                    "Promotion ledger has {} record(s), {} promoted candidate(s), and rollback reference {}.",
                    ledger.record_count,
                    ledger.promoted_count,
                    ledger
                        .active_rollback_reference
                        .as_deref()
                        .unwrap_or("not recorded")
                ),
                "Record the promoted candidate and active rollback reference before stability signoff.",
            )
        }
        None => source_check(
            "promotion-state",
            "Promotion state",
            FridayReleaseStabilityBoardCategory::PromotionState,
            path,
            true,
            FridayReleaseStabilityBoardCheckStatus::Missing,
            "Promotion ledger JSON is missing.",
            "Generate the promotion ledger before reviewing stability.",
        ),
    }
}

fn post_promotion_check(
    path: &Path,
    monitor: Option<&FridayReleasePostPromotionMonitorReport>,
) -> FridayReleaseStabilityBoardCheck {
    match monitor {
        Some(monitor) => {
            let status = if monitor.blocking_count > 0 {
                FridayReleaseStabilityBoardCheckStatus::Failed
            } else if monitor.stale_count > 0 {
                FridayReleaseStabilityBoardCheckStatus::Stale
            } else if monitor.warning_count > 0 || !monitor.ready_for_stable {
                FridayReleaseStabilityBoardCheckStatus::Warning
            } else {
                FridayReleaseStabilityBoardCheckStatus::Passed
            };
            source_check(
                "post-promotion-freshness",
                "Post-promotion freshness",
                FridayReleaseStabilityBoardCategory::PostPromotionFreshness,
                path,
                true,
                status,
                &format!(
                    "Post-promotion monitor has {} blocking issue(s), {} warning(s), and {} stale check(s).",
                    monitor.blocking_count, monitor.warning_count, monitor.stale_count
                ),
                "Resolve post-promotion evidence before marking the release stable.",
            )
        }
        None => source_check(
            "post-promotion-freshness",
            "Post-promotion freshness",
            FridayReleaseStabilityBoardCategory::PostPromotionFreshness,
            path,
            true,
            FridayReleaseStabilityBoardCheckStatus::Missing,
            "Post-promotion monitor JSON is missing.",
            "Generate the post-promotion monitor before reviewing stability.",
        ),
    }
}

fn rollback_recovery_check(
    path: &Path,
    drill: Option<&FridayReleaseRollbackDrillReport>,
) -> FridayReleaseStabilityBoardCheck {
    match drill {
        Some(drill) => {
            let status = if drill.blocking_count > 0 {
                FridayReleaseStabilityBoardCheckStatus::Failed
            } else if drill.stale_count > 0 {
                FridayReleaseStabilityBoardCheckStatus::Stale
            } else if drill.warning_count > 0 || !drill.ready_to_rollback {
                FridayReleaseStabilityBoardCheckStatus::Warning
            } else {
                FridayReleaseStabilityBoardCheckStatus::Passed
            };
            source_check(
                "rollback-recovery",
                "Rollback recovery",
                FridayReleaseStabilityBoardCategory::RollbackRecovery,
                path,
                true,
                status,
                &format!(
                    "Rollback drill has {} blocking issue(s), {} warning(s), and {} stale check(s).",
                    drill.blocking_count, drill.warning_count, drill.stale_count
                ),
                "Run a clean rollback drill before treating this candidate as recoverable.",
            )
        }
        None => source_check(
            "rollback-recovery",
            "Rollback recovery",
            FridayReleaseStabilityBoardCategory::RollbackRecovery,
            path,
            true,
            FridayReleaseStabilityBoardCheckStatus::Missing,
            "Rollback drill JSON is missing.",
            "Generate a rollback drill before reviewing stability.",
        ),
    }
}

fn source_check(
    id: &str,
    label: &str,
    category: FridayReleaseStabilityBoardCategory,
    path: &Path,
    required: bool,
    status: FridayReleaseStabilityBoardCheckStatus,
    summary: &str,
    next_action: &str,
) -> FridayReleaseStabilityBoardCheck {
    let (present, bytes) = file_info(path);
    FridayReleaseStabilityBoardCheck {
        id: id.to_string(),
        label: label.to_string(),
        category,
        source_path: path_string(path),
        required,
        present,
        stale: status == FridayReleaseStabilityBoardCheckStatus::Stale,
        bytes,
        status,
        summary: summary.to_string(),
        next_action: next_action.to_string(),
    }
}

fn evidence_link(id: &str, label: &str, path: &Path) -> FridayReleaseStabilityBoardEvidenceLink {
    let (present, _) = file_info(path);
    FridayReleaseStabilityBoardEvidenceLink {
        id: id.to_string(),
        label: label.to_string(),
        path: path_string(path),
        present,
    }
}

fn unique_next_actions(checks: &[FridayReleaseStabilityBoardCheck]) -> Vec<String> {
    let mut actions = Vec::new();
    for check in checks {
        if check.status != FridayReleaseStabilityBoardCheckStatus::Passed
            && !actions.contains(&check.next_action)
        {
            actions.push(check.next_action.clone());
        }
    }
    actions
}

fn score_checks(checks: &[FridayReleaseStabilityBoardCheck]) -> u8 {
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
