use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleasePreventionAction, FridayReleasePreventionActionStatus,
    FridayReleasePreventionPlanReport, read_friday_release_prevention_plan_report,
};

const IMMEDIATE_DUE_WINDOW_MS: u128 = 0;
const NORMAL_DUE_WINDOW_MS: u128 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseOwnerFollowUpCompletionState {
    Ready,
    NeedsEvidence,
    Blocked,
    Complete,
    Overdue,
}

impl FridayReleaseOwnerFollowUpCompletionState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NeedsEvidence => "needs-evidence",
            Self::Blocked => "blocked",
            Self::Complete => "complete",
            Self::Overdue => "overdue",
        }
    }

    fn score_multiplier(self) -> f32 {
        match self {
            Self::Complete => 1.0,
            Self::Ready => 0.8,
            Self::NeedsEvidence => 0.45,
            Self::Overdue => 0.2,
            Self::Blocked => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseOwnerFollowUpEvidenceState {
    Present,
    Missing,
    NotRequired,
}

impl FridayReleaseOwnerFollowUpEvidenceState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Missing => "missing",
            Self::NotRequired => "not-required",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseOwnerFollowUpRecord {
    pub id: String,
    pub action_id: String,
    pub owner: String,
    pub title: String,
    pub summary: String,
    pub completion_state: FridayReleaseOwnerFollowUpCompletionState,
    pub evidence_state: FridayReleaseOwnerFollowUpEvidenceState,
    pub source_path: String,
    pub evidence_path: String,
    pub evidence_request: String,
    pub due_after_unix_ms: u128,
    pub due_before_unix_ms: u128,
    pub overdue: bool,
    pub required: bool,
    pub release_gate_blocking: bool,
    pub command: String,
    pub assignment_copy: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseOwnerFollowUpGroup {
    pub owner: String,
    pub record_count: usize,
    pub ready_count: usize,
    pub waiting_count: usize,
    pub blocked_count: usize,
    pub overdue_count: usize,
    pub complete_count: usize,
    pub evidence_missing_count: usize,
    pub records: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseOwnerFollowUpBoardReport {
    pub board_id: String,
    pub board_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub ready_for_next_checkpoint: bool,
    pub prevention_plan_json: String,
    pub incident_archive_json: String,
    pub stability_board_json: String,
    pub record_count: usize,
    pub owner_count: usize,
    pub ready_count: usize,
    pub waiting_count: usize,
    pub blocked_count: usize,
    pub overdue_count: usize,
    pub complete_count: usize,
    pub evidence_missing_count: usize,
    pub gate_blocking_count: usize,
    pub owner_groups: Vec<FridayReleaseOwnerFollowUpGroup>,
    pub records: Vec<FridayReleaseOwnerFollowUpRecord>,
    pub assignment_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseOwnerFollowUpBoardReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_owner_followup_board_report(
    board_path: impl AsRef<Path>,
    prevention_plan_path: impl AsRef<Path>,
) -> FridayReleaseOwnerFollowUpBoardReport {
    friday_release_owner_followup_board_report_at(board_path, prevention_plan_path, unix_ms())
}

pub fn friday_release_owner_followup_board_report_at(
    board_path: impl AsRef<Path>,
    prevention_plan_path: impl AsRef<Path>,
    generated_at_unix_ms: u128,
) -> FridayReleaseOwnerFollowUpBoardReport {
    let board_path = board_path.as_ref();
    let prevention_plan_path = prevention_plan_path.as_ref();
    let prevention_plan = read_friday_release_prevention_plan_report(prevention_plan_path).ok();
    let records = followup_records(
        board_path,
        prevention_plan_path,
        prevention_plan.as_ref(),
        generated_at_unix_ms,
    );
    let owner_groups = owner_groups(&records);
    let record_count = records.len();
    let owner_count = owner_groups.len();
    let ready_count = count_state(&records, FridayReleaseOwnerFollowUpCompletionState::Ready);
    let waiting_count = count_state(
        &records,
        FridayReleaseOwnerFollowUpCompletionState::NeedsEvidence,
    );
    let blocked_count = count_state(&records, FridayReleaseOwnerFollowUpCompletionState::Blocked);
    let overdue_count = count_state(&records, FridayReleaseOwnerFollowUpCompletionState::Overdue);
    let complete_count = count_state(
        &records,
        FridayReleaseOwnerFollowUpCompletionState::Complete,
    );
    let evidence_missing_count = records
        .iter()
        .filter(|record| record.evidence_state == FridayReleaseOwnerFollowUpEvidenceState::Missing)
        .count();
    let gate_blocking_count = records
        .iter()
        .filter(|record| record.release_gate_blocking)
        .count();
    let score_out_of_100 = score_records(&records, evidence_missing_count, overdue_count);
    let ready_for_next_checkpoint = prevention_plan
        .as_ref()
        .map(|plan| plan.ready_for_next_checkpoint)
        .unwrap_or(false)
        && gate_blocking_count == 0
        && evidence_missing_count == 0
        && overdue_count == 0
        && blocked_count == 0;
    let status = if blocked_count > 0 || overdue_count > 0 || gate_blocking_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if waiting_count > 0 || evidence_missing_count > 0 || !ready_for_next_checkpoint {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };
    let board_json = path_string(board_path);
    let prevention_plan_json = path_string(prevention_plan_path);
    let incident_archive_json = prevention_plan
        .as_ref()
        .map(|plan| plan.incident_archive_json.clone())
        .unwrap_or_else(|| "tmp/friday-dashboard/release-incident-archive.json".to_string());
    let stability_board_json = prevention_plan
        .as_ref()
        .map(|plan| plan.stability_board_json.clone())
        .unwrap_or_else(|| "tmp/friday-dashboard/release-stability-board.json".to_string());

    FridayReleaseOwnerFollowUpBoardReport {
        board_id: format!("friday-release-owner-followup-board-{generated_at_unix_ms}"),
        board_json: board_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        score_out_of_100,
        ready_for_next_checkpoint,
        prevention_plan_json: prevention_plan_json.clone(),
        incident_archive_json,
        stability_board_json,
        record_count,
        owner_count,
        ready_count,
        waiting_count,
        blocked_count,
        overdue_count,
        complete_count,
        evidence_missing_count,
        gate_blocking_count,
        assignment_copy: assignment_copy(&records),
        summary: format!(
            "Friday owner follow-up board is {score_out_of_100}/100 with {owner_count} owner(s), {record_count} assignment(s), {overdue_count} overdue item(s), and {gate_blocking_count} release gate blocker(s)."
        ),
        commands: vec![
            format!(
                "flow --friday-release-owner-followup-board --output {} --prevention-plan {}",
                board_json, prevention_plan_json
            ),
            format!(
                "flow --friday-release-owner-followup-board-json --output {} --prevention-plan {}",
                board_json, prevention_plan_json
            ),
        ],
        owner_groups,
        records,
    }
}

pub fn write_friday_release_owner_followup_board_report(
    board_path: impl AsRef<Path>,
    report: &FridayReleaseOwnerFollowUpBoardReport,
) -> Result<()> {
    let board_path = board_path.as_ref();
    if let Some(parent) = board_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release owner follow-up board directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(board_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release owner follow-up board {}",
            board_path.display()
        )
    })
}

pub fn read_friday_release_owner_followup_board_report(
    board_path: impl AsRef<Path>,
) -> Result<FridayReleaseOwnerFollowUpBoardReport> {
    let board_path = board_path.as_ref();
    let bytes = fs::read(board_path).with_context(|| {
        format!(
            "Could not read Friday release owner follow-up board {}",
            board_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release owner follow-up board {}",
            board_path.display()
        )
    })
}

fn followup_records(
    board_path: &Path,
    prevention_plan_path: &Path,
    prevention_plan: Option<&FridayReleasePreventionPlanReport>,
    generated_at_unix_ms: u128,
) -> Vec<FridayReleaseOwnerFollowUpRecord> {
    if let Some(plan) = prevention_plan {
        let mut records = plan
            .actions
            .iter()
            .map(|action| followup_record(action, generated_at_unix_ms))
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            left.owner
                .cmp(&right.owner)
                .then_with(|| {
                    left.completion_state
                        .label()
                        .cmp(right.completion_state.label())
                })
                .then_with(|| left.id.cmp(&right.id))
        });
        records
    } else {
        let evidence_path = path_string(prevention_plan_path);
        let command = format!(
            "flow --friday-release-prevention-plan --output {} --incident-archive tmp/friday-dashboard/release-incident-archive.json --stability-board tmp/friday-dashboard/release-stability-board.json",
            evidence_path
        );
        let id = "followup-create-prevention-plan".to_string();
        let title = "Create release prevention plan".to_string();
        let next_action = "Generate the prevention plan before assigning owners.".to_string();
        let evidence_request = format!(
            "Create {} first, then regenerate {}.",
            evidence_path,
            path_string(board_path)
        );
        let assignment_copy = format!(
            "@release-operator - {title}\nState: blocked\nEvidence: {evidence_request}\nCommand: {command}"
        );
        vec![FridayReleaseOwnerFollowUpRecord {
            id,
            action_id: "create-prevention-plan".to_string(),
            owner: "release-operator".to_string(),
            title,
            summary:
                "Friday cannot create owner follow-up records until the prevention plan exists."
                    .to_string(),
            completion_state: FridayReleaseOwnerFollowUpCompletionState::Blocked,
            evidence_state: FridayReleaseOwnerFollowUpEvidenceState::Missing,
            source_path: evidence_path.clone(),
            evidence_path,
            evidence_request,
            due_after_unix_ms: generated_at_unix_ms,
            due_before_unix_ms: generated_at_unix_ms,
            overdue: true,
            required: true,
            release_gate_blocking: true,
            command,
            assignment_copy,
            next_action,
        }]
    }
}

fn followup_record(
    action: &FridayReleasePreventionAction,
    generated_at_unix_ms: u128,
) -> FridayReleaseOwnerFollowUpRecord {
    let owner = normalize_owner(&action.owner);
    let evidence_state = evidence_state(action);
    let due_window = if action.release_gate_blocking
        || action.status != FridayReleasePreventionActionStatus::OwnerReady
        || evidence_state == FridayReleaseOwnerFollowUpEvidenceState::Missing
    {
        IMMEDIATE_DUE_WINDOW_MS
    } else {
        NORMAL_DUE_WINDOW_MS
    };
    let due_before_unix_ms = generated_at_unix_ms + due_window;
    let overdue = due_before_unix_ms <= generated_at_unix_ms
        && action.required
        && evidence_state == FridayReleaseOwnerFollowUpEvidenceState::Missing;
    let completion_state = completion_state(action, evidence_state, overdue);
    let evidence_request = evidence_request(action, evidence_state);
    let assignment_copy = assignment_text(
        &owner,
        &action.title,
        completion_state,
        &evidence_request,
        &action.command,
        &action.next_action,
        due_before_unix_ms,
    );

    FridayReleaseOwnerFollowUpRecord {
        id: format!("followup-{}", action.id),
        action_id: action.id.clone(),
        owner,
        title: action.title.clone(),
        summary: action.summary.clone(),
        completion_state,
        evidence_state,
        source_path: action.source_path.clone(),
        evidence_path: action.evidence_path.clone(),
        evidence_request,
        due_after_unix_ms: generated_at_unix_ms,
        due_before_unix_ms,
        overdue,
        required: action.required,
        release_gate_blocking: action.release_gate_blocking
            || overdue
            || (action.required
                && evidence_state == FridayReleaseOwnerFollowUpEvidenceState::Missing),
        command: action.command.clone(),
        assignment_copy,
        next_action: action.next_action.clone(),
    }
}

fn evidence_state(
    action: &FridayReleasePreventionAction,
) -> FridayReleaseOwnerFollowUpEvidenceState {
    if !action.required && action.evidence_path.trim().is_empty() {
        return FridayReleaseOwnerFollowUpEvidenceState::NotRequired;
    }
    if action.required || action.status == FridayReleasePreventionActionStatus::NeedsEvidence {
        return FridayReleaseOwnerFollowUpEvidenceState::Missing;
    }
    if action.evidence_path.trim().is_empty() || action.evidence_path == "inline" {
        return FridayReleaseOwnerFollowUpEvidenceState::NotRequired;
    }
    if Path::new(&action.evidence_path).exists() {
        FridayReleaseOwnerFollowUpEvidenceState::Present
    } else if action.required {
        FridayReleaseOwnerFollowUpEvidenceState::Missing
    } else {
        FridayReleaseOwnerFollowUpEvidenceState::NotRequired
    }
}

fn completion_state(
    action: &FridayReleasePreventionAction,
    evidence_state: FridayReleaseOwnerFollowUpEvidenceState,
    overdue: bool,
) -> FridayReleaseOwnerFollowUpCompletionState {
    if action.status == FridayReleasePreventionActionStatus::Blocked {
        FridayReleaseOwnerFollowUpCompletionState::Blocked
    } else if overdue {
        FridayReleaseOwnerFollowUpCompletionState::Overdue
    } else if evidence_state == FridayReleaseOwnerFollowUpEvidenceState::Missing
        || action.status == FridayReleasePreventionActionStatus::NeedsEvidence
    {
        FridayReleaseOwnerFollowUpCompletionState::NeedsEvidence
    } else if !action.required && action.status == FridayReleasePreventionActionStatus::OwnerReady {
        FridayReleaseOwnerFollowUpCompletionState::Complete
    } else {
        FridayReleaseOwnerFollowUpCompletionState::Ready
    }
}

fn evidence_request(
    action: &FridayReleasePreventionAction,
    evidence_state: FridayReleaseOwnerFollowUpEvidenceState,
) -> String {
    match evidence_state {
        FridayReleaseOwnerFollowUpEvidenceState::Present => format!(
            "Review the attached evidence at {} and confirm the owner follow-up is still current.",
            action.evidence_path
        ),
        FridayReleaseOwnerFollowUpEvidenceState::Missing => format!(
            "Attach evidence at {} before the next checkpoint: {}",
            action.evidence_path, action.next_action
        ),
        FridayReleaseOwnerFollowUpEvidenceState::NotRequired => {
            format!(
                "No evidence attachment is required yet: {}",
                action.next_action
            )
        }
    }
}

fn owner_groups(
    records: &[FridayReleaseOwnerFollowUpRecord],
) -> Vec<FridayReleaseOwnerFollowUpGroup> {
    let mut groups: BTreeMap<String, Vec<&FridayReleaseOwnerFollowUpRecord>> = BTreeMap::new();
    for record in records {
        groups.entry(record.owner.clone()).or_default().push(record);
    }

    groups
        .into_iter()
        .map(|(owner, records)| FridayReleaseOwnerFollowUpGroup {
            owner,
            record_count: records.len(),
            ready_count: records
                .iter()
                .filter(|record| {
                    record.completion_state == FridayReleaseOwnerFollowUpCompletionState::Ready
                })
                .count(),
            waiting_count: records
                .iter()
                .filter(|record| {
                    record.completion_state
                        == FridayReleaseOwnerFollowUpCompletionState::NeedsEvidence
                })
                .count(),
            blocked_count: records
                .iter()
                .filter(|record| {
                    record.completion_state == FridayReleaseOwnerFollowUpCompletionState::Blocked
                })
                .count(),
            overdue_count: records
                .iter()
                .filter(|record| {
                    record.completion_state == FridayReleaseOwnerFollowUpCompletionState::Overdue
                })
                .count(),
            complete_count: records
                .iter()
                .filter(|record| {
                    record.completion_state == FridayReleaseOwnerFollowUpCompletionState::Complete
                })
                .count(),
            evidence_missing_count: records
                .iter()
                .filter(|record| {
                    record.evidence_state == FridayReleaseOwnerFollowUpEvidenceState::Missing
                })
                .count(),
            records: records.iter().map(|record| record.id.clone()).collect(),
        })
        .collect()
}

fn count_state(
    records: &[FridayReleaseOwnerFollowUpRecord],
    state: FridayReleaseOwnerFollowUpCompletionState,
) -> usize {
    records
        .iter()
        .filter(|record| record.completion_state == state)
        .count()
}

fn score_records(
    records: &[FridayReleaseOwnerFollowUpRecord],
    evidence_missing_count: usize,
    overdue_count: usize,
) -> u8 {
    if records.is_empty() {
        return 0;
    }
    let earned = records
        .iter()
        .map(|record| record.completion_state.score_multiplier())
        .sum::<f32>();
    let penalty = (evidence_missing_count as f32 * 5.0 + overdue_count as f32 * 10.0).min(35.0);
    (((earned / records.len() as f32) * 100.0) - penalty)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn assignment_copy(records: &[FridayReleaseOwnerFollowUpRecord]) -> String {
    let mut lines = vec!["Friday release owner follow-up board".to_string()];
    for record in records {
        lines.push(format!(
            "- @{} [{}] {} -> {}",
            record.owner,
            record.completion_state.label(),
            record.title,
            record.next_action
        ));
    }
    lines.join("\n")
}

fn assignment_text(
    owner: &str,
    title: &str,
    state: FridayReleaseOwnerFollowUpCompletionState,
    evidence_request: &str,
    command: &str,
    next_action: &str,
    due_before_unix_ms: u128,
) -> String {
    format!(
        "@{owner} - {title}\nState: {}\nDue before: {due_before_unix_ms}\nEvidence: {evidence_request}\nNext: {next_action}\nCommand: {command}",
        state.label()
    )
}

fn normalize_owner(owner: &str) -> String {
    let owner = owner.trim().trim_start_matches('@');
    if owner.is_empty() {
        "release-operator".to_string()
    } else {
        owner.to_string()
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
