use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseHandoffDispatchAuditState,
    FridayReleaseHandoffDispatchAuditTrail, read_friday_release_handoff_dispatch_audit_trail,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseHandoffDispatchGovernanceState {
    Approved,
    Held,
    NeedsFinalDecision,
    StaleChecklist,
    RevokedActiveDecision,
    BlockedCarryover,
}

impl FridayReleaseHandoffDispatchGovernanceState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Held => "held",
            Self::NeedsFinalDecision => "needs-final-decision",
            Self::StaleChecklist => "stale-checklist",
            Self::RevokedActiveDecision => "revoked-active-decision",
            Self::BlockedCarryover => "blocked-carryover",
        }
    }

    fn score_multiplier(self) -> f32 {
        match self {
            Self::Approved => 1.0,
            Self::NeedsFinalDecision => 0.55,
            Self::Held => 0.35,
            Self::StaleChecklist => 0.2,
            Self::RevokedActiveDecision => 0.1,
            Self::BlockedCarryover => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseHandoffDispatchGovernanceSource {
    LatestDecision,
    ActiveDecision,
    FinalDecision,
    BlockerCarryover,
    AuditTrail,
}

impl FridayReleaseHandoffDispatchGovernanceSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::LatestDecision => "latest-decision",
            Self::ActiveDecision => "active-decision",
            Self::FinalDecision => "final-decision",
            Self::BlockerCarryover => "blocker-carryover",
            Self::AuditTrail => "audit-trail",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffDispatchGovernanceFinding {
    pub id: String,
    pub source: FridayReleaseHandoffDispatchGovernanceSource,
    pub state: FridayReleaseHandoffDispatchGovernanceState,
    pub release_gate_blocking: bool,
    pub audit_id: String,
    pub checklist_id: String,
    pub title: String,
    pub evidence_path: String,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffDispatchGovernanceReview {
    pub review_id: String,
    pub review_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub state: FridayReleaseHandoffDispatchGovernanceState,
    pub approved_for_external_handoff: bool,
    pub trail_id: String,
    pub trail_json: String,
    pub latest_audit_id: Option<String>,
    pub latest_checklist_id: Option<String>,
    pub active_audit_id: Option<String>,
    pub active_checklist_id: Option<String>,
    pub latest_state: Option<FridayReleaseHandoffDispatchAuditState>,
    pub record_count: usize,
    pub finding_count: usize,
    pub final_decision_gap_count: usize,
    pub stale_checklist_count: usize,
    pub revoked_active_decision_count: usize,
    pub blocked_carryover_count: usize,
    pub held_count: usize,
    pub release_gate_blocking_count: usize,
    pub unresolved_blocker_count: usize,
    pub findings: Vec<FridayReleaseHandoffDispatchGovernanceFinding>,
    pub governance_notes_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseHandoffDispatchGovernanceReview {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_handoff_dispatch_governance_review_report(
    review_path: impl AsRef<Path>,
    trail_path: impl AsRef<Path>,
) -> FridayReleaseHandoffDispatchGovernanceReview {
    let review_path = review_path.as_ref();
    let trail_path = trail_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let trail = read_friday_release_handoff_dispatch_audit_trail(trail_path).ok();
    let fallback = fallback_dispatch_audit_trail(trail_path);
    let trail = trail.as_ref().unwrap_or(&fallback);

    let mut findings = dispatch_governance_findings(trail);
    findings.sort_by(|left, right| {
        state_rank(left.state)
            .cmp(&state_rank(right.state))
            .then_with(|| left.id.cmp(&right.id))
    });

    let finding_count = findings.len();
    let final_decision_gap_count = state_count(
        &findings,
        FridayReleaseHandoffDispatchGovernanceState::NeedsFinalDecision,
    );
    let stale_checklist_count = state_count(
        &findings,
        FridayReleaseHandoffDispatchGovernanceState::StaleChecklist,
    );
    let revoked_active_decision_count = state_count(
        &findings,
        FridayReleaseHandoffDispatchGovernanceState::RevokedActiveDecision,
    );
    let blocked_carryover_count = state_count(
        &findings,
        FridayReleaseHandoffDispatchGovernanceState::BlockedCarryover,
    );
    let held_count = state_count(&findings, FridayReleaseHandoffDispatchGovernanceState::Held);
    let release_gate_blocking_count = findings
        .iter()
        .filter(|finding| finding.release_gate_blocking)
        .count();
    let state = if blocked_carryover_count > 0 {
        FridayReleaseHandoffDispatchGovernanceState::BlockedCarryover
    } else if revoked_active_decision_count > 0 {
        FridayReleaseHandoffDispatchGovernanceState::RevokedActiveDecision
    } else if stale_checklist_count > 0 {
        FridayReleaseHandoffDispatchGovernanceState::StaleChecklist
    } else if final_decision_gap_count > 0 {
        FridayReleaseHandoffDispatchGovernanceState::NeedsFinalDecision
    } else if held_count > 0 {
        FridayReleaseHandoffDispatchGovernanceState::Held
    } else {
        FridayReleaseHandoffDispatchGovernanceState::Approved
    };
    let approved_for_external_handoff = state
        == FridayReleaseHandoffDispatchGovernanceState::Approved
        && trail.record_count > 0
        && trail.unresolved_blocker_count == 0
        && matches!(
            trail.latest_state,
            Some(
                FridayReleaseHandoffDispatchAuditState::Approved
                    | FridayReleaseHandoffDispatchAuditState::SentManually
            )
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

    FridayReleaseHandoffDispatchGovernanceReview {
        review_id: format!(
            "friday-release-handoff-dispatch-governance-review-{generated_at_unix_ms}"
        ),
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
        latest_checklist_id: trail.latest_checklist_id.clone(),
        active_audit_id: trail.active_audit_id.clone(),
        active_checklist_id: trail.active_checklist_id.clone(),
        latest_state: trail.latest_state,
        record_count: trail.record_count,
        finding_count,
        final_decision_gap_count,
        stale_checklist_count,
        revoked_active_decision_count,
        blocked_carryover_count,
        held_count,
        release_gate_blocking_count,
        unresolved_blocker_count: trail.unresolved_blocker_count,
        governance_notes_copy: dispatch_governance_notes_copy(
            &findings,
            approved_for_external_handoff,
        ),
        summary: format!(
            "Friday release handoff dispatch governance is {} with score {}/100, {} finding(s), {} final decision gap(s), {} stale checklist warning(s), {} revoked decision issue(s), and {} blocker carryover issue(s).",
            state.label(),
            score_out_of_100,
            finding_count,
            final_decision_gap_count,
            stale_checklist_count,
            revoked_active_decision_count,
            blocked_carryover_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-handoff-dispatch-governance --output {} --trail {}",
                review_json, trail_json
            ),
            format!(
                "flow --friday-release-handoff-dispatch-governance-json --output {} --trail {}",
                review_json, trail_json
            ),
        ],
        findings,
    }
}

pub fn write_friday_release_handoff_dispatch_governance_review(
    review_path: impl AsRef<Path>,
    review: &FridayReleaseHandoffDispatchGovernanceReview,
) -> Result<()> {
    let review_path = review_path.as_ref();
    if let Some(parent) = review_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday handoff dispatch governance review directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(review_path, review.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday handoff dispatch governance review {}",
            review_path.display()
        )
    })
}

pub fn read_friday_release_handoff_dispatch_governance_review(
    review_path: impl AsRef<Path>,
) -> Result<FridayReleaseHandoffDispatchGovernanceReview> {
    let review_path = review_path.as_ref();
    let bytes = fs::read(review_path).with_context(|| {
        format!(
            "Could not read Friday handoff dispatch governance review {}",
            review_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday handoff dispatch governance review {}",
            review_path.display()
        )
    })
}

fn dispatch_governance_findings(
    trail: &FridayReleaseHandoffDispatchAuditTrail,
) -> Vec<FridayReleaseHandoffDispatchGovernanceFinding> {
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
            "missing-dispatch-audit-record",
            FridayReleaseHandoffDispatchGovernanceSource::AuditTrail,
            FridayReleaseHandoffDispatchGovernanceState::Held,
            true,
            "",
            "",
            "No dispatch audit record",
            &trail.trail_json,
            "No dispatch checklist has been recorded in the audit trail.",
            "Record a dispatch audit entry before considering external handoff complete.",
        ));
        return findings;
    }

    if trail.unresolved_blocker_count > 0 {
        findings.push(finding(
            "active-dispatch-blocker-carryover",
            FridayReleaseHandoffDispatchGovernanceSource::BlockerCarryover,
            FridayReleaseHandoffDispatchGovernanceState::BlockedCarryover,
            true,
            active.map(|record| record.audit_id.as_str()).unwrap_or(""),
            active
                .map(|record| record.checklist_id.as_str())
                .unwrap_or(""),
            "Active dispatch decision has blocker carryover",
            active
                .map(|record| record.checklist_json.as_str())
                .unwrap_or(&trail.trail_json),
            &format!(
                "Active dispatch decision still carries {} unresolved blocker(s).",
                trail.unresolved_blocker_count
            ),
            "Resolve blockers before considering the external handoff complete.",
        ));
    }

    if let Some(record) = latest {
        if record.final_decision_note.trim().is_empty() {
            findings.push(finding(
                "latest-final-decision-missing",
                FridayReleaseHandoffDispatchGovernanceSource::FinalDecision,
                FridayReleaseHandoffDispatchGovernanceState::NeedsFinalDecision,
                true,
                &record.audit_id,
                &record.checklist_id,
                "Latest dispatch final decision is missing",
                &record.checklist_json,
                "Latest dispatch audit record has no final decision note.",
                "Add an operator final decision note before completing the handoff.",
            ));
        }

        if record.state == FridayReleaseHandoffDispatchAuditState::Revoked {
            findings.push(finding(
                "latest-dispatch-decision-revoked",
                FridayReleaseHandoffDispatchGovernanceSource::LatestDecision,
                FridayReleaseHandoffDispatchGovernanceState::RevokedActiveDecision,
                true,
                &record.audit_id,
                &record.checklist_id,
                "Latest dispatch decision is revoked",
                &record.checklist_json,
                "Latest dispatch decision was revoked.",
                "Create a new approved or sent-manually dispatch decision before closing the handoff.",
            ));
        } else if !matches!(
            record.state,
            FridayReleaseHandoffDispatchAuditState::Approved
                | FridayReleaseHandoffDispatchAuditState::SentManually
        ) {
            findings.push(finding(
                "latest-dispatch-decision-held",
                FridayReleaseHandoffDispatchGovernanceSource::LatestDecision,
                FridayReleaseHandoffDispatchGovernanceState::Held,
                record.state == FridayReleaseHandoffDispatchAuditState::Blocked,
                &record.audit_id,
                &record.checklist_id,
                "Latest dispatch decision is not approved",
                &record.checklist_json,
                &format!("Latest dispatch decision is {}.", record.state.label()),
                "Move the latest decision to approved or sent-manually only after blockers are resolved.",
            ));
        }
    }

    if let Some(record) = active {
        if !record.active || record.state == FridayReleaseHandoffDispatchAuditState::Revoked {
            findings.push(finding(
                "active-dispatch-decision-revoked",
                FridayReleaseHandoffDispatchGovernanceSource::ActiveDecision,
                FridayReleaseHandoffDispatchGovernanceState::RevokedActiveDecision,
                true,
                &record.audit_id,
                &record.checklist_id,
                "Active dispatch decision is revoked or inactive",
                &record.checklist_json,
                "The selected active dispatch decision is revoked or inactive.",
                "Select or create a current approved dispatch decision before completing the handoff.",
            ));
        }
        if !record.checklist_ready_to_dispatch
            && matches!(
                record.state,
                FridayReleaseHandoffDispatchAuditState::Approved
                    | FridayReleaseHandoffDispatchAuditState::SentManually
            )
        {
            findings.push(finding(
                "active-checklist-not-ready",
                FridayReleaseHandoffDispatchGovernanceSource::ActiveDecision,
                FridayReleaseHandoffDispatchGovernanceState::StaleChecklist,
                true,
                &record.audit_id,
                &record.checklist_id,
                "Active checklist is not ready",
                &record.checklist_json,
                "The active dispatch decision is approved/sent, but the checklist payload still says it is not ready to dispatch.",
                "Regenerate the checklist after clearing recipient, attachment, privacy, and blocker issues.",
            ));
        }
    }

    findings
}

