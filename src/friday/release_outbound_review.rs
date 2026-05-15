use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleasePublicationControl, FridayReleasePublicationState,
    read_friday_release_publication_control,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseOutboundReviewState {
    Draft,
    Reviewed,
    ChangesRequested,
    Held,
    Blocked,
    ManuallyPublished,
    Revoked,
    Superseded,
}

impl FridayReleaseOutboundReviewState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Reviewed => "reviewed",
            Self::ChangesRequested => "changes-requested",
            Self::Held => "held",
            Self::Blocked => "blocked",
            Self::ManuallyPublished => "manually-published",
            Self::Revoked => "revoked",
            Self::Superseded => "superseded",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "reviewed" | "review" | "approved" => Ok(Self::Reviewed),
            "changes-requested" | "changes" | "request-changes" => Ok(Self::ChangesRequested),
            "held" | "hold" => Ok(Self::Held),
            "blocked" | "block" => Ok(Self::Blocked),
            "manually-published" | "manual" | "manual-publish" | "published" => {
                Ok(Self::ManuallyPublished)
            }
            "revoked" | "revoke" => Ok(Self::Revoked),
            "superseded" | "supersede" => Ok(Self::Superseded),
            other => anyhow::bail!(
                "Unknown Friday release outbound review state `{}`. Use draft, reviewed, changes-requested, held, blocked, manually-published, revoked, or superseded.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseOutboundReviewRequest {
    pub state: FridayReleaseOutboundReviewState,
    pub reviewer: String,
    pub review_note: String,
    pub manual_publication_reference: Option<String>,
    pub supersedes_review_id: Option<String>,
}

