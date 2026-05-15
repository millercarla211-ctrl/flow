use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseOwnerFollowUpBoardReport,
    FridayReleaseOwnerFollowUpCompletionState, FridayReleaseOwnerFollowUpEvidenceState,
    FridayReleaseOwnerFollowUpRecord, FridayReleasePreventionEvidenceLink,
    FridayReleasePreventionPlanReport, FridayReleaseStabilityBoardCheck,
    FridayReleaseStabilityBoardCheckStatus, FridayReleaseStabilityBoardReport,
    read_friday_release_owner_followup_board_report, read_friday_release_prevention_plan_report,
    read_friday_release_stability_board_report,
};

const OWNER_SLA_WINDOW_MS: u128 = 4 * 60 * 60 * 1000;
const STABILITY_SLA_WINDOW_MS: u128 = 2 * 60 * 60 * 1000;
const WARNING_WINDOW_MS: u128 = 60 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseEvidenceSlaState {
    Fresh,
    DueSoon,
    Overdue,
    Missing,
    Blocked,
    Acknowledged,
}

impl FridayReleaseEvidenceSlaState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::DueSoon => "due-soon",
            Self::Overdue => "overdue",
            Self::Missing => "missing",
            Self::Blocked => "blocked",
            Self::Acknowledged => "acknowledged",
        }
    }

    fn score_multiplier(self) -> f32 {
        match self {
            Self::Acknowledged => 1.0,
            Self::Fresh => 0.9,
            Self::DueSoon => 0.65,
            Self::Missing => 0.25,
            Self::Overdue => 0.15,
            Self::Blocked => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseEvidenceEscalationLevel {
    None,
    Owner,
    ReleaseGate,
    Checkpoint,
}

impl FridayReleaseEvidenceEscalationLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Owner => "owner",
            Self::ReleaseGate => "release-gate",
            Self::Checkpoint => "checkpoint",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseEvidenceRequirementSource {
    OwnerFollowUp,
    PreventionPlan,
    StabilityBoard,
}

impl FridayReleaseEvidenceRequirementSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::OwnerFollowUp => "owner-follow-up",
            Self::PreventionPlan => "prevention-plan",
            Self::StabilityBoard => "stability-board",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseEvidenceSlaRequirement {
    pub id: String,
    pub source: FridayReleaseEvidenceRequirementSource,
    pub owner: String,
    pub title: String,
    pub state: FridayReleaseEvidenceSlaState,
    pub escalation_level: FridayReleaseEvidenceEscalationLevel,
    pub evidence_path: String,
    pub evidence_present: bool,
    pub due_after_unix_ms: u128,
    pub due_before_unix_ms: u128,
    pub sla_window_ms: u128,
    pub age_ms: u128,
    pub acknowledgement_required: bool,
    pub release_gate_blocking: bool,
    pub escalation_copy: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseEvidenceSlaOwnerGroup {
    pub owner: String,
    pub requirement_count: usize,
    pub fresh_count: usize,
    pub due_soon_count: usize,
    pub overdue_count: usize,
    pub missing_count: usize,
    pub blocked_count: usize,
    pub acknowledged_count: usize,
    pub escalation_count: usize,
    pub release_gate_blocking_count: usize,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseEvidenceSlaMonitorReport {
    pub monitor_id: String,
    pub monitor_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub ready_for_next_checkpoint: bool,
    pub owner_followup_board_json: String,
    pub prevention_plan_json: String,
    pub stability_board_json: String,
    pub requirement_count: usize,
    pub owner_count: usize,
    pub fresh_count: usize,
    pub due_soon_count: usize,
    pub overdue_count: usize,
    pub missing_count: usize,
    pub blocked_count: usize,
    pub acknowledged_count: usize,
    pub escalation_count: usize,
    pub gate_blocking_count: usize,
    pub owner_groups: Vec<FridayReleaseEvidenceSlaOwnerGroup>,
    pub requirements: Vec<FridayReleaseEvidenceSlaRequirement>,
    pub escalation_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseEvidenceSlaMonitorReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_evidence_sla_monitor_report(
    monitor_path: impl AsRef<Path>,
    owner_followup_board_path: impl AsRef<Path>,
    prevention_plan_path: impl AsRef<Path>,
    stability_board_path: impl AsRef<Path>,
) -> FridayReleaseEvidenceSlaMonitorReport {
    friday_release_evidence_sla_monitor_report_at(
        monitor_path,
        owner_followup_board_path,
        prevention_plan_path,
        stability_board_path,
        unix_ms(),
    )
}

pub fn friday_release_evidence_sla_monitor_report_at(
    monitor_path: impl AsRef<Path>,
    owner_followup_board_path: impl AsRef<Path>,
    prevention_plan_path: impl AsRef<Path>,
    stability_board_path: impl AsRef<Path>,
    generated_at_unix_ms: u128,
) -> FridayReleaseEvidenceSlaMonitorReport {
    let monitor_path = monitor_path.as_ref();
    let owner_followup_board_path = owner_followup_board_path.as_ref();
    let prevention_plan_path = prevention_plan_path.as_ref();
    let stability_board_path = stability_board_path.as_ref();
    let owner_board =
        read_friday_release_owner_followup_board_report(owner_followup_board_path).ok();
    let prevention_plan = read_friday_release_prevention_plan_report(prevention_plan_path).ok();
    let stability_board = read_friday_release_stability_board_report(stability_board_path).ok();
    let requirements = sla_requirements(
        owner_followup_board_path,
        prevention_plan_path,
        stability_board_path,
        owner_board.as_ref(),
        prevention_plan.as_ref(),
        stability_board.as_ref(),
        generated_at_unix_ms,
    );
    let owner_groups = owner_groups(&requirements);
    let requirement_count = requirements.len();
    let owner_count = owner_groups.len();
    let fresh_count = count_state(&requirements, FridayReleaseEvidenceSlaState::Fresh);
    let due_soon_count = count_state(&requirements, FridayReleaseEvidenceSlaState::DueSoon);
    let overdue_count = count_state(&requirements, FridayReleaseEvidenceSlaState::Overdue);
    let missing_count = count_state(&requirements, FridayReleaseEvidenceSlaState::Missing);
    let blocked_count = count_state(&requirements, FridayReleaseEvidenceSlaState::Blocked);
    let acknowledged_count =
        count_state(&requirements, FridayReleaseEvidenceSlaState::Acknowledged);
    let escalation_count = requirements
        .iter()
        .filter(|requirement| {
            requirement.escalation_level != FridayReleaseEvidenceEscalationLevel::None
        })
        .count();
    let gate_blocking_count = requirements
        .iter()
        .filter(|requirement| requirement.release_gate_blocking)
        .count();
    let score_out_of_100 = score_requirements(&requirements, escalation_count);
    let ready_for_next_checkpoint = owner_board
        .as_ref()
        .map(|board| board.ready_for_next_checkpoint)
        .unwrap_or(false)
        && prevention_plan
            .as_ref()
            .map(|plan| plan.ready_for_next_checkpoint)
            .unwrap_or(false)
        && stability_board
            .as_ref()
            .map(|board| board.ready_for_checkpoint)
            .unwrap_or(false)
        && overdue_count == 0
        && missing_count == 0
        && blocked_count == 0
        && gate_blocking_count == 0;
    let status = if blocked_count > 0 || overdue_count > 0 || gate_blocking_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if missing_count > 0 || due_soon_count > 0 || !ready_for_next_checkpoint {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };
    let monitor_json = path_string(monitor_path);
    let owner_followup_board_json = path_string(owner_followup_board_path);
    let prevention_plan_json = path_string(prevention_plan_path);
    let stability_board_json = path_string(stability_board_path);

    FridayReleaseEvidenceSlaMonitorReport {
        monitor_id: format!("friday-release-evidence-sla-monitor-{generated_at_unix_ms}"),
        monitor_json: monitor_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        score_out_of_100,
        ready_for_next_checkpoint,
        owner_followup_board_json: owner_followup_board_json.clone(),
        prevention_plan_json: prevention_plan_json.clone(),
        stability_board_json: stability_board_json.clone(),
        requirement_count,
        owner_count,
        fresh_count,
        due_soon_count,
        overdue_count,
        missing_count,
        blocked_count,
        acknowledged_count,
        escalation_count,
        gate_blocking_count,
        escalation_copy: escalation_copy(&requirements),
        summary: format!(
            "Friday release evidence SLA monitor is {score_out_of_100}/100 with {requirement_count} requirement(s), {overdue_count} overdue, {missing_count} missing, and {escalation_count} escalation(s)."
        ),
        commands: vec![
            format!(
                "flow --friday-release-evidence-sla-monitor --output {} --owner-followup-board {} --prevention-plan {} --stability-board {}",
                monitor_json, owner_followup_board_json, prevention_plan_json, stability_board_json
            ),
            format!(
                "flow --friday-release-evidence-sla-monitor-json --output {} --owner-followup-board {} --prevention-plan {} --stability-board {}",
                monitor_json, owner_followup_board_json, prevention_plan_json, stability_board_json
            ),
        ],
        owner_groups,
        requirements,
    }
}

pub fn write_friday_release_evidence_sla_monitor_report(
    monitor_path: impl AsRef<Path>,
    report: &FridayReleaseEvidenceSlaMonitorReport,
) -> Result<()> {
    let monitor_path = monitor_path.as_ref();
    if let Some(parent) = monitor_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release evidence SLA monitor directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(monitor_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release evidence SLA monitor {}",
            monitor_path.display()
        )
    })
}

pub fn read_friday_release_evidence_sla_monitor_report(
    monitor_path: impl AsRef<Path>,
) -> Result<FridayReleaseEvidenceSlaMonitorReport> {
    let monitor_path = monitor_path.as_ref();
    let bytes = fs::read(monitor_path).with_context(|| {
        format!(
            "Could not read Friday release evidence SLA monitor {}",
            monitor_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release evidence SLA monitor {}",
            monitor_path.display()
        )
    })
}

#[allow(clippy::too_many_arguments)]
fn sla_requirements(
    owner_followup_board_path: &Path,
    prevention_plan_path: &Path,
    stability_board_path: &Path,
    owner_board: Option<&FridayReleaseOwnerFollowUpBoardReport>,
    prevention_plan: Option<&FridayReleasePreventionPlanReport>,
    stability_board: Option<&FridayReleaseStabilityBoardReport>,
    generated_at_unix_ms: u128,
) -> Vec<FridayReleaseEvidenceSlaRequirement> {
    let mut requirements = Vec::new();

    match owner_board {
        Some(board) => {
            requirements.extend(
                board
                    .records
                    .iter()
                    .map(|record| owner_requirement(record, generated_at_unix_ms)),
            );
        }
        None => requirements.push(missing_source_requirement(
            "owner-followup-board-missing",
            FridayReleaseEvidenceRequirementSource::OwnerFollowUp,
            "Owner follow-up board is missing",
            owner_followup_board_path,
            generated_at_unix_ms,
        )),
    }

    match prevention_plan {
        Some(plan) => {
            requirements.extend(
                plan.evidence_links
                    .iter()
                    .filter_map(|link| prevention_requirement(link, generated_at_unix_ms)),
            );
        }
        None => requirements.push(missing_source_requirement(
            "prevention-plan-missing",
            FridayReleaseEvidenceRequirementSource::PreventionPlan,
            "Prevention plan is missing",
            prevention_plan_path,
            generated_at_unix_ms,
        )),
    }

    match stability_board {
        Some(board) => {
            requirements.extend(
                board
                    .checks
                    .iter()
                    .filter(|check| check.required || !check.present || check.stale)
                    .map(|check| stability_requirement(check, generated_at_unix_ms)),
            );
        }
        None => requirements.push(missing_source_requirement(
            "stability-board-missing",
            FridayReleaseEvidenceRequirementSource::StabilityBoard,
            "Stability board is missing",
            stability_board_path,
            generated_at_unix_ms,
        )),
    }

    requirements.sort_by(|left, right| {
        left.escalation_level
            .cmp(&right.escalation_level)
            .reverse()
            .then_with(|| left.owner.cmp(&right.owner))
            .then_with(|| left.id.cmp(&right.id))
    });
    requirements
}

fn owner_requirement(
    record: &FridayReleaseOwnerFollowUpRecord,
    generated_at_unix_ms: u128,
) -> FridayReleaseEvidenceSlaRequirement {
    let evidence_present = record.evidence_state
        == FridayReleaseOwnerFollowUpEvidenceState::Present
        || record.evidence_state == FridayReleaseOwnerFollowUpEvidenceState::NotRequired;
    let age_ms = generated_at_unix_ms.saturating_sub(record.due_after_unix_ms);
    let state = if record.completion_state == FridayReleaseOwnerFollowUpCompletionState::Blocked {
        FridayReleaseEvidenceSlaState::Blocked
    } else if record.evidence_state == FridayReleaseOwnerFollowUpEvidenceState::Missing
        && generated_at_unix_ms >= record.due_before_unix_ms
    {
        FridayReleaseEvidenceSlaState::Overdue
    } else if record.evidence_state == FridayReleaseOwnerFollowUpEvidenceState::Missing {
        FridayReleaseEvidenceSlaState::Missing
    } else if record.completion_state == FridayReleaseOwnerFollowUpCompletionState::Complete {
        FridayReleaseEvidenceSlaState::Acknowledged
    } else if record
        .due_before_unix_ms
        .saturating_sub(generated_at_unix_ms)
        <= WARNING_WINDOW_MS
    {
        FridayReleaseEvidenceSlaState::DueSoon
    } else {
        FridayReleaseEvidenceSlaState::Fresh
    };
    let escalation_level = escalation_level(state, record.release_gate_blocking, record.required);
    let next_action = match state {
        FridayReleaseEvidenceSlaState::Overdue => {
            "Escalate this overdue owner evidence before the next checkpoint."
        }
        FridayReleaseEvidenceSlaState::Missing => "Attach the requested owner evidence.",
        FridayReleaseEvidenceSlaState::Blocked => "Resolve the blocked owner follow-up first.",
        FridayReleaseEvidenceSlaState::DueSoon => {
            "Confirm the owner evidence before the SLA closes."
        }
        FridayReleaseEvidenceSlaState::Acknowledged => "Keep the acknowledgement attached.",
        FridayReleaseEvidenceSlaState::Fresh => "Keep the owner evidence current.",
    };

    FridayReleaseEvidenceSlaRequirement {
        id: format!("sla-{}", record.id),
        source: FridayReleaseEvidenceRequirementSource::OwnerFollowUp,
        owner: record.owner.clone(),
        title: record.title.clone(),
        state,
        escalation_level,
        evidence_path: record.evidence_path.clone(),
        evidence_present,
        due_after_unix_ms: record.due_after_unix_ms,
        due_before_unix_ms: record.due_before_unix_ms,
        sla_window_ms: record
            .due_before_unix_ms
            .saturating_sub(record.due_after_unix_ms)
            .max(OWNER_SLA_WINDOW_MS.min(record.due_before_unix_ms)),
        age_ms,
        acknowledgement_required: record.required && evidence_present,
        release_gate_blocking: record.release_gate_blocking
            || escalation_level >= FridayReleaseEvidenceEscalationLevel::ReleaseGate,
        escalation_copy: escalation_text(
            &record.owner,
            &record.title,
            state,
            escalation_level,
            &record.evidence_request,
            &record.command,
        ),
        next_action: next_action.to_string(),
    }
}

fn prevention_requirement(
    link: &FridayReleasePreventionEvidenceLink,
    generated_at_unix_ms: u128,
) -> Option<FridayReleaseEvidenceSlaRequirement> {
    if link.present {
        return None;
    }
    let state = FridayReleaseEvidenceSlaState::Missing;
    let escalation_level = FridayReleaseEvidenceEscalationLevel::ReleaseGate;
    Some(FridayReleaseEvidenceSlaRequirement {
        id: format!("sla-prevention-{}", link.id),
        source: FridayReleaseEvidenceRequirementSource::PreventionPlan,
        owner: "release-operator".to_string(),
        title: format!("Attach prevention evidence: {}", link.label),
        state,
        escalation_level,
        evidence_path: link.path.clone(),
        evidence_present: false,
        due_after_unix_ms: generated_at_unix_ms,
        due_before_unix_ms: generated_at_unix_ms,
        sla_window_ms: 0,
        age_ms: 0,
        acknowledgement_required: false,
        release_gate_blocking: true,
        escalation_copy: escalation_text(
            "release-operator",
            &format!("Attach prevention evidence: {}", link.label),
            state,
            escalation_level,
            &format!("Attach missing prevention evidence at {}.", link.path),
            "flow --friday-release-prevention-plan",
        ),
        next_action: "Attach the missing prevention evidence before checkpoint review.".to_string(),
    })
}

fn stability_requirement(
    check: &FridayReleaseStabilityBoardCheck,
    generated_at_unix_ms: u128,
) -> FridayReleaseEvidenceSlaRequirement {
    let evidence_present = check.present && !check.stale;
    let state = match check.status {
        FridayReleaseStabilityBoardCheckStatus::Failed => FridayReleaseEvidenceSlaState::Blocked,
        FridayReleaseStabilityBoardCheckStatus::Missing => FridayReleaseEvidenceSlaState::Missing,
        FridayReleaseStabilityBoardCheckStatus::Stale => FridayReleaseEvidenceSlaState::Overdue,
        FridayReleaseStabilityBoardCheckStatus::Warning if check.stale => {
            FridayReleaseEvidenceSlaState::Overdue
        }
        FridayReleaseStabilityBoardCheckStatus::Warning => FridayReleaseEvidenceSlaState::DueSoon,
        FridayReleaseStabilityBoardCheckStatus::Passed => FridayReleaseEvidenceSlaState::Fresh,
    };
    let escalation_level = escalation_level(state, check.required, check.required);
    let due_before_unix_ms = if matches!(
        state,
        FridayReleaseEvidenceSlaState::Blocked
            | FridayReleaseEvidenceSlaState::Missing
            | FridayReleaseEvidenceSlaState::Overdue
    ) {
        generated_at_unix_ms
    } else {
        generated_at_unix_ms + STABILITY_SLA_WINDOW_MS
    };

    FridayReleaseEvidenceSlaRequirement {
        id: format!("sla-stability-{}", check.id),
        source: FridayReleaseEvidenceRequirementSource::StabilityBoard,
        owner: "release-operator".to_string(),
        title: check.label.clone(),
        state,
        escalation_level,
        evidence_path: check.source_path.clone(),
        evidence_present,
        due_after_unix_ms: generated_at_unix_ms,
        due_before_unix_ms,
        sla_window_ms: STABILITY_SLA_WINDOW_MS,
        age_ms: 0,
        acknowledgement_required: check.required && evidence_present,
        release_gate_blocking: check.required
            && escalation_level >= FridayReleaseEvidenceEscalationLevel::ReleaseGate,
        escalation_copy: escalation_text(
            "release-operator",
            &check.label,
            state,
            escalation_level,
            &check.summary,
            &check.next_action,
        ),
        next_action: check.next_action.clone(),
    }
}

fn missing_source_requirement(
    id: &str,
    source: FridayReleaseEvidenceRequirementSource,
    title: &str,
    path: &Path,
    generated_at_unix_ms: u128,
) -> FridayReleaseEvidenceSlaRequirement {
    let state = FridayReleaseEvidenceSlaState::Missing;
    let escalation_level = FridayReleaseEvidenceEscalationLevel::Checkpoint;
    let path = path_string(path);
    FridayReleaseEvidenceSlaRequirement {
        id: id.to_string(),
        source,
        owner: "release-operator".to_string(),
        title: title.to_string(),
        state,
        escalation_level,
        evidence_path: path.clone(),
        evidence_present: false,
        due_after_unix_ms: generated_at_unix_ms,
        due_before_unix_ms: generated_at_unix_ms,
        sla_window_ms: 0,
        age_ms: 0,
        acknowledgement_required: false,
        release_gate_blocking: true,
        escalation_copy: escalation_text(
            "release-operator",
            title,
            state,
            escalation_level,
            &format!("Generate the missing source report at {path}."),
            "flow --friday-release-evidence-sla-monitor",
        ),
        next_action: "Generate the missing release evidence source and rerun the SLA monitor."
            .to_string(),
    }
}

fn owner_groups(
    requirements: &[FridayReleaseEvidenceSlaRequirement],
) -> Vec<FridayReleaseEvidenceSlaOwnerGroup> {
    let mut groups: BTreeMap<String, Vec<&FridayReleaseEvidenceSlaRequirement>> = BTreeMap::new();
    for requirement in requirements {
        groups
            .entry(requirement.owner.clone())
            .or_default()
            .push(requirement);
    }

    groups
        .into_iter()
        .map(|(owner, requirements)| FridayReleaseEvidenceSlaOwnerGroup {
            owner,
            requirement_count: requirements.len(),
            fresh_count: requirements
                .iter()
                .filter(|requirement| requirement.state == FridayReleaseEvidenceSlaState::Fresh)
                .count(),
            due_soon_count: requirements
                .iter()
                .filter(|requirement| requirement.state == FridayReleaseEvidenceSlaState::DueSoon)
                .count(),
            overdue_count: requirements
                .iter()
                .filter(|requirement| requirement.state == FridayReleaseEvidenceSlaState::Overdue)
                .count(),
            missing_count: requirements
                .iter()
                .filter(|requirement| requirement.state == FridayReleaseEvidenceSlaState::Missing)
                .count(),
            blocked_count: requirements
                .iter()
                .filter(|requirement| requirement.state == FridayReleaseEvidenceSlaState::Blocked)
                .count(),
            acknowledged_count: requirements
                .iter()
                .filter(|requirement| {
                    requirement.state == FridayReleaseEvidenceSlaState::Acknowledged
                })
                .count(),
            escalation_count: requirements
                .iter()
                .filter(|requirement| {
                    requirement.escalation_level != FridayReleaseEvidenceEscalationLevel::None
                })
                .count(),
            release_gate_blocking_count: requirements
                .iter()
                .filter(|requirement| requirement.release_gate_blocking)
                .count(),
            requirements: requirements
                .iter()
                .map(|requirement| requirement.id.clone())
                .collect(),
        })
        .collect()
}

fn count_state(
    requirements: &[FridayReleaseEvidenceSlaRequirement],
    state: FridayReleaseEvidenceSlaState,
) -> usize {
    requirements
        .iter()
        .filter(|requirement| requirement.state == state)
        .count()
}

fn escalation_level(
    state: FridayReleaseEvidenceSlaState,
    release_gate_blocking: bool,
    required: bool,
) -> FridayReleaseEvidenceEscalationLevel {
    match state {
        FridayReleaseEvidenceSlaState::Blocked | FridayReleaseEvidenceSlaState::Overdue
            if release_gate_blocking =>
        {
            FridayReleaseEvidenceEscalationLevel::Checkpoint
        }
        FridayReleaseEvidenceSlaState::Blocked | FridayReleaseEvidenceSlaState::Overdue => {
            FridayReleaseEvidenceEscalationLevel::ReleaseGate
        }
        FridayReleaseEvidenceSlaState::Missing if required || release_gate_blocking => {
            FridayReleaseEvidenceEscalationLevel::ReleaseGate
        }
        FridayReleaseEvidenceSlaState::Missing | FridayReleaseEvidenceSlaState::DueSoon => {
            FridayReleaseEvidenceEscalationLevel::Owner
        }
        FridayReleaseEvidenceSlaState::Fresh | FridayReleaseEvidenceSlaState::Acknowledged => {
            FridayReleaseEvidenceEscalationLevel::None
        }
    }
}

fn score_requirements(
    requirements: &[FridayReleaseEvidenceSlaRequirement],
    escalation_count: usize,
) -> u8 {
    if requirements.is_empty() {
        return 0;
    }
    let earned = requirements
        .iter()
        .map(|requirement| requirement.state.score_multiplier())
        .sum::<f32>();
    let penalty = (escalation_count as f32 * 4.0).min(30.0);
    (((earned / requirements.len() as f32) * 100.0) - penalty)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn escalation_copy(requirements: &[FridayReleaseEvidenceSlaRequirement]) -> String {
    let mut lines = vec!["Friday release evidence SLA monitor".to_string()];
    for requirement in requirements.iter().filter(|requirement| {
        requirement.escalation_level != FridayReleaseEvidenceEscalationLevel::None
    }) {
        lines.push(format!(
            "- @{} [{} / {}] {} -> {}",
            requirement.owner,
            requirement.state.label(),
            requirement.escalation_level.label(),
            requirement.title,
            requirement.next_action
        ));
    }
    if lines.len() == 1 {
        lines.push("No SLA escalations are active.".to_string());
    }
    lines.join("\n")
}

fn escalation_text(
    owner: &str,
    title: &str,
    state: FridayReleaseEvidenceSlaState,
    escalation_level: FridayReleaseEvidenceEscalationLevel,
    evidence_request: &str,
    command: &str,
) -> String {
    format!(
        "@{owner} - {title}\nSLA: {}\nEscalation: {}\nEvidence: {evidence_request}\nCommand: {command}",
        state.label(),
        escalation_level.label()
    )
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
