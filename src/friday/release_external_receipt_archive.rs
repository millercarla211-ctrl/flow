use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayReleaseOutboundReviewLedger, FridayReleaseOutboundReviewRecord,
    FridayReleaseOutboundReviewState, FridayReleasePublicationState,
    read_friday_release_outbound_review_ledger,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseExternalReceiptState {
    Draft,
    Attached,
    Verified,
    Stale,
    Missing,
    Revoked,
    Superseded,
    Blocked,
}

impl FridayReleaseExternalReceiptState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Attached => "attached",
            Self::Verified => "verified",
            Self::Stale => "stale",
            Self::Missing => "missing",
            Self::Revoked => "revoked",
            Self::Superseded => "superseded",
            Self::Blocked => "blocked",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "attached" | "attach" => Ok(Self::Attached),
            "verified" | "verify" | "ready" => Ok(Self::Verified),
            "stale" | "expired" => Ok(Self::Stale),
            "missing" | "missing-evidence" => Ok(Self::Missing),
            "revoked" | "revoke" => Ok(Self::Revoked),
            "superseded" | "supersede" => Ok(Self::Superseded),
            "blocked" | "block" => Ok(Self::Blocked),
            other => anyhow::bail!(
                "Unknown Friday release external receipt state `{}`. Use draft, attached, verified, stale, missing, revoked, superseded, or blocked.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseExternalReceiptKind {
    Publication,
    Send,
    Deploy,
    Upload,
    Announcement,
    Other,
}

impl FridayReleaseExternalReceiptKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Publication => "publication",
            Self::Send => "send",
            Self::Deploy => "deploy",
            Self::Upload => "upload",
            Self::Announcement => "announcement",
            Self::Other => "other",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "publication" | "publish" | "post" => Self::Publication,
            "send" | "email" | "handoff" => Self::Send,
            "deploy" | "deployment" => Self::Deploy,
            "upload" | "attachment" => Self::Upload,
            "announcement" | "announce" => Self::Announcement,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseExternalReceiptRequest {
    pub state: FridayReleaseExternalReceiptState,
    pub receipt_kind: FridayReleaseExternalReceiptKind,
    pub operator: String,
    pub receipt_note: String,
    pub evidence_path: Option<String>,
    pub external_reference: Option<String>,
    pub supersedes_receipt_id: Option<String>,
}

