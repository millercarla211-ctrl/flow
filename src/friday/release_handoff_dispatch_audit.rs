use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseHandoffDispatchChecklist,
    FridayReleaseHandoffDispatchChecklistState, read_friday_release_handoff_dispatch_checklist,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseHandoffDispatchAuditState {
    Draft,
    Ready,
    Held,
    Approved,
    SentManually,
    Revoked,
    Blocked,
}

impl FridayReleaseHandoffDispatchAuditState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Ready => "ready",
            Self::Held => "held",
            Self::Approved => "approved",
            Self::SentManually => "sent-manually",
            Self::Revoked => "revoked",
            Self::Blocked => "blocked",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "ready" | "ready-to-dispatch" | "ready_to_dispatch" => Ok(Self::Ready),
            "held" | "hold" => Ok(Self::Held),
            "approved" | "approve" => Ok(Self::Approved),
            "sent-manually" | "sent_manually" | "manual-sent" | "sent" => Ok(Self::SentManually),
            "revoked" | "revoke" => Ok(Self::Revoked),
            "blocked" | "block" => Ok(Self::Blocked),
            other => anyhow::bail!(
                "Unknown Friday handoff dispatch audit state `{}`. Use draft, ready, held, approved, sent-manually, revoked, or blocked.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffDispatchAuditRequest {
    pub state: FridayReleaseHandoffDispatchAuditState,
    pub operator: String,
    pub final_decision_note: String,
    pub supersedes_checklist_id: Option<String>,
}

