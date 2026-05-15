use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseHandoffDispatchGovernanceReview,
    FridayReleaseHandoffDispatchGovernanceState,
    read_friday_release_handoff_dispatch_governance_review,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseHandoffCompletionState {
    Draft,
    Completed,
    ManuallySent,
    Held,
    Revoked,
    Superseded,
    Blocked,
}

impl FridayReleaseHandoffCompletionState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Completed => "completed",
            Self::ManuallySent => "manually-sent",
            Self::Held => "held",
            Self::Revoked => "revoked",
            Self::Superseded => "superseded",
            Self::Blocked => "blocked",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "completed" | "complete" | "done" => Ok(Self::Completed),
            "manually-sent" | "manual" | "manual-send" | "sent-manually" | "sent" => {
                Ok(Self::ManuallySent)
            }
            "held" | "hold" => Ok(Self::Held),
            "revoked" | "revoke" => Ok(Self::Revoked),
            "superseded" | "supersede" => Ok(Self::Superseded),
            "blocked" | "block" => Ok(Self::Blocked),
            other => anyhow::bail!(
                "Unknown Friday handoff completion state `{}`. Use draft, completed, manually-sent, held, revoked, superseded, or blocked.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffCompletionRequest {
    pub state: FridayReleaseHandoffCompletionState,
    pub operator: String,
    pub outcome_note: String,
    pub external_reference: Option<String>,
    pub supersedes_completion_id: Option<String>,
}

