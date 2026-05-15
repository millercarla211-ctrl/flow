use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseHandoffCompletionLedger,
    FridayReleaseHandoffCompletionRecord, FridayReleaseHandoffCompletionState,
    read_friday_release_handoff_completion_ledger,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleasePublicationState {
    Draft,
    Ready,
    Held,
    Blocked,
    PublishedManually,
    Revoked,
    Superseded,
}

impl FridayReleasePublicationState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Ready => "ready",
            Self::Held => "held",
            Self::Blocked => "blocked",
            Self::PublishedManually => "published-manually",
            Self::Revoked => "revoked",
            Self::Superseded => "superseded",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "ready" | "ready-to-publish" => Ok(Self::Ready),
            "held" | "hold" => Ok(Self::Held),
            "blocked" | "block" => Ok(Self::Blocked),
            "published-manually" | "published" | "manual" | "manual-publish" => {
                Ok(Self::PublishedManually)
            }
            "revoked" | "revoke" => Ok(Self::Revoked),
            "superseded" | "supersede" => Ok(Self::Superseded),
            other => anyhow::bail!(
                "Unknown Friday release publication state `{}`. Use draft, ready, held, blocked, published-manually, revoked, or superseded.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleasePublicationBlockerKind {
    MissingCompletionLedger,
    BlockedCompletion,
    UnapprovedCompletion,
    DraftCompletion,
    RevokedCompletion,
    SupersededCompletion,
    UnresolvedBlocker,
    ManualReferenceMissing,
}

impl FridayReleasePublicationBlockerKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::MissingCompletionLedger => "missing-completion-ledger",
            Self::BlockedCompletion => "blocked-completion",
            Self::UnapprovedCompletion => "unapproved-completion",
            Self::DraftCompletion => "draft-completion",
            Self::RevokedCompletion => "revoked-completion",
            Self::SupersededCompletion => "superseded-completion",
            Self::UnresolvedBlocker => "unresolved-blocker",
            Self::ManualReferenceMissing => "manual-reference-missing",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePublicationRequest {
    pub state: FridayReleasePublicationState,
    pub operator: String,
    pub publication_note: String,
    pub manual_publication_reference: Option<String>,
}

