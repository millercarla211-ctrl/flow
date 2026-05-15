use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseHandoffAuditState, FridayReleaseHandoffAuditTrail,
    read_friday_release_handoff_audit_trail,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseHandoffGovernanceState {
    Approved,
    Held,
    NeedsAcknowledgement,
    StaleActivePacket,
    BlockedCarryover,
}

impl FridayReleaseHandoffGovernanceState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Held => "held",
            Self::NeedsAcknowledgement => "needs-acknowledgement",
            Self::StaleActivePacket => "stale-active-packet",
            Self::BlockedCarryover => "blocked-carryover",
        }
    }

    fn score_multiplier(self) -> f32 {
        match self {
            Self::Approved => 1.0,
            Self::NeedsAcknowledgement => 0.55,
            Self::Held => 0.35,
            Self::StaleActivePacket => 0.2,
            Self::BlockedCarryover => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseHandoffGovernanceSource {
    LatestPacket,
    ActivePacket,
    Acknowledgement,
    BlockerCarryover,
    AuditTrail,
}

impl FridayReleaseHandoffGovernanceSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::LatestPacket => "latest-packet",
            Self::ActivePacket => "active-packet",
            Self::Acknowledgement => "acknowledgement",
            Self::BlockerCarryover => "blocker-carryover",
            Self::AuditTrail => "audit-trail",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffGovernanceFinding {
    pub id: String,
    pub source: FridayReleaseHandoffGovernanceSource,
    pub state: FridayReleaseHandoffGovernanceState,
    pub release_gate_blocking: bool,
    pub audit_id: String,
    pub packet_id: String,
    pub title: String,
    pub evidence_path: String,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffGovernanceReview {
    pub review_id: String,
    pub review_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub state: FridayReleaseHandoffGovernanceState,
    pub approved_for_external_handoff: bool,
    pub trail_id: String,
    pub trail_json: String,
    pub latest_audit_id: Option<String>,
    pub latest_packet_id: Option<String>,
    pub active_audit_id: Option<String>,
    pub active_packet_id: Option<String>,
    pub latest_state: Option<FridayReleaseHandoffAuditState>,
    pub record_count: usize,
    pub finding_count: usize,
    pub acknowledgement_gap_count: usize,
    pub stale_active_packet_count: usize,
    pub blocked_carryover_count: usize,
    pub held_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub findings: Vec<FridayReleaseHandoffGovernanceFinding>,
    pub governance_notes_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseHandoffGovernanceReview {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_handoff_governance_review_report(
    review_path: impl AsRef<Path>,
    trail_path: impl AsRef<Path>,
) -> FridayReleaseHandoffGovernanceReview {
    let review_path = review_path.as_ref();
    let trail_path = trail_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let trail = read_friday_release_handoff_audit_trail(trail_path).ok();
    let fallback = fallback_audit_trail(trail_path);
    let trail = trail.as_ref().unwrap_or(&fallback);

    let mut findings = governance_findings(trail);
    findings.sort_by(|left, right| {
        state_rank(left.state)
            .cmp(&state_rank(right.state))
            .then_with(|| left.id.cmp(&right.id))
    });

    let finding_count = findings.len();
    let acknowledgement_gap_count = state_count(
        &findings,
        FridayReleaseHandoffGovernanceState::NeedsAcknowledgement,
    );
    let stale_active_packet_count = state_count(
        &findings,
        FridayReleaseHandoffGovernanceState::StaleActivePacket,
    );
    let blocked_carryover_count = state_count(
        &findings,
        FridayReleaseHandoffGovernanceState::BlockedCarryover,
    );
    let held_count = state_count(&findings, FridayReleaseHandoffGovernanceState::Held);
    let release_gate_blocking_count = findings
        .iter()
        .filter(|finding| finding.release_gate_blocking)
        .count();
    let state = if blocked_carryover_count > 0 {
        FridayReleaseHandoffGovernanceState::BlockedCarryover
    } else if stale_active_packet_count > 0 {
        FridayReleaseHandoffGovernanceState::StaleActivePacket
    } else if acknowledgement_gap_count > 0 {
        FridayReleaseHandoffGovernanceState::NeedsAcknowledgement
    } else if held_count > 0 {
        FridayReleaseHandoffGovernanceState::Held
    } else {
        FridayReleaseHandoffGovernanceState::Approved
    };
    let approved_for_external_handoff = state == FridayReleaseHandoffGovernanceState::Approved
        && trail.record_count > 0
        && trail.unresolved_blocker_count == 0
        && matches!(
            trail.latest_state,
            Some(FridayReleaseHandoffAuditState::Ready | FridayReleaseHandoffAuditState::Sent)
        );
    let score_out_of_100 = score_findings(&findings, trail.record_count > 0);
    let status = if release_gate_blocking_count > 0 || blocked_carryover_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if approved_for_external_handoff {
        FridayDashboardPanelStatus::Ready
    } else {
        FridayDashboardPanelStatus::Warning
    };
    let review_json = path_string(review_path);
    let trail_json = path_string(trail_path);

    FridayReleaseHandoffGovernanceReview {
        review_id: format!("friday-release-handoff-governance-review-{generated_at_unix_ms}"),
        review_json: review_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        score_out_of_100,
        state,
        approved_for_external_handoff,
        trail_id: trail.trail_id.clone(),
        trail_json: trail_json.clone(),
        latest_audit_id: trail.latest_audit_id.clone(),
        latest_packet_id: trail.latest_packet_id.clone(),
        active_audit_id: trail.active_audit_id.clone(),
        active_packet_id: trail.active_packet_id.clone(),
        latest_state: trail.latest_state,
        record_count: trail.record_count,
        finding_count,
        acknowledgement_gap_count,
        stale_active_packet_count,
        blocked_carryover_count,
        held_count,
        release_gate_blocking_count,
        unresolved_blocker_count: trail.unresolved_blocker_count,
        governance_notes_copy: governance_notes_copy(&findings, approved_for_external_handoff),
        summary: format!(
            "Friday release handoff governance review is {} with score {}/100, {} finding(s), {} acknowledgement gap(s), {} stale packet warning(s), and {} blocker carryover issue(s).",
            state.label(),
            score_out_of_100,
            finding_count,
            acknowledgement_gap_count,
            stale_active_packet_count,
            blocked_carryover_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-handoff-governance-review --output {} --trail {}",
                review_json, trail_json
            ),
            format!(
                "flow --friday-release-handoff-governance-review-json --output {} --trail {}",
                review_json, trail_json
            ),
        ],
        findings,
    }
}

pub fn write_friday_release_handoff_governance_review(
    review_path: impl AsRef<Path>,
    review: &FridayReleaseHandoffGovernanceReview,
) -> Result<()> {
    let review_path = review_path.as_ref();
    if let Some(parent) = review_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday handoff governance review directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(review_path, review.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday handoff governance review {}",
            review_path.display()
        )
    })
}

pub fn read_friday_release_handoff_governance_review(
    review_path: impl AsRef<Path>,
) -> Result<FridayReleaseHandoffGovernanceReview> {
    let review_path = review_path.as_ref();
    let bytes = fs::read(review_path).with_context(|| {
        format!(
            "Could not read Friday handoff governance review {}",
            review_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday handoff governance review {}",
            review_path.display()
        )
    })
}

fn governance_findings(
    trail: &FridayReleaseHandoffAuditTrail,
) -> Vec<FridayReleaseHandoffGovernanceFinding> {
    let mut findings = Vec::new();
    let latest = trail.records.last();
    let active = trail
        .active_audit_id
        .as_deref()
        .and_then(|audit_id| {
            trail
                .records
                .iter()
                .find(|record| record.audit_id == audit_id)
        })
        .or(latest);

    if trail.record_count == 0 {
        findings.push(finding(
            "missing-audit-trail-record",
            FridayReleaseHandoffGovernanceSource::AuditTrail,
            FridayReleaseHandoffGovernanceState::Held,
            true,
            "",
            "",
            "No handoff audit record",
            &trail.trail_json,
            "No handoff packet has been recorded in the audit trail.",
            "Record a handoff packet audit entry before preparing external handoff notes.",
        ));
        return findings;
    }

    if trail.unresolved_blocker_count > 0 {
        findings.push(finding(
            "active-blocker-carryover",
            FridayReleaseHandoffGovernanceSource::BlockerCarryover,
            FridayReleaseHandoffGovernanceState::BlockedCarryover,
            true,
            active.map(|record| record.audit_id.as_str()).unwrap_or(""),
            active.map(|record| record.packet_id.as_str()).unwrap_or(""),
            "Active packet has blocker carryover",
            active
                .map(|record| record.packet_json.as_str())
                .unwrap_or(&trail.trail_json),
            &format!(
                "Active packet still carries {} unresolved blocker(s).",
                trail.unresolved_blocker_count
            ),
            "Resolve or explicitly carry the blockers before external handoff.",
        ));
    }

    if let Some(record) = latest {
        if record.acknowledgement_note.trim().is_empty() {
            findings.push(finding(
                "latest-acknowledgement-missing",
                FridayReleaseHandoffGovernanceSource::Acknowledgement,
                FridayReleaseHandoffGovernanceState::NeedsAcknowledgement,
                true,
                &record.audit_id,
                &record.packet_id,
                "Latest audit acknowledgement is missing",
                &record.packet_json,
                "Latest handoff audit record has no acknowledgement note.",
                "Add an operator acknowledgement note before external handoff.",
            ));
        }

        if matches!(
            record.state,
            FridayReleaseHandoffAuditState::Superseded | FridayReleaseHandoffAuditState::Revoked
        ) {
            findings.push(finding(
                "latest-packet-not-active",
                FridayReleaseHandoffGovernanceSource::LatestPacket,
                FridayReleaseHandoffGovernanceState::StaleActivePacket,
                true,
                &record.audit_id,
                &record.packet_id,
                "Latest packet is not active",
                &record.packet_json,
                &format!("Latest handoff packet is {}.", record.state.label()),
                "Create or select a ready active handoff packet before external handoff.",
            ));
        } else if !matches!(
            record.state,
            FridayReleaseHandoffAuditState::Ready | FridayReleaseHandoffAuditState::Sent
        ) {
            findings.push(finding(
                "latest-packet-held",
                FridayReleaseHandoffGovernanceSource::LatestPacket,
                FridayReleaseHandoffGovernanceState::Held,
                record.state == FridayReleaseHandoffAuditState::Blocked,
                &record.audit_id,
                &record.packet_id,
                "Latest packet is not ready or sent",
                &record.packet_json,
                &format!("Latest handoff packet is {}.", record.state.label()),
                "Move the latest packet to ready or sent only after blockers and acknowledgements are resolved.",
            ));
        }
    }

    if let Some(record) = active {
        if !record.active {
            findings.push(finding(
                "active-packet-marked-inactive",
                FridayReleaseHandoffGovernanceSource::ActivePacket,
                FridayReleaseHandoffGovernanceState::StaleActivePacket,
                true,
                &record.audit_id,
                &record.packet_id,
                "Active packet is marked inactive",
                &record.packet_json,
                "The selected active packet is marked inactive.",
                "Select a current ready packet before external handoff.",
            ));
        }
        if !record.packet_ready_to_send
            && matches!(
                record.state,
                FridayReleaseHandoffAuditState::Ready | FridayReleaseHandoffAuditState::Sent
            )
        {
            findings.push(finding(
                "active-packet-not-ready-to-send",
                FridayReleaseHandoffGovernanceSource::ActivePacket,
                FridayReleaseHandoffGovernanceState::Held,
                true,
                &record.audit_id,
                &record.packet_id,
                "Active packet payload is not ready to send",
                &record.packet_json,
                "The active packet state is ready/sent, but the packet payload still says it is not ready to send.",
                "Regenerate the packet after clearing blocker and missing evidence counts.",
            ));
        }
    }

    findings
}

#[allow(clippy::too_many_arguments)]
fn finding(
    id: &str,
    source: FridayReleaseHandoffGovernanceSource,
    state: FridayReleaseHandoffGovernanceState,
    release_gate_blocking: bool,
    audit_id: &str,
    packet_id: &str,
    title: &str,
    evidence_path: &str,
    summary: &str,
    next_action: &str,
) -> FridayReleaseHandoffGovernanceFinding {
    FridayReleaseHandoffGovernanceFinding {
        id: id.to_string(),
        source,
        state,
        release_gate_blocking,
        audit_id: audit_id.to_string(),
        packet_id: packet_id.to_string(),
        title: title.to_string(),
        evidence_path: evidence_path.to_string(),
        summary: summary.to_string(),
        next_action: next_action.to_string(),
    }
}

fn score_findings(findings: &[FridayReleaseHandoffGovernanceFinding], has_records: bool) -> u8 {
    if !has_records {
        return 0;
    }
    if findings.is_empty() {
        return 100;
    }

    let earned = findings
        .iter()
        .map(|finding| finding.state.score_multiplier())
        .sum::<f32>();
    ((earned / findings.len() as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn state_count(
    findings: &[FridayReleaseHandoffGovernanceFinding],
    state: FridayReleaseHandoffGovernanceState,
) -> usize {
    findings
        .iter()
        .filter(|finding| finding.state == state)
        .count()
}

fn state_rank(state: FridayReleaseHandoffGovernanceState) -> u8 {
    match state {
        FridayReleaseHandoffGovernanceState::BlockedCarryover => 0,
        FridayReleaseHandoffGovernanceState::StaleActivePacket => 1,
        FridayReleaseHandoffGovernanceState::NeedsAcknowledgement => 2,
        FridayReleaseHandoffGovernanceState::Held => 3,
        FridayReleaseHandoffGovernanceState::Approved => 4,
    }
}

fn governance_notes_copy(
    findings: &[FridayReleaseHandoffGovernanceFinding],
    approved_for_external_handoff: bool,
) -> String {
    let mut lines = vec![
        "Friday release handoff governance review".to_string(),
        format!(
            "Status: {}",
            if approved_for_external_handoff {
                "approved for external handoff"
            } else {
                "hold external handoff"
            }
        ),
    ];
    if findings.is_empty() {
        lines.push("- No governance findings remain.".to_string());
    } else {
        for finding in findings.iter().take(8) {
            lines.push(format!(
                "- [{}] {} -> {}",
                finding.state.label(),
                finding.title,
                finding.next_action
            ));
        }
    }
    lines.join("\n")
}

fn fallback_audit_trail(trail_path: &Path) -> FridayReleaseHandoffAuditTrail {
    FridayReleaseHandoffAuditTrail {
        trail_id: "missing-release-handoff-audit-trail".to_string(),
        trail_json: path_string(trail_path),
        generated_at_unix_ms: unix_ms(),
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: 0,
        draft_count: 0,
        ready_count: 0,
        sent_count: 0,
        superseded_count: 0,
        revoked_count: 0,
        blocked_count: 0,
        active_audit_id: None,
        active_packet_id: None,
        latest_audit_id: None,
        latest_packet_id: None,
        latest_state: None,
        latest_ready_to_send: false,
        unresolved_blocker_count: 0,
        blocker_carryover_count: 0,
        acknowledgement_count: 0,
        records: Vec::new(),
        audit_summary_copy: String::new(),
        summary: "Release handoff audit trail could not be loaded.".to_string(),
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