impl Default for FridayReleaseOutboundReviewRequest {
    fn default() -> Self {
        Self {
            state: FridayReleaseOutboundReviewState::Draft,
            reviewer: "operator".to_string(),
            review_note: "Reviewed local-only outbound release copy.".to_string(),
            manual_publication_reference: None,
            supersedes_review_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseOutboundReviewRecord {
    pub review_id: String,
    pub publication_control_id: String,
    pub publication_control_json: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub state: FridayReleaseOutboundReviewState,
    pub reviewer: String,
    pub review_note: String,
    pub manual_publication_reference: Option<String>,
    pub supersedes_review_id: Option<String>,
    pub publication_state: FridayReleasePublicationState,
    pub publication_status: FridayDashboardPanelStatus,
    pub ready_to_publish: bool,
    pub publication_score_out_of_100: u8,
    pub publication_blocker_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub active_completion_id: Option<String>,
    pub latest_governance_review_id: Option<String>,
    pub release_notes_copy: String,
    pub deployment_note_copy: String,
    pub announcement_copy: String,
    pub external_send_instructions_copy: String,
    pub active: bool,
    pub copy_safe: bool,
    pub externally_mutated_by_friday: bool,
    pub review_notes_copy: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseOutboundReviewLedger {
    pub ledger_id: String,
    pub ledger_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub record_count: usize,
    pub draft_count: usize,
    pub reviewed_count: usize,
    pub changes_requested_count: usize,
    pub held_count: usize,
    pub blocked_count: usize,
    pub manually_published_count: usize,
    pub revoked_count: usize,
    pub superseded_count: usize,
    pub active_review_id: Option<String>,
    pub latest_review_id: Option<String>,
    pub latest_state: Option<FridayReleaseOutboundReviewState>,
    pub latest_publication_control_id: Option<String>,
    pub latest_publication_state: Option<FridayReleasePublicationState>,
    pub copy_safe_count: usize,
    pub reviewed_safe_count: usize,
    pub blocked_review_count: usize,
    pub manual_publication_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub records: Vec<FridayReleaseOutboundReviewRecord>,
    pub outbound_summary_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseOutboundReviewLedger {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_outbound_review_ledger_report(
    ledger_path: impl AsRef<Path>,
    mut records: Vec<FridayReleaseOutboundReviewRecord>,
) -> FridayReleaseOutboundReviewLedger {
    let ledger_path = ledger_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    records.sort_by(|left, right| {
        left.recorded_at_unix_ms
            .cmp(&right.recorded_at_unix_ms)
            .then_with(|| left.review_id.cmp(&right.review_id))
    });
    records.dedup_by(|left, right| left.review_id == right.review_id);
    let latest = records.last();
    let active = records
        .iter()
        .rev()
        .find(|record| {
            !matches!(
                record.state,
                FridayReleaseOutboundReviewState::Revoked
                    | FridayReleaseOutboundReviewState::Superseded
            )
        })
        .or(latest);
    let copy_safe_count = records.iter().filter(|record| record.copy_safe).count();
    let blocked_review_count = records
        .iter()
        .filter(|record| {
            record.state == FridayReleaseOutboundReviewState::Blocked
                || record.release_gate_blocking_count > 0
                || record.unresolved_blocker_count > 0
                || record.publication_blocker_count > 0
                || !record.ready_to_publish
        })
        .count();
    let release_gate_blocking_count = active
        .map(|record| record.release_gate_blocking_count)
        .unwrap_or(0);
    let unresolved_blocker_count = active
        .map(|record| record.unresolved_blocker_count)
        .unwrap_or(0);
    let ledger_json = path_string(ledger_path);
    let manually_published_count = state_count(
        &records,
        FridayReleaseOutboundReviewState::ManuallyPublished,
    );

    FridayReleaseOutboundReviewLedger {
        ledger_id: format!("friday-release-outbound-review-ledger-{generated_at_unix_ms}"),
        ledger_json: ledger_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: records.len(),
        draft_count: state_count(&records, FridayReleaseOutboundReviewState::Draft),
        reviewed_count: state_count(&records, FridayReleaseOutboundReviewState::Reviewed),
        changes_requested_count: state_count(
            &records,
            FridayReleaseOutboundReviewState::ChangesRequested,
        ),
        held_count: state_count(&records, FridayReleaseOutboundReviewState::Held),
        blocked_count: state_count(&records, FridayReleaseOutboundReviewState::Blocked),
        manually_published_count: state_count(
            &records,
            FridayReleaseOutboundReviewState::ManuallyPublished,
        ),
        revoked_count: state_count(&records, FridayReleaseOutboundReviewState::Revoked),
        superseded_count: state_count(&records, FridayReleaseOutboundReviewState::Superseded),
        active_review_id: active.map(|record| record.review_id.clone()),
        latest_review_id: latest.map(|record| record.review_id.clone()),
        latest_state: latest.map(|record| record.state),
        latest_publication_control_id: latest.map(|record| record.publication_control_id.clone()),
        latest_publication_state: latest.map(|record| record.publication_state),
        copy_safe_count,
        reviewed_safe_count: copy_safe_count,
        blocked_review_count,
        manual_publication_count: manually_published_count,
        release_gate_blocking_count,
        unresolved_blocker_count,
        outbound_summary_copy: outbound_summary_copy(&records),
        summary: format!(
            "Friday release outbound review ledger has {} record(s), {} reviewed, {} changes requested, {} manually published, {} blocked, and {} safe reviewed outcome(s).",
            records.len(),
            state_count(&records, FridayReleaseOutboundReviewState::Reviewed),
            state_count(&records, FridayReleaseOutboundReviewState::ChangesRequested),
            manually_published_count,
            state_count(&records, FridayReleaseOutboundReviewState::Blocked),
            copy_safe_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-outbound-review --ledger {} --publication-control <release-publication-control.json> --state draft --operator <name>",
                ledger_json
            ),
            format!(
                "flow --friday-release-outbound-review-list --ledger {}",
                ledger_json
            ),
            format!(
                "flow --friday-release-outbound-review-export --ledger {} --output {}",
                ledger_json, ledger_json
            ),
            format!(
                "flow --friday-release-outbound-review-json --ledger {} --publication-control <release-publication-control.json>",
                ledger_json
            ),
        ],
        records,
    }
}

pub fn append_friday_release_outbound_review_to_ledger(
    ledger_path: impl AsRef<Path>,
    publication_control_path: impl AsRef<Path>,
    request: FridayReleaseOutboundReviewRequest,
) -> Result<FridayReleaseOutboundReviewLedger> {
    let ledger_path = ledger_path.as_ref();
    let publication_control_path = publication_control_path.as_ref();
    let mut records = read_friday_release_outbound_review_ledger(ledger_path)
        .map(|ledger| ledger.records)
        .unwrap_or_default();
    records.push(
        friday_release_outbound_review_record_from_publication_control(
            publication_control_path,
            request,
        )?,
    );
    let ledger = friday_release_outbound_review_ledger_report(ledger_path, records);
    write_friday_release_outbound_review_ledger(ledger_path, &ledger)?;
    Ok(ledger)
}

pub fn friday_release_outbound_review_record_from_publication_control(
    publication_control_path: impl AsRef<Path>,
    request: FridayReleaseOutboundReviewRequest,
) -> Result<FridayReleaseOutboundReviewRecord> {
    let publication_control_path = publication_control_path.as_ref();
    let control = read_friday_release_publication_control(publication_control_path)?;
    Ok(outbound_review_record(
        publication_control_path,
        &control,
        request,
    ))
}

pub fn write_friday_release_outbound_review_ledger(
    ledger_path: impl AsRef<Path>,
    ledger: &FridayReleaseOutboundReviewLedger,
) -> Result<()> {
    let ledger_path = ledger_path.as_ref();
    if let Some(parent) = ledger_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release outbound review ledger directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(ledger_path, ledger.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release outbound review ledger {}",
            ledger_path.display()
        )
    })
}

pub fn read_friday_release_outbound_review_ledger(
    ledger_path: impl AsRef<Path>,
) -> Result<FridayReleaseOutboundReviewLedger> {
    let ledger_path = ledger_path.as_ref();
    let bytes = fs::read(ledger_path).with_context(|| {
        format!(
            "Could not read Friday release outbound review ledger {}",
            ledger_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release outbound review ledger {}",
            ledger_path.display()
        )
    })
}

fn outbound_review_record(
    publication_control_path: &Path,
    control: &FridayReleasePublicationControl,
    request: FridayReleaseOutboundReviewRequest,
) -> FridayReleaseOutboundReviewRecord {
    let recorded_at_unix_ms = unix_ms();
    let manual_reference = request
        .manual_publication_reference
        .clone()
        .or_else(|| control.manual_publication_reference.clone());
    let unsafe_review = !control.ready_to_publish
        || control.release_gate_blocking_count > 0
        || control.unresolved_blocker_count > 0
        || control.publication_blocker_count > 0;
    let manual_reference_missing = request.state
        == FridayReleaseOutboundReviewState::ManuallyPublished
        && manual_reference
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty();
    let state = if (unsafe_review || manual_reference_missing)
        && matches!(
            request.state,
            FridayReleaseOutboundReviewState::Reviewed
                | FridayReleaseOutboundReviewState::ManuallyPublished
        ) {
        FridayReleaseOutboundReviewState::Blocked
    } else {
        request.state
    };
    let active = !matches!(
        state,
        FridayReleaseOutboundReviewState::Revoked | FridayReleaseOutboundReviewState::Superseded
    );
    let copy_safe = matches!(
        state,
        FridayReleaseOutboundReviewState::Reviewed
            | FridayReleaseOutboundReviewState::ManuallyPublished
    ) && !unsafe_review
        && !manual_reference_missing;
    let review_notes_copy = format!(
        "Friday release outbound review\nState: {}\nReviewer: {}\nPublication control: {}\nReady to publish: {}\nManual reference: {}\nNote: {}\nFriday did not send, publish, deploy, upload, or email.\nFriday external mutation: false",
        state.label(),
        request.reviewer,
        control.control_id,
        control.ready_to_publish,
        manual_reference.as_deref().unwrap_or("not-recorded"),
        request.review_note
    );
    let summary = format!(
        "{} recorded outbound review {} as {} with {} publication blocker(s).",
        request.reviewer,
        control.control_id,
        state.label(),
        control.release_gate_blocking_count
    );

    FridayReleaseOutboundReviewRecord {
        review_id: format!(
            "friday-release-outbound-review-{}-{recorded_at_unix_ms}",
            control.control_id
        ),
        publication_control_id: control.control_id.clone(),
        publication_control_json: path_string(publication_control_path),
        recorded_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        state,
        reviewer: request.reviewer,
        review_note: request.review_note,
        manual_publication_reference: manual_reference,
        supersedes_review_id: request.supersedes_review_id,
        publication_state: control.state,
        publication_status: control.status,
        ready_to_publish: control.ready_to_publish,
        publication_score_out_of_100: control.score_out_of_100,
        publication_blocker_count: control.publication_blocker_count,
        release_gate_blocking_count: control.release_gate_blocking_count,
        unresolved_blocker_count: control.unresolved_blocker_count,
        active_completion_id: control.active_completion_id.clone(),
        latest_governance_review_id: control.latest_governance_review_id.clone(),
        release_notes_copy: control.release_notes_copy.clone(),
        deployment_note_copy: control.deployment_note_copy.clone(),
        announcement_copy: control.announcement_copy.clone(),
        external_send_instructions_copy: control.external_send_instructions_copy.clone(),
        active,
        copy_safe,
        externally_mutated_by_friday: false,
        review_notes_copy,
        summary,
    }
}

fn state_count(
    records: &[FridayReleaseOutboundReviewRecord],
    state: FridayReleaseOutboundReviewState,
) -> usize {
    records
        .iter()
        .filter(|record| record.state == state)
        .count()
}

fn outbound_summary_copy(records: &[FridayReleaseOutboundReviewRecord]) -> String {
    let mut lines = vec!["Friday release outbound review".to_string()];
    for record in records.iter().rev().take(8) {
        lines.push(format!(
            "- {} [{}] {} -> {}",
            record.reviewer,
            record.state.label(),
            record.publication_control_id,
            record.review_note
        ));
        if record.release_gate_blocking_count > 0 {
            lines.push(format!(
                "  publication blockers: {}",
                record.release_gate_blocking_count
            ));
        }
        if record.state == FridayReleaseOutboundReviewState::ManuallyPublished {
            lines.push(
                "  manual publication recorded; Friday did not publish externally".to_string(),
            );
        }
    }
    if lines.len() == 1 {
        lines.push("No outbound review records are recorded.".to_string());
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
