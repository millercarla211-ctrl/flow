use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleasePostPromotionMonitorReport,
    FridayReleasePromotionDecision, FridayReleasePromotionLedger, FridayReleaseRollbackDrillReport,
    FridayReleaseStabilityBoardReport, read_friday_release_post_promotion_monitor_report,
    read_friday_release_promotion_ledger, read_friday_release_rollback_drill_report,
    read_friday_release_stability_board_report,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseRecoveryRunbookPhaseKind {
    Pause,
    Diagnose,
    Rollback,
    Verify,
    Resume,
    FollowUp,
}

impl FridayReleaseRecoveryRunbookPhaseKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Diagnose => "diagnose",
            Self::Rollback => "rollback",
            Self::Verify => "verify",
            Self::Resume => "resume",
            Self::FollowUp => "follow-up",
        }
    }

    fn order(self) -> u8 {
        match self {
            Self::Pause => 1,
            Self::Diagnose => 2,
            Self::Rollback => 3,
            Self::Verify => 4,
            Self::Resume => 5,
            Self::FollowUp => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseRecoveryRunbookPhaseStatus {
    Ready,
    RequiresApproval,
    Blocked,
}

impl FridayReleaseRecoveryRunbookPhaseStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::RequiresApproval => "requires-approval",
            Self::Blocked => "blocked",
        }
    }

    fn score_multiplier(self) -> f32 {
        match self {
            Self::Ready => 1.0,
            Self::RequiresApproval => 0.75,
            Self::Blocked => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseRecoveryRunbookPhase {
    pub kind: FridayReleaseRecoveryRunbookPhaseKind,
    pub order: u8,
    pub label: String,
    pub status: FridayReleaseRecoveryRunbookPhaseStatus,
    pub approval_required: bool,
    pub source_path: String,
    pub objective: String,
    pub command: String,
    pub verification: String,
    pub risks: Vec<String>,
    pub evidence_paths: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseRecoveryRunbookApprovalGate {
    pub id: String,
    pub label: String,
    pub phase: FridayReleaseRecoveryRunbookPhaseKind,
    pub required: bool,
    pub satisfied: bool,
    pub summary: String,
    pub operator_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseRecoveryRunbookReport {
    pub runbook_id: String,
    pub runbook_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub ready_for_operator_review: bool,
    pub ready_to_execute_recovery: bool,
    pub active_candidate_id: Option<String>,
    pub active_promotion_id: Option<String>,
    pub active_rollback_reference: Option<String>,
    pub latest_promotion_decision: Option<FridayReleasePromotionDecision>,
    pub stability_board_json: String,
    pub rollback_drill_json: String,
    pub promotion_ledger_json: String,
    pub post_promotion_monitor_json: String,
    pub phase_count: usize,
    pub blocked_phase_count: usize,
    pub approval_gate_count: usize,
    pub unsatisfied_approval_gate_count: usize,
    pub command_count: usize,
    pub active_risks: Vec<String>,
    pub phases: Vec<FridayReleaseRecoveryRunbookPhase>,
    pub approval_gates: Vec<FridayReleaseRecoveryRunbookApprovalGate>,
    pub recovery_commands: Vec<String>,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseRecoveryRunbookReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_recovery_runbook_report(
    runbook_path: impl AsRef<Path>,
    stability_board_path: impl AsRef<Path>,
    rollback_drill_path: impl AsRef<Path>,
    promotion_ledger_path: impl AsRef<Path>,
    post_promotion_monitor_path: impl AsRef<Path>,
) -> FridayReleaseRecoveryRunbookReport {
    let runbook_path = runbook_path.as_ref();
    let stability_board_path = stability_board_path.as_ref();
    let rollback_drill_path = rollback_drill_path.as_ref();
    let promotion_ledger_path = promotion_ledger_path.as_ref();
    let post_promotion_monitor_path = post_promotion_monitor_path.as_ref();
    let generated_at_unix_ms = unix_ms();

    let stability_board = read_friday_release_stability_board_report(stability_board_path).ok();
    let rollback_drill = read_friday_release_rollback_drill_report(rollback_drill_path).ok();
    let promotion_ledger = read_friday_release_promotion_ledger(promotion_ledger_path).ok();
    let post_promotion_monitor =
        read_friday_release_post_promotion_monitor_report(post_promotion_monitor_path).ok();

    let active_candidate_id = stability_board
        .as_ref()
        .and_then(|board| board.active_candidate_id.clone())
        .or_else(|| {
            rollback_drill
                .as_ref()
                .and_then(|drill| drill.active_candidate_id.clone())
        })
        .or_else(|| {
            promotion_ledger
                .as_ref()
                .and_then(|ledger| ledger.active_candidate_id.clone())
        })
        .or_else(|| {
            post_promotion_monitor
                .as_ref()
                .and_then(|monitor| monitor.active_candidate_id.clone())
        });
    let active_promotion_id = stability_board
        .as_ref()
        .and_then(|board| board.active_promotion_id.clone())
        .or_else(|| {
            rollback_drill
                .as_ref()
                .and_then(|drill| drill.active_promotion_id.clone())
        })
        .or_else(|| {
            promotion_ledger
                .as_ref()
                .and_then(|ledger| ledger.active_promotion_id.clone())
        });
    let active_rollback_reference = stability_board
        .as_ref()
        .and_then(|board| board.active_rollback_reference.clone())
        .or_else(|| {
            rollback_drill
                .as_ref()
                .and_then(|drill| drill.active_rollback_reference.clone())
        })
        .or_else(|| {
            promotion_ledger
                .as_ref()
                .and_then(|ledger| ledger.active_rollback_reference.clone())
        });
    let latest_promotion_decision = stability_board
        .as_ref()
        .and_then(|board| board.latest_promotion_decision)
        .or_else(|| {
            rollback_drill
                .as_ref()
                .and_then(|drill| drill.latest_promotion_decision)
        })
        .or_else(|| {
            post_promotion_monitor
                .as_ref()
                .and_then(|monitor| monitor.latest_decision)
        })
        .or_else(|| {
            promotion_ledger
                .as_ref()
                .and_then(|ledger| ledger.latest_decision)
        });
    let active_risks = recovery_risks(
        stability_board.as_ref(),
        rollback_drill.as_ref(),
        post_promotion_monitor.as_ref(),
    );
    let phases = recovery_phases(
        stability_board_path,
        rollback_drill_path,
        promotion_ledger_path,
        post_promotion_monitor_path,
        stability_board.as_ref(),
        rollback_drill.as_ref(),
        promotion_ledger.as_ref(),
        post_promotion_monitor.as_ref(),
        &active_risks,
    );
    let approval_gates = approval_gates(&phases, active_rollback_reference.as_deref());
    let blocked_phase_count = phases
        .iter()
        .filter(|phase| phase.status == FridayReleaseRecoveryRunbookPhaseStatus::Blocked)
        .count();
    let approval_gate_count = approval_gates.iter().filter(|gate| gate.required).count();
    let unsatisfied_approval_gate_count = approval_gates
        .iter()
        .filter(|gate| gate.required && !gate.satisfied)
        .count();
    let score_out_of_100 = score_phases(&phases);
    let ready_for_operator_review = stability_board.is_some()
        && rollback_drill.is_some()
        && promotion_ledger.is_some()
        && post_promotion_monitor.is_some();
    let ready_to_execute_recovery = ready_for_operator_review
        && blocked_phase_count == 0
        && unsatisfied_approval_gate_count == 0;
    let status = if !ready_for_operator_review || blocked_phase_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if unsatisfied_approval_gate_count > 0 || !active_risks.is_empty() {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };
    let recovery_commands = phases
        .iter()
        .map(|phase| phase.command.clone())
        .filter(|command| !command.trim().is_empty())
        .collect::<Vec<_>>();
    let runbook_json = path_string(runbook_path);
    let stability_board_json = path_string(stability_board_path);
    let rollback_drill_json = path_string(rollback_drill_path);
    let promotion_ledger_json = path_string(promotion_ledger_path);
    let post_promotion_monitor_json = path_string(post_promotion_monitor_path);

    FridayReleaseRecoveryRunbookReport {
        runbook_id: format!("friday-release-recovery-runbook-{generated_at_unix_ms}"),
        runbook_json: runbook_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        score_out_of_100,
        ready_for_operator_review,
        ready_to_execute_recovery,
        active_candidate_id,
        active_promotion_id,
        active_rollback_reference,
        latest_promotion_decision,
        stability_board_json: stability_board_json.clone(),
        rollback_drill_json: rollback_drill_json.clone(),
        promotion_ledger_json: promotion_ledger_json.clone(),
        post_promotion_monitor_json: post_promotion_monitor_json.clone(),
        phase_count: phases.len(),
        blocked_phase_count,
        approval_gate_count,
        unsatisfied_approval_gate_count,
        command_count: recovery_commands.len(),
        active_risks,
        phases,
        approval_gates,
        recovery_commands,
        summary: format!(
            "Friday recovery runbook is {score_out_of_100}/100 with {blocked_phase_count} blocked phase(s) and {unsatisfied_approval_gate_count} approval gate(s) still unsatisfied."
        ),
        commands: vec![
            format!(
                "flow --friday-release-recovery-runbook --output {} --stability-board {} --rollback-drill {} --promotion-ledger {} --post-promotion-monitor {}",
                runbook_json,
                stability_board_json,
                rollback_drill_json,
                promotion_ledger_json,
                post_promotion_monitor_json
            ),
            format!(
                "flow --friday-release-recovery-runbook-json --output {} --stability-board {} --rollback-drill {} --promotion-ledger {} --post-promotion-monitor {}",
                runbook_json,
                stability_board_json,
                rollback_drill_json,
                promotion_ledger_json,
                post_promotion_monitor_json
            ),
        ],
    }
}

pub fn write_friday_release_recovery_runbook_report(
    runbook_path: impl AsRef<Path>,
    report: &FridayReleaseRecoveryRunbookReport,
) -> Result<()> {
    let runbook_path = runbook_path.as_ref();
    if let Some(parent) = runbook_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday recovery runbook directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(runbook_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday recovery runbook {}",
            runbook_path.display()
        )
    })
}

pub fn read_friday_release_recovery_runbook_report(
    runbook_path: impl AsRef<Path>,
) -> Result<FridayReleaseRecoveryRunbookReport> {
    let runbook_path = runbook_path.as_ref();
    let bytes = fs::read(runbook_path).with_context(|| {
        format!(
            "Could not read Friday recovery runbook {}",
            runbook_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday recovery runbook {}",
            runbook_path.display()
        )
    })
}

fn recovery_risks(
    stability_board: Option<&FridayReleaseStabilityBoardReport>,
    rollback_drill: Option<&FridayReleaseRollbackDrillReport>,
    post_promotion_monitor: Option<&FridayReleasePostPromotionMonitorReport>,
) -> Vec<String> {
    let mut risks = stability_board
        .map(|board| board.active_risks.clone())
        .unwrap_or_default();

    if stability_board.is_none() {
        risks.push("Stability board evidence is missing.".to_string());
    }
    if rollback_drill.is_none() {
        risks.push("Rollback drill evidence is missing.".to_string());
    }
    if let Some(drill) = rollback_drill {
        risks.extend(
            drill
                .blocked_reasons
                .iter()
                .map(|reason| format!("Rollback drill: {reason}")),
        );
    }
    if let Some(monitor) = post_promotion_monitor {
        risks.extend(
            monitor
                .warnings
                .iter()
                .map(|warning| format!("Post-promotion monitor: {warning}")),
        );
    } else {
        risks.push("Post-promotion monitor evidence is missing.".to_string());
    }
    risks.sort();
    risks.dedup();
    risks
}

#[allow(clippy::too_many_arguments)]
fn recovery_phases(
    stability_board_path: &Path,
    rollback_drill_path: &Path,
    promotion_ledger_path: &Path,
    post_promotion_monitor_path: &Path,
    stability_board: Option<&FridayReleaseStabilityBoardReport>,
    rollback_drill: Option<&FridayReleaseRollbackDrillReport>,
    promotion_ledger: Option<&FridayReleasePromotionLedger>,
    post_promotion_monitor: Option<&FridayReleasePostPromotionMonitorReport>,
    active_risks: &[String],
) -> Vec<FridayReleaseRecoveryRunbookPhase> {
    let rollback_reference = rollback_drill
        .and_then(|drill| drill.active_rollback_reference.clone())
        .or_else(|| stability_board.and_then(|board| board.active_rollback_reference.clone()))
        .or_else(|| promotion_ledger.and_then(|ledger| ledger.active_rollback_reference.clone()));
    let rollback_command = rollback_drill
        .map(|drill| drill.dry_run_command.clone())
        .filter(|command| command.contains("--dry-run") || command.contains("rollback-drill"))
        .unwrap_or_else(|| {
            format!(
                "flow --friday-release-rollback-drill-json --output {} --dry-run",
                path_string(rollback_drill_path)
            )
        });
    let blocked_by_missing_sources = stability_board.is_none()
        || rollback_drill.is_none()
        || promotion_ledger.is_none()
        || post_promotion_monitor.is_none();
    let rollback_blocked =
        rollback_drill.is_none_or(|drill| !drill.ready_to_rollback || drill.blocking_count > 0);
    let post_promotion_blocked = post_promotion_monitor
        .is_none_or(|monitor| monitor.blocking_count > 0 || monitor.stale_count > 0);
    let stability_blocked =
        stability_board.is_none_or(|board| board.blocking_count > 0 || board.stale_count > 0);

    vec![
        phase(
            FridayReleaseRecoveryRunbookPhaseKind::Pause,
            "Pause Friday runtime",
            if blocked_by_missing_sources {
                FridayReleaseRecoveryRunbookPhaseStatus::Blocked
            } else {
                FridayReleaseRecoveryRunbookPhaseStatus::RequiresApproval
            },
            true,
            stability_board_path,
            "Pause live Friday automation before touching release state.",
            "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json",
            "Confirm the live runner has no active unsafe release action before proceeding.",
            active_risks.iter().take(3).cloned().collect(),
            vec![path_string(stability_board_path)],
            "Get explicit operator approval before pausing or cancelling live work.",
        ),
        phase(
            FridayReleaseRecoveryRunbookPhaseKind::Diagnose,
            "Diagnose release risk",
            if stability_board.is_some() {
                FridayReleaseRecoveryRunbookPhaseStatus::Ready
            } else {
                FridayReleaseRecoveryRunbookPhaseStatus::Blocked
            },
            false,
            stability_board_path,
            "Review the consolidated stability board before deciding whether rollback is necessary.",
            &format!(
                "flow --friday-release-stability-board-json --output {}",
                path_string(stability_board_path)
            ),
            "The stability board should explain every active risk and evidence link.",
            stability_board
                .map(|board| board.active_risks.iter().take(4).cloned().collect())
                .unwrap_or_else(|| vec!["Stability board is missing.".to_string()]),
            vec![path_string(stability_board_path)],
            "Regenerate the stability board if any evidence is stale or missing.",
        ),
        phase(
            FridayReleaseRecoveryRunbookPhaseKind::Rollback,
            "Rollback dry run",
            if rollback_blocked {
                FridayReleaseRecoveryRunbookPhaseStatus::Blocked
            } else {
                FridayReleaseRecoveryRunbookPhaseStatus::RequiresApproval
            },
            true,
            rollback_drill_path,
            "Rehearse rollback using only local dry-run commands before any real recovery action.",
            &rollback_command,
            "The dry-run command must include a rollback reference and must not execute deployment or destructive commands.",
            rollback_drill
                .map(|drill| drill.blocked_reasons.iter().take(4).cloned().collect())
                .unwrap_or_else(|| vec!["Rollback drill is missing.".to_string()]),
            vec![path_string(rollback_drill_path)],
            rollback_reference
                .as_ref()
                .map(|reference| format!("Operator must approve rollback reference `{reference}`."))
                .unwrap_or_else(|| "Record a rollback reference before rehearsal.".to_string()),
        ),
        phase(
            FridayReleaseRecoveryRunbookPhaseKind::Verify,
            "Verify recovery state",
            if post_promotion_blocked {
                FridayReleaseRecoveryRunbookPhaseStatus::Blocked
            } else {
                FridayReleaseRecoveryRunbookPhaseStatus::Ready
            },
            false,
            post_promotion_monitor_path,
            "Verify post-promotion checks after recovery rehearsal.",
            &format!(
                "flow --friday-release-post-promotion-monitor-json --output {}",
                path_string(post_promotion_monitor_path)
            ),
            "The post-promotion monitor should have no blocking, stale, or missing required evidence.",
            post_promotion_monitor
                .map(|monitor| monitor.warnings.iter().take(4).cloned().collect())
                .unwrap_or_else(|| vec!["Post-promotion monitor is missing.".to_string()]),
            vec![path_string(post_promotion_monitor_path)],
            "Refresh post-promotion evidence after rollback rehearsal.",
        ),
        phase(
            FridayReleaseRecoveryRunbookPhaseKind::Resume,
            "Resume Friday runtime",
            if stability_blocked {
                FridayReleaseRecoveryRunbookPhaseStatus::Blocked
            } else {
                FridayReleaseRecoveryRunbookPhaseStatus::RequiresApproval
            },
            true,
            stability_board_path,
            "Resume Friday only after recovery evidence is clean.",
            "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json --history tmp/friday-dashboard/trusted-host-runner-history.json",
            "The stability board should be ready and no recovery phase should remain blocked.",
            stability_board
                .map(|board| board.next_actions.iter().take(4).cloned().collect())
                .unwrap_or_default(),
            vec![path_string(stability_board_path)],
            "Get explicit operator approval before resuming live runtime work.",
        ),
        phase(
            FridayReleaseRecoveryRunbookPhaseKind::FollowUp,
            "Follow-up incident notes",
            FridayReleaseRecoveryRunbookPhaseStatus::Ready,
            false,
            promotion_ledger_path,
            "Capture incident notes and promotion-ledger context for the next release loop.",
            &format!(
                "flow --friday-release-post-promotion-monitor --output {} --promotion-ledger {} --incident-note <incident-note.md>",
                path_string(post_promotion_monitor_path),
                path_string(promotion_ledger_path)
            ),
            "The follow-up note should explain the trigger, operator action, verification evidence, and next prevention work.",
            Vec::new(),
            vec![
                path_string(promotion_ledger_path),
                path_string(post_promotion_monitor_path),
            ],
            "Attach an incident note before closing the recovery loop.",
        ),
    ]
}

fn approval_gates(
    phases: &[FridayReleaseRecoveryRunbookPhase],
    rollback_reference: Option<&str>,
) -> Vec<FridayReleaseRecoveryRunbookApprovalGate> {
    phases
        .iter()
        .filter(|phase| phase.approval_required)
        .map(|phase| FridayReleaseRecoveryRunbookApprovalGate {
            id: format!("approve-{}", phase.kind.label()),
            label: format!("Approve {}", phase.label),
            phase: phase.kind,
            required: true,
            satisfied: false,
            summary: if phase.kind == FridayReleaseRecoveryRunbookPhaseKind::Rollback {
                rollback_reference
                    .map(|reference| {
                        format!(
                            "Rollback reference `{reference}` requires explicit operator approval."
                        )
                    })
                    .unwrap_or_else(|| {
                        "Rollback reference is missing and cannot be approved yet.".to_string()
                    })
            } else {
                format!("{} requires explicit operator approval.", phase.label)
            },
            operator_action: format!(
                "Review phase `{}` and record approval outside this generated runbook.",
                phase.kind.label()
            ),
        })
        .collect()
}

fn phase(
    kind: FridayReleaseRecoveryRunbookPhaseKind,
    label: &str,
    status: FridayReleaseRecoveryRunbookPhaseStatus,
    approval_required: bool,
    source_path: &Path,
    objective: &str,
    command: &str,
    verification: &str,
    risks: Vec<String>,
    evidence_paths: Vec<String>,
    next_action: impl Into<String>,
) -> FridayReleaseRecoveryRunbookPhase {
    FridayReleaseRecoveryRunbookPhase {
        kind,
        order: kind.order(),
        label: label.to_string(),
        status,
        approval_required,
        source_path: path_string(source_path),
        objective: objective.to_string(),
        command: command.to_string(),
        verification: verification.to_string(),
        risks,
        evidence_paths,
        next_action: next_action.into(),
    }
}

fn score_phases(phases: &[FridayReleaseRecoveryRunbookPhase]) -> u8 {
    if phases.is_empty() {
        return 0;
    }

    let earned = phases
        .iter()
        .map(|phase| phase.status.score_multiplier())
        .sum::<f32>();

    ((earned / phases.len() as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
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
