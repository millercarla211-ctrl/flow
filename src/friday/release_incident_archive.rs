use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayReleasePostPromotionIncidentNote, FridayReleasePostPromotionMonitorReport,
    FridayReleaseRecoveryRunbookReport, FridayReleaseRollbackDrillReport,
    FridayReleaseStabilityBoardReport, read_friday_release_post_promotion_monitor_report,
    read_friday_release_recovery_runbook_report, read_friday_release_rollback_drill_report,
    read_friday_release_stability_board_report,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseIncidentSeverity {
    Info,
    Watch,
    Blocking,
    Critical,
}

impl FridayReleaseIncidentSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Watch => "watch",
            Self::Blocking => "blocking",
            Self::Critical => "critical",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "info" | "note" => Ok(Self::Info),
            "watch" | "warning" => Ok(Self::Watch),
            "blocking" | "blocked" | "blocker" => Ok(Self::Blocking),
            "critical" | "severe" => Ok(Self::Critical),
            other => anyhow::bail!(
                "Unknown Friday incident severity `{}`. Use info, watch, blocking, or critical.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseIncidentOutcome {
    Open,
    Monitoring,
    Resolved,
    RolledBack,
    Prevented,
}

impl FridayReleaseIncidentOutcome {
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Monitoring => "monitoring",
            Self::Resolved => "resolved",
            Self::RolledBack => "rolled-back",
            Self::Prevented => "prevented",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "open" => Ok(Self::Open),
            "monitoring" | "monitor" => Ok(Self::Monitoring),
            "resolved" | "resolve" | "closed" => Ok(Self::Resolved),
            "rolled-back" | "rolled_back" | "rollback" => Ok(Self::RolledBack),
            "prevented" | "prevent" => Ok(Self::Prevented),
            other => anyhow::bail!(
                "Unknown Friday incident outcome `{}`. Use open, monitoring, resolved, rolled-back, or prevented.",
                other
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseIncidentArchiveEntry {
    pub incident_id: String,
    pub recorded_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub severity: FridayReleaseIncidentSeverity,
    pub outcome: FridayReleaseIncidentOutcome,
    pub title: String,
    pub summary: String,
    pub recovery_runbook_id: Option<String>,
    pub recovery_runbook_json: String,
    pub stability_board_json: String,
    pub rollback_drill_json: String,
    pub post_promotion_monitor_json: String,
    pub active_candidate_id: Option<String>,
    pub active_promotion_id: Option<String>,
    pub active_rollback_reference: Option<String>,
    pub blocked_phase_count: usize,
    pub active_risk_count: usize,
    pub incident_notes: Vec<FridayReleasePostPromotionIncidentNote>,
    pub follow_up_actions: Vec<String>,
    pub prevention_items: Vec<String>,
    pub evidence_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseIncidentArchive {
    pub archive_id: String,
    pub archive_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub incident_count: usize,
    pub open_count: usize,
    pub monitoring_count: usize,
    pub resolved_count: usize,
    pub rolled_back_count: usize,
    pub prevented_count: usize,
    pub critical_count: usize,
    pub blocking_count: usize,
    pub follow_up_count: usize,
    pub latest_incident_id: Option<String>,
    pub latest_severity: Option<FridayReleaseIncidentSeverity>,
    pub latest_outcome: Option<FridayReleaseIncidentOutcome>,
    pub latest_rollback_reference: Option<String>,
    pub entries: Vec<FridayReleaseIncidentArchiveEntry>,
    pub commands: Vec<String>,
}

impl FridayReleaseIncidentArchive {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_incident_archive_report(
    archive_path: impl AsRef<Path>,
    mut entries: Vec<FridayReleaseIncidentArchiveEntry>,
) -> FridayReleaseIncidentArchive {
    let archive_path = archive_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    entries.sort_by_key(|entry| entry.recorded_at_unix_ms);
    entries.dedup_by(|left, right| {
        left.recovery_runbook_json == right.recovery_runbook_json
            && left.recorded_at_unix_ms == right.recorded_at_unix_ms
    });
    let latest = entries.last();
    let follow_up_count = entries
        .iter()
        .map(|entry| entry.follow_up_actions.len())
        .sum::<usize>();
    let archive_json = path_string(archive_path);

    FridayReleaseIncidentArchive {
        archive_id: format!("friday-release-incident-archive-{generated_at_unix_ms}"),
        archive_json: archive_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        incident_count: entries.len(),
        open_count: outcome_count(&entries, FridayReleaseIncidentOutcome::Open),
        monitoring_count: outcome_count(&entries, FridayReleaseIncidentOutcome::Monitoring),
        resolved_count: outcome_count(&entries, FridayReleaseIncidentOutcome::Resolved),
        rolled_back_count: outcome_count(&entries, FridayReleaseIncidentOutcome::RolledBack),
        prevented_count: outcome_count(&entries, FridayReleaseIncidentOutcome::Prevented),
        critical_count: severity_count(&entries, FridayReleaseIncidentSeverity::Critical),
        blocking_count: severity_count(&entries, FridayReleaseIncidentSeverity::Blocking)
            + severity_count(&entries, FridayReleaseIncidentSeverity::Critical),
        follow_up_count,
        latest_incident_id: latest.map(|entry| entry.incident_id.clone()),
        latest_severity: latest.map(|entry| entry.severity),
        latest_outcome: latest.map(|entry| entry.outcome),
        latest_rollback_reference: latest.and_then(|entry| entry.active_rollback_reference.clone()),
        commands: vec![
            format!(
                "flow --friday-release-incident-archive --archive {} --runbook <release-recovery-runbook.json> --incident-note <incident-note.md>",
                archive_json
            ),
            format!(
                "flow --friday-release-incident-archive-list --archive {}",
                archive_json
            ),
            format!(
                "flow --friday-release-incident-archive-export --archive {} --output {}",
                archive_json, archive_json
            ),
            format!(
                "flow --friday-release-incident-archive-json --archive {}",
                archive_json
            ),
        ],
        entries,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn friday_release_incident_entry_from_sources(
    recovery_runbook_path: impl AsRef<Path>,
    stability_board_path: impl AsRef<Path>,
    rollback_drill_path: impl AsRef<Path>,
    post_promotion_monitor_path: impl AsRef<Path>,
    incident_note_paths: Vec<String>,
    outcome: FridayReleaseIncidentOutcome,
) -> FridayReleaseIncidentArchiveEntry {
    let recovery_runbook_path = recovery_runbook_path.as_ref();
    let stability_board_path = stability_board_path.as_ref();
    let rollback_drill_path = rollback_drill_path.as_ref();
    let post_promotion_monitor_path = post_promotion_monitor_path.as_ref();
    let recorded_at_unix_ms = unix_ms();
    let runbook = read_friday_release_recovery_runbook_report(recovery_runbook_path).ok();
    let stability_board = read_friday_release_stability_board_report(stability_board_path).ok();
    let rollback_drill = read_friday_release_rollback_drill_report(rollback_drill_path).ok();
    let post_promotion_monitor =
        read_friday_release_post_promotion_monitor_report(post_promotion_monitor_path).ok();
    let incident_notes = incident_note_paths
        .iter()
        .map(|path| incident_note(path))
        .collect::<Vec<_>>();
    let severity = incident_severity(
        runbook.as_ref(),
        stability_board.as_ref(),
        rollback_drill.as_ref(),
        post_promotion_monitor.as_ref(),
    );
    let active_candidate_id = runbook
        .as_ref()
        .and_then(|runbook| runbook.active_candidate_id.clone())
        .or_else(|| {
            stability_board
                .as_ref()
                .and_then(|board| board.active_candidate_id.clone())
        })
        .or_else(|| {
            rollback_drill
                .as_ref()
                .and_then(|drill| drill.active_candidate_id.clone())
        })
        .or_else(|| {
            post_promotion_monitor
                .as_ref()
                .and_then(|monitor| monitor.active_candidate_id.clone())
        });
    let active_promotion_id = runbook
        .as_ref()
        .and_then(|runbook| runbook.active_promotion_id.clone())
        .or_else(|| {
            stability_board
                .as_ref()
                .and_then(|board| board.active_promotion_id.clone())
        })
        .or_else(|| {
            rollback_drill
                .as_ref()
                .and_then(|drill| drill.active_promotion_id.clone())
        })
        .or_else(|| {
            post_promotion_monitor
                .as_ref()
                .and_then(|monitor| monitor.active_promotion_id.clone())
        });
    let active_rollback_reference = runbook
        .as_ref()
        .and_then(|runbook| runbook.active_rollback_reference.clone())
        .or_else(|| {
            stability_board
                .as_ref()
                .and_then(|board| board.active_rollback_reference.clone())
        })
        .or_else(|| {
            rollback_drill
                .as_ref()
                .and_then(|drill| drill.active_rollback_reference.clone())
        })
        .or_else(|| {
            post_promotion_monitor
                .as_ref()
                .and_then(|monitor| monitor.active_rollback_reference.clone())
        });
    let active_risk_count = runbook
        .as_ref()
        .map(|runbook| runbook.active_risks.len())
        .or_else(|| {
            stability_board
                .as_ref()
                .map(|board| board.active_risks.len())
        })
        .unwrap_or_default();
    let blocked_phase_count = runbook
        .as_ref()
        .map(|runbook| runbook.blocked_phase_count)
        .unwrap_or_default();
    let follow_up_actions = follow_up_actions(
        runbook.as_ref(),
        stability_board.as_ref(),
        rollback_drill.as_ref(),
        post_promotion_monitor.as_ref(),
    );
    let prevention_items = prevention_items(
        stability_board.as_ref(),
        rollback_drill.as_ref(),
        post_promotion_monitor.as_ref(),
    );
    let evidence_paths = evidence_paths(
        recovery_runbook_path,
        stability_board_path,
        rollback_drill_path,
        post_promotion_monitor_path,
        &incident_notes,
    );
    let recovery_runbook_id = runbook.as_ref().map(|runbook| runbook.runbook_id.clone());
    let title = active_rollback_reference
        .as_ref()
        .map(|reference| format!("Release recovery review for {reference}"))
        .unwrap_or_else(|| "Release recovery review".to_string());

    FridayReleaseIncidentArchiveEntry {
        incident_id: format!("friday-release-incident-{recorded_at_unix_ms}"),
        recorded_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        severity,
        outcome,
        title,
        summary: incident_summary(severity, outcome, blocked_phase_count, active_risk_count),
        recovery_runbook_id,
        recovery_runbook_json: path_string(recovery_runbook_path),
        stability_board_json: path_string(stability_board_path),
        rollback_drill_json: path_string(rollback_drill_path),
        post_promotion_monitor_json: path_string(post_promotion_monitor_path),
        active_candidate_id,
        active_promotion_id,
        active_rollback_reference,
        blocked_phase_count,
        active_risk_count,
        incident_notes,
        follow_up_actions,
        prevention_items,
        evidence_paths,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn append_friday_release_incident_to_archive(
    archive_path: impl AsRef<Path>,
    recovery_runbook_path: impl AsRef<Path>,
    stability_board_path: impl AsRef<Path>,
    rollback_drill_path: impl AsRef<Path>,
    post_promotion_monitor_path: impl AsRef<Path>,
    incident_note_paths: Vec<String>,
    outcome: FridayReleaseIncidentOutcome,
) -> Result<FridayReleaseIncidentArchive> {
    let archive_path = archive_path.as_ref();
    let mut entries = read_friday_release_incident_archive(archive_path)
        .map(|archive| archive.entries)
        .unwrap_or_default();
    entries.push(friday_release_incident_entry_from_sources(
        recovery_runbook_path,
        stability_board_path,
        rollback_drill_path,
        post_promotion_monitor_path,
        incident_note_paths,
        outcome,
    ));
    let archive = friday_release_incident_archive_report(archive_path, entries);
    write_friday_release_incident_archive(archive_path, &archive)?;
    Ok(archive)
}

pub fn write_friday_release_incident_archive(
    archive_path: impl AsRef<Path>,
    archive: &FridayReleaseIncidentArchive,
) -> Result<()> {
    let archive_path = archive_path.as_ref();
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release incident archive directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(archive_path, archive.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release incident archive {}",
            archive_path.display()
        )
    })
}

pub fn read_friday_release_incident_archive(
    archive_path: impl AsRef<Path>,
) -> Result<FridayReleaseIncidentArchive> {
    let archive_path = archive_path.as_ref();
    let bytes = fs::read(archive_path).with_context(|| {
        format!(
            "Could not read Friday release incident archive {}",
            archive_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release incident archive {}",
            archive_path.display()
        )
    })
}

fn incident_severity(
    runbook: Option<&FridayReleaseRecoveryRunbookReport>,
    stability_board: Option<&FridayReleaseStabilityBoardReport>,
    rollback_drill: Option<&FridayReleaseRollbackDrillReport>,
    post_promotion_monitor: Option<&FridayReleasePostPromotionMonitorReport>,
) -> FridayReleaseIncidentSeverity {
    if runbook.is_none() || stability_board.is_none() || rollback_drill.is_none() {
        return FridayReleaseIncidentSeverity::Blocking;
    }
    if runbook.is_some_and(|runbook| runbook.blocked_phase_count >= 2)
        || rollback_drill.is_some_and(|drill| drill.blocking_count > 0)
        || post_promotion_monitor.is_some_and(|monitor| monitor.blocking_count > 0)
    {
        return FridayReleaseIncidentSeverity::Critical;
    }
    if stability_board.is_some_and(|board| board.blocking_count > 0)
        || runbook.is_some_and(|runbook| runbook.unsatisfied_approval_gate_count > 0)
    {
        return FridayReleaseIncidentSeverity::Blocking;
    }
    if stability_board.is_some_and(|board| board.warning_count > 0 || board.stale_count > 0)
        || post_promotion_monitor.is_some_and(|monitor| monitor.warning_count > 0)
    {
        return FridayReleaseIncidentSeverity::Watch;
    }
    FridayReleaseIncidentSeverity::Info
}

fn follow_up_actions(
    runbook: Option<&FridayReleaseRecoveryRunbookReport>,
    stability_board: Option<&FridayReleaseStabilityBoardReport>,
    rollback_drill: Option<&FridayReleaseRollbackDrillReport>,
    post_promotion_monitor: Option<&FridayReleasePostPromotionMonitorReport>,
) -> Vec<String> {
    let mut actions = Vec::new();
    if let Some(runbook) = runbook {
        actions.extend(
            runbook
                .phases
                .iter()
                .filter(|phase| !phase.next_action.trim().is_empty())
                .map(|phase| format!("{}: {}", phase.label, phase.next_action)),
        );
    }
    if let Some(board) = stability_board {
        actions.extend(board.next_actions.iter().cloned());
    }
    if let Some(drill) = rollback_drill {
        actions.extend(
            drill
                .blocked_reasons
                .iter()
                .map(|reason| format!("Resolve rollback blocker: {reason}")),
        );
    }
    if let Some(monitor) = post_promotion_monitor {
        actions.extend(
            monitor
                .warnings
                .iter()
                .map(|warning| format!("Resolve post-promotion warning: {warning}")),
        );
    }
    actions.sort();
    actions.dedup();
    actions
}

fn prevention_items(
    stability_board: Option<&FridayReleaseStabilityBoardReport>,
    rollback_drill: Option<&FridayReleaseRollbackDrillReport>,
    post_promotion_monitor: Option<&FridayReleasePostPromotionMonitorReport>,
) -> Vec<String> {
    let mut items = Vec::new();
    if let Some(board) = stability_board {
        items.extend(
            board
                .active_risks
                .iter()
                .map(|risk| format!("Prevent recurrence of: {risk}")),
        );
    }
    if let Some(drill) = rollback_drill {
        items.extend(
            drill
                .checks
                .iter()
                .filter(|check| check.status.label() != "passed")
                .map(|check| format!("Improve rollback evidence: {}", check.next_action)),
        );
    }
    if let Some(monitor) = post_promotion_monitor {
        items.extend(
            monitor
                .checks
                .iter()
                .filter(|check| check.status.label() != "passed")
                .map(|check| format!("Improve post-promotion evidence: {}", check.next_action)),
        );
    }
    items.sort();
    items.dedup();
    items
}

fn incident_note(path: &str) -> FridayReleasePostPromotionIncidentNote {
    let path_ref = Path::new(path);
    let metadata = fs::metadata(path_ref).ok();
    let present = metadata.is_some();
    let bytes = metadata.map(|metadata| metadata.len()).unwrap_or_default();
    let summary = if present {
        format!("Incident note `{}` is present.", path_string(path_ref))
    } else {
        format!("Incident note `{}` is missing.", path_string(path_ref))
    };

    FridayReleasePostPromotionIncidentNote {
        id: note_id(path_ref),
        path: path_string(path_ref),
        present,
        bytes,
        summary,
    }
}

fn evidence_paths(
    recovery_runbook_path: &Path,
    stability_board_path: &Path,
    rollback_drill_path: &Path,
    post_promotion_monitor_path: &Path,
    incident_notes: &[FridayReleasePostPromotionIncidentNote],
) -> Vec<String> {
    let mut paths = vec![
        path_string(recovery_runbook_path),
        path_string(stability_board_path),
        path_string(rollback_drill_path),
        path_string(post_promotion_monitor_path),
    ];
    paths.extend(incident_notes.iter().map(|note| note.path.clone()));
    paths.sort();
    paths.dedup();
    paths
}

fn incident_summary(
    severity: FridayReleaseIncidentSeverity,
    outcome: FridayReleaseIncidentOutcome,
    blocked_phase_count: usize,
    active_risk_count: usize,
) -> String {
    format!(
        "Friday release incident is {} with outcome {}; {} blocked recovery phase(s), {} active risk(s).",
        severity.label(),
        outcome.label(),
        blocked_phase_count,
        active_risk_count
    )
}

fn outcome_count(
    entries: &[FridayReleaseIncidentArchiveEntry],
    outcome: FridayReleaseIncidentOutcome,
) -> usize {
    entries
        .iter()
        .filter(|entry| entry.outcome == outcome)
        .count()
}

fn severity_count(
    entries: &[FridayReleaseIncidentArchiveEntry],
    severity: FridayReleaseIncidentSeverity,
) -> usize {
    entries
        .iter()
        .filter(|entry| entry.severity == severity)
        .count()
}

fn note_id(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("incident-note")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
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
