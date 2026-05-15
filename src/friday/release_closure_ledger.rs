use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseExternalReceiptState,
    FridayReleaseOutboundReviewState, FridayReleaseReceiptReviewBoardReport,
    FridayReleaseReceiptReviewDecision, read_friday_release_receipt_review_board_report,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseClosureState {
    Draft,
    Closed,
    Held,
    Carryover,
    Blocked,
    Revoked,
    Superseded,
}

impl FridayReleaseClosureState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Closed => "closed",
            Self::Held => "held",
            Self::Carryover => "carryover",
            Self::Blocked => "blocked",
            Self::Revoked => "revoked",
            Self::Superseded => "superseded",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "closed" | "close" | "complete" | "completed" | "done" => Ok(Self::Closed),
            "held" | "hold" => Ok(Self::Held),
            "carryover" | "carry-over" | "carried-over" => Ok(Self::Carryover),
            "blocked" | "block" => Ok(Self::Blocked),
            "revoked" | "revoke" => Ok(Self::Revoked),
            "superseded" | "supersede" => Ok(Self::Superseded),
            other => anyhow::bail!(
                "Unknown Friday release closure state `{}`. Use draft, closed, held, carryover, blocked, revoked, or superseded.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseClosureRequest {
    pub state: FridayReleaseClosureState,
    pub operator: String,
    pub closure_note: String,
    pub external_reference: Option<String>,
    pub carryover_commitment: Option<String>,
    pub supersedes_closure_id: Option<String>,
}