impl Default for FridayReleaseHandoffDispatchAuditRequest {
    fn default() -> Self {
        Self {
            state: FridayReleaseHandoffDispatchAuditState::Draft,
            operator: "operator".to_string(),
            final_decision_note: "Recorded local dispatch checklist decision.".to_string(),
            supersedes_checklist_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffDispatchAuditRecord {
    pub audit_id: String,
    pub checklist_id: String,
    pub checklist_json: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub state: FridayReleaseHandoffDispatchAuditState,
    pub operator: String,
    pub final_decision_note: String,
    pub supersedes_checklist_id: Option<String>,
    pub checklist_ready_to_dispatch: bool,
    pub checklist_status: FridayDashboardPanelStatus,
    pub checklist_state: FridayReleaseHandoffDispatchChecklistState,
    pub item_count: usize,
    pub ready_count: usize,
    pub recipient_count: usize,
    pub attachment_count: usize,
    pub privacy_review_count: usize,
    pub missing_recipient_count: usize,
    pub missing_attachment_count: usize,
    pub blocked_count: usize,
    pub release_gate_blocking_count: usize,
    pub active: bool,
    pub blocker_carryover: usize,
    pub audit_notes: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffDispatchAuditTrail {
    pub trail_id: String,
    pub trail_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub record_count: usize,
    pub draft_count: usize,
    pub ready_count: usize,
    pub held_count: usize,
    pub approved_count: usize,
    pub sent_manually_count: usize,
    pub revoked_count: usize,
    pub blocked_count: usize,
    pub active_audit_id: Option<String>,
    pub active_checklist_id: Option<String>,
    pub latest_audit_id: Option<String>,
    pub latest_checklist_id: Option<String>,
    pub latest_state: Option<FridayReleaseHandoffDispatchAuditState>,
    pub latest_ready_to_dispatch: bool,
    pub unresolved_blocker_count: usize,
    pub blocker_carryover_count: usize,
    pub final_decision_count: usize,
    pub records: Vec<FridayReleaseHandoffDispatchAuditRecord>,
    pub audit_summary_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseHandoffDispatchAuditTrail {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_handoff_dispatch_audit_trail_report(
    trail_path: impl AsRef<Path>,
    mut records: Vec<FridayReleaseHandoffDispatchAuditRecord>,
) -> FridayReleaseHandoffDispatchAuditTrail {
    let trail_path = trail_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    records.sort_by(|left, right| {
        left.recorded_at_unix_ms
            .cmp(&right.recorded_at_unix_ms)
            .then_with(|| left.audit_id.cmp(&right.audit_id))
    });
    records.dedup_by(|left, right| left.audit_id == right.audit_id);
    let latest = records.last();
    let active = records
        .iter()
        .rev()
        .find(|record| record.state != FridayReleaseHandoffDispatchAuditState::Revoked)
        .or(latest);
    let blocker_carryover_count = records
        .iter()
        .filter(|record| record.blocker_carryover > 0)
        .count();
    let final_decision_count = records
        .iter()
        .filter(|record| !record.final_decision_note.trim().is_empty())
        .count();
    let unresolved_blocker_count = active.map(|record| record.blocker_carryover).unwrap_or(0);
    let trail_json = path_string(trail_path);

    FridayReleaseHandoffDispatchAuditTrail {
        trail_id: format!("friday-release-handoff-dispatch-audit-trail-{generated_at_unix_ms}"),
        trail_json: trail_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: records.len(),
        draft_count: state_count(&records, FridayReleaseHandoffDispatchAuditState::Draft),
        ready_count: state_count(&records, FridayReleaseHandoffDispatchAuditState::Ready),
        held_count: state_count(&records, FridayReleaseHandoffDispatchAuditState::Held),
        approved_count: state_count(&records, FridayReleaseHandoffDispatchAuditState::Approved),
        sent_manually_count: state_count(
            &records,
            FridayReleaseHandoffDispatchAuditState::SentManually,
        ),
        revoked_count: state_count(&records, FridayReleaseHandoffDispatchAuditState::Revoked),
        blocked_count: state_count(&records, FridayReleaseHandoffDispatchAuditState::Blocked),
        active_audit_id: active.map(|record| record.audit_id.clone()),
        active_checklist_id: active.map(|record| record.checklist_id.clone()),
        latest_audit_id: latest.map(|record| record.audit_id.clone()),
        latest_checklist_id: latest.map(|record| record.checklist_id.clone()),
        latest_state: latest.map(|record| record.state),
        latest_ready_to_dispatch: latest
            .map(|record| record.checklist_ready_to_dispatch)
            .unwrap_or(false),
        unresolved_blocker_count,
        blocker_carryover_count,
        final_decision_count,
        audit_summary_copy: dispatch_audit_summary_copy(&records),
        summary: format!(
            "Friday release handoff dispatch audit has {} record(s), {} approved, {} sent manually, {} held, {} blocked, and {} blocker carryover record(s).",
            records.len(),
            state_count(&records, FridayReleaseHandoffDispatchAuditState::Approved),
            state_count(
                &records,
                FridayReleaseHandoffDispatchAuditState::SentManually
            ),
            state_count(&records, FridayReleaseHandoffDispatchAuditState::Held),
            state_count(&records, FridayReleaseHandoffDispatchAuditState::Blocked),
            blocker_carryover_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-handoff-dispatch-audit --trail {} --checklist <release-handoff-dispatch-checklist.json> --state draft --operator <name>",
                trail_json
            ),
            format!(
                "flow --friday-release-handoff-dispatch-audit-list --trail {}",
                trail_json
            ),
            format!(
                "flow --friday-release-handoff-dispatch-audit-export --trail {} --output {}",
                trail_json, trail_json
            ),
            format!(
                "flow --friday-release-handoff-dispatch-audit-json --trail {} --checklist <release-handoff-dispatch-checklist.json>",
                trail_json
            ),
        ],
        records,
    }
}

pub fn append_friday_release_handoff_dispatch_audit_to_trail(
    trail_path: impl AsRef<Path>,
    checklist_path: impl AsRef<Path>,
    request: FridayReleaseHandoffDispatchAuditRequest,
) -> Result<FridayReleaseHandoffDispatchAuditTrail> {
    let trail_path = trail_path.as_ref();
    let checklist_path = checklist_path.as_ref();
    let mut records = read_friday_release_handoff_dispatch_audit_trail(trail_path)
        .map(|trail| trail.records)
        .unwrap_or_default();
    records.push(friday_release_handoff_dispatch_audit_record_from_checklist(
        checklist_path,
        request,
    )?);
    let trail = friday_release_handoff_dispatch_audit_trail_report(trail_path, records);
    write_friday_release_handoff_dispatch_audit_trail(trail_path, &trail)?;
    Ok(trail)
}

pub fn friday_release_handoff_dispatch_audit_record_from_checklist(
    checklist_path: impl AsRef<Path>,
    request: FridayReleaseHandoffDispatchAuditRequest,
) -> Result<FridayReleaseHandoffDispatchAuditRecord> {
    let checklist_path = checklist_path.as_ref();
    let checklist = read_friday_release_handoff_dispatch_checklist(checklist_path)?;
    Ok(dispatch_audit_record(checklist_path, &checklist, request))
}

pub fn write_friday_release_handoff_dispatch_audit_trail(
    trail_path: impl AsRef<Path>,
    trail: &FridayReleaseHandoffDispatchAuditTrail,
) -> Result<()> {
    let trail_path = trail_path.as_ref();
    if let Some(parent) = trail_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release handoff dispatch audit trail directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(trail_path, trail.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release handoff dispatch audit trail {}",
            trail_path.display()
        )
    })
}

pub fn read_friday_release_handoff_dispatch_audit_trail(
    trail_path: impl AsRef<Path>,
) -> Result<FridayReleaseHandoffDispatchAuditTrail> {
    let trail_path = trail_path.as_ref();
    let bytes = fs::read(trail_path).with_context(|| {
        format!(
            "Could not read Friday release handoff dispatch audit trail {}",
            trail_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release handoff dispatch audit trail {}",
            trail_path.display()
        )
    })
}

fn dispatch_audit_record(
    checklist_path: &Path,
    checklist: &FridayReleaseHandoffDispatchChecklist,
    request: FridayReleaseHandoffDispatchAuditRequest,
) -> FridayReleaseHandoffDispatchAuditRecord {
    let recorded_at_unix_ms = unix_ms();
    let blocker_carryover = checklist.release_gate_blocking_count;
    let active = request.state != FridayReleaseHandoffDispatchAuditState::Revoked;
    let audit_notes = format!(
        "Friday handoff dispatch audit: {}\nOperator: {}\nChecklist: {}\nFinal decision: {}\nBlocker carryover: {}\nNo automatic send: true",
        request.state.label(),
        request.operator,
        checklist.checklist_id,
        request.final_decision_note,
        blocker_carryover
    );
    let summary = format!(
        "{} recorded dispatch checklist {} as {} with {} release-gate blocker(s).",
        request.operator,
        checklist.checklist_id,
        request.state.label(),
        checklist.release_gate_blocking_count
    );

    FridayReleaseHandoffDispatchAuditRecord {
        audit_id: format!(
            "friday-release-handoff-dispatch-audit-{}-{recorded_at_unix_ms}",
            checklist.checklist_id
        ),
        checklist_id: checklist.checklist_id.clone(),
        checklist_json: path_string(checklist_path),
        recorded_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        state: request.state,
        operator: request.operator,
        final_decision_note: request.final_decision_note,
        supersedes_checklist_id: request.supersedes_checklist_id,
        checklist_ready_to_dispatch: checklist.ready_to_dispatch,
        checklist_status: checklist.status,
        checklist_state: checklist.state,
        item_count: checklist.item_count,
        ready_count: checklist.ready_count,
        recipient_count: checklist.recipient_count,
        attachment_count: checklist.attachment_count,
        privacy_review_count: checklist.privacy_review_count,
        missing_recipient_count: checklist.missing_recipient_count,
        missing_attachment_count: checklist.missing_attachment_count,
        blocked_count: checklist.blocked_count,
        release_gate_blocking_count: checklist.release_gate_blocking_count,
        active,
        blocker_carryover,
        audit_notes,
        summary,
    }
}

fn state_count(
    records: &[FridayReleaseHandoffDispatchAuditRecord],
    state: FridayReleaseHandoffDispatchAuditState,
) -> usize {
    records
        .iter()
        .filter(|record| record.state == state)
        .count()
}

fn dispatch_audit_summary_copy(records: &[FridayReleaseHandoffDispatchAuditRecord]) -> String {
    let mut lines = vec!["Friday release handoff dispatch audit".to_string()];
    for record in records.iter().rev().take(8) {
        lines.push(format!(
            "- {} [{}] {} -> {}",
            record.operator,
            record.state.label(),
            record.checklist_id,
            record.final_decision_note
        ));
        if record.blocker_carryover > 0 {
            lines.push(format!(
                "  carryover blockers: {}",
                record.blocker_carryover
            ));
        }
        if record.state == FridayReleaseHandoffDispatchAuditState::SentManually {
            lines.push("  manual send recorded; Friday did not send externally".to_string());
        }
    }
    if lines.len() == 1 {
        lines.push("No handoff dispatch audit records are recorded.".to_string());
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
