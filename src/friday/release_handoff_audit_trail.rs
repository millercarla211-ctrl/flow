use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseHandoffPacket, read_friday_release_handoff_packet,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseHandoffAuditState {
    Draft,
    Ready,
    Sent,
    Superseded,
    Revoked,
    Blocked,
}

impl FridayReleaseHandoffAuditState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Ready => "ready",
            Self::Sent => "sent",
            Self::Superseded => "superseded",
            Self::Revoked => "revoked",
            Self::Blocked => "blocked",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "ready" | "ready-to-send" | "ready_to_send" => Ok(Self::Ready),
            "sent" | "send" => Ok(Self::Sent),
            "superseded" | "supersede" => Ok(Self::Superseded),
            "revoked" | "revoke" => Ok(Self::Revoked),
            "blocked" | "block" | "held" | "hold" => Ok(Self::Blocked),
            other => anyhow::bail!(
                "Unknown Friday handoff audit state `{}`. Use draft, ready, sent, superseded, revoked, or blocked.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffAuditRequest {
    pub state: FridayReleaseHandoffAuditState,
    pub operator: String,
    pub acknowledgement_note: String,
    pub supersedes_packet_id: Option<String>,
}

impl Default for FridayReleaseHandoffAuditRequest {
    fn default() -> Self {
        Self {
            state: FridayReleaseHandoffAuditState::Draft,
            operator: "operator".to_string(),
            acknowledgement_note: "Recorded local handoff packet state.".to_string(),
            supersedes_packet_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffAuditRecord {
    pub audit_id: String,
    pub packet_id: String,
    pub packet_json: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub state: FridayReleaseHandoffAuditState,
    pub operator: String,
    pub acknowledgement_note: String,
    pub supersedes_packet_id: Option<String>,
    pub packet_ready_to_send: bool,
    pub packet_status: FridayDashboardPanelStatus,
    pub packet_section_count: usize,
    pub attachable_file_count: usize,
    pub inline_note_count: usize,
    pub unresolved_blocker_count: usize,
    pub missing_count: usize,
    pub manifest_sha256: String,
    pub active: bool,
    pub blocker_carryover: usize,
    pub audit_notes: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffAuditTrail {
    pub trail_id: String,
    pub trail_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub record_count: usize,
    pub draft_count: usize,
    pub ready_count: usize,
    pub sent_count: usize,
    pub superseded_count: usize,
    pub revoked_count: usize,
    pub blocked_count: usize,
    pub active_audit_id: Option<String>,
    pub active_packet_id: Option<String>,
    pub latest_audit_id: Option<String>,
    pub latest_packet_id: Option<String>,
    pub latest_state: Option<FridayReleaseHandoffAuditState>,
    pub latest_ready_to_send: bool,
    pub unresolved_blocker_count: usize,
    pub blocker_carryover_count: usize,
    pub acknowledgement_count: usize,
    pub records: Vec<FridayReleaseHandoffAuditRecord>,
    pub audit_summary_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseHandoffAuditTrail {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_handoff_audit_trail_report(
    trail_path: impl AsRef<Path>,
    mut records: Vec<FridayReleaseHandoffAuditRecord>,
) -> FridayReleaseHandoffAuditTrail {
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
        .find(|record| {
            !matches!(
                record.state,
                FridayReleaseHandoffAuditState::Superseded
                    | FridayReleaseHandoffAuditState::Revoked
            )
        })
        .or(latest);
    let acknowledgement_count = records
        .iter()
        .filter(|record| !record.acknowledgement_note.trim().is_empty())
        .count();
    let blocker_carryover_count = records
        .iter()
        .filter(|record| record.blocker_carryover > 0)
        .count();
    let unresolved_blocker_count = active.map(|record| record.blocker_carryover).unwrap_or(0);
    let trail_json = path_string(trail_path);

    FridayReleaseHandoffAuditTrail {
        trail_id: format!("friday-release-handoff-audit-trail-{generated_at_unix_ms}"),
        trail_json: trail_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        record_count: records.len(),
        draft_count: state_count(&records, FridayReleaseHandoffAuditState::Draft),
        ready_count: state_count(&records, FridayReleaseHandoffAuditState::Ready),
        sent_count: state_count(&records, FridayReleaseHandoffAuditState::Sent),
        superseded_count: state_count(&records, FridayReleaseHandoffAuditState::Superseded),
        revoked_count: state_count(&records, FridayReleaseHandoffAuditState::Revoked),
        blocked_count: state_count(&records, FridayReleaseHandoffAuditState::Blocked),
        active_audit_id: active.map(|record| record.audit_id.clone()),
        active_packet_id: active.map(|record| record.packet_id.clone()),
        latest_audit_id: latest.map(|record| record.audit_id.clone()),
        latest_packet_id: latest.map(|record| record.packet_id.clone()),
        latest_state: latest.map(|record| record.state),
        latest_ready_to_send: latest
            .map(|record| record.packet_ready_to_send)
            .unwrap_or(false),
        unresolved_blocker_count,
        blocker_carryover_count,
        acknowledgement_count,
        audit_summary_copy: audit_summary_copy(&records),
        summary: format!(
            "Friday release handoff audit trail has {} record(s), {} ready, {} sent, {} blocked, and {} blocker carryover record(s).",
            records.len(),
            state_count(&records, FridayReleaseHandoffAuditState::Ready),
            state_count(&records, FridayReleaseHandoffAuditState::Sent),
            state_count(&records, FridayReleaseHandoffAuditState::Blocked),
            blocker_carryover_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-handoff-audit --trail {} --packet <release-handoff-packet.json> --state draft --operator <name>",
                trail_json
            ),
            format!(
                "flow --friday-release-handoff-audit-list --trail {}",
                trail_json
            ),
            format!(
                "flow --friday-release-handoff-audit-export --trail {} --output {}",
                trail_json, trail_json
            ),
            format!(
                "flow --friday-release-handoff-audit-json --trail {} --packet <release-handoff-packet.json>",
                trail_json
            ),
        ],
        records,
    }
}

pub fn append_friday_release_handoff_audit_to_trail(
    trail_path: impl AsRef<Path>,
    packet_path: impl AsRef<Path>,
    request: FridayReleaseHandoffAuditRequest,
) -> Result<FridayReleaseHandoffAuditTrail> {
    let trail_path = trail_path.as_ref();
    let packet_path = packet_path.as_ref();
    let mut records = read_friday_release_handoff_audit_trail(trail_path)
        .map(|trail| trail.records)
        .unwrap_or_default();
    records.push(friday_release_handoff_audit_record_from_packet(
        packet_path,
        request,
    )?);
    let trail = friday_release_handoff_audit_trail_report(trail_path, records);
    write_friday_release_handoff_audit_trail(trail_path, &trail)?;
    Ok(trail)
}

pub fn friday_release_handoff_audit_record_from_packet(
    packet_path: impl AsRef<Path>,
    request: FridayReleaseHandoffAuditRequest,
) -> Result<FridayReleaseHandoffAuditRecord> {
    let packet_path = packet_path.as_ref();
    let packet = read_friday_release_handoff_packet(packet_path)?;
    Ok(handoff_audit_record(packet_path, &packet, request))
}

pub fn write_friday_release_handoff_audit_trail(
    trail_path: impl AsRef<Path>,
    trail: &FridayReleaseHandoffAuditTrail,
) -> Result<()> {
    let trail_path = trail_path.as_ref();
    if let Some(parent) = trail_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release handoff audit trail directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(trail_path, trail.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release handoff audit trail {}",
            trail_path.display()
        )
    })
}

pub fn read_friday_release_handoff_audit_trail(
    trail_path: impl AsRef<Path>,
) -> Result<FridayReleaseHandoffAuditTrail> {
    let trail_path = trail_path.as_ref();
    let bytes = fs::read(trail_path).with_context(|| {
        format!(
            "Could not read Friday release handoff audit trail {}",
            trail_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release handoff audit trail {}",
            trail_path.display()
        )
    })
}

fn handoff_audit_record(
    packet_path: &Path,
    packet: &FridayReleaseHandoffPacket,
    request: FridayReleaseHandoffAuditRequest,
) -> FridayReleaseHandoffAuditRecord {
    let recorded_at_unix_ms = unix_ms();
    let blocker_carryover = packet.unresolved_blocker_count + packet.missing_count;
    let active = !matches!(
        request.state,
        FridayReleaseHandoffAuditState::Superseded | FridayReleaseHandoffAuditState::Revoked
    );
    let audit_notes = format!(
        "Friday handoff audit: {}\nOperator: {}\nPacket: {}\nAcknowledgement: {}\nBlocker carryover: {}",
        request.state.label(),
        request.operator,
        packet.packet_id,
        request.acknowledgement_note,
        blocker_carryover
    );
    let summary = format!(
        "{} recorded packet {} as {} with {} unresolved blocker(s) and {} missing item(s).",
        request.operator,
        packet.packet_id,
        request.state.label(),
        packet.unresolved_blocker_count,
        packet.missing_count
    );

    FridayReleaseHandoffAuditRecord {
        audit_id: format!(
            "friday-release-handoff-audit-{}-{recorded_at_unix_ms}",
            packet.packet_id
        ),
        packet_id: packet.packet_id.clone(),
        packet_json: path_string(packet_path),
        recorded_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        state: request.state,
        operator: request.operator,
        acknowledgement_note: request.acknowledgement_note,
        supersedes_packet_id: request.supersedes_packet_id,
        packet_ready_to_send: packet.ready_to_send,
        packet_status: packet.status,
        packet_section_count: packet.section_count,
        attachable_file_count: packet.attachable_file_count,
        inline_note_count: packet.inline_note_count,
        unresolved_blocker_count: packet.unresolved_blocker_count,
        missing_count: packet.missing_count,
        manifest_sha256: packet.manifest_sha256.clone(),
        active,
        blocker_carryover,
        audit_notes,
        summary,
    }
}

fn state_count(
    records: &[FridayReleaseHandoffAuditRecord],
    state: FridayReleaseHandoffAuditState,
) -> usize {
    records
        .iter()
        .filter(|record| record.state == state)
        .count()
}

fn audit_summary_copy(records: &[FridayReleaseHandoffAuditRecord]) -> String {
    let mut lines = vec!["Friday release handoff audit trail".to_string()];
    for record in records.iter().rev().take(8) {
        lines.push(format!(
            "- {} [{}] {} -> {}",
            record.operator,
            record.state.label(),
            record.packet_id,
            record.acknowledgement_note
        ));
        if record.blocker_carryover > 0 {
            lines.push(format!(
                "  carryover blockers: {}",
                record.blocker_carryover
            ));
        }
    }
    if lines.len() == 1 {
        lines.push("No handoff audit records are recorded.".to_string());
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