impl Default for FridayReleasePublicationRequest {
    fn default() -> Self {
        Self {
            state: FridayReleasePublicationState::Draft,
            operator: "operator".to_string(),
            publication_note: "Prepared local-only publication control.".to_string(),
            manual_publication_reference: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePublicationBlocker {
    pub id: String,
    pub kind: FridayReleasePublicationBlockerKind,
    pub release_gate_blocking: bool,
    pub completion_id: String,
    pub evidence_path: String,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePublicationControl {
    pub control_id: String,
    pub control_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub state: FridayReleasePublicationState,
    pub ready_to_publish: bool,
    pub score_out_of_100: u8,
    pub ledger_id: String,
    pub ledger_json: String,
    pub active_completion_id: Option<String>,
    pub latest_completion_id: Option<String>,
    pub latest_completion_state: Option<FridayReleaseHandoffCompletionState>,
    pub latest_governance_review_id: Option<String>,
    pub completion_record_count: usize,
    pub approved_outcome_count: usize,
    pub blocked_outcome_count: usize,
    pub publication_blocker_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub operator: String,
    pub publication_note: String,
    pub manual_publication_reference: Option<String>,
    pub blockers: Vec<FridayReleasePublicationBlocker>,
    pub release_notes_copy: String,
    pub deployment_note_copy: String,
    pub announcement_copy: String,
    pub external_send_instructions_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleasePublicationControl {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_publication_control_report(
    control_path: impl AsRef<Path>,
    ledger_path: impl AsRef<Path>,
    request: FridayReleasePublicationRequest,
) -> FridayReleasePublicationControl {
    let control_path = control_path.as_ref();
    let ledger_path = ledger_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let ledger = read_friday_release_handoff_completion_ledger(ledger_path).ok();
    let fallback = fallback_completion_ledger(ledger_path);
    let ledger = ledger.as_ref().unwrap_or(&fallback);
    let active = ledger.records.iter().find(|record| {
        Some(record.completion_id.as_str()) == ledger.active_completion_id.as_deref()
    });
    let mut blockers = publication_blockers(ledger, &request);
    blockers.sort_by(|left, right| {
        blocker_rank(left.kind)
            .cmp(&blocker_rank(right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });

    let release_gate_blocking_count = blockers
        .iter()
        .filter(|blocker| blocker.release_gate_blocking)
        .count();
    let ready_to_publish = release_gate_blocking_count == 0
        && active.is_some()
        && matches!(
            request.state,
            FridayReleasePublicationState::Ready | FridayReleasePublicationState::PublishedManually
        );
    let state = if release_gate_blocking_count > 0 {
        FridayReleasePublicationState::Blocked
    } else {
        request.state
    };
    let status = if state == FridayReleasePublicationState::Blocked {
        FridayDashboardPanelStatus::Blocked
    } else if ready_to_publish {
        FridayDashboardPanelStatus::Ready
    } else {
        FridayDashboardPanelStatus::Warning
    };
    let score_out_of_100 = score_publication_control(
        ledger.record_count,
        release_gate_blocking_count,
        ready_to_publish,
    );
    let control_json = path_string(control_path);
    let ledger_json = path_string(ledger_path);
    let release_notes_copy = release_notes_copy(ledger, active, state, &request, &blockers);
    let deployment_note_copy = deployment_note_copy(ledger, active, state, &request);
    let announcement_copy = announcement_copy(ledger, active, state, &request);
    let external_send_instructions_copy =
        external_send_instructions_copy(ledger, active, state, &request, &blockers);

    FridayReleasePublicationControl {
        control_id: format!("friday-release-publication-control-{generated_at_unix_ms}"),
        control_json: control_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        state,
        ready_to_publish,
        score_out_of_100,
        ledger_id: ledger.ledger_id.clone(),
        ledger_json: ledger_json.clone(),
        active_completion_id: ledger.active_completion_id.clone(),
        latest_completion_id: ledger.latest_completion_id.clone(),
        latest_completion_state: ledger.latest_state,
        latest_governance_review_id: ledger.latest_governance_review_id.clone(),
        completion_record_count: ledger.record_count,
        approved_outcome_count: ledger.approved_outcome_count,
        blocked_outcome_count: ledger.blocked_outcome_count,
        publication_blocker_count: blockers.len(),
        release_gate_blocking_count,
        unresolved_blocker_count: ledger.unresolved_blocker_count,
        operator: request.operator,
        publication_note: request.publication_note,
        manual_publication_reference: request.manual_publication_reference,
        blockers,
        release_notes_copy,
        deployment_note_copy,
        announcement_copy,
        external_send_instructions_copy,
        summary: format!(
            "Friday release publication control is {} with score {}/100, {} completion record(s), {} approved outcome(s), {} blocked outcome(s), and {} publication blocker(s).",
            state.label(),
            score_out_of_100,
            ledger.record_count,
            ledger.approved_outcome_count,
            ledger.blocked_outcome_count,
            release_gate_blocking_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-publication-control --output {} --completion-ledger {} --state draft --operator <name>",
                control_json, ledger_json
            ),
            format!(
                "flow --friday-release-publication-control-json --output {} --completion-ledger {} --state draft --operator <name>",
                control_json, ledger_json
            ),
        ],
    }
}

pub fn write_friday_release_publication_control(
    control_path: impl AsRef<Path>,
    control: &FridayReleasePublicationControl,
) -> Result<()> {
    let control_path = control_path.as_ref();
    if let Some(parent) = control_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release publication control directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(control_path, control.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release publication control {}",
            control_path.display()
        )
    })
}

pub fn read_friday_release_publication_control(
    control_path: impl AsRef<Path>,
) -> Result<FridayReleasePublicationControl> {
    let control_path = control_path.as_ref();
    let bytes = fs::read(control_path).with_context(|| {
        format!(
            "Could not read Friday release publication control {}",
            control_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release publication control {}",
            control_path.display()
        )
    })
}

fn publication_blockers(
    ledger: &FridayReleaseHandoffCompletionLedger,
    request: &FridayReleasePublicationRequest,
) -> Vec<FridayReleasePublicationBlocker> {
    let mut blockers = Vec::new();
    if ledger.record_count == 0 {
        blockers.push(blocker(
            "missing-completion-ledger",
            FridayReleasePublicationBlockerKind::MissingCompletionLedger,
            true,
            "missing",
            &ledger.ledger_json,
            "No governed handoff completion record is available.",
            "Record a completed or manually-sent handoff outcome before preparing publication copy.",
        ));
        return blockers;
    }

    let active = ledger.records.iter().find(|record| {
        Some(record.completion_id.as_str()) == ledger.active_completion_id.as_deref()
    });
    let Some(record) = active else {
        blockers.push(blocker(
            "missing-active-completion",
            FridayReleasePublicationBlockerKind::MissingCompletionLedger,
            true,
            "missing",
            &ledger.ledger_json,
            "No active handoff completion record is available.",
            "Refresh the completion ledger and keep one non-revoked, non-superseded active record.",
        ));
        return blockers;
    };

    match record.state {
        FridayReleaseHandoffCompletionState::Completed
        | FridayReleaseHandoffCompletionState::ManuallySent => {}
        FridayReleaseHandoffCompletionState::Draft | FridayReleaseHandoffCompletionState::Held => {
            blockers.push(blocker(
                "active-completion-not-final",
                FridayReleasePublicationBlockerKind::DraftCompletion,
                true,
                &record.completion_id,
                &record.governance_review_json,
                "The active handoff completion is not final.",
                "Move the completion ledger to completed or manually-sent only after governance is clear.",
            ));
        }
        FridayReleaseHandoffCompletionState::Blocked => blockers.push(blocker(
            "active-completion-blocked",
            FridayReleasePublicationBlockerKind::BlockedCompletion,
            true,
            &record.completion_id,
            &record.governance_review_json,
            "The active handoff completion is blocked.",
            "Resolve completion blockers before preparing release notes or external-send copy.",
        )),
        FridayReleaseHandoffCompletionState::Revoked => blockers.push(blocker(
            "active-completion-revoked",
            FridayReleasePublicationBlockerKind::RevokedCompletion,
            true,
            &record.completion_id,
            &record.governance_review_json,
            "The active handoff completion has been revoked.",
            "Create a new governed completion record before publication control is marked ready.",
        )),
        FridayReleaseHandoffCompletionState::Superseded => blockers.push(blocker(
            "active-completion-superseded",
            FridayReleasePublicationBlockerKind::SupersededCompletion,
            true,
            &record.completion_id,
            &record.governance_review_json,
            "The active handoff completion has been superseded.",
            "Use the replacement completion record before preparing publication copy.",
        )),
    }

    if !record.approved_for_external_handoff {
        blockers.push(blocker(
            "active-completion-unapproved",
            FridayReleasePublicationBlockerKind::UnapprovedCompletion,
            true,
            &record.completion_id,
            &record.governance_review_json,
            "The active handoff completion was not approved for external handoff.",
            "Clear dispatch governance before release notes, deployment notes, or external-send instructions leave local review.",
        ));
    }
    if record.release_gate_blocking_count > 0 || record.unresolved_blocker_count > 0 {
        blockers.push(blocker(
            "active-completion-unresolved-blockers",
            FridayReleasePublicationBlockerKind::UnresolvedBlocker,
            true,
            &record.completion_id,
            &record.governance_review_json,
            "The active completion still carries unresolved blockers.",
            "Resolve blocker carryover and regenerate the completion ledger.",
        ));
    }
    if request.state == FridayReleasePublicationState::PublishedManually
        && request
            .manual_publication_reference
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        blockers.push(blocker(
            "manual-publication-reference-missing",
            FridayReleasePublicationBlockerKind::ManualReferenceMissing,
            true,
            &record.completion_id,
            &record.governance_review_json,
            "Manual publication was requested without an external reference.",
            "Record the human-owned URL, ticket, email, or publication reference before marking published-manually.",
        ));
    }

    blockers
}

fn blocker(
    id: &str,
    kind: FridayReleasePublicationBlockerKind,
    release_gate_blocking: bool,
    completion_id: &str,
    evidence_path: &str,
    summary: &str,
    next_action: &str,
) -> FridayReleasePublicationBlocker {
    FridayReleasePublicationBlocker {
        id: id.to_string(),
        kind,
        release_gate_blocking,
        completion_id: completion_id.to_string(),
        evidence_path: evidence_path.to_string(),
        summary: summary.to_string(),
        next_action: next_action.to_string(),
    }
}

fn blocker_rank(kind: FridayReleasePublicationBlockerKind) -> u8 {
    match kind {
        FridayReleasePublicationBlockerKind::MissingCompletionLedger => 0,
        FridayReleasePublicationBlockerKind::BlockedCompletion => 1,
        FridayReleasePublicationBlockerKind::UnapprovedCompletion => 2,
        FridayReleasePublicationBlockerKind::UnresolvedBlocker => 3,
        FridayReleasePublicationBlockerKind::DraftCompletion => 4,
        FridayReleasePublicationBlockerKind::RevokedCompletion => 5,
        FridayReleasePublicationBlockerKind::SupersededCompletion => 6,
        FridayReleasePublicationBlockerKind::ManualReferenceMissing => 7,
    }
}

fn score_publication_control(
    record_count: usize,
    release_gate_blocking_count: usize,
    ready_to_publish: bool,
) -> u8 {
    if record_count == 0 {
        0
    } else if ready_to_publish {
        100
    } else {
        (85_i16 - (release_gate_blocking_count as i16 * 18)).clamp(10, 85) as u8
    }
}

fn release_notes_copy(
    ledger: &FridayReleaseHandoffCompletionLedger,
    active: Option<&FridayReleaseHandoffCompletionRecord>,
    state: FridayReleasePublicationState,
    request: &FridayReleasePublicationRequest,
    blockers: &[FridayReleasePublicationBlocker],
) -> String {
    let mut lines = vec![
        "Friday release notes".to_string(),
        format!("Publication state: {}", state.label()),
        format!("Operator: {}", request.operator),
        format!("Completion ledger: {}", ledger.ledger_id),
        format!(
            "Active completion: {}",
            ledger.active_completion_id.as_deref().unwrap_or("none")
        ),
        format!(
            "Latest governance review: {}",
            ledger
                .latest_governance_review_id
                .as_deref()
                .unwrap_or("none")
        ),
        format!("Note: {}", request.publication_note),
        "No external publication by Friday: true".to_string(),
    ];
    if let Some(record) = active {
        lines.push(format!("Outcome: {}", record.outcome_note));
    }
    if !blockers.is_empty() {
        lines.push("Publication blockers:".to_string());
        for blocker in blockers {
            lines.push(format!(
                "- [{}] {} -> {}",
                blocker.kind.label(),
                blocker.summary,
                blocker.next_action
            ));
        }
    }
    lines.join("\n")
}

fn deployment_note_copy(
    ledger: &FridayReleaseHandoffCompletionLedger,
    active: Option<&FridayReleaseHandoffCompletionRecord>,
    state: FridayReleasePublicationState,
    request: &FridayReleasePublicationRequest,
) -> String {
    format!(
        "Friday deployment note\nState: {}\nCompletion: {}\nGovernance: {}\nOperator: {}\nNote: {}\nFriday did not deploy or mutate external systems.",
        state.label(),
        ledger.active_completion_id.as_deref().unwrap_or("none"),
        active
            .map(|record| record.governance_review_id.as_str())
            .unwrap_or("none"),
        request.operator,
        request.publication_note
    )
}

fn announcement_copy(
    ledger: &FridayReleaseHandoffCompletionLedger,
    active: Option<&FridayReleaseHandoffCompletionRecord>,
    state: FridayReleasePublicationState,
    request: &FridayReleasePublicationRequest,
) -> String {
    format!(
        "Friday update\nStatus: {}\n{} governed handoff record(s) reviewed. {}\nPrepared by {} for local operator review.",
        state.label(),
        ledger.record_count,
        active
            .map(|record| record.outcome_note.as_str())
            .unwrap_or("No completion outcome is available."),
        request.operator
    )
}

fn external_send_instructions_copy(
    ledger: &FridayReleaseHandoffCompletionLedger,
    active: Option<&FridayReleaseHandoffCompletionRecord>,
    state: FridayReleasePublicationState,
    request: &FridayReleasePublicationRequest,
    blockers: &[FridayReleasePublicationBlocker],
) -> String {
    let mut lines = vec![
        "Friday external-send instructions".to_string(),
        "Friday will not send, publish, deploy, upload, or email this automatically.".to_string(),
        format!("State: {}", state.label()),
        format!(
            "Manual reference: {}",
            request
                .manual_publication_reference
                .as_deref()
                .unwrap_or("not-recorded")
        ),
        format!(
            "Completion: {}",
            active
                .map(|record| record.completion_id.as_str())
                .unwrap_or("none")
        ),
        format!("Ledger: {}", ledger.ledger_json),
    ];
    if blockers.is_empty() {
        lines.push("Operator may manually publish using the copied release notes.".to_string());
    } else {
        lines.push("Do not manually publish until these blockers are resolved:".to_string());
        lines.extend(
            blockers
                .iter()
                .map(|blocker| format!("- {}", blocker.summary)),
        );
    }
    lines.join("\n")
}

fn fallback_completion_ledger(ledger_path: &Path) -> FridayReleaseHandoffCompletionLedger {
    FridayReleaseHandoffCompletionLedger {
        ledger_id: "missing-release-handoff-completion-ledger".to_string(),
        ledger_json: path_string(ledger_path),
        generated_at_unix_ms: 0,
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: 0,
        draft_count: 0,
        completed_count: 0,
        manually_sent_count: 0,
        held_count: 0,
        revoked_count: 0,
        superseded_count: 0,
        blocked_count: 0,
        active_completion_id: None,
        latest_completion_id: None,
        latest_state: None,
        latest_governance_review_id: None,
        latest_governance_state: None,
        approved_outcome_count: 0,
        blocked_outcome_count: 0,
        release_gate_blocking_count: 0,
        unresolved_blocker_count: 0,
        records: Vec::new(),
        completion_summary_copy: "No handoff completion ledger could be loaded.".to_string(),
        summary: "Release handoff completion ledger could not be loaded.".to_string(),
        commands: Vec::new(),
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