impl Default for FridayReleaseHandoffCompletionRequest {
    fn default() -> Self {
        Self {
            state: FridayReleaseHandoffCompletionState::Draft,
            operator: "operator".to_string(),
            outcome_note: "Recorded governed local handoff outcome.".to_string(),
            external_reference: None,
            supersedes_completion_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffCompletionRecord {
    pub completion_id: String,
    pub governance_review_id: String,
    pub governance_review_json: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub state: FridayReleaseHandoffCompletionState,
    pub operator: String,
    pub outcome_note: String,
    pub external_reference: Option<String>,
    pub supersedes_completion_id: Option<String>,
    pub governance_state: FridayReleaseHandoffDispatchGovernanceState,
    pub governance_status: FridayDashboardPanelStatus,
    pub governance_score_out_of_100: u8,
    pub approved_for_external_handoff: bool,
    pub latest_audit_id: Option<String>,
    pub latest_checklist_id: Option<String>,
    pub active_audit_id: Option<String>,
    pub active_checklist_id: Option<String>,
    pub finding_count: usize,
    pub release_gate_blocking_count: usize,
    pub blocker_carryover_count: usize,
    pub unresolved_blocker_count: usize,
    pub active: bool,
    pub externally_mutated_by_friday: bool,
    pub completion_notes: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffCompletionLedger {
    pub ledger_id: String,
    pub ledger_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub record_count: usize,
    pub draft_count: usize,
    pub completed_count: usize,
    pub manually_sent_count: usize,
    pub held_count: usize,
    pub revoked_count: usize,
    pub superseded_count: usize,
    pub blocked_count: usize,
    pub active_completion_id: Option<String>,
    pub latest_completion_id: Option<String>,
    pub latest_state: Option<FridayReleaseHandoffCompletionState>,
    pub latest_governance_review_id: Option<String>,
    pub latest_governance_state: Option<FridayReleaseHandoffDispatchGovernanceState>,
    pub approved_outcome_count: usize,
    pub blocked_outcome_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub records: Vec<FridayReleaseHandoffCompletionRecord>,
    pub completion_summary_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseHandoffCompletionLedger {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_handoff_completion_ledger_report(
    ledger_path: impl AsRef<Path>,
    mut records: Vec<FridayReleaseHandoffCompletionRecord>,
) -> FridayReleaseHandoffCompletionLedger {
    let ledger_path = ledger_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    records.sort_by(|left, right| {
        left.recorded_at_unix_ms
            .cmp(&right.recorded_at_unix_ms)
            .then_with(|| left.completion_id.cmp(&right.completion_id))
    });
    records.dedup_by(|left, right| left.completion_id == right.completion_id);
    let latest = records.last();
    let active = records
        .iter()
        .rev()
        .find(|record| {
            !matches!(
                record.state,
                FridayReleaseHandoffCompletionState::Superseded
                    | FridayReleaseHandoffCompletionState::Revoked
            )
        })
        .or(latest);
    let ledger_json = path_string(ledger_path);
    let approved_outcome_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.state,
                FridayReleaseHandoffCompletionState::Completed
                    | FridayReleaseHandoffCompletionState::ManuallySent
            ) && record.approved_for_external_handoff
                && record.release_gate_blocking_count == 0
        })
        .count();
    let blocked_outcome_count = records
        .iter()
        .filter(|record| {
            record.state == FridayReleaseHandoffCompletionState::Blocked
                || record.release_gate_blocking_count > 0
                || record.unresolved_blocker_count > 0
        })
        .count();
    let release_gate_blocking_count = active
        .map(|record| record.release_gate_blocking_count)
        .unwrap_or(0);
    let unresolved_blocker_count = active
        .map(|record| record.unresolved_blocker_count)
        .unwrap_or(0);

    FridayReleaseHandoffCompletionLedger {
        ledger_id: format!("friday-release-handoff-completion-ledger-{generated_at_unix_ms}"),
        ledger_json: ledger_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: records.len(),
        draft_count: state_count(&records, FridayReleaseHandoffCompletionState::Draft),
        completed_count: state_count(&records, FridayReleaseHandoffCompletionState::Completed),
        manually_sent_count: state_count(
            &records,
            FridayReleaseHandoffCompletionState::ManuallySent,
        ),
        held_count: state_count(&records, FridayReleaseHandoffCompletionState::Held),
        revoked_count: state_count(&records, FridayReleaseHandoffCompletionState::Revoked),
        superseded_count: state_count(&records, FridayReleaseHandoffCompletionState::Superseded),
        blocked_count: state_count(&records, FridayReleaseHandoffCompletionState::Blocked),
        active_completion_id: active.map(|record| record.completion_id.clone()),
        latest_completion_id: latest.map(|record| record.completion_id.clone()),
        latest_state: latest.map(|record| record.state),
        latest_governance_review_id: latest.map(|record| record.governance_review_id.clone()),
        latest_governance_state: latest.map(|record| record.governance_state),
        approved_outcome_count,
        blocked_outcome_count,
        release_gate_blocking_count,
        unresolved_blocker_count,
        completion_summary_copy: completion_summary_copy(&records),
        summary: format!(
            "Friday release handoff completion ledger has {} record(s), {} completed, {} manually sent, {} held, {} blocked, and {} approved governed outcome(s).",
            records.len(),
            state_count(&records, FridayReleaseHandoffCompletionState::Completed),
            state_count(&records, FridayReleaseHandoffCompletionState::ManuallySent),
            state_count(&records, FridayReleaseHandoffCompletionState::Held),
            state_count(&records, FridayReleaseHandoffCompletionState::Blocked),
            approved_outcome_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-handoff-completion --ledger {} --governance-review <release-handoff-dispatch-governance-review.json> --state draft --operator <name>",
                ledger_json
            ),
            format!(
                "flow --friday-release-handoff-completion-list --ledger {}",
                ledger_json
            ),
            format!(
                "flow --friday-release-handoff-completion-export --ledger {} --output {}",
                ledger_json, ledger_json
            ),
            format!(
                "flow --friday-release-handoff-completion-json --ledger {} --governance-review <release-handoff-dispatch-governance-review.json>",
                ledger_json
            ),
        ],
        records,
    }
}

pub fn append_friday_release_handoff_completion_to_ledger(
    ledger_path: impl AsRef<Path>,
    governance_review_path: impl AsRef<Path>,
    request: FridayReleaseHandoffCompletionRequest,
) -> Result<FridayReleaseHandoffCompletionLedger> {
    let ledger_path = ledger_path.as_ref();
    let governance_review_path = governance_review_path.as_ref();
    let mut records = read_friday_release_handoff_completion_ledger(ledger_path)
        .map(|ledger| ledger.records)
        .unwrap_or_default();
    records.push(
        friday_release_handoff_completion_record_from_governance_review(
            governance_review_path,
            request,
        )?,
    );
    let ledger = friday_release_handoff_completion_ledger_report(ledger_path, records);
    write_friday_release_handoff_completion_ledger(ledger_path, &ledger)?;
    Ok(ledger)
}

pub fn friday_release_handoff_completion_record_from_governance_review(
    governance_review_path: impl AsRef<Path>,
    request: FridayReleaseHandoffCompletionRequest,
) -> Result<FridayReleaseHandoffCompletionRecord> {
    let governance_review_path = governance_review_path.as_ref();
    let review = read_friday_release_handoff_dispatch_governance_review(governance_review_path)?;
    Ok(completion_record(governance_review_path, &review, request))
}

pub fn write_friday_release_handoff_completion_ledger(
    ledger_path: impl AsRef<Path>,
    ledger: &FridayReleaseHandoffCompletionLedger,
) -> Result<()> {
    let ledger_path = ledger_path.as_ref();
    if let Some(parent) = ledger_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release handoff completion ledger directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(ledger_path, ledger.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release handoff completion ledger {}",
            ledger_path.display()
        )
    })
}

