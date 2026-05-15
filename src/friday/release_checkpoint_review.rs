use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseEscalationLedger,
    FridayReleaseEscalationOwnerResponse, FridayReleaseEvidenceSlaMonitorReport,
    FridayReleaseEvidenceSlaState, FridayReleaseOwnerFollowUpBoardReport,
    FridayReleaseOwnerFollowUpCompletionState, FridayReleasePreventionActionStatus,
    FridayReleasePreventionPlanReport, FridayReleaseStabilityBoardCheckStatus,
    FridayReleaseStabilityBoardReport, read_friday_release_escalation_ledger,
    read_friday_release_evidence_sla_monitor_report,
    read_friday_release_owner_followup_board_report, read_friday_release_prevention_plan_report,
    read_friday_release_stability_board_report,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseCheckpointDecision {
    Ready,
    Hold,
    CarryOver,
    NeedsReview,
}

impl FridayReleaseCheckpointDecision {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Hold => "hold",
            Self::CarryOver => "carry-over",
            Self::NeedsReview => "needs-review",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseCheckpointReviewState {
    Ready,
    Hold,
    CarryOver,
    ReviewRequired,
}

impl FridayReleaseCheckpointReviewState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Hold => "hold",
            Self::CarryOver => "carry-over",
            Self::ReviewRequired => "review-required",
        }
    }

    fn score_multiplier(self) -> f32 {
        match self {
            Self::Ready => 1.0,
            Self::ReviewRequired => 0.55,
            Self::CarryOver => 0.3,
            Self::Hold => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseCheckpointReviewSource {
    EscalationLedger,
    SlaMonitor,
    OwnerFollowUp,
    PreventionPlan,
    StabilityBoard,
    MissingEvidence,
}

impl FridayReleaseCheckpointReviewSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::EscalationLedger => "escalation-ledger",
            Self::SlaMonitor => "sla-monitor",
            Self::OwnerFollowUp => "owner-follow-up",
            Self::PreventionPlan => "prevention-plan",
            Self::StabilityBoard => "stability-board",
            Self::MissingEvidence => "missing-evidence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseCheckpointReviewItem {
    pub id: String,
    pub source: FridayReleaseCheckpointReviewSource,
    pub owner: String,
    pub title: String,
    pub state: FridayReleaseCheckpointReviewState,
    pub decision: FridayReleaseCheckpointDecision,
    pub acknowledgement_required: bool,
    pub acknowledged: bool,
    pub active_carryover: bool,
    pub release_gate_blocking: bool,
    pub evidence_path: String,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseCheckpointReviewOwnerGroup {
    pub owner: String,
    pub item_count: usize,
    pub hold_count: usize,
    pub carryover_count: usize,
    pub review_required_count: usize,
    pub acknowledgement_blocker_count: usize,
    pub release_gate_blocking_count: usize,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseCheckpointReviewBoardReport {
    pub review_id: String,
    pub review_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub decision: FridayReleaseCheckpointDecision,
    pub ready_for_checkpoint: bool,
    pub item_count: usize,
    pub owner_count: usize,
    pub hold_count: usize,
    pub carryover_count: usize,
    pub review_required_count: usize,
    pub acknowledgement_required_count: usize,
    pub acknowledgement_blocker_count: usize,
    pub active_escalation_count: usize,
    pub release_gate_blocking_count: usize,
    pub escalation_ledger_json: String,
    pub sla_monitor_json: String,
    pub owner_followup_board_json: String,
    pub prevention_plan_json: String,
    pub stability_board_json: String,
    pub owner_groups: Vec<FridayReleaseCheckpointReviewOwnerGroup>,
    pub items: Vec<FridayReleaseCheckpointReviewItem>,
    pub review_notes_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseCheckpointReviewBoardReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_checkpoint_review_board_report(
    review_path: impl AsRef<Path>,
    escalation_ledger_path: impl AsRef<Path>,
    sla_monitor_path: impl AsRef<Path>,
    owner_followup_board_path: impl AsRef<Path>,
    prevention_plan_path: impl AsRef<Path>,
    stability_board_path: impl AsRef<Path>,
) -> FridayReleaseCheckpointReviewBoardReport {
    let review_path = review_path.as_ref();
    let escalation_ledger_path = escalation_ledger_path.as_ref();
    let sla_monitor_path = sla_monitor_path.as_ref();
    let owner_followup_board_path = owner_followup_board_path.as_ref();
    let prevention_plan_path = prevention_plan_path.as_ref();
    let stability_board_path = stability_board_path.as_ref();
    let generated_at_unix_ms = unix_ms();

    let escalation_ledger = read_friday_release_escalation_ledger(escalation_ledger_path).ok();
    let sla_monitor = read_friday_release_evidence_sla_monitor_report(sla_monitor_path).ok();
    let owner_board =
        read_friday_release_owner_followup_board_report(owner_followup_board_path).ok();
    let prevention_plan = read_friday_release_prevention_plan_report(prevention_plan_path).ok();
    let stability_board = read_friday_release_stability_board_report(stability_board_path).ok();

    let mut items = Vec::new();
    append_ledger_items(&mut items, escalation_ledger.as_ref());
    append_sla_items(&mut items, sla_monitor.as_ref());
    append_owner_followup_items(&mut items, owner_board.as_ref());
    append_prevention_items(&mut items, prevention_plan.as_ref());
    append_stability_items(&mut items, stability_board.as_ref());
    append_missing_source_item(
        &mut items,
        "escalation-ledger",
        escalation_ledger.is_some(),
        escalation_ledger_path,
    );
    append_missing_source_item(
        &mut items,
        "sla-monitor",
        sla_monitor.is_some(),
        sla_monitor_path,
    );
    append_missing_source_item(
        &mut items,
        "owner-follow-up",
        owner_board.is_some(),
        owner_followup_board_path,
    );
    append_missing_source_item(
        &mut items,
        "prevention-plan",
        prevention_plan.is_some(),
        prevention_plan_path,
    );
    append_missing_source_item(
        &mut items,
        "stability-board",
        stability_board.is_some(),
        stability_board_path,
    );
    items.sort_by(|left, right| {
        left.owner
            .cmp(&right.owner)
            .then_with(|| state_rank(left.state).cmp(&state_rank(right.state)))
            .then_with(|| left.id.cmp(&right.id))
    });

    let owner_groups = owner_groups(&items);
    let item_count = items.len();
    let hold_count = state_count(&items, FridayReleaseCheckpointReviewState::Hold);
    let carryover_count = state_count(&items, FridayReleaseCheckpointReviewState::CarryOver);
    let review_required_count =
        state_count(&items, FridayReleaseCheckpointReviewState::ReviewRequired);
    let acknowledgement_required_count = items
        .iter()
        .filter(|item| item.acknowledgement_required)
        .count();
    let acknowledgement_blocker_count = items
        .iter()
        .filter(|item| item.acknowledgement_required && !item.acknowledged)
        .count();
    let active_escalation_count = escalation_ledger
        .as_ref()
        .map(|ledger| ledger.active_count)
        .unwrap_or(0);
    let release_gate_blocking_count = items
        .iter()
        .filter(|item| item.release_gate_blocking)
        .count();
    let decision =
        if hold_count > 0 || acknowledgement_blocker_count > 0 || release_gate_blocking_count > 0 {
            FridayReleaseCheckpointDecision::Hold
        } else if carryover_count > 0 || active_escalation_count > 0 {
            FridayReleaseCheckpointDecision::CarryOver
        } else if review_required_count > 0 {
            FridayReleaseCheckpointDecision::NeedsReview
        } else {
            FridayReleaseCheckpointDecision::Ready
        };
    let ready_for_checkpoint = decision == FridayReleaseCheckpointDecision::Ready;
    let status = match decision {
        FridayReleaseCheckpointDecision::Ready => FridayDashboardPanelStatus::Ready,
        FridayReleaseCheckpointDecision::NeedsReview => FridayDashboardPanelStatus::Warning,
        FridayReleaseCheckpointDecision::CarryOver | FridayReleaseCheckpointDecision::Hold => {
            FridayDashboardPanelStatus::Blocked
        }
    };
    let score_out_of_100 = score_items(&items, acknowledgement_blocker_count);
    let review_json = path_string(review_path);
    let escalation_ledger_json = path_string(escalation_ledger_path);
    let sla_monitor_json = path_string(sla_monitor_path);
    let owner_followup_board_json = path_string(owner_followup_board_path);
    let prevention_plan_json = path_string(prevention_plan_path);
    let stability_board_json = path_string(stability_board_path);

    FridayReleaseCheckpointReviewBoardReport {
        review_id: format!("friday-release-checkpoint-review-{generated_at_unix_ms}"),
        review_json: review_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        score_out_of_100,
        decision,
        ready_for_checkpoint,
        item_count,
        owner_count: owner_groups.len(),
        hold_count,
        carryover_count,
        review_required_count,
        acknowledgement_required_count,
        acknowledgement_blocker_count,
        active_escalation_count,
        release_gate_blocking_count,
        escalation_ledger_json: escalation_ledger_json.clone(),
        sla_monitor_json: sla_monitor_json.clone(),
        owner_followup_board_json: owner_followup_board_json.clone(),
        prevention_plan_json: prevention_plan_json.clone(),
        stability_board_json: stability_board_json.clone(),
        owner_groups,
        review_notes_copy: review_notes_copy(decision, &items),
        summary: format!(
            "Friday checkpoint review is {} at {score_out_of_100}/100 with {item_count} review item(s), {hold_count} hold(s), {carryover_count} carryover(s), and {acknowledgement_blocker_count} acknowledgement blocker(s).",
            decision.label()
        ),
        commands: vec![
            format!(
                "flow --friday-release-checkpoint-review --output {} --ledger {} --monitor {} --owner-followup-board {} --prevention-plan {} --stability-board {}",
                review_json,
                escalation_ledger_json,
                sla_monitor_json,
                owner_followup_board_json,
                prevention_plan_json,
                stability_board_json
            ),
            format!(
                "flow --friday-release-checkpoint-review-json --output {} --ledger {} --monitor {} --owner-followup-board {} --prevention-plan {} --stability-board {}",
                review_json,
                escalation_ledger_json,
                sla_monitor_json,
                owner_followup_board_json,
                prevention_plan_json,
                stability_board_json
            ),
        ],
        items,
    }
}

pub fn write_friday_release_checkpoint_review_board_report(
    review_path: impl AsRef<Path>,
    report: &FridayReleaseCheckpointReviewBoardReport,
) -> Result<()> {
    let review_path = review_path.as_ref();
    if let Some(parent) = review_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release checkpoint review directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(review_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release checkpoint review {}",
            review_path.display()
        )
    })
}

pub fn read_friday_release_checkpoint_review_board_report(
    review_path: impl AsRef<Path>,
) -> Result<FridayReleaseCheckpointReviewBoardReport> {
    let review_path = review_path.as_ref();
    let bytes = fs::read(review_path).with_context(|| {
        format!(
            "Could not read Friday release checkpoint review {}",
            review_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release checkpoint review {}",
            review_path.display()
        )
    })
}

fn append_ledger_items(
    items: &mut Vec<FridayReleaseCheckpointReviewItem>,
    ledger: Option<&FridayReleaseEscalationLedger>,
) {
    let Some(ledger) = ledger else {
        return;
    };

    for entry in ledger.entries.iter().filter(|entry| {
        entry.active_carryover
            || entry.release_gate_blocking
            || (entry.acknowledgement_required && !entry.acknowledged)
    }) {
        let state = if entry.release_gate_blocking
            || (entry.acknowledgement_required && !entry.acknowledged)
        {
            FridayReleaseCheckpointReviewState::Hold
        } else if entry.active_carryover {
            FridayReleaseCheckpointReviewState::CarryOver
        } else {
            FridayReleaseCheckpointReviewState::ReviewRequired
        };
        items.push(FridayReleaseCheckpointReviewItem {
            id: format!("checkpoint-ledger-{}", entry.escalation_id),
            source: FridayReleaseCheckpointReviewSource::EscalationLedger,
            owner: entry.owner.clone(),
            title: entry.title.clone(),
            state,
            decision: decision_for_state(state),
            acknowledgement_required: entry.acknowledgement_required,
            acknowledged: entry.acknowledged,
            active_carryover: entry.active_carryover,
            release_gate_blocking: entry.release_gate_blocking,
            evidence_path: entry.evidence_path.clone(),
            summary: format!(
                "Owner response is {} and gate outcome is {}.",
                entry.owner_response.label(),
                entry.gate_outcome.label()
            ),
            next_action: if entry.owner_response == FridayReleaseEscalationOwnerResponse::Pending {
                format!(
                    "Collect @{} acknowledgement or hold the checkpoint.",
                    entry.owner
                )
            } else {
                entry.next_action.clone()
            },
        });
    }
}

fn append_sla_items(
    items: &mut Vec<FridayReleaseCheckpointReviewItem>,
    monitor: Option<&FridayReleaseEvidenceSlaMonitorReport>,
) {
    let Some(monitor) = monitor else {
        return;
    };

    for requirement in monitor.requirements.iter().filter(|requirement| {
        requirement.release_gate_blocking
            || matches!(
                requirement.state,
                FridayReleaseEvidenceSlaState::Overdue
                    | FridayReleaseEvidenceSlaState::Missing
                    | FridayReleaseEvidenceSlaState::Blocked
                    | FridayReleaseEvidenceSlaState::DueSoon
            )
    }) {
        let state = if requirement.release_gate_blocking
            || matches!(
                requirement.state,
                FridayReleaseEvidenceSlaState::Overdue
                    | FridayReleaseEvidenceSlaState::Missing
                    | FridayReleaseEvidenceSlaState::Blocked
            ) {
            FridayReleaseCheckpointReviewState::Hold
        } else {
            FridayReleaseCheckpointReviewState::ReviewRequired
        };
        items.push(FridayReleaseCheckpointReviewItem {
            id: format!("checkpoint-sla-{}", requirement.id),
            source: FridayReleaseCheckpointReviewSource::SlaMonitor,
            owner: requirement.owner.clone(),
            title: requirement.title.clone(),
            state,
            decision: decision_for_state(state),
            acknowledgement_required: requirement.acknowledgement_required
                || requirement.release_gate_blocking,
            acknowledged: requirement.state == FridayReleaseEvidenceSlaState::Acknowledged,
            active_carryover: false,
            release_gate_blocking: requirement.release_gate_blocking,
            evidence_path: requirement.evidence_path.clone(),
            summary: format!(
                "SLA state is {} with escalation level {}.",
                requirement.state.label(),
                requirement.escalation_level.label()
            ),
            next_action: requirement.next_action.clone(),
        });
    }
}

fn append_owner_followup_items(
    items: &mut Vec<FridayReleaseCheckpointReviewItem>,
    board: Option<&FridayReleaseOwnerFollowUpBoardReport>,
) {
    let Some(board) = board else {
        return;
    };

    for record in board.records.iter().filter(|record| {
        record.release_gate_blocking
            || record.overdue
            || record.completion_state != FridayReleaseOwnerFollowUpCompletionState::Complete
    }) {
        let state = if record.release_gate_blocking || record.overdue {
            FridayReleaseCheckpointReviewState::Hold
        } else {
            FridayReleaseCheckpointReviewState::ReviewRequired
        };
        items.push(FridayReleaseCheckpointReviewItem {
            id: format!("checkpoint-owner-{}", record.id),
            source: FridayReleaseCheckpointReviewSource::OwnerFollowUp,
            owner: record.owner.clone(),
            title: record.title.clone(),
            state,
            decision: decision_for_state(state),
            acknowledgement_required: record.release_gate_blocking || record.required,
            acknowledged: record.completion_state
                == FridayReleaseOwnerFollowUpCompletionState::Complete,
            active_carryover: record.overdue,
            release_gate_blocking: record.release_gate_blocking,
            evidence_path: record.evidence_path.clone(),
            summary: format!(
                "Owner follow-up is {} with evidence request: {}",
                record.completion_state.label(),
                record.evidence_request
            ),
            next_action: record.next_action.clone(),
        });
    }
}

fn append_prevention_items(
    items: &mut Vec<FridayReleaseCheckpointReviewItem>,
    plan: Option<&FridayReleasePreventionPlanReport>,
) {
    let Some(plan) = plan else {
        return;
    };

    for action in plan.actions.iter().filter(|action| {
        action.release_gate_blocking
            || action.status != FridayReleasePreventionActionStatus::OwnerReady
    }) {
        let state = if action.release_gate_blocking
            || action.status == FridayReleasePreventionActionStatus::Blocked
        {
            FridayReleaseCheckpointReviewState::Hold
        } else {
            FridayReleaseCheckpointReviewState::ReviewRequired
        };
        items.push(FridayReleaseCheckpointReviewItem {
            id: format!("checkpoint-prevention-{}", action.id),
            source: FridayReleaseCheckpointReviewSource::PreventionPlan,
            owner: action.owner.clone(),
            title: action.title.clone(),
            state,
            decision: decision_for_state(state),
            acknowledgement_required: action.release_gate_blocking,
            acknowledged: action.status == FridayReleasePreventionActionStatus::OwnerReady,
            active_carryover: action.status != FridayReleasePreventionActionStatus::OwnerReady,
            release_gate_blocking: action.release_gate_blocking,
            evidence_path: action.evidence_path.clone(),
            summary: action.summary.clone(),
            next_action: action.next_action.clone(),
        });
    }
}

fn append_stability_items(
    items: &mut Vec<FridayReleaseCheckpointReviewItem>,
    board: Option<&FridayReleaseStabilityBoardReport>,
) {
    let Some(board) = board else {
        return;
    };

    for check in board.checks.iter().filter(|check| {
        check.required && check.status != FridayReleaseStabilityBoardCheckStatus::Passed
    }) {
        let state = if matches!(
            check.status,
            FridayReleaseStabilityBoardCheckStatus::Failed
                | FridayReleaseStabilityBoardCheckStatus::Missing
        ) {
            FridayReleaseCheckpointReviewState::Hold
        } else {
            FridayReleaseCheckpointReviewState::ReviewRequired
        };
        items.push(FridayReleaseCheckpointReviewItem {
            id: format!("checkpoint-stability-{}", check.id),
            source: FridayReleaseCheckpointReviewSource::StabilityBoard,
            owner: "release-operator".to_string(),
            title: check.label.clone(),
            state,
            decision: decision_for_state(state),
            acknowledgement_required: check.required,
            acknowledged: check.status == FridayReleaseStabilityBoardCheckStatus::Passed,
            active_carryover: check.stale || !check.present,
            release_gate_blocking: matches!(state, FridayReleaseCheckpointReviewState::Hold),
            evidence_path: check.source_path.clone(),
            summary: check.summary.clone(),
            next_action: check.next_action.clone(),
        });
    }
}

fn append_missing_source_item(
    items: &mut Vec<FridayReleaseCheckpointReviewItem>,
    source_name: &str,
    present: bool,
    path: &Path,
) {
    if present {
        return;
    }

    items.push(FridayReleaseCheckpointReviewItem {
        id: format!("checkpoint-missing-{source_name}"),
        source: FridayReleaseCheckpointReviewSource::MissingEvidence,
        owner: "release-operator".to_string(),
        title: format!("Attach {source_name} evidence"),
        state: FridayReleaseCheckpointReviewState::Hold,
        decision: FridayReleaseCheckpointDecision::Hold,
        acknowledgement_required: true,
        acknowledged: false,
        active_carryover: true,
        release_gate_blocking: true,
        evidence_path: path_string(path),
        summary: format!("Required checkpoint evidence `{source_name}` is missing."),
        next_action: format!("Attach {} before checkpoint review.", path.display()),
    });
}

fn owner_groups(
    items: &[FridayReleaseCheckpointReviewItem],
) -> Vec<FridayReleaseCheckpointReviewOwnerGroup> {
    let mut groups: BTreeMap<String, Vec<&FridayReleaseCheckpointReviewItem>> = BTreeMap::new();
    for item in items {
        groups.entry(item.owner.clone()).or_default().push(item);
    }

    groups
        .into_iter()
        .map(|(owner, items)| FridayReleaseCheckpointReviewOwnerGroup {
            owner,
            item_count: items.len(),
            hold_count: items
                .iter()
                .filter(|item| item.state == FridayReleaseCheckpointReviewState::Hold)
                .count(),
            carryover_count: items
                .iter()
                .filter(|item| item.state == FridayReleaseCheckpointReviewState::CarryOver)
                .count(),
            review_required_count: items
                .iter()
                .filter(|item| item.state == FridayReleaseCheckpointReviewState::ReviewRequired)
                .count(),
            acknowledgement_blocker_count: items
                .iter()
                .filter(|item| item.acknowledgement_required && !item.acknowledged)
                .count(),
            release_gate_blocking_count: items
                .iter()
                .filter(|item| item.release_gate_blocking)
                .count(),
            items: items.iter().map(|item| item.id.clone()).collect(),
        })
        .collect()
}

fn state_count(
    items: &[FridayReleaseCheckpointReviewItem],
    state: FridayReleaseCheckpointReviewState,
) -> usize {
    items.iter().filter(|item| item.state == state).count()
}

fn score_items(items: &[FridayReleaseCheckpointReviewItem], acknowledgement_blockers: usize) -> u8 {
    if items.is_empty() {
        return 100;
    }
    let earned = items
        .iter()
        .map(|item| item.state.score_multiplier())
        .sum::<f32>();
    let base = ((earned / items.len() as f32) * 100.0).round();
    (base as i32 - acknowledgement_blockers as i32 * 8).clamp(0, 100) as u8
}

fn decision_for_state(
    state: FridayReleaseCheckpointReviewState,
) -> FridayReleaseCheckpointDecision {
    match state {
        FridayReleaseCheckpointReviewState::Ready => FridayReleaseCheckpointDecision::Ready,
        FridayReleaseCheckpointReviewState::Hold => FridayReleaseCheckpointDecision::Hold,
        FridayReleaseCheckpointReviewState::CarryOver => FridayReleaseCheckpointDecision::CarryOver,
        FridayReleaseCheckpointReviewState::ReviewRequired => {
            FridayReleaseCheckpointDecision::NeedsReview
        }
    }
}

fn state_rank(state: FridayReleaseCheckpointReviewState) -> u8 {
    match state {
        FridayReleaseCheckpointReviewState::Hold => 0,
        FridayReleaseCheckpointReviewState::CarryOver => 1,
        FridayReleaseCheckpointReviewState::ReviewRequired => 2,
        FridayReleaseCheckpointReviewState::Ready => 3,
    }
}

fn review_notes_copy(
    decision: FridayReleaseCheckpointDecision,
    items: &[FridayReleaseCheckpointReviewItem],
) -> String {
    let mut lines = vec![format!(
        "Friday release checkpoint review: {}",
        decision.label()
    )];
    for item in items
        .iter()
        .filter(|item| item.state != FridayReleaseCheckpointReviewState::Ready)
        .take(12)
    {
        lines.push(format!(
            "- @{} [{} / {}] {} -> {}",
            item.owner,
            item.source.label(),
            item.state.label(),
            item.title,
            item.next_action
        ));
    }
    if lines.len() == 1 {
        lines.push("No checkpoint blockers are recorded.".to_string());
    }
    lines.join("\n")
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