#[allow(clippy::too_many_arguments)]
fn finding(
    id: &str,
    source: FridayReleaseHandoffDispatchGovernanceSource,
    state: FridayReleaseHandoffDispatchGovernanceState,
    release_gate_blocking: bool,
    audit_id: &str,
    checklist_id: &str,
    title: &str,
    evidence_path: &str,
    summary: &str,
    next_action: &str,
) -> FridayReleaseHandoffDispatchGovernanceFinding {
    FridayReleaseHandoffDispatchGovernanceFinding {
        id: id.to_string(),
        source,
        state,
        release_gate_blocking,
        audit_id: audit_id.to_string(),
        checklist_id: checklist_id.to_string(),
        title: title.to_string(),
        evidence_path: evidence_path.to_string(),
        summary: summary.to_string(),
        next_action: next_action.to_string(),
    }
}

fn score_findings(
    findings: &[FridayReleaseHandoffDispatchGovernanceFinding],
    has_records: bool,
) -> u8 {
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
    findings: &[FridayReleaseHandoffDispatchGovernanceFinding],
    state: FridayReleaseHandoffDispatchGovernanceState,
) -> usize {
    findings
        .iter()
        .filter(|finding| finding.state == state)
        .count()
}

fn state_rank(state: FridayReleaseHandoffDispatchGovernanceState) -> u8 {
    match state {
        FridayReleaseHandoffDispatchGovernanceState::BlockedCarryover => 0,
        FridayReleaseHandoffDispatchGovernanceState::RevokedActiveDecision => 1,
        FridayReleaseHandoffDispatchGovernanceState::StaleChecklist => 2,
        FridayReleaseHandoffDispatchGovernanceState::NeedsFinalDecision => 3,
        FridayReleaseHandoffDispatchGovernanceState::Held => 4,
        FridayReleaseHandoffDispatchGovernanceState::Approved => 5,
    }
}