pub fn read_friday_release_handoff_completion_ledger(
    ledger_path: impl AsRef<Path>,
) -> Result<FridayReleaseHandoffCompletionLedger> {
    let ledger_path = ledger_path.as_ref();
    let bytes = fs::read(ledger_path).with_context(|| {
        format!(
            "Could not read Friday release handoff completion ledger {}",
            ledger_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release handoff completion ledger {}",
            ledger_path.display()
        )
    })
}

fn completion_record(
    governance_review_path: &Path,
    review: &FridayReleaseHandoffDispatchGovernanceReview,
    request: FridayReleaseHandoffCompletionRequest,
) -> FridayReleaseHandoffCompletionRecord {
    let recorded_at_unix_ms = unix_ms();
    let release_gate_blocking_count = review.release_gate_blocking_count;
    let blocked_by_governance = !review.approved_for_external_handoff
        || release_gate_blocking_count > 0
        || review.unresolved_blocker_count > 0;
    let state = if blocked_by_governance
        && matches!(
            request.state,
            FridayReleaseHandoffCompletionState::Completed
                | FridayReleaseHandoffCompletionState::ManuallySent
        ) {
        FridayReleaseHandoffCompletionState::Blocked
    } else {
        request.state
    };
    let active = !matches!(
        state,
        FridayReleaseHandoffCompletionState::Revoked
            | FridayReleaseHandoffCompletionState::Superseded
    );
    let completion_notes = format!(
        "Friday handoff completion: {}\nOperator: {}\nGovernance: {}\nOutcome: {}\nExternal reference: {}\nNo external mutation by Friday: true",
        state.label(),
        request.operator,
        review.review_id,
        request.outcome_note,
        request
            .external_reference
            .as_deref()
            .unwrap_or("not-recorded")
    );
    let summary = format!(
        "{} recorded handoff completion {} as {} with {} release-gate blocker(s).",
        request.operator,
        review.review_id,
        state.label(),
        release_gate_blocking_count
    );

    FridayReleaseHandoffCompletionRecord {
        completion_id: format!(
            "friday-release-handoff-completion-{}-{recorded_at_unix_ms}",
            review.review_id
        ),
        governance_review_id: review.review_id.clone(),
        governance_review_json: path_string(governance_review_path),
        recorded_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        state,
        operator: request.operator,
        outcome_note: request.outcome_note,
        external_reference: request.external_reference,
        supersedes_completion_id: request.supersedes_completion_id,
        governance_state: review.state,
        governance_status: review.status,
        governance_score_out_of_100: review.score_out_of_100,
        approved_for_external_handoff: review.approved_for_external_handoff,
        latest_audit_id: review.latest_audit_id.clone(),
        latest_checklist_id: review.latest_checklist_id.clone(),
        active_audit_id: review.active_audit_id.clone(),
        active_checklist_id: review.active_checklist_id.clone(),
        finding_count: review.finding_count,
        release_gate_blocking_count,
        blocker_carryover_count: review.blocked_carryover_count,
        unresolved_blocker_count: review.unresolved_blocker_count,
        active,
        externally_mutated_by_friday: false,
        completion_notes,
        summary,
    }
}

fn state_count(
    records: &[FridayReleaseHandoffCompletionRecord],
    state: FridayReleaseHandoffCompletionState,
) -> usize {
    records
        .iter()
        .filter(|record| record.state == state)
        .count()
}

fn completion_summary_copy(records: &[FridayReleaseHandoffCompletionRecord]) -> String {
    let mut lines = vec!["Friday release handoff completion ledger".to_string()];
    for record in records.iter().rev().take(8) {
        lines.push(format!(
            "- {} [{}] {} -> {}",
            record.operator,
            record.state.label(),
            record.governance_review_id,
            record.outcome_note
        ));
        if record.release_gate_blocking_count > 0 || record.unresolved_blocker_count > 0 {
            lines.push(format!(
                "  release gate blockers: {}, unresolved blockers: {}",
                record.release_gate_blocking_count, record.unresolved_blocker_count
            ));
        }
        if record.state == FridayReleaseHandoffCompletionState::ManuallySent {
            lines.push("  manual send was recorded; Friday did not send externally".to_string());
        }
    }
    if lines.len() == 1 {
        lines.push("No handoff completion records are recorded.".to_string());
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
