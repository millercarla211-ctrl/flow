use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseExternalReceiptArchive,
    FridayReleaseExternalReceiptRecord, FridayReleaseExternalReceiptState,
    FridayReleaseOutboundReviewState, read_friday_release_external_receipt_archive,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseReceiptReviewDecision {
    Verified,
    Held,
    MissingReceipt,
    StaleEvidence,
    BlockedReview,
    RevokedReceipt,
    Carryover,
}

impl FridayReleaseReceiptReviewDecision {
    pub fn label(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Held => "held",
            Self::MissingReceipt => "missing-receipt",
            Self::StaleEvidence => "stale-evidence",
            Self::BlockedReview => "blocked-review",
            Self::RevokedReceipt => "revoked-receipt",
            Self::Carryover => "carryover",
        }
    }

    fn score_multiplier(self) -> f32 {
        match self {
            Self::Verified => 1.0,
            Self::Carryover => 0.55,
            Self::Held => 0.35,
            Self::StaleEvidence => 0.25,
            Self::MissingReceipt => 0.15,
            Self::RevokedReceipt | Self::BlockedReview => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseReceiptReviewSource {
    ActiveReceipt,
    ReceiptArchive,
    EvidenceFreshness,
    OutboundReview,
    MissingArchive,
}

impl FridayReleaseReceiptReviewSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::ActiveReceipt => "active-receipt",
            Self::ReceiptArchive => "receipt-archive",
            Self::EvidenceFreshness => "evidence-freshness",
            Self::OutboundReview => "outbound-review",
            Self::MissingArchive => "missing-archive",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseReceiptReviewFinding {
    pub id: String,
    pub source: FridayReleaseReceiptReviewSource,
    pub decision: FridayReleaseReceiptReviewDecision,
    pub release_gate_blocking: bool,
    pub receipt_id: String,
    pub outbound_review_id: String,
    pub evidence_path: String,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseReceiptReviewBoardReport {
    pub review_id: String,
    pub review_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub decision: FridayReleaseReceiptReviewDecision,
    pub ready_for_external_completion: bool,
    pub archive_id: String,
    pub archive_json: String,
    pub active_receipt_id: Option<String>,
    pub latest_receipt_id: Option<String>,
    pub latest_receipt_state: Option<FridayReleaseExternalReceiptState>,
    pub latest_outbound_review_id: Option<String>,
    pub latest_outbound_review_state: Option<FridayReleaseOutboundReviewState>,
    pub record_count: usize,
    pub finding_count: usize,
    pub verified_count: usize,
    pub held_count: usize,
    pub missing_receipt_count: usize,
    pub stale_evidence_count: usize,
    pub blocked_review_count: usize,
    pub revoked_receipt_count: usize,
    pub carryover_count: usize,
    pub attached_receipt_count: usize,
    pub verified_receipt_count: usize,
    pub stale_or_missing_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub findings: Vec<FridayReleaseReceiptReviewFinding>,
    pub review_notes_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseReceiptReviewBoardReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_receipt_review_board_report(
    review_path: impl AsRef<Path>,
    archive_path: impl AsRef<Path>,
) -> FridayReleaseReceiptReviewBoardReport {
    let review_path = review_path.as_ref();
    let archive_path = archive_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let archive = read_friday_release_external_receipt_archive(archive_path).ok();
    let fallback = fallback_receipt_archive(archive_path);
    let archive = archive.as_ref().unwrap_or(&fallback);
    let active = active_receipt(archive);

    let mut findings = receipt_review_findings(archive, active);
    findings.sort_by(|left, right| {
        decision_rank(left.decision)
            .cmp(&decision_rank(right.decision))
            .then_with(|| left.id.cmp(&right.id))
    });

    let finding_count = findings.len();
    let verified_count = decision_count(&findings, FridayReleaseReceiptReviewDecision::Verified);
    let held_count = decision_count(&findings, FridayReleaseReceiptReviewDecision::Held);
    let missing_receipt_count = decision_count(
        &findings,
        FridayReleaseReceiptReviewDecision::MissingReceipt,
    );
    let stale_evidence_count =
        decision_count(&findings, FridayReleaseReceiptReviewDecision::StaleEvidence);
    let blocked_review_count =
        decision_count(&findings, FridayReleaseReceiptReviewDecision::BlockedReview);
    let revoked_receipt_count = decision_count(
        &findings,
        FridayReleaseReceiptReviewDecision::RevokedReceipt,
    );
    let carryover_count = decision_count(&findings, FridayReleaseReceiptReviewDecision::Carryover);
    let release_gate_blocking_count = findings
        .iter()
        .filter(|finding| finding.release_gate_blocking)
        .count();
    let decision = if blocked_review_count > 0 {
        FridayReleaseReceiptReviewDecision::BlockedReview
    } else if revoked_receipt_count > 0 {
        FridayReleaseReceiptReviewDecision::RevokedReceipt
    } else if missing_receipt_count > 0 {
        FridayReleaseReceiptReviewDecision::MissingReceipt
    } else if stale_evidence_count > 0 {
        FridayReleaseReceiptReviewDecision::StaleEvidence
    } else if held_count > 0 {
        FridayReleaseReceiptReviewDecision::Held
    } else if carryover_count > 0 {
        FridayReleaseReceiptReviewDecision::Carryover
    } else {
        FridayReleaseReceiptReviewDecision::Verified
    };
    let ready_for_external_completion = decision == FridayReleaseReceiptReviewDecision::Verified
        && archive.record_count > 0
        && archive.verified_receipt_count > 0
        && release_gate_blocking_count == 0
        && archive.unresolved_blocker_count == 0;
    let status = if release_gate_blocking_count > 0
        || decision == FridayReleaseReceiptReviewDecision::BlockedReview
    {
        FridayDashboardPanelStatus::Blocked
    } else if ready_for_external_completion {
        FridayDashboardPanelStatus::Ready
    } else {
        FridayDashboardPanelStatus::Warning
    };
    let score_out_of_100 = score_findings(&findings, archive.record_count > 0);
    let review_json = path_string(review_path);
    let archive_json = path_string(archive_path);

    FridayReleaseReceiptReviewBoardReport {
        review_id: format!("friday-release-receipt-review-board-{generated_at_unix_ms}"),
        review_json: review_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        score_out_of_100,
        decision,
        ready_for_external_completion,
        archive_id: archive.archive_id.clone(),
        archive_json: archive_json.clone(),
        active_receipt_id: archive.active_receipt_id.clone(),
        latest_receipt_id: archive.latest_receipt_id.clone(),
        latest_receipt_state: archive.latest_state,
        latest_outbound_review_id: archive.latest_outbound_review_id.clone(),
        latest_outbound_review_state: archive.latest_outbound_review_state,
        record_count: archive.record_count,
        finding_count,
        verified_count,
        held_count,
        missing_receipt_count,
        stale_evidence_count,
        blocked_review_count,
        revoked_receipt_count,
        carryover_count,
        attached_receipt_count: archive.attached_receipt_count,
        verified_receipt_count: archive.verified_receipt_count,
        stale_or_missing_count: archive.stale_or_missing_count,
        release_gate_blocking_count,
        unresolved_blocker_count: archive.unresolved_blocker_count,
        review_notes_copy: review_notes_copy(
            decision,
            archive,
            active,
            &findings,
            ready_for_external_completion,
        ),
        summary: format!(
            "Friday release receipt review board is {} at {}/100 with {} receipt record(s), {} finding(s), {} verified receipt(s), and {} gate blocker(s).",
            decision.label(),
            score_out_of_100,
            archive.record_count,
            finding_count,
            archive.verified_receipt_count,
            release_gate_blocking_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-receipt-review-board --output {} --receipt-archive {}",
                review_json, archive_json
            ),
            format!(
                "flow --friday-release-receipt-review-board-json --output {} --receipt-archive {}",
                review_json, archive_json
            ),
        ],
        findings,
    }
}

pub fn write_friday_release_receipt_review_board_report(
    review_path: impl AsRef<Path>,
    report: &FridayReleaseReceiptReviewBoardReport,
) -> Result<()> {
    let review_path = review_path.as_ref();
    if let Some(parent) = review_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release receipt review board directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(review_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release receipt review board {}",
            review_path.display()
        )
    })
}

pub fn read_friday_release_receipt_review_board_report(
    review_path: impl AsRef<Path>,
) -> Result<FridayReleaseReceiptReviewBoardReport> {
    let review_path = review_path.as_ref();
    let bytes = fs::read(review_path).with_context(|| {
        format!(
            "Could not read Friday release receipt review board {}",
            review_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release receipt review board {}",
            review_path.display()
        )
    })
}

fn active_receipt(
    archive: &FridayReleaseExternalReceiptArchive,
) -> Option<&FridayReleaseExternalReceiptRecord> {
    archive
        .records
        .iter()
        .rev()
        .find(|record| {
            Some(record.receipt_id.as_str()) == archive.active_receipt_id.as_deref()
                && record.active
        })
        .or_else(|| archive.records.iter().rev().find(|record| record.active))
        .or_else(|| archive.records.last())
}

fn receipt_review_findings(
    archive: &FridayReleaseExternalReceiptArchive,
    active: Option<&FridayReleaseExternalReceiptRecord>,
) -> Vec<FridayReleaseReceiptReviewFinding> {
    let mut findings = Vec::new();
    if archive.record_count == 0 {
        findings.push(finding(
            "missing-receipt-archive-record",
            FridayReleaseReceiptReviewSource::MissingArchive,
            FridayReleaseReceiptReviewDecision::MissingReceipt,
            true,
            "none",
            "none",
            &archive.archive_json,
            "No external receipt records are archived.",
            "Attach operator-owned receipt evidence before treating the release as externally complete.",
        ));
    }

    let Some(active) = active else {
        return findings;
    };

    if !active.outbound_review_copy_safe
        || active.outbound_review_state == FridayReleaseOutboundReviewState::Blocked
        || active.release_gate_blocking_count > 0
        || active.unresolved_blocker_count > 0
    {
        findings.push(finding(
            "active-outbound-review-blocked",
            FridayReleaseReceiptReviewSource::OutboundReview,
            FridayReleaseReceiptReviewDecision::BlockedReview,
            true,
            &active.receipt_id,
            &active.outbound_review_id,
            active.evidence_path.as_deref().unwrap_or("not-recorded"),
            "The active receipt points to an outbound review that is not safe for external completion.",
            "Resolve outbound review blockers before verifying receipt evidence.",
        ));
    }

    match active.state {
        FridayReleaseExternalReceiptState::Verified if active.receipt_verified => {
            findings.push(finding(
                "active-receipt-verified",
                FridayReleaseReceiptReviewSource::ActiveReceipt,
                FridayReleaseReceiptReviewDecision::Verified,
                false,
                &active.receipt_id,
                &active.outbound_review_id,
                active
                    .evidence_path
                    .as_deref()
                    .unwrap_or("operator-reference"),
                "The active receipt is attached and verified by the operator.",
                "Preserve this receipt with the release evidence package.",
            ));
        }
        FridayReleaseExternalReceiptState::Attached => findings.push(finding(
            "active-receipt-attached-not-verified",
            FridayReleaseReceiptReviewSource::ActiveReceipt,
            FridayReleaseReceiptReviewDecision::Carryover,
            false,
            &active.receipt_id,
            &active.outbound_review_id,
            active
                .evidence_path
                .as_deref()
                .unwrap_or("operator-reference"),
            "The active receipt is attached but not yet verified.",
            "Have an operator verify the receipt before marking the external outcome complete.",
        )),
        FridayReleaseExternalReceiptState::Stale => findings.push(finding(
            "active-receipt-stale",
            FridayReleaseReceiptReviewSource::EvidenceFreshness,
            FridayReleaseReceiptReviewDecision::StaleEvidence,
            true,
            &active.receipt_id,
            &active.outbound_review_id,
            active.evidence_path.as_deref().unwrap_or("not-recorded"),
            "The active receipt evidence is stale.",
            "Refresh or replace the receipt evidence before final review.",
        )),
        FridayReleaseExternalReceiptState::Missing => findings.push(finding(
            "active-receipt-missing",
            FridayReleaseReceiptReviewSource::ActiveReceipt,
            FridayReleaseReceiptReviewDecision::MissingReceipt,
            true,
            &active.receipt_id,
            &active.outbound_review_id,
            active.evidence_path.as_deref().unwrap_or("not-recorded"),
            "The active receipt is missing evidence.",
            "Attach an operator-owned evidence path or external reference.",
        )),
        FridayReleaseExternalReceiptState::Revoked => findings.push(finding(
            "active-receipt-revoked",
            FridayReleaseReceiptReviewSource::ActiveReceipt,
            FridayReleaseReceiptReviewDecision::RevokedReceipt,
            true,
            &active.receipt_id,
            &active.outbound_review_id,
            active.evidence_path.as_deref().unwrap_or("not-recorded"),
            "The active receipt was revoked.",
            "Record a new receipt or carry the external-completion decision forward.",
        )),
        FridayReleaseExternalReceiptState::Blocked => findings.push(finding(
            "active-receipt-blocked",
            FridayReleaseReceiptReviewSource::ActiveReceipt,
            FridayReleaseReceiptReviewDecision::BlockedReview,
            true,
            &active.receipt_id,
            &active.outbound_review_id,
            active.evidence_path.as_deref().unwrap_or("not-recorded"),
            "The active receipt is blocked.",
            "Clear receipt and outbound review blockers before completion.",
        )),
        FridayReleaseExternalReceiptState::Draft => findings.push(finding(
            "active-receipt-draft",
            FridayReleaseReceiptReviewSource::ActiveReceipt,
            FridayReleaseReceiptReviewDecision::Held,
            true,
            &active.receipt_id,
            &active.outbound_review_id,
            active.evidence_path.as_deref().unwrap_or("not-recorded"),
            "The active receipt is still a draft.",
            "Attach and verify receipt evidence before treating the release as complete.",
        )),
        FridayReleaseExternalReceiptState::Superseded => findings.push(finding(
            "active-receipt-superseded",
            FridayReleaseReceiptReviewSource::ActiveReceipt,
            FridayReleaseReceiptReviewDecision::Carryover,
            false,
            &active.receipt_id,
            &active.outbound_review_id,
            active.evidence_path.as_deref().unwrap_or("not-recorded"),
            "The active receipt was superseded.",
            "Review the latest receipt before final external completion.",
        )),
        FridayReleaseExternalReceiptState::Verified => findings.push(finding(
            "active-receipt-verified-state-without-proof",
            FridayReleaseReceiptReviewSource::ActiveReceipt,
            FridayReleaseReceiptReviewDecision::MissingReceipt,
            true,
            &active.receipt_id,
            &active.outbound_review_id,
            active.evidence_path.as_deref().unwrap_or("not-recorded"),
            "The receipt state is verified but attached verification proof is missing.",
            "Attach the operator-owned evidence path or external reference.",
        )),
    }

    if archive.stale_or_missing_count > 0 {
        findings.push(finding(
            "archive-stale-or-missing-carryover",
            FridayReleaseReceiptReviewSource::ReceiptArchive,
            FridayReleaseReceiptReviewDecision::StaleEvidence,
            true,
            active.receipt_id.as_str(),
            active.outbound_review_id.as_str(),
            &archive.archive_json,
            "The archive still contains stale or missing receipt evidence.",
            "Resolve stale or missing receipt records before closing the release.",
        ));
    }

    if archive.blocked_receipt_count > 0 {
        findings.push(finding(
            "archive-blocked-receipt-carryover",
            FridayReleaseReceiptReviewSource::ReceiptArchive,
            FridayReleaseReceiptReviewDecision::BlockedReview,
            true,
            active.receipt_id.as_str(),
            active.outbound_review_id.as_str(),
            &archive.archive_json,
            "The archive still carries blocked receipt evidence.",
            "Clear blocked receipt records or document carryover before final signoff.",
        ));
    }

    findings
}

#[allow(clippy::too_many_arguments)]
fn finding(
    id: &str,
    source: FridayReleaseReceiptReviewSource,
    decision: FridayReleaseReceiptReviewDecision,
    release_gate_blocking: bool,
    receipt_id: &str,
    outbound_review_id: &str,
    evidence_path: &str,
    summary: &str,
    next_action: &str,
) -> FridayReleaseReceiptReviewFinding {
    FridayReleaseReceiptReviewFinding {
        id: id.to_string(),
        source,
        decision,
        release_gate_blocking,
        receipt_id: receipt_id.to_string(),
        outbound_review_id: outbound_review_id.to_string(),
        evidence_path: evidence_path.to_string(),
        summary: summary.to_string(),
        next_action: next_action.to_string(),
    }
}

fn decision_count(
    findings: &[FridayReleaseReceiptReviewFinding],
    decision: FridayReleaseReceiptReviewDecision,
) -> usize {
    findings
        .iter()
        .filter(|finding| finding.decision == decision)
        .count()
}

fn score_findings(findings: &[FridayReleaseReceiptReviewFinding], has_archive: bool) -> u8 {
    if !has_archive {
        return 0;
    }
    if findings.is_empty() {
        return 100;
    }
    let average = findings
        .iter()
        .map(|finding| finding.decision.score_multiplier())
        .sum::<f32>()
        / findings.len() as f32;
    (average * 100.0).round().clamp(0.0, 100.0) as u8
}

fn decision_rank(decision: FridayReleaseReceiptReviewDecision) -> u8 {
    match decision {
        FridayReleaseReceiptReviewDecision::BlockedReview => 0,
        FridayReleaseReceiptReviewDecision::RevokedReceipt => 1,
        FridayReleaseReceiptReviewDecision::MissingReceipt => 2,
        FridayReleaseReceiptReviewDecision::StaleEvidence => 3,
        FridayReleaseReceiptReviewDecision::Held => 4,
        FridayReleaseReceiptReviewDecision::Carryover => 5,
        FridayReleaseReceiptReviewDecision::Verified => 6,
    }
}

fn review_notes_copy(
    decision: FridayReleaseReceiptReviewDecision,
    archive: &FridayReleaseExternalReceiptArchive,
    active: Option<&FridayReleaseExternalReceiptRecord>,
    findings: &[FridayReleaseReceiptReviewFinding],
    ready_for_external_completion: bool,
) -> String {
    let mut lines = vec![
        "Friday release receipt review board".to_string(),
        format!("Decision: {}", decision.label()),
        format!(
            "Ready for external completion: {}",
            yes_no(ready_for_external_completion)
        ),
        format!("Archive: {}", archive.archive_json),
        format!("Records reviewed: {}", archive.record_count),
        "Friday did not fetch, send, publish, deploy, upload, or email.".to_string(),
    ];
    if let Some(active) = active {
        lines.push(format!("Active receipt: {}", active.receipt_id));
        lines.push(format!("Outbound review: {}", active.outbound_review_id));
        if let Some(path) = &active.evidence_path {
            lines.push(format!("Evidence path: {path}"));
        }
        if let Some(reference) = &active.external_reference {
            lines.push(format!("External reference: {reference}"));
        }
    }
    lines.push("Findings:".to_string());
    for finding in findings.iter().take(10) {
        lines.push(format!(
            "- [{}] {} -> {}",
            finding.decision.label(),
            finding.summary,
            finding.next_action
        ));
    }
    lines.join("\n")
}

fn fallback_receipt_archive(archive_path: &Path) -> FridayReleaseExternalReceiptArchive {
    let archive_json = path_string(archive_path);
    FridayReleaseExternalReceiptArchive {
        archive_id: "friday-release-external-receipt-archive-missing".to_string(),
        archive_json,
        generated_at_unix_ms: unix_ms(),
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: 0,
        draft_count: 0,
        attached_count: 0,
        verified_count: 0,
        stale_count: 0,
        missing_count: 0,
        revoked_count: 0,
        superseded_count: 0,
        blocked_count: 0,
        active_receipt_id: None,
        latest_receipt_id: None,
        latest_state: None,
        latest_outbound_review_id: None,
        latest_outbound_review_state: None,
        attached_receipt_count: 0,
        verified_receipt_count: 0,
        blocked_receipt_count: 0,
        stale_or_missing_count: 0,
        release_gate_blocking_count: 0,
        unresolved_blocker_count: 0,
        records: Vec::new(),
        audit_notes_copy: "No external receipt archive found.".to_string(),
        summary: "Friday release external receipt archive is missing.".to_string(),
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

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