impl Default for FridayReleaseExternalReceiptRequest {
    fn default() -> Self {
        Self {
            state: FridayReleaseExternalReceiptState::Draft,
            receipt_kind: FridayReleaseExternalReceiptKind::Publication,
            operator: "operator".to_string(),
            receipt_note: "Recorded operator-owned external receipt evidence.".to_string(),
            evidence_path: None,
            external_reference: None,
            supersedes_receipt_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseExternalReceiptRecord {
    pub receipt_id: String,
    pub outbound_review_id: String,
    pub outbound_review_ledger_id: String,
    pub outbound_review_ledger_json: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub state: FridayReleaseExternalReceiptState,
    pub receipt_kind: FridayReleaseExternalReceiptKind,
    pub operator: String,
    pub receipt_note: String,
    pub evidence_path: Option<String>,
    pub external_reference: Option<String>,
    pub supersedes_receipt_id: Option<String>,
    pub outbound_review_state: FridayReleaseOutboundReviewState,
    pub outbound_review_copy_safe: bool,
    pub outbound_review_active: bool,
    pub publication_control_id: String,
    pub publication_state: FridayReleasePublicationState,
    pub manual_publication_reference: Option<String>,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub source_review_notes_copy: String,
    pub reviewed_release_notes_copy: String,
    pub active: bool,
    pub receipt_attached: bool,
    pub receipt_verified: bool,
    pub externally_mutated_by_friday: bool,
    pub receipt_notes_copy: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseExternalReceiptArchive {
    pub archive_id: String,
    pub archive_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub record_count: usize,
    pub draft_count: usize,
    pub attached_count: usize,
    pub verified_count: usize,
    pub stale_count: usize,
    pub missing_count: usize,
    pub revoked_count: usize,
    pub superseded_count: usize,
    pub blocked_count: usize,
    pub active_receipt_id: Option<String>,
    pub latest_receipt_id: Option<String>,
    pub latest_state: Option<FridayReleaseExternalReceiptState>,
    pub latest_outbound_review_id: Option<String>,
    pub latest_outbound_review_state: Option<FridayReleaseOutboundReviewState>,
    pub attached_receipt_count: usize,
    pub verified_receipt_count: usize,
    pub blocked_receipt_count: usize,
    pub stale_or_missing_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub records: Vec<FridayReleaseExternalReceiptRecord>,
    pub audit_notes_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseExternalReceiptArchive {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_external_receipt_archive_report(
    archive_path: impl AsRef<Path>,
    mut records: Vec<FridayReleaseExternalReceiptRecord>,
) -> FridayReleaseExternalReceiptArchive {
    let archive_path = archive_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    records.sort_by(|left, right| {
        left.recorded_at_unix_ms
            .cmp(&right.recorded_at_unix_ms)
            .then_with(|| left.receipt_id.cmp(&right.receipt_id))
    });
    records.dedup_by(|left, right| left.receipt_id == right.receipt_id);
    let latest = records.last();
    let active = records
        .iter()
        .rev()
        .find(|record| {
            !matches!(
                record.state,
                FridayReleaseExternalReceiptState::Revoked
                    | FridayReleaseExternalReceiptState::Superseded
            )
        })
        .or(latest);
    let archive_json = path_string(archive_path);
    let attached_receipt_count = records
        .iter()
        .filter(|record| record.receipt_attached)
        .count();
    let verified_receipt_count = records
        .iter()
        .filter(|record| record.receipt_verified)
        .count();
    let blocked_receipt_count = records
        .iter()
        .filter(|record| {
            record.state == FridayReleaseExternalReceiptState::Blocked
                || record.release_gate_blocking_count > 0
                || record.unresolved_blocker_count > 0
                || !record.outbound_review_copy_safe
        })
        .count();
    let stale_or_missing_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.state,
                FridayReleaseExternalReceiptState::Stale
                    | FridayReleaseExternalReceiptState::Missing
            )
        })
        .count();
    let release_gate_blocking_count = active
        .map(|record| record.release_gate_blocking_count)
        .unwrap_or(0);
    let unresolved_blocker_count = active
        .map(|record| record.unresolved_blocker_count)
        .unwrap_or(0);
    let audit_notes_copy = audit_notes_copy(&records);
    let summary = format!(
        "Friday release external receipt archive has {} record(s), {} attached, {} verified, {} stale/missing, and {} blocked receipt(s).",
        records.len(),
        attached_receipt_count,
        verified_receipt_count,
        stale_or_missing_count,
        blocked_receipt_count
    );

    FridayReleaseExternalReceiptArchive {
        archive_id: format!("friday-release-external-receipt-archive-{generated_at_unix_ms}"),
        archive_json: archive_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: records.len(),
        draft_count: state_count(&records, FridayReleaseExternalReceiptState::Draft),
        attached_count: state_count(&records, FridayReleaseExternalReceiptState::Attached),
        verified_count: state_count(&records, FridayReleaseExternalReceiptState::Verified),
        stale_count: state_count(&records, FridayReleaseExternalReceiptState::Stale),
        missing_count: state_count(&records, FridayReleaseExternalReceiptState::Missing),
        revoked_count: state_count(&records, FridayReleaseExternalReceiptState::Revoked),
        superseded_count: state_count(&records, FridayReleaseExternalReceiptState::Superseded),
        blocked_count: state_count(&records, FridayReleaseExternalReceiptState::Blocked),
        active_receipt_id: active.map(|record| record.receipt_id.clone()),
        latest_receipt_id: latest.map(|record| record.receipt_id.clone()),
        latest_state: latest.map(|record| record.state),
        latest_outbound_review_id: latest.map(|record| record.outbound_review_id.clone()),
        latest_outbound_review_state: latest.map(|record| record.outbound_review_state),
        attached_receipt_count,
        verified_receipt_count,
        blocked_receipt_count,
        stale_or_missing_count,
        release_gate_blocking_count,
        unresolved_blocker_count,
        audit_notes_copy,
        summary,
        commands: vec![
            format!(
                "flow --friday-release-external-receipt --archive {} --outbound-review-ledger <release-outbound-review-ledger.json> --state draft --operator <name> --evidence <path>",
                archive_json
            ),
            format!(
                "flow --friday-release-external-receipt-list --archive {}",
                archive_json
            ),
            format!(
                "flow --friday-release-external-receipt-export --archive {} --output {}",
                archive_json, archive_json
            ),
            format!(
                "flow --friday-release-external-receipt-json --archive {} --outbound-review-ledger <release-outbound-review-ledger.json>",
                archive_json
            ),
        ],
        records,
    }
}

pub fn append_friday_release_external_receipt_to_archive(
    archive_path: impl AsRef<Path>,
    outbound_review_ledger_path: impl AsRef<Path>,
    request: FridayReleaseExternalReceiptRequest,
) -> Result<FridayReleaseExternalReceiptArchive> {
    let archive_path = archive_path.as_ref();
    let outbound_review_ledger_path = outbound_review_ledger_path.as_ref();
    let mut records = read_friday_release_external_receipt_archive(archive_path)
        .map(|archive| archive.records)
        .unwrap_or_default();
    records.push(friday_release_external_receipt_record_from_outbound_review(
        outbound_review_ledger_path,
        request,
    )?);
    let archive = friday_release_external_receipt_archive_report(archive_path, records);
    write_friday_release_external_receipt_archive(archive_path, &archive)?;
    Ok(archive)
}

pub fn friday_release_external_receipt_record_from_outbound_review(
    outbound_review_ledger_path: impl AsRef<Path>,
    request: FridayReleaseExternalReceiptRequest,
) -> Result<FridayReleaseExternalReceiptRecord> {
    let outbound_review_ledger_path = outbound_review_ledger_path.as_ref();
    let ledger = read_friday_release_outbound_review_ledger(outbound_review_ledger_path)?;
    let review = active_outbound_review(&ledger).with_context(|| {
        format!(
            "Friday external receipt archive needs at least one outbound review in {}",
            outbound_review_ledger_path.display()
        )
    })?;
    Ok(external_receipt_record(
        outbound_review_ledger_path,
        &ledger,
        review,
        request,
    ))
}

pub fn write_friday_release_external_receipt_archive(
    archive_path: impl AsRef<Path>,
    archive: &FridayReleaseExternalReceiptArchive,
) -> Result<()> {
    let archive_path = archive_path.as_ref();
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release external receipt archive directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(archive_path, archive.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release external receipt archive {}",
            archive_path.display()
        )
    })
}

pub fn read_friday_release_external_receipt_archive(
    archive_path: impl AsRef<Path>,
) -> Result<FridayReleaseExternalReceiptArchive> {
    let archive_path = archive_path.as_ref();
    let bytes = fs::read(archive_path).with_context(|| {
        format!(
            "Could not read Friday release external receipt archive {}",
            archive_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release external receipt archive {}",
            archive_path.display()
        )
    })
}

fn active_outbound_review(
    ledger: &FridayReleaseOutboundReviewLedger,
) -> Option<&FridayReleaseOutboundReviewRecord> {
    ledger
        .records
        .iter()
        .rev()
        .find(|record| {
            Some(record.review_id.as_str()) == ledger.active_review_id.as_deref() && record.active
        })
        .or_else(|| ledger.records.iter().rev().find(|record| record.active))
        .or_else(|| ledger.records.last())
}

fn external_receipt_record(
    outbound_review_ledger_path: &Path,
    ledger: &FridayReleaseOutboundReviewLedger,
    review: &FridayReleaseOutboundReviewRecord,
    request: FridayReleaseExternalReceiptRequest,
) -> FridayReleaseExternalReceiptRecord {
    let recorded_at_unix_ms = unix_ms();
    let evidence_path = request
        .evidence_path
        .clone()
        .filter(|value| !value.trim().is_empty());
    let external_reference = request
        .external_reference
        .clone()
        .or_else(|| review.manual_publication_reference.clone())
        .filter(|value| !value.trim().is_empty());
    let has_receipt_evidence = evidence_path.is_some() || external_reference.is_some();
    let unsafe_review = !review.copy_safe
        || review.release_gate_blocking_count > 0
        || review.unresolved_blocker_count > 0
        || review.state == FridayReleaseOutboundReviewState::Blocked;
    let state = if unsafe_review
        && matches!(
            request.state,
            FridayReleaseExternalReceiptState::Attached
                | FridayReleaseExternalReceiptState::Verified
        ) {
        FridayReleaseExternalReceiptState::Blocked
    } else if !has_receipt_evidence
        && matches!(
            request.state,
            FridayReleaseExternalReceiptState::Attached
                | FridayReleaseExternalReceiptState::Verified
        )
    {
        FridayReleaseExternalReceiptState::Missing
    } else {
        request.state
    };
    let active = !matches!(
        state,
        FridayReleaseExternalReceiptState::Revoked | FridayReleaseExternalReceiptState::Superseded
    );
    let receipt_attached = has_receipt_evidence
        && matches!(
            state,
            FridayReleaseExternalReceiptState::Attached
                | FridayReleaseExternalReceiptState::Verified
        )
        && !unsafe_review;
    let receipt_verified = state == FridayReleaseExternalReceiptState::Verified
        && receipt_attached
        && review.copy_safe;
    let receipt_notes_copy = format!(
        "Friday release external receipt archive\nState: {}\nKind: {}\nOperator: {}\nOutbound review: {}\nEvidence path: {}\nExternal reference: {}\nNote: {}\nFriday did not fetch, send, publish, deploy, upload, or email.\nReceipt evidence is operator-owned.\nFriday external mutation: false",
        state.label(),
        request.receipt_kind.label(),
        request.operator,
        review.review_id,
        evidence_path.as_deref().unwrap_or("not-recorded"),
        external_reference.as_deref().unwrap_or("not-recorded"),
        request.receipt_note
    );
    let summary = format!(
        "{} recorded {} receipt {} for outbound review {} with {} gate blocker(s).",
        request.operator,
        request.receipt_kind.label(),
        state.label(),
        review.review_id,
        review.release_gate_blocking_count
    );

    FridayReleaseExternalReceiptRecord {
        receipt_id: format!(
            "friday-release-external-receipt-{}-{recorded_at_unix_ms}",
            review.review_id
        ),
        outbound_review_id: review.review_id.clone(),
        outbound_review_ledger_id: ledger.ledger_id.clone(),
        outbound_review_ledger_json: path_string(outbound_review_ledger_path),
        recorded_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        state,
        receipt_kind: request.receipt_kind,
        operator: request.operator,
        receipt_note: request.receipt_note,
        evidence_path,
        external_reference,
        supersedes_receipt_id: request.supersedes_receipt_id,
        outbound_review_state: review.state,
        outbound_review_copy_safe: review.copy_safe,
        outbound_review_active: review.active,
        publication_control_id: review.publication_control_id.clone(),
        publication_state: review.publication_state,
        manual_publication_reference: review.manual_publication_reference.clone(),
        release_gate_blocking_count: review.release_gate_blocking_count,
        unresolved_blocker_count: review.unresolved_blocker_count,
        source_review_notes_copy: review.review_notes_copy.clone(),
        reviewed_release_notes_copy: review.release_notes_copy.clone(),
        active,
        receipt_attached,
        receipt_verified,
        externally_mutated_by_friday: false,
        receipt_notes_copy,
        summary,
    }
}

fn state_count(
    records: &[FridayReleaseExternalReceiptRecord],
    state: FridayReleaseExternalReceiptState,
) -> usize {
    records
        .iter()
        .filter(|record| record.state == state)
        .count()
}

fn audit_notes_copy(records: &[FridayReleaseExternalReceiptRecord]) -> String {
    let mut lines = vec!["Friday release external receipt archive".to_string()];
    for record in records.iter().rev().take(8) {
        lines.push(format!(
            "- {} [{}:{}] {} -> {}",
            record.operator,
            record.receipt_kind.label(),
            record.state.label(),
            record.outbound_review_id,
            record.receipt_note
        ));
        if let Some(path) = &record.evidence_path {
            lines.push(format!("  evidence: {path}"));
        }
        if let Some(reference) = &record.external_reference {
            lines.push(format!("  reference: {reference}"));
        }
        if record.release_gate_blocking_count > 0 {
            lines.push(format!(
                "  release gate blockers: {}",
                record.release_gate_blocking_count
            ));
        }
    }
    if lines.len() == 1 {
        lines.push("No external receipt records are archived.".to_string());
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
