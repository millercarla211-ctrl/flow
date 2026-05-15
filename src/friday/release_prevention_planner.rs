use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseIncidentArchive, FridayReleaseIncidentSeverity,
    FridayReleaseStabilityBoardReport, read_friday_release_incident_archive,
    read_friday_release_stability_board_report,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleasePreventionFindingKind {
    CriticalIncident,
    RepeatedFailureClass,
    StaleEvidence,
    MissingEvidence,
    MissingIncidentNote,
    RollbackGap,
    StabilityGate,
}

impl FridayReleasePreventionFindingKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::CriticalIncident => "critical-incident",
            Self::RepeatedFailureClass => "repeated-failure-class",
            Self::StaleEvidence => "stale-evidence",
            Self::MissingEvidence => "missing-evidence",
            Self::MissingIncidentNote => "missing-incident-note",
            Self::RollbackGap => "rollback-gap",
            Self::StabilityGate => "stability-gate",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleasePreventionActionKind {
    RefreshEvidence,
    HardenRollback,
    AttachIncidentNote,
    ResolveRecurrence,
    ReviewReleaseGate,
}

impl FridayReleasePreventionActionKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::RefreshEvidence => "refresh-evidence",
            Self::HardenRollback => "harden-rollback",
            Self::AttachIncidentNote => "attach-incident-note",
            Self::ResolveRecurrence => "resolve-recurrence",
            Self::ReviewReleaseGate => "review-release-gate",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleasePreventionActionStatus {
    OwnerReady,
    NeedsEvidence,
    Blocked,
}

impl FridayReleasePreventionActionStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::OwnerReady => "owner-ready",
            Self::NeedsEvidence => "needs-evidence",
            Self::Blocked => "blocked",
        }
    }

    fn score_multiplier(self) -> f32 {
        match self {
            Self::OwnerReady => 1.0,
            Self::NeedsEvidence => 0.45,
            Self::Blocked => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePreventionFinding {
    pub id: String,
    pub kind: FridayReleasePreventionFindingKind,
    pub severity: FridayReleaseIncidentSeverity,
    pub title: String,
    pub recurrence_count: usize,
    pub source_paths: Vec<String>,
    pub summary: String,
    pub next_action: String,
    pub release_gate_blocking: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePreventionAction {
    pub id: String,
    pub kind: FridayReleasePreventionActionKind,
    pub status: FridayReleasePreventionActionStatus,
    pub owner: String,
    pub title: String,
    pub summary: String,
    pub source_path: String,
    pub evidence_path: String,
    pub command: String,
    pub required: bool,
    pub release_gate_blocking: bool,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePreventionEvidenceLink {
    pub id: String,
    pub label: String,
    pub path: String,
    pub present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePreventionPlanReport {
    pub plan_id: String,
    pub plan_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub ready_for_next_checkpoint: bool,
    pub incident_archive_json: String,
    pub stability_board_json: String,
    pub incident_count: usize,
    pub finding_count: usize,
    pub recurring_issue_count: usize,
    pub action_count: usize,
    pub owner_ready_count: usize,
    pub blocker_count: usize,
    pub evidence_missing_count: usize,
    pub gate_blocking_count: usize,
    pub latest_incident_id: Option<String>,
    pub active_rollback_reference: Option<String>,
    pub findings: Vec<FridayReleasePreventionFinding>,
    pub actions: Vec<FridayReleasePreventionAction>,
    pub evidence_links: Vec<FridayReleasePreventionEvidenceLink>,
    pub owner_ready_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleasePreventionPlanReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_prevention_plan_report(
    plan_path: impl AsRef<Path>,
    incident_archive_path: impl AsRef<Path>,
    stability_board_path: impl AsRef<Path>,
) -> FridayReleasePreventionPlanReport {
    let plan_path = plan_path.as_ref();
    let incident_archive_path = incident_archive_path.as_ref();
    let stability_board_path = stability_board_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let archive = read_friday_release_incident_archive(incident_archive_path).ok();
    let board = read_friday_release_stability_board_report(stability_board_path).ok();
    let findings = prevention_findings(
        incident_archive_path,
        stability_board_path,
        archive.as_ref(),
        board.as_ref(),
    );
    let evidence_links = prevention_evidence_links(
        incident_archive_path,
        stability_board_path,
        archive.as_ref(),
        board.as_ref(),
    );
    let actions = prevention_actions(
        plan_path,
        incident_archive_path,
        stability_board_path,
        &findings,
        archive.as_ref(),
        board.as_ref(),
    );
    let action_count = actions.len();
    let owner_ready_count = actions
        .iter()
        .filter(|action| action.status == FridayReleasePreventionActionStatus::OwnerReady)
        .count();
    let blocker_count = findings
        .iter()
        .filter(|finding| {
            matches!(
                finding.severity,
                FridayReleaseIncidentSeverity::Blocking | FridayReleaseIncidentSeverity::Critical
            )
        })
        .count();
    let evidence_missing_count = evidence_links.iter().filter(|link| !link.present).count();
    let gate_blocking_count = findings
        .iter()
        .filter(|finding| finding.release_gate_blocking)
        .count()
        + actions
            .iter()
            .filter(|action| action.release_gate_blocking)
            .count();
    let recurring_issue_count = findings
        .iter()
        .filter(|finding| finding.kind == FridayReleasePreventionFindingKind::RepeatedFailureClass)
        .count();
    let score_out_of_100 = score_actions(&actions, evidence_missing_count);
    let ready_for_next_checkpoint = archive.is_some()
        && board.is_some()
        && gate_blocking_count == 0
        && evidence_missing_count == 0
        && actions.iter().all(|action| {
            action.status == FridayReleasePreventionActionStatus::OwnerReady || !action.required
        });
    let status = if !ready_for_next_checkpoint && (blocker_count > 0 || gate_blocking_count > 0) {
        FridayDashboardPanelStatus::Blocked
    } else if !ready_for_next_checkpoint {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };
    let incident_count = archive
        .as_ref()
        .map(|archive| archive.incident_count)
        .unwrap_or_default();
    let latest_incident_id = archive
        .as_ref()
        .and_then(|archive| archive.latest_incident_id.clone());
    let active_rollback_reference = board
        .as_ref()
        .and_then(|board| board.active_rollback_reference.clone())
        .or_else(|| {
            archive
                .as_ref()
                .and_then(|archive| archive.latest_rollback_reference.clone())
        });
    let plan_json = path_string(plan_path);
    let incident_archive_json = path_string(incident_archive_path);
    let stability_board_json = path_string(stability_board_path);

    FridayReleasePreventionPlanReport {
        plan_id: format!("friday-release-prevention-plan-{generated_at_unix_ms}"),
        plan_json: plan_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        score_out_of_100,
        ready_for_next_checkpoint,
        incident_archive_json: incident_archive_json.clone(),
        stability_board_json: stability_board_json.clone(),
        incident_count,
        finding_count: findings.len(),
        recurring_issue_count,
        action_count,
        owner_ready_count,
        blocker_count,
        evidence_missing_count,
        gate_blocking_count,
        latest_incident_id,
        active_rollback_reference,
        owner_ready_copy: owner_ready_copy(&actions),
        summary: format!(
            "Friday prevention plan is {score_out_of_100}/100 with {blocker_count} blocker(s), {recurring_issue_count} recurring issue class(es), and {gate_blocking_count} release gate blocker(s)."
        ),
        commands: vec![
            format!(
                "flow --friday-release-prevention-plan --output {} --incident-archive {} --stability-board {}",
                plan_json, incident_archive_json, stability_board_json
            ),
            format!(
                "flow --friday-release-prevention-plan-json --output {} --incident-archive {} --stability-board {}",
                plan_json, incident_archive_json, stability_board_json
            ),
        ],
        findings,
        actions,
        evidence_links,
    }
}

pub fn write_friday_release_prevention_plan_report(
    plan_path: impl AsRef<Path>,
    report: &FridayReleasePreventionPlanReport,
) -> Result<()> {
    let plan_path = plan_path.as_ref();
    if let Some(parent) = plan_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release prevention plan directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(plan_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release prevention plan {}",
            plan_path.display()
        )
    })
}

pub fn read_friday_release_prevention_plan_report(
    plan_path: impl AsRef<Path>,
) -> Result<FridayReleasePreventionPlanReport> {
    let plan_path = plan_path.as_ref();
    let bytes = fs::read(plan_path).with_context(|| {
        format!(
            "Could not read Friday release prevention plan {}",
            plan_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release prevention plan {}",
            plan_path.display()
        )
    })
}

fn prevention_findings(
    incident_archive_path: &Path,
    stability_board_path: &Path,
    archive: Option<&FridayReleaseIncidentArchive>,
    board: Option<&FridayReleaseStabilityBoardReport>,
) -> Vec<FridayReleasePreventionFinding> {
    let mut findings = Vec::new();

    if archive.is_none() {
        findings.push(finding(
            "incident-archive-missing",
            FridayReleasePreventionFindingKind::MissingEvidence,
            FridayReleaseIncidentSeverity::Blocking,
            "Incident archive is missing",
            1,
            vec![path_string(incident_archive_path)],
            "Friday cannot learn from release recovery decisions until the incident archive exists.",
            "Generate the incident archive before the next checkpoint.",
            true,
        ));
    }
    if board.is_none() {
        findings.push(finding(
            "stability-board-missing",
            FridayReleasePreventionFindingKind::MissingEvidence,
            FridayReleaseIncidentSeverity::Blocking,
            "Stability board is missing",
            1,
            vec![path_string(stability_board_path)],
            "Friday cannot block the next checkpoint without the current stability board.",
            "Generate the stability board before the prevention plan.",
            true,
        ));
    }

    if let Some(archive) = archive {
        if archive.critical_count > 0 {
            findings.push(finding(
                "critical-incidents",
                FridayReleasePreventionFindingKind::CriticalIncident,
                FridayReleaseIncidentSeverity::Critical,
                "Critical incidents are still in the archive",
                archive.critical_count,
                archive
                    .entries
                    .iter()
                    .filter(|entry| entry.severity == FridayReleaseIncidentSeverity::Critical)
                    .flat_map(|entry| entry.evidence_paths.clone())
                    .collect(),
                "Critical incident history must produce explicit prevention evidence before another checkpoint.",
                "Assign prevention owners for every critical incident.",
                true,
            ));
        }
        let missing_note_count = archive
            .entries
            .iter()
            .filter(|entry| {
                entry.incident_notes.is_empty()
                    || entry.incident_notes.iter().any(|note| !note.present)
            })
            .count();
        if missing_note_count > 0 {
            findings.push(finding(
                "missing-incident-notes",
                FridayReleasePreventionFindingKind::MissingIncidentNote,
                FridayReleaseIncidentSeverity::Blocking,
                "Incident notes are missing",
                missing_note_count,
                archive.entries.iter().map(|entry| entry.recovery_runbook_json.clone()).collect(),
                "Every release incident needs an attached note before prevention work can be reviewed.",
                "Attach incident-note markdown files to the archive.",
                true,
            ));
        }
        findings.extend(repeated_failure_findings(archive));
    }

    if let Some(board) = board {
        if board.stale_count > 0 {
            findings.push(finding(
                "stale-stability-evidence",
                FridayReleasePreventionFindingKind::StaleEvidence,
                FridayReleaseIncidentSeverity::Blocking,
                "Stability evidence is stale",
                board.stale_count,
                board
                    .checks
                    .iter()
                    .filter(|check| check.stale)
                    .map(|check| check.source_path.clone())
                    .collect(),
                "The next release checkpoint must not proceed while stability evidence is stale.",
                "Refresh stale stability evidence and regenerate the board.",
                true,
            ));
        }
        if board.missing_evidence_count > 0 {
            findings.push(finding(
                "missing-stability-evidence",
                FridayReleasePreventionFindingKind::MissingEvidence,
                FridayReleaseIncidentSeverity::Blocking,
                "Stability evidence is missing",
                board.missing_evidence_count,
                board
                    .evidence_links
                    .iter()
                    .filter(|link| !link.present)
                    .map(|link| link.path.clone())
                    .collect(),
                "Missing release evidence blocks prevention review.",
                "Attach the missing stability evidence before the next checkpoint.",
                true,
            ));
        }
        if !board.recoverable
            || board
                .active_risks
                .iter()
                .any(|risk| risk.to_ascii_lowercase().contains("rollback"))
        {
            findings.push(finding(
                "rollback-recovery-gap",
                FridayReleasePreventionFindingKind::RollbackGap,
                FridayReleaseIncidentSeverity::Blocking,
                "Rollback recovery is not clean",
                1,
                vec![board.rollback_drill_json.clone()],
                "Friday needs a clean rollback recovery path before another checkpoint.",
                "Run a clean rollback drill and attach the result.",
                true,
            ));
        }
        if !board.ready_for_checkpoint {
            findings.push(finding(
                "checkpoint-gate-blocked",
                FridayReleasePreventionFindingKind::StabilityGate,
                FridayReleaseIncidentSeverity::Blocking,
                "Next checkpoint is blocked by stability board",
                board.blocking_count.max(1),
                vec![path_string(stability_board_path)],
                "The stability board is not ready for the next checkpoint.",
                "Resolve stability-board blockers before checkpoint review.",
                true,
            ));
        }
    }

    findings.sort_by(|left, right| left.id.cmp(&right.id));
    findings.dedup_by(|left, right| left.id == right.id);
    findings
}

fn repeated_failure_findings(
    archive: &FridayReleaseIncidentArchive,
) -> Vec<FridayReleasePreventionFinding> {
    let mut classes: BTreeMap<String, (usize, Vec<String>)> = BTreeMap::new();
    for entry in &archive.entries {
        for text in entry
            .prevention_items
            .iter()
            .chain(entry.follow_up_actions.iter())
        {
            let class = failure_class(text);
            let record = classes.entry(class).or_insert_with(|| (0, Vec::new()));
            record.0 += 1;
            record.1.extend(entry.evidence_paths.clone());
        }
    }

    classes
        .into_iter()
        .filter(|(_, (count, _))| *count >= 2)
        .map(|(class, (count, paths))| {
            finding(
                &format!("repeated-{class}"),
                FridayReleasePreventionFindingKind::RepeatedFailureClass,
                if count >= 3 {
                    FridayReleaseIncidentSeverity::Critical
                } else {
                    FridayReleaseIncidentSeverity::Blocking
                },
                &format!("Repeated {class} release failure class"),
                count,
                paths,
                &format!(
                    "The incident archive shows {count} repeated {class} prevention signal(s)."
                ),
                "Create one owner-ready prevention action for this repeated failure class.",
                true,
            )
        })
        .collect()
}

fn prevention_actions(
    plan_path: &Path,
    incident_archive_path: &Path,
    stability_board_path: &Path,
    findings: &[FridayReleasePreventionFinding],
    archive: Option<&FridayReleaseIncidentArchive>,
    board: Option<&FridayReleaseStabilityBoardReport>,
) -> Vec<FridayReleasePreventionAction> {
    let mut actions = findings
        .iter()
        .map(|finding| {
            action_for_finding(
                plan_path,
                incident_archive_path,
                stability_board_path,
                finding,
                archive,
                board,
            )
        })
        .collect::<Vec<_>>();

    if actions.is_empty() {
        actions.push(FridayReleasePreventionAction {
            id: "keep-prevention-plan-current".to_string(),
            kind: FridayReleasePreventionActionKind::ReviewReleaseGate,
            status: FridayReleasePreventionActionStatus::OwnerReady,
            owner: "release-operator".to_string(),
            title: "Keep prevention plan current".to_string(),
            summary: "No blocking prevention findings are active.".to_string(),
            source_path: path_string(stability_board_path),
            evidence_path: path_string(plan_path),
            command: format!(
                "flow --friday-release-prevention-plan --output {} --incident-archive {} --stability-board {}",
                path_string(plan_path),
                path_string(incident_archive_path),
                path_string(stability_board_path)
            ),
            required: false,
            release_gate_blocking: false,
            next_action: "Regenerate the prevention plan after the next release incident.".to_string(),
        });
    }

    actions.sort_by(|left, right| left.id.cmp(&right.id));
    actions
}

fn action_for_finding(
    plan_path: &Path,
    incident_archive_path: &Path,
    stability_board_path: &Path,
    finding: &FridayReleasePreventionFinding,
    archive: Option<&FridayReleaseIncidentArchive>,
    board: Option<&FridayReleaseStabilityBoardReport>,
) -> FridayReleasePreventionAction {
    let (kind, title, command, evidence_path) = match finding.kind {
        FridayReleasePreventionFindingKind::StaleEvidence
        | FridayReleasePreventionFindingKind::MissingEvidence => (
            FridayReleasePreventionActionKind::RefreshEvidence,
            "Refresh release evidence",
            format!(
                "flow --friday-release-stability-board-json --output {}",
                path_string(stability_board_path)
            ),
            path_string(stability_board_path),
        ),
        FridayReleasePreventionFindingKind::RollbackGap => (
            FridayReleasePreventionActionKind::HardenRollback,
            "Harden rollback drill evidence",
            format!(
                "flow --friday-release-rollback-drill-json --output {} --dry-run",
                board
                    .map(|board| board.rollback_drill_json.clone())
                    .unwrap_or_else(
                        || "tmp/friday-dashboard/release-rollback-drill.json".to_string()
                    )
            ),
            board
                .map(|board| board.rollback_drill_json.clone())
                .unwrap_or_else(|| "tmp/friday-dashboard/release-rollback-drill.json".to_string()),
        ),
        FridayReleasePreventionFindingKind::MissingIncidentNote => (
            FridayReleasePreventionActionKind::AttachIncidentNote,
            "Attach missing incident notes",
            format!(
                "flow --friday-release-incident-archive --archive {} --runbook <release-recovery-runbook.json> --incident-note <incident-note.md>",
                path_string(incident_archive_path)
            ),
            path_string(incident_archive_path),
        ),
        FridayReleasePreventionFindingKind::RepeatedFailureClass
        | FridayReleasePreventionFindingKind::CriticalIncident => (
            FridayReleasePreventionActionKind::ResolveRecurrence,
            "Assign prevention owner",
            format!(
                "flow --friday-release-prevention-plan --output {} --incident-archive {} --stability-board {}",
                path_string(plan_path),
                path_string(incident_archive_path),
                path_string(stability_board_path)
            ),
            path_string(plan_path),
        ),
        FridayReleasePreventionFindingKind::StabilityGate => (
            FridayReleasePreventionActionKind::ReviewReleaseGate,
            "Resolve checkpoint gate blockers",
            format!(
                "flow --friday-release-stability-board-json --output {}",
                path_string(stability_board_path)
            ),
            path_string(stability_board_path),
        ),
    };
    let evidence_missing = finding
        .source_paths
        .iter()
        .any(|path| path != "inline" && !Path::new(path).exists());
    let status = if archive.is_none() || board.is_none() {
        FridayReleasePreventionActionStatus::Blocked
    } else if evidence_missing
        || finding.kind == FridayReleasePreventionFindingKind::MissingEvidence
    {
        FridayReleasePreventionActionStatus::NeedsEvidence
    } else {
        FridayReleasePreventionActionStatus::OwnerReady
    };

    FridayReleasePreventionAction {
        id: format!("prevent-{}", finding.id),
        kind,
        status,
        owner: "release-operator".to_string(),
        title: title.to_string(),
        summary: finding.summary.clone(),
        source_path: finding
            .source_paths
            .first()
            .cloned()
            .unwrap_or_else(|| "inline".to_string()),
        evidence_path,
        command,
        required: finding.release_gate_blocking,
        release_gate_blocking: finding.release_gate_blocking
            && status != FridayReleasePreventionActionStatus::OwnerReady,
        next_action: finding.next_action.clone(),
    }
}

fn prevention_evidence_links(
    incident_archive_path: &Path,
    stability_board_path: &Path,
    archive: Option<&FridayReleaseIncidentArchive>,
    board: Option<&FridayReleaseStabilityBoardReport>,
) -> Vec<FridayReleasePreventionEvidenceLink> {
    let mut links = vec![
        evidence_link(
            "incident-archive",
            "Incident archive",
            incident_archive_path,
        ),
        evidence_link("stability-board", "Stability board", stability_board_path),
    ];
    if let Some(archive) = archive {
        for entry in &archive.entries {
            links.push(FridayReleasePreventionEvidenceLink {
                id: format!("incident-{}", entry.incident_id),
                label: entry.title.clone(),
                path: entry.recovery_runbook_json.clone(),
                present: Path::new(&entry.recovery_runbook_json).exists(),
            });
        }
    }
    if let Some(board) = board {
        links.extend(
            board
                .evidence_links
                .iter()
                .map(|link| FridayReleasePreventionEvidenceLink {
                    id: format!("stability-{}", link.id),
                    label: link.label.clone(),
                    path: link.path.clone(),
                    present: link.present,
                }),
        );
    }
    links
}

fn finding(
    id: &str,
    kind: FridayReleasePreventionFindingKind,
    severity: FridayReleaseIncidentSeverity,
    title: &str,
    recurrence_count: usize,
    mut source_paths: Vec<String>,
    summary: &str,
    next_action: &str,
    release_gate_blocking: bool,
) -> FridayReleasePreventionFinding {
    source_paths.sort();
    source_paths.dedup();
    FridayReleasePreventionFinding {
        id: id.to_string(),
        kind,
        severity,
        title: title.to_string(),
        recurrence_count,
        source_paths,
        summary: summary.to_string(),
        next_action: next_action.to_string(),
        release_gate_blocking,
    }
}

fn evidence_link(id: &str, label: &str, path: &Path) -> FridayReleasePreventionEvidenceLink {
    FridayReleasePreventionEvidenceLink {
        id: id.to_string(),
        label: label.to_string(),
        path: path_string(path),
        present: path.exists(),
    }
}

fn failure_class(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    if lower.contains("rollback") {
        "rollback".to_string()
    } else if lower.contains("post-promotion") || lower.contains("promotion") {
        "post-promotion".to_string()
    } else if lower.contains("stale") || lower.contains("refresh") {
        "stale-evidence".to_string()
    } else if lower.contains("missing") || lower.contains("attach") {
        "missing-evidence".to_string()
    } else if lower.contains("deployment") || lower.contains("gate") {
        "deployment-gate".to_string()
    } else {
        "release-process".to_string()
    }
}

fn score_actions(actions: &[FridayReleasePreventionAction], missing_evidence_count: usize) -> u8 {
    if actions.is_empty() {
        return 0;
    }
    let earned = actions
        .iter()
        .map(|action| action.status.score_multiplier())
        .sum::<f32>();
    let penalty = (missing_evidence_count as f32 * 8.0).min(30.0);
    (((earned / actions.len() as f32) * 100.0) - penalty)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn owner_ready_copy(actions: &[FridayReleasePreventionAction]) -> String {
    let mut lines = vec!["Friday release prevention plan".to_string()];
    for action in actions {
        lines.push(format!(
            "- [{}] {} -> {}",
            action.status.label(),
            action.title,
            action.next_action
        ));
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
