use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayReleaseCheckpointDecision, FridayReleaseCheckpointReviewBoardReport,
    read_friday_release_checkpoint_review_board_report,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseCheckpointSignoffDecision {
    SignedOff,
    Held,
    CarriedOver,
    Superseded,
    Revoked,
}

impl FridayReleaseCheckpointSignoffDecision {
    pub fn label(self) -> &'static str {
        match self {
            Self::SignedOff => "signed-off",
            Self::Held => "held",
            Self::CarriedOver => "carried-over",
            Self::Superseded => "superseded",
            Self::Revoked => "revoked",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "signed-off" | "signed_off" | "signedoff" | "approve" | "approved" | "ready" => {
                Ok(Self::SignedOff)
            }
            "held" | "hold" | "blocked" => Ok(Self::Held),
            "carried-over" | "carried_over" | "carry-over" | "carryover" => Ok(Self::CarriedOver),
            "superseded" | "supersede" => Ok(Self::Superseded),
            "revoked" | "revoke" => Ok(Self::Revoked),
            other => anyhow::bail!(
                "Unknown Friday checkpoint signoff decision `{}`. Use signed-off, held, carried-over, superseded, or revoked.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseCheckpointSignoffRequest {
    pub decision: FridayReleaseCheckpointSignoffDecision,
    pub operator: String,
    pub reason: String,
    pub acknowledgement_evidence_path: String,
    pub carryover_commitment: String,
}

impl Default for FridayReleaseCheckpointSignoffRequest {
    fn default() -> Self {
        Self {
            decision: FridayReleaseCheckpointSignoffDecision::Held,
            operator: "operator".to_string(),
            reason: "Reviewed the checkpoint board and kept the release local-only.".to_string(),
            acknowledgement_evidence_path: String::new(),
            carryover_commitment: "Carry unresolved checkpoint work into the next release loop."
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseCheckpointSignoffRecord {
    pub signoff_id: String,
    pub review_id: String,
    pub review_json: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub decision: FridayReleaseCheckpointSignoffDecision,
    pub operator: String,
    pub reason: String,
    pub acknowledgement_evidence_path: String,
    pub acknowledgement_evidence_present: bool,
    pub acknowledgement_evidence_bytes: u64,
    pub carryover_commitment: String,
    pub review_decision: FridayReleaseCheckpointDecision,
    pub review_score_out_of_100: u8,
    pub review_ready_for_checkpoint: bool,
    pub review_hold_count: usize,
    pub review_carryover_count: usize,
    pub review_acknowledgement_blocker_count: usize,
    pub release_gate_blocking_count: usize,
    pub active_hold: bool,
    pub active_carryover: bool,
    pub release_notes: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseCheckpointSignoffLedger {
    pub ledger_id: String,
    pub ledger_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub record_count: usize,
    pub signed_off_count: usize,
    pub held_count: usize,
    pub carried_over_count: usize,
    pub superseded_count: usize,
    pub revoked_count: usize,
    pub active_signoff_id: Option<String>,
    pub active_review_id: Option<String>,
    pub active_decision: Option<FridayReleaseCheckpointSignoffDecision>,
    pub active_hold_count: usize,
    pub active_carryover_count: usize,
    pub acknowledgement_evidence_missing_count: usize,
    pub release_gate_blocking_count: usize,
    pub records: Vec<FridayReleaseCheckpointSignoffRecord>,
    pub release_notes_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseCheckpointSignoffLedger {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_checkpoint_signoff_ledger_report(
    ledger_path: impl AsRef<Path>,
    mut records: Vec<FridayReleaseCheckpointSignoffRecord>,
) -> FridayReleaseCheckpointSignoffLedger {
    let ledger_path = ledger_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    records.sort_by(|left, right| {
        left.recorded_at_unix_ms
            .cmp(&right.recorded_at_unix_ms)
            .then_with(|| left.signoff_id.cmp(&right.signoff_id))
    });
    records.dedup_by(|left, right| left.signoff_id == right.signoff_id);
    let latest = records.last();
    let active = records
        .iter()
        .rev()
        .find(|record| {
            !matches!(
                record.decision,
                FridayReleaseCheckpointSignoffDecision::Superseded
                    | FridayReleaseCheckpointSignoffDecision::Revoked
            )
        })
        .or(latest);
    let acknowledgement_evidence_missing_count = records
        .iter()
        .filter(|record| {
            record.decision == FridayReleaseCheckpointSignoffDecision::SignedOff
                && record.review_acknowledgement_blocker_count > 0
                && !record.acknowledgement_evidence_present
        })
        .count();
    let release_gate_blocking_count = records
        .iter()
        .filter(|record| record.release_gate_blocking_count > 0 || record.active_hold)
        .count();
    let active_hold_count = records.iter().filter(|record| record.active_hold).count();
    let active_carryover_count = records
        .iter()
        .filter(|record| record.active_carryover)
        .count();
    let ledger_json = path_string(ledger_path);

    FridayReleaseCheckpointSignoffLedger {
        ledger_id: format!("friday-release-checkpoint-signoff-ledger-{generated_at_unix_ms}"),
        ledger_json: ledger_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: records.len(),
        signed_off_count: decision_count(
            &records,
            FridayReleaseCheckpointSignoffDecision::SignedOff,
        ),
        held_count: decision_count(&records, FridayReleaseCheckpointSignoffDecision::Held),
        carried_over_count: decision_count(
            &records,
            FridayReleaseCheckpointSignoffDecision::CarriedOver,
        ),
        superseded_count: decision_count(
            &records,
            FridayReleaseCheckpointSignoffDecision::Superseded,
        ),
        revoked_count: decision_count(&records, FridayReleaseCheckpointSignoffDecision::Revoked),
        active_signoff_id: active.map(|record| record.signoff_id.clone()),
        active_review_id: active.map(|record| record.review_id.clone()),
        active_decision: active.map(|record| record.decision),
        active_hold_count,
        active_carryover_count,
        acknowledgement_evidence_missing_count,
        release_gate_blocking_count,
        release_notes_copy: release_notes_copy(&records),
        summary: format!(
            "Friday checkpoint signoff ledger has {} record(s), {} signed off, {} held, {} carried over, and {} missing acknowledgement evidence item(s).",
            records.len(),
            decision_count(&records, FridayReleaseCheckpointSignoffDecision::SignedOff),
            decision_count(&records, FridayReleaseCheckpointSignoffDecision::Held),
            decision_count(
                &records,
                FridayReleaseCheckpointSignoffDecision::CarriedOver
            ),
            acknowledgement_evidence_missing_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-checkpoint-signoff --ledger {} --review <release-checkpoint-review.json> --decision held --operator <name> --reason \"<reason>\"",
                ledger_json
            ),
            format!(
                "flow --friday-release-checkpoint-signoff-list --ledger {}",
                ledger_json
            ),
            format!(
                "flow --friday-release-checkpoint-signoff-export --ledger {} --output {}",
                ledger_json, ledger_json
            ),
            format!(
                "flow --friday-release-checkpoint-signoff-json --ledger {} --review <release-checkpoint-review.json>",
                ledger_json
            ),
        ],
        records,
    }
}

pub fn append_friday_release_checkpoint_signoff_to_ledger(
    ledger_path: impl AsRef<Path>,
    review_path: impl AsRef<Path>,
    request: FridayReleaseCheckpointSignoffRequest,
) -> Result<FridayReleaseCheckpointSignoffLedger> {
    let ledger_path = ledger_path.as_ref();
    let review_path = review_path.as_ref();
    let review = read_friday_release_checkpoint_review_board_report(review_path)?;
    let mut records = read_friday_release_checkpoint_signoff_ledger(ledger_path)
        .map(|ledger| ledger.records)
        .unwrap_or_default();
    records.push(signoff_record(review_path, &review, request));
    let ledger = friday_release_checkpoint_signoff_ledger_report(ledger_path, records);
    write_friday_release_checkpoint_signoff_ledger(ledger_path, &ledger)?;
    Ok(ledger)
}

pub fn friday_release_checkpoint_signoff_record_from_review(
    review_path: impl AsRef<Path>,
    request: FridayReleaseCheckpointSignoffRequest,
) -> Result<FridayReleaseCheckpointSignoffRecord> {
    let review_path = review_path.as_ref();
    let review = read_friday_release_checkpoint_review_board_report(review_path)?;
    Ok(signoff_record(review_path, &review, request))
}

pub fn write_friday_release_checkpoint_signoff_ledger(
    ledger_path: impl AsRef<Path>,
    ledger: &FridayReleaseCheckpointSignoffLedger,
) -> Result<()> {
    let ledger_path = ledger_path.as_ref();
    if let Some(parent) = ledger_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday checkpoint signoff ledger directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(ledger_path, ledger.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday checkpoint signoff ledger {}",
            ledger_path.display()
        )
    })
}

pub fn read_friday_release_checkpoint_signoff_ledger(
    ledger_path: impl AsRef<Path>,
) -> Result<FridayReleaseCheckpointSignoffLedger> {
    let ledger_path = ledger_path.as_ref();
    let bytes = fs::read(ledger_path).with_context(|| {
        format!(
            "Could not read Friday checkpoint signoff ledger {}",
            ledger_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday checkpoint signoff ledger {}",
            ledger_path.display()
        )
    })
}

fn signoff_record(
    review_path: &Path,
    review: &FridayReleaseCheckpointReviewBoardReport,
    request: FridayReleaseCheckpointSignoffRequest,
) -> FridayReleaseCheckpointSignoffRecord {
    let recorded_at_unix_ms = unix_ms();
    let evidence_path = request.acknowledgement_evidence_path.trim();
    let (acknowledgement_evidence_present, acknowledgement_evidence_bytes) =
        if evidence_path.is_empty() {
            (false, 0)
        } else {
            fs::metadata(evidence_path)
                .map(|metadata| (metadata.is_file(), metadata.len()))
                .unwrap_or((false, 0))
        };
    let active_hold = matches!(
        request.decision,
        FridayReleaseCheckpointSignoffDecision::Held
            | FridayReleaseCheckpointSignoffDecision::Revoked
    ) || (request.decision != FridayReleaseCheckpointSignoffDecision::SignedOff
        && review.decision == FridayReleaseCheckpointDecision::Hold);
    let active_carryover = request.decision == FridayReleaseCheckpointSignoffDecision::CarriedOver
        || (!request.carryover_commitment.trim().is_empty()
            && request.decision != FridayReleaseCheckpointSignoffDecision::SignedOff)
        || (request.decision != FridayReleaseCheckpointSignoffDecision::SignedOff
            && review.carryover_count > 0);
    let release_gate_blocking_count = if active_hold
        || (request.decision == FridayReleaseCheckpointSignoffDecision::SignedOff
            && review.acknowledgement_blocker_count > 0
            && !acknowledgement_evidence_present)
    {
        review.release_gate_blocking_count.max(1)
    } else {
        review.release_gate_blocking_count
    };
    let release_notes = format!(
        "Friday checkpoint signoff: {}\nOperator: {}\nReason: {}\nReview: {}\nCarryover: {}",
        request.decision.label(),
        request.operator,
        request.reason,
        review.review_id,
        request.carryover_commitment
    );
    let summary = format!(
        "{} {} checkpoint {} with {} hold(s), {} carryover(s), and {} acknowledgement blocker(s).",
        request.operator,
        request.decision.label(),
        review.review_id,
        review.hold_count,
        review.carryover_count,
        review.acknowledgement_blocker_count
    );

    FridayReleaseCheckpointSignoffRecord {
        signoff_id: format!(
            "friday-release-checkpoint-signoff-{}-{recorded_at_unix_ms}",
            review.review_id
        ),
        review_id: review.review_id.clone(),
        review_json: path_string(review_path),
        recorded_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        decision: request.decision,
        operator: request.operator,
        reason: request.reason,
        acknowledgement_evidence_path: request.acknowledgement_evidence_path,
        acknowledgement_evidence_present,
        acknowledgement_evidence_bytes,
        carryover_commitment: request.carryover_commitment,
        review_decision: review.decision,
        review_score_out_of_100: review.score_out_of_100,
        review_ready_for_checkpoint: review.ready_for_checkpoint,
        review_hold_count: review.hold_count,
        review_carryover_count: review.carryover_count,
        review_acknowledgement_blocker_count: review.acknowledgement_blocker_count,
        release_gate_blocking_count,
        active_hold,
        active_carryover,
        release_notes,
        summary,
    }
}

fn decision_count(
    records: &[FridayReleaseCheckpointSignoffRecord],
    decision: FridayReleaseCheckpointSignoffDecision,
) -> usize {
    records
        .iter()
        .filter(|record| record.decision == decision)
        .count()
}

fn release_notes_copy(records: &[FridayReleaseCheckpointSignoffRecord]) -> String {
    let mut lines = vec!["Friday checkpoint signoff ledger".to_string()];
    for record in records.iter().rev().take(8) {
        lines.push(format!(
            "- {} [{}] {} -> {}",
            record.operator,
            record.decision.label(),
            record.review_id,
            record.reason
        ));
    }
    if lines.len() == 1 {
        lines.push("No checkpoint signoffs are recorded.".to_string());
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
