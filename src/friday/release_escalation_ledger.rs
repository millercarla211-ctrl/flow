use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayReleaseEvidenceEscalationLevel, FridayReleaseEvidenceRequirementSource,
    FridayReleaseEvidenceSlaMonitorReport, FridayReleaseEvidenceSlaRequirement,
    FridayReleaseEvidenceSlaState, read_friday_release_evidence_sla_monitor_report,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseEscalationOwnerResponse {
    Pending,
    Acknowledged,
    Resolved,
    Rejected,
    CarriedOver,
}

impl FridayReleaseEscalationOwnerResponse {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Acknowledged => "acknowledged",
            Self::Resolved => "resolved",
            Self::Rejected => "rejected",
            Self::CarriedOver => "carried-over",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "pending" | "open" => Ok(Self::Pending),
            "acknowledged" | "ack" => Ok(Self::Acknowledged),
            "resolved" | "resolve" | "closed" => Ok(Self::Resolved),
            "rejected" | "reject" => Ok(Self::Rejected),
            "carried-over" | "carried_over" | "carry-over" | "carryover" => Ok(Self::CarriedOver),
            other => anyhow::bail!(
                "Unknown Friday escalation owner response `{}`. Use pending, acknowledged, resolved, rejected, or carried-over.",
                other
            ),
        }
    }

    fn acknowledged(self) -> bool {
        matches!(self, Self::Acknowledged | Self::Resolved)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseEscalationGateOutcome {
    Blocked,
    CarryOver,
    Cleared,
    Monitoring,
}

impl FridayReleaseEscalationGateOutcome {
    pub fn label(self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::CarryOver => "carry-over",
            Self::Cleared => "cleared",
            Self::Monitoring => "monitoring",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "blocked" | "block" => Ok(Self::Blocked),
            "carry-over" | "carryover" | "carried-over" | "carried_over" => Ok(Self::CarryOver),
            "cleared" | "clear" | "resolved" => Ok(Self::Cleared),
            "monitoring" | "monitor" => Ok(Self::Monitoring),
            other => anyhow::bail!(
                "Unknown Friday escalation gate outcome `{}`. Use blocked, carry-over, cleared, or monitoring.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseEscalationLedgerEntry {
    pub escalation_id: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub monitor_id: Option<String>,
    pub monitor_json: String,
    pub requirement_id: String,
    pub source: FridayReleaseEvidenceRequirementSource,
    pub owner: String,
    pub title: String,
    pub sla_state: FridayReleaseEvidenceSlaState,
    pub escalation_level: FridayReleaseEvidenceEscalationLevel,
    pub owner_response: FridayReleaseEscalationOwnerResponse,
    pub gate_outcome: FridayReleaseEscalationGateOutcome,
    pub acknowledgement_required: bool,
    pub acknowledged: bool,
    pub active_carryover: bool,
    pub release_gate_blocking: bool,
    pub evidence_path: String,
    pub escalation_copy: String,
    pub owner_response_copy: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseEscalationOwnerGroup {
    pub owner: String,
    pub entry_count: usize,
    pub active_count: usize,
    pub acknowledged_count: usize,
    pub acknowledgement_blocker_count: usize,
    pub carryover_count: usize,
    pub release_gate_blocking_count: usize,
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseEscalationLedger {
    pub ledger_id: String,
    pub ledger_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub entry_count: usize,
    pub active_count: usize,
    pub acknowledged_count: usize,
    pub response_pending_count: usize,
    pub rejected_count: usize,
    pub resolved_count: usize,
    pub carryover_count: usize,
    pub release_gate_blocking_count: usize,
    pub acknowledgement_blocker_count: usize,
    pub owner_count: usize,
    pub latest_escalation_id: Option<String>,
    pub latest_gate_outcome: Option<FridayReleaseEscalationGateOutcome>,
    pub owner_groups: Vec<FridayReleaseEscalationOwnerGroup>,
    pub entries: Vec<FridayReleaseEscalationLedgerEntry>,
    pub owner_response_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseEscalationLedger {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_escalation_ledger_report(
    ledger_path: impl AsRef<Path>,
    mut entries: Vec<FridayReleaseEscalationLedgerEntry>,
) -> FridayReleaseEscalationLedger {
    let ledger_path = ledger_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    entries.sort_by(|left, right| {
        left.recorded_at_unix_ms
            .cmp(&right.recorded_at_unix_ms)
            .then_with(|| left.requirement_id.cmp(&right.requirement_id))
            .then_with(|| {
                left.owner_response
                    .label()
                    .cmp(right.owner_response.label())
            })
            .then_with(|| left.gate_outcome.label().cmp(right.gate_outcome.label()))
    });
    entries.dedup_by(|left, right| {
        left.requirement_id == right.requirement_id
            && left.recorded_at_unix_ms == right.recorded_at_unix_ms
            && left.owner_response == right.owner_response
            && left.gate_outcome == right.gate_outcome
    });
    let latest = entries.last();
    let owner_groups = owner_groups(&entries);
    let entry_count = entries.len();
    let active_count = entries
        .iter()
        .filter(|entry| entry.active_carryover)
        .count();
    let acknowledged_count = entries.iter().filter(|entry| entry.acknowledged).count();
    let response_pending_count =
        response_count(&entries, FridayReleaseEscalationOwnerResponse::Pending);
    let rejected_count = response_count(&entries, FridayReleaseEscalationOwnerResponse::Rejected);
    let resolved_count = response_count(&entries, FridayReleaseEscalationOwnerResponse::Resolved);
    let carryover_count = entries
        .iter()
        .filter(|entry| {
            entry.gate_outcome == FridayReleaseEscalationGateOutcome::CarryOver
                || entry.owner_response == FridayReleaseEscalationOwnerResponse::CarriedOver
        })
        .count();
    let release_gate_blocking_count = entries
        .iter()
        .filter(|entry| entry.release_gate_blocking)
        .count();
    let acknowledgement_blocker_count = entries
        .iter()
        .filter(|entry| entry.acknowledgement_required && !entry.acknowledged)
        .count();
    let ledger_json = path_string(ledger_path);

    FridayReleaseEscalationLedger {
        ledger_id: format!("friday-release-escalation-ledger-{generated_at_unix_ms}"),
        ledger_json: ledger_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        entry_count,
        active_count,
        acknowledged_count,
        response_pending_count,
        rejected_count,
        resolved_count,
        carryover_count,
        release_gate_blocking_count,
        acknowledgement_blocker_count,
        owner_count: owner_groups.len(),
        latest_escalation_id: latest.map(|entry| entry.escalation_id.clone()),
        latest_gate_outcome: latest.map(|entry| entry.gate_outcome),
        owner_response_copy: owner_response_copy(&entries),
        summary: format!(
            "Friday release escalation ledger has {entry_count} record(s), {active_count} active carryover(s), {acknowledgement_blocker_count} acknowledgement blocker(s), and {release_gate_blocking_count} release gate blocker(s)."
        ),
        commands: vec![
            format!(
                "flow --friday-release-escalation-ledger --ledger {} --monitor <release-evidence-sla-monitor.json> --response pending --outcome carry-over",
                ledger_json
            ),
            format!(
                "flow --friday-release-escalation-ledger-list --ledger {}",
                ledger_json
            ),
            format!(
                "flow --friday-release-escalation-ledger-export --ledger {} --output {}",
                ledger_json, ledger_json
            ),
            format!(
                "flow --friday-release-escalation-ledger-json --ledger {} --monitor <release-evidence-sla-monitor.json>",
                ledger_json
            ),
        ],
        owner_groups,
        entries,
    }
}

pub fn friday_release_escalation_entries_from_monitor(
    monitor_path: impl AsRef<Path>,
    owner_response: FridayReleaseEscalationOwnerResponse,
    gate_outcome: FridayReleaseEscalationGateOutcome,
) -> Result<Vec<FridayReleaseEscalationLedgerEntry>> {
    let monitor_path = monitor_path.as_ref();
    let monitor = read_friday_release_evidence_sla_monitor_report(monitor_path)?;
    Ok(escalation_entries_from_monitor(
        monitor_path,
        &monitor,
        owner_response,
        gate_outcome,
        unix_ms(),
    ))
}

pub fn friday_release_escalation_entries_from_monitor_at(
    monitor_path: impl AsRef<Path>,
    monitor: &FridayReleaseEvidenceSlaMonitorReport,
    owner_response: FridayReleaseEscalationOwnerResponse,
    gate_outcome: FridayReleaseEscalationGateOutcome,
    recorded_at_unix_ms: u128,
) -> Vec<FridayReleaseEscalationLedgerEntry> {
    escalation_entries_from_monitor(
        monitor_path.as_ref(),
        monitor,
        owner_response,
        gate_outcome,
        recorded_at_unix_ms,
    )
}

pub fn append_friday_release_escalation_to_ledger(
    ledger_path: impl AsRef<Path>,
    monitor_path: impl AsRef<Path>,
    owner_response: FridayReleaseEscalationOwnerResponse,
    gate_outcome: FridayReleaseEscalationGateOutcome,
) -> Result<FridayReleaseEscalationLedger> {
    let ledger_path = ledger_path.as_ref();
    let monitor_path = monitor_path.as_ref();
    let mut entries = read_friday_release_escalation_ledger(ledger_path)
        .map(|ledger| ledger.entries)
        .unwrap_or_default();
    entries.extend(friday_release_escalation_entries_from_monitor(
        monitor_path,
        owner_response,
        gate_outcome,
    )?);
    let ledger = friday_release_escalation_ledger_report(ledger_path, entries);
    write_friday_release_escalation_ledger(ledger_path, &ledger)?;
    Ok(ledger)
}

pub fn write_friday_release_escalation_ledger(
    ledger_path: impl AsRef<Path>,
    ledger: &FridayReleaseEscalationLedger,
) -> Result<()> {
    let ledger_path = ledger_path.as_ref();
    if let Some(parent) = ledger_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release escalation ledger directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(ledger_path, ledger.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release escalation ledger {}",
            ledger_path.display()
        )
    })
}

pub fn read_friday_release_escalation_ledger(
    ledger_path: impl AsRef<Path>,
) -> Result<FridayReleaseEscalationLedger> {
    let ledger_path = ledger_path.as_ref();
    let bytes = fs::read(ledger_path).with_context(|| {
        format!(
            "Could not read Friday release escalation ledger {}",
            ledger_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release escalation ledger {}",
            ledger_path.display()
        )
    })
}

fn escalation_entries_from_monitor(
    monitor_path: &Path,
    monitor: &FridayReleaseEvidenceSlaMonitorReport,
    owner_response: FridayReleaseEscalationOwnerResponse,
    gate_outcome: FridayReleaseEscalationGateOutcome,
    recorded_at_unix_ms: u128,
) -> Vec<FridayReleaseEscalationLedgerEntry> {
    let mut entries = monitor
        .requirements
        .iter()
        .filter(|requirement| {
            requirement.escalation_level != FridayReleaseEvidenceEscalationLevel::None
                || requirement.release_gate_blocking
        })
        .map(|requirement| {
            escalation_entry(
                monitor_path,
                monitor,
                requirement,
                owner_response,
                gate_outcome,
                recorded_at_unix_ms,
            )
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.owner
            .cmp(&right.owner)
            .then_with(|| left.escalation_level.cmp(&right.escalation_level).reverse())
            .then_with(|| left.requirement_id.cmp(&right.requirement_id))
    });
    entries
}

fn escalation_entry(
    monitor_path: &Path,
    monitor: &FridayReleaseEvidenceSlaMonitorReport,
    requirement: &FridayReleaseEvidenceSlaRequirement,
    owner_response: FridayReleaseEscalationOwnerResponse,
    gate_outcome: FridayReleaseEscalationGateOutcome,
    recorded_at_unix_ms: u128,
) -> FridayReleaseEscalationLedgerEntry {
    let acknowledged = owner_response.acknowledged();
    let acknowledgement_required =
        requirement.acknowledgement_required || requirement.release_gate_blocking;
    let active_carryover = !matches!(gate_outcome, FridayReleaseEscalationGateOutcome::Cleared)
        && !matches!(
            owner_response,
            FridayReleaseEscalationOwnerResponse::Resolved
        )
        && (requirement.escalation_level != FridayReleaseEvidenceEscalationLevel::None
            || requirement.release_gate_blocking);
    let release_gate_blocking = requirement.release_gate_blocking
        && !matches!(gate_outcome, FridayReleaseEscalationGateOutcome::Cleared);
    let owner_response_copy = format!(
        "@{} - {}\nResponse: {}\nGate outcome: {}\nAcknowledgement required: {}\nNext: {}",
        requirement.owner,
        requirement.title,
        owner_response.label(),
        gate_outcome.label(),
        yes_no(acknowledgement_required && !acknowledged),
        requirement.next_action
    );

    FridayReleaseEscalationLedgerEntry {
        escalation_id: format!(
            "friday-release-escalation-{}-{recorded_at_unix_ms}",
            requirement.id
        ),
        recorded_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        monitor_id: Some(monitor.monitor_id.clone()),
        monitor_json: path_string(monitor_path),
        requirement_id: requirement.id.clone(),
        source: requirement.source,
        owner: requirement.owner.clone(),
        title: requirement.title.clone(),
        sla_state: requirement.state,
        escalation_level: requirement.escalation_level,
        owner_response,
        gate_outcome,
        acknowledgement_required,
        acknowledged,
        active_carryover,
        release_gate_blocking,
        evidence_path: requirement.evidence_path.clone(),
        escalation_copy: requirement.escalation_copy.clone(),
        owner_response_copy,
        next_action: requirement.next_action.clone(),
    }
}

fn owner_groups(
    entries: &[FridayReleaseEscalationLedgerEntry],
) -> Vec<FridayReleaseEscalationOwnerGroup> {
    let mut groups: BTreeMap<String, Vec<&FridayReleaseEscalationLedgerEntry>> = BTreeMap::new();
    for entry in entries {
        groups.entry(entry.owner.clone()).or_default().push(entry);
    }

    groups
        .into_iter()
        .map(|(owner, entries)| FridayReleaseEscalationOwnerGroup {
            owner,
            entry_count: entries.len(),
            active_count: entries
                .iter()
                .filter(|entry| entry.active_carryover)
                .count(),
            acknowledged_count: entries.iter().filter(|entry| entry.acknowledged).count(),
            acknowledgement_blocker_count: entries
                .iter()
                .filter(|entry| entry.acknowledgement_required && !entry.acknowledged)
                .count(),
            carryover_count: entries
                .iter()
                .filter(|entry| {
                    entry.gate_outcome == FridayReleaseEscalationGateOutcome::CarryOver
                        || entry.owner_response == FridayReleaseEscalationOwnerResponse::CarriedOver
                })
                .count(),
            release_gate_blocking_count: entries
                .iter()
                .filter(|entry| entry.release_gate_blocking)
                .count(),
            entries: entries
                .iter()
                .map(|entry| entry.escalation_id.clone())
                .collect(),
        })
        .collect()
}

fn response_count(
    entries: &[FridayReleaseEscalationLedgerEntry],
    response: FridayReleaseEscalationOwnerResponse,
) -> usize {
    entries
        .iter()
        .filter(|entry| entry.owner_response == response)
        .count()
}

fn owner_response_copy(entries: &[FridayReleaseEscalationLedgerEntry]) -> String {
    let mut lines = vec!["Friday release escalation ledger".to_string()];
    for entry in entries.iter().filter(|entry| entry.active_carryover) {
        lines.push(format!(
            "- @{} [{} / {}] {} -> {}",
            entry.owner,
            entry.owner_response.label(),
            entry.gate_outcome.label(),
            entry.title,
            entry.next_action
        ));
    }
    if lines.len() == 1 {
        lines.push("No active escalation carryovers are recorded.".to_string());
    }
    lines.join("\n")
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