impl Default for FridayReleaseClosureRequest {
    fn default() -> Self {
        Self {
            state: FridayReleaseClosureState::Draft,
            operator: "operator".to_string(),
            closure_note: "Recorded local release closure outcome.".to_string(),
            external_reference: None,
            carryover_commitment: None,
            supersedes_closure_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseClosureRecord {
    pub closure_id: String,
    pub receipt_review_id: String,
    pub receipt_review_json: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub state: FridayReleaseClosureState,
    pub operator: String,
    pub closure_note: String,
    pub external_reference: Option<String>,
    pub carryover_commitment: Option<String>,
    pub supersedes_closure_id: Option<String>,
    pub review_decision: FridayReleaseReceiptReviewDecision,
    pub review_status: FridayDashboardPanelStatus,
    pub review_score_out_of_100: u8,
    pub ready_for_external_completion: bool,
    pub archive_id: String,
    pub active_receipt_id: Option<String>,
    pub latest_receipt_state: Option<FridayReleaseExternalReceiptState>,
    pub latest_outbound_review_id: Option<String>,
    pub latest_outbound_review_state: Option<FridayReleaseOutboundReviewState>,
    pub finding_count: usize,
    pub verified_receipt_count: usize,
    pub stale_or_missing_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub active: bool,
    pub externally_mutated_by_friday: bool,
    pub closure_notes_copy: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseClosureLedger {
    pub ledger_id: String,
    pub ledger_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub record_count: usize,
    pub draft_count: usize,
    pub closed_count: usize,
    pub held_count: usize,
    pub carryover_count: usize,
    pub blocked_count: usize,
    pub revoked_count: usize,
    pub superseded_count: usize,
    pub active_closure_id: Option<String>,
    pub latest_closure_id: Option<String>,
    pub latest_state: Option<FridayReleaseClosureState>,
    pub latest_receipt_review_id: Option<String>,
    pub latest_review_decision: Option<FridayReleaseReceiptReviewDecision>,
    pub closed_outcome_count: usize,
    pub carryover_outcome_count: usize,
    pub blocked_outcome_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub records: Vec<FridayReleaseClosureRecord>,
    pub closure_summary_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseClosureLedger {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_closure_ledger_report(
    ledger_path: impl AsRef<Path>,
    mut records: Vec<FridayReleaseClosureRecord>,
) -> FridayReleaseClosureLedger {
    let ledger_path = ledger_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    records.sort_by(|left, right| {
        left.recorded_at_unix_ms
            .cmp(&right.recorded_at_unix_ms)
            .then_with(|| left.closure_id.cmp(&right.closure_id))
    });
    records.dedup_by(|left, right| left.closure_id == right.closure_id);
    let latest = records.last();
    let active = records
        .iter()
        .rev()
        .find(|record| {
            !matches!(
                record.state,
                FridayReleaseClosureState::Revoked | FridayReleaseClosureState::Superseded
            )
        })
        .or(latest);
    let ledger_json = path_string(ledger_path);
    let closed_outcome_count = records
        .iter()
        .filter(|record| {
            record.state == FridayReleaseClosureState::Closed
                && record.ready_for_external_completion
                && record.release_gate_blocking_count == 0
                && record.unresolved_blocker_count == 0
                && !record.externally_mutated_by_friday
        })
        .count();
    let carryover_outcome_count = records
        .iter()
        .filter(|record| {
            record.state == FridayReleaseClosureState::Carryover
                || record.carryover_commitment.is_some()
        })
        .count();
    let blocked_outcome_count = records
        .iter()
        .filter(|record| {
            record.state == FridayReleaseClosureState::Blocked
                || record.release_gate_blocking_count > 0
                || record.unresolved_blocker_count > 0
                || !record.ready_for_external_completion
        })
        .count();
    let release_gate_blocking_count = active
        .map(|record| record.release_gate_blocking_count)
        .unwrap_or(0);
    let unresolved_blocker_count = active
        .map(|record| record.unresolved_blocker_count)
        .unwrap_or(0);

    FridayReleaseClosureLedger {
        ledger_id: format!("friday-release-closure-ledger-{generated_at_unix_ms}"),
        ledger_json: ledger_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: records.len(),
        draft_count: state_count(&records, FridayReleaseClosureState::Draft),
        closed_count: state_count(&records, FridayReleaseClosureState::Closed),
        held_count: state_count(&records, FridayReleaseClosureState::Held),
        carryover_count: state_count(&records, FridayReleaseClosureState::Carryover),
        blocked_count: state_count(&records, FridayReleaseClosureState::Blocked),
        revoked_count: state_count(&records, FridayReleaseClosureState::Revoked),
        superseded_count: state_count(&records, FridayReleaseClosureState::Superseded),
        active_closure_id: active.map(|record| record.closure_id.clone()),
        latest_closure_id: latest.map(|record| record.closure_id.clone()),
        latest_state: latest.map(|record| record.state),
        latest_receipt_review_id: latest.map(|record| record.receipt_review_id.clone()),
        latest_review_decision: latest.map(|record| record.review_decision),
        closed_outcome_count,
        carryover_outcome_count,
        blocked_outcome_count,
        release_gate_blocking_count,
        unresolved_blocker_count,
        closure_summary_copy: closure_summary_copy(&records),
        summary: format!(
            "Friday release closure ledger has {} record(s), {} closed, {} held, {} carryover, {} blocked, and {} externally complete local outcome(s).",
            records.len(),
            state_count(&records, FridayReleaseClosureState::Closed),
            state_count(&records, FridayReleaseClosureState::Held),
            state_count(&records, FridayReleaseClosureState::Carryover),
            state_count(&records, FridayReleaseClosureState::Blocked),
            closed_outcome_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-closure --ledger {} --receipt-review <release-receipt-review-board.json> --state draft --operator <name>",
                ledger_json
            ),
            format!(
                "flow --friday-release-closure-list --ledger {}",
                ledger_json
            ),
            format!(
                "flow --friday-release-closure-export --ledger {} --output {}",
                ledger_json, ledger_json
            ),
            format!(
                "flow --friday-release-closure-json --ledger {} --receipt-review <release-receipt-review-board.json>",
                ledger_json
            ),
        ],
        records,
    }
}

pub fn append_friday_release_closure_to_ledger(
    ledger_path: impl AsRef<Path>,
    receipt_review_path: impl AsRef<Path>,
    request: FridayReleaseClosureRequest,
) -> Result<FridayReleaseClosureLedger> {
    let ledger_path = ledger_path.as_ref();
    let receipt_review_path = receipt_review_path.as_ref();
    let mut records = read_friday_release_closure_ledger(ledger_path)
        .map(|ledger| ledger.records)
        .unwrap_or_default();
    records.push(friday_release_closure_record_from_receipt_review(
        receipt_review_path,
        request,
    )?);
    let ledger = friday_release_closure_ledger_report(ledger_path, records);
    write_friday_release_closure_ledger(ledger_path, &ledger)?;
    Ok(ledger)
}

pub fn friday_release_closure_record_from_receipt_review(
    receipt_review_path: impl AsRef<Path>,
    request: FridayReleaseClosureRequest,
) -> Result<FridayReleaseClosureRecord> {
    let receipt_review_path = receipt_review_path.as_ref();
    let review = read_friday_release_receipt_review_board_report(receipt_review_path)?;
    Ok(closure_record(receipt_review_path, &review, request))
}

pub fn write_friday_release_closure_ledger(
    ledger_path: impl AsRef<Path>,
    ledger: &FridayReleaseClosureLedger,
) -> Result<()> {
    let ledger_path = ledger_path.as_ref();
    if let Some(parent) = ledger_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release closure ledger directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(ledger_path, ledger.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release closure ledger {}",
            ledger_path.display()
        )
    })
}

pub fn read_friday_release_closure_ledger(
    ledger_path: impl AsRef<Path>,
) -> Result<FridayReleaseClosureLedger> {
    let ledger_path = ledger_path.as_ref();
    let bytes = fs::read(ledger_path).with_context(|| {
        format!(
            "Could not read Friday release closure ledger {}",
            ledger_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release closure ledger {}",
            ledger_path.display()
        )
    })
}

fn closure_record(
    receipt_review_path: &Path,
    review: &FridayReleaseReceiptReviewBoardReport,
    request: FridayReleaseClosureRequest,
) -> FridayReleaseClosureRecord {
    let recorded_at_unix_ms = unix_ms();
    let blocked_by_review = !review.ready_for_external_completion
        || review.release_gate_blocking_count > 0
        || review.unresolved_blocker_count > 0
        || matches!(
            review.decision,
            FridayReleaseReceiptReviewDecision::BlockedReview
                | FridayReleaseReceiptReviewDecision::MissingReceipt
                | FridayReleaseReceiptReviewDecision::StaleEvidence
                | FridayReleaseReceiptReviewDecision::RevokedReceipt
        );
    let state = if blocked_by_review && request.state == FridayReleaseClosureState::Closed {
        FridayReleaseClosureState::Blocked
    } else {
        request.state
    };
    let active = !matches!(
        state,
        FridayReleaseClosureState::Revoked | FridayReleaseClosureState::Superseded
    );
    let closure_notes_copy = format!(
        "Friday release closure ledger\nState: {}\nOperator: {}\nReceipt review: {}\nReview decision: {}\nClosure note: {}\nExternal reference: {}\nCarryover: {}\nFriday did not fetch, send, publish, deploy, upload, or email.\nNo external mutation by Friday: true",
        state.label(),
        request.operator,
        review.review_id,
        review.decision.label(),
        request.closure_note,
        request
            .external_reference
            .as_deref()
            .unwrap_or("not-recorded"),
        request
            .carryover_commitment
            .as_deref()
            .unwrap_or("not-recorded")
    );
    let summary = format!(
        "{} recorded release closure {} as {} with {} release-gate blocker(s).",
        request.operator,
        review.review_id,
        state.label(),
        review.release_gate_blocking_count
    );

    FridayReleaseClosureRecord {
        closure_id: format!(
            "friday-release-closure-{}-{recorded_at_unix_ms}",
            review.review_id
        ),
        receipt_review_id: review.review_id.clone(),
        receipt_review_json: path_string(receipt_review_path),
        recorded_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        state,
        operator: request.operator,
        closure_note: request.closure_note,
        external_reference: request.external_reference,
        carryover_commitment: request.carryover_commitment,
        supersedes_closure_id: request.supersedes_closure_id,
        review_decision: review.decision,
        review_status: review.status,
        review_score_out_of_100: review.score_out_of_100,
        ready_for_external_completion: review.ready_for_external_completion,
        archive_id: review.archive_id.clone(),
        active_receipt_id: review.active_receipt_id.clone(),
        latest_receipt_state: review.latest_receipt_state,
        latest_outbound_review_id: review.latest_outbound_review_id.clone(),
        latest_outbound_review_state: review.latest_outbound_review_state,
        finding_count: review.finding_count,
        verified_receipt_count: review.verified_receipt_count,
        stale_or_missing_count: review.stale_or_missing_count,
        release_gate_blocking_count: review.release_gate_blocking_count,
        unresolved_blocker_count: review.unresolved_blocker_count,
        active,
        externally_mutated_by_friday: false,
        closure_notes_copy,
        summary,
    }
}

fn state_count(records: &[FridayReleaseClosureRecord], state: FridayReleaseClosureState) -> usize {
    records
        .iter()
        .filter(|record| record.state == state)
        .count()
}

fn closure_summary_copy(records: &[FridayReleaseClosureRecord]) -> String {
    let mut lines = vec!["Friday release closure ledger".to_string()];
    for record in records.iter().rev().take(8) {
        lines.push(format!(
            "- {} [{}] {} -> {}",
            record.operator,
            record.state.label(),
            record.receipt_review_id,
            record.closure_note
        ));
        if record.release_gate_blocking_count > 0 || record.unresolved_blocker_count > 0 {
            lines.push(format!(
                "  release gate blockers: {}, unresolved blockers: {}",
                record.release_gate_blocking_count, record.unresolved_blocker_count
            ));
        }
        if let Some(carryover) = &record.carryover_commitment {
            lines.push(format!("  carryover: {carryover}"));
        }
        if record.state == FridayReleaseClosureState::Closed {
            lines.push("  closed locally from verified receipt review evidence".to_string());
        }
    }
    if lines.len() == 1 {
        lines.push("No release closure records are recorded.".to_string());
    }
    lines.push("Friday did not fetch, send, publish, deploy, upload, or email.".to_string());
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