fn dispatch_governance_notes_copy(
    findings: &[FridayReleaseHandoffDispatchGovernanceFinding],
    approved_for_external_handoff: bool,
) -> String {
    let mut lines = vec![
        "Friday release handoff dispatch governance".to_string(),
        format!(
            "Status: {}",
            if approved_for_external_handoff {
                "approved for completed external handoff"
            } else {
                "hold dispatch completion"
            }
        ),
    ];
    if findings.is_empty() {
        lines.push("- No dispatch governance findings remain.".to_string());
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

fn fallback_dispatch_audit_trail(trail_path: &Path) -> FridayReleaseHandoffDispatchAuditTrail {
    FridayReleaseHandoffDispatchAuditTrail {
        trail_id: "missing-release-handoff-dispatch-audit-trail".to_string(),
        trail_json: path_string(trail_path),
        generated_at_unix_ms: unix_ms(),
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: 0,
        draft_count: 0,
        ready_count: 0,
        held_count: 0,
        approved_count: 0,
        sent_manually_count: 0,
        revoked_count: 0,
        blocked_count: 0,
        active_audit_id: None,
        active_checklist_id: None,
        latest_audit_id: None,
        latest_checklist_id: None,
        latest_state: None,
        latest_ready_to_dispatch: false,
        unresolved_blocker_count: 0,
        blocker_carryover_count: 0,
        final_decision_count: 0,
        records: Vec::new(),
        audit_summary_copy: String::new(),
        summary: "Release handoff dispatch audit trail could not be loaded.".to_string(),
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
