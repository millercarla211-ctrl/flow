use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleasePromotionDecision, FridayReleasePromotionLedger,
    FridayReleasePromotionRecord, FridayReleaseQaCommandCenterReport,
    read_friday_release_promotion_ledger, read_friday_release_qa_command_center_report,
};

const STALE_AFTER_MS: u128 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleasePostPromotionCheckStatus {
    Passed,
    Warning,
    Failed,
    Missing,
    Stale,
}

impl FridayReleasePostPromotionCheckStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
            Self::Missing => "missing",
            Self::Stale => "stale",
        }
    }

    fn score_multiplier(self) -> f32 {
        match self {
            Self::Passed => 1.0,
            Self::Warning | Self::Stale => 0.5,
            Self::Failed | Self::Missing => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePostPromotionCheck {
    pub id: String,
    pub label: String,
    pub source_path: String,
    pub required: bool,
    pub present: bool,
    pub stale: bool,
    pub bytes: u64,
    pub status: FridayReleasePostPromotionCheckStatus,
    pub summary: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePostPromotionIncidentNote {
    pub id: String,
    pub path: String,
    pub present: bool,
    pub bytes: u64,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleasePostPromotionMonitorReport {
    pub monitor_id: String,
    pub monitor_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub ready_for_stable: bool,
    pub promotion_ledger_json: String,
    pub qa_json: String,
    pub dashboard_smoke_result_path: String,
    pub active_candidate_id: Option<String>,
    pub active_promotion_id: Option<String>,
    pub active_rollback_reference: Option<String>,
    pub latest_decision: Option<FridayReleasePromotionDecision>,
    pub promoted_count: usize,
    pub incident_note_count: usize,
    pub missing_evidence_count: usize,
    pub stale_count: usize,
    pub warning_count: usize,
    pub blocking_count: usize,
    pub checks: Vec<FridayReleasePostPromotionCheck>,
    pub incident_notes: Vec<FridayReleasePostPromotionIncidentNote>,
    pub warnings: Vec<String>,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleasePostPromotionMonitorReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_post_promotion_monitor_report(
    monitor_path: impl AsRef<Path>,
    promotion_ledger_path: impl AsRef<Path>,
    qa_path: impl AsRef<Path>,
    dashboard_smoke_result_path: impl AsRef<Path>,
    incident_note_paths: Vec<String>,
) -> FridayReleasePostPromotionMonitorReport {
    let monitor_path = monitor_path.as_ref();
    let promotion_ledger_path = promotion_ledger_path.as_ref();
    let qa_path = qa_path.as_ref();
    let dashboard_smoke_result_path = dashboard_smoke_result_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let ledger = read_friday_release_promotion_ledger(promotion_ledger_path).ok();
    let qa = read_friday_release_qa_command_center_report(qa_path).ok();
    let active_promotion = ledger.as_ref().and_then(active_promoted_record);
    let incident_notes = incident_note_paths
        .iter()
        .map(|path| incident_note(path))
        .collect::<Vec<_>>();
    let mut checks = vec![
        promotion_ledger_check(promotion_ledger_path, ledger.as_ref()),
        active_promotion_check(active_promotion),
        rollback_reference_check(ledger.as_ref(), active_promotion),
        qa_check(qa_path, qa.as_ref()),
        dashboard_smoke_check(dashboard_smoke_result_path),
    ];

    if let Some(record) = active_promotion {
        checks.extend(record.post_promotion_checks.iter().map(|check| {
            source_check(
                &format!("post-promotion-{}", check.id),
                &check.label,
                Path::new(&check.result_path),
                check.required,
                if check.present {
                    FridayReleasePostPromotionCheckStatus::Passed
                } else if check.required {
                    FridayReleasePostPromotionCheckStatus::Missing
                } else {
                    FridayReleasePostPromotionCheckStatus::Warning
                },
                &check.summary,
                &check.next_action,
            )
        }));
    }

    if incident_notes.is_empty() {
        checks.push(inline_check(
            "incident-notes",
            "Incident notes",
            FridayReleasePostPromotionCheckStatus::Passed,
            false,
            "No post-promotion incident notes were supplied.",
            "Attach an incident-note file only when a post-promotion issue needs operator review.",
        ));
    } else {
        checks.extend(incident_notes.iter().map(|note| incident_note_check(note)));
    }

    let blocking_count = checks
        .iter()
        .filter(|check| {
            matches!(
                check.status,
                FridayReleasePostPromotionCheckStatus::Failed
                    | FridayReleasePostPromotionCheckStatus::Missing
            )
        })
        .count();
    let warning_count = checks
        .iter()
        .filter(|check| {
            matches!(
                check.status,
                FridayReleasePostPromotionCheckStatus::Warning
                    | FridayReleasePostPromotionCheckStatus::Stale
            )
        })
        .count();
    let stale_count = checks.iter().filter(|check| check.stale).count();
    let missing_evidence_count = checks.iter().filter(|check| !check.present).count();
    let score_out_of_100 = score_checks(&checks);
    let status = if blocking_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if warning_count > 0 {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };
    let ready_for_stable = active_promotion.is_some()
        && blocking_count == 0
        && warning_count == 0
        && stale_count == 0
        && qa.as_ref().is_some_and(|qa| qa.ready_to_ship);
    let mut warnings = ledger
        .as_ref()
        .map(|ledger| ledger.warnings.clone())
        .unwrap_or_default();
    if active_promotion.is_none() {
        warnings
            .push("No promoted candidate is available for post-promotion monitoring.".to_string());
    }
    if missing_evidence_count > 0 {
        warnings.push(format!(
            "{missing_evidence_count} post-promotion evidence item(s) are missing."
        ));
    }
    if stale_count > 0 {
        warnings.push(format!(
            "{stale_count} post-promotion evidence item(s) are stale."
        ));
    }
    let promoted_count = ledger
        .as_ref()
        .map(|ledger| ledger.promoted_count)
        .unwrap_or_default();
    let latest_decision = ledger.as_ref().and_then(|ledger| ledger.latest_decision);
    let active_candidate_id = active_promotion.map(|record| record.candidate_id.clone());
    let active_promotion_id = active_promotion.map(|record| record.promotion_id.clone());
    let active_rollback_reference = active_promotion
        .map(|record| record.rollback_reference.clone())
        .or_else(|| {
            ledger
                .as_ref()
                .and_then(|ledger| ledger.active_rollback_reference.clone())
        });

    FridayReleasePostPromotionMonitorReport {
        monitor_id: format!("friday-release-post-promotion-monitor-{generated_at_unix_ms}"),
        monitor_json: path_string(monitor_path),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        score_out_of_100,
        ready_for_stable,
        promotion_ledger_json: path_string(promotion_ledger_path),
        qa_json: path_string(qa_path),
        dashboard_smoke_result_path: path_string(dashboard_smoke_result_path),
        active_candidate_id,
        active_promotion_id,
        active_rollback_reference,
        latest_decision,
        promoted_count,
        incident_note_count: incident_notes.iter().filter(|note| note.present).count(),
        missing_evidence_count,
        stale_count,
        warning_count,
        blocking_count,
        checks,
        incident_notes,
        warnings,
        summary: format!(
            "Friday post-promotion monitor is {score_out_of_100}/100 with {blocking_count} blocking issue(s), {warning_count} warning(s), and {stale_count} stale check(s)."
        ),
        commands: vec![
            format!(
                "flow --friday-release-post-promotion-monitor --output {} --promotion-ledger {} --qa {} --dashboard-smoke-result {}",
                path_string(monitor_path),
                path_string(promotion_ledger_path),
                path_string(qa_path),
                path_string(dashboard_smoke_result_path)
            ),
            format!(
                "flow --friday-release-post-promotion-monitor-json --output {}",
                path_string(monitor_path)
            ),
        ],
    }
}

pub fn write_friday_release_post_promotion_monitor_report(
    monitor_path: impl AsRef<Path>,
    report: &FridayReleasePostPromotionMonitorReport,
) -> Result<()> {
    let monitor_path = monitor_path.as_ref();
    if let Some(parent) = monitor_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday post-promotion monitor directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(monitor_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday post-promotion monitor {}",
            monitor_path.display()
        )
    })
}

pub fn read_friday_release_post_promotion_monitor_report(
    monitor_path: impl AsRef<Path>,
) -> Result<FridayReleasePostPromotionMonitorReport> {
    let monitor_path = monitor_path.as_ref();
    let bytes = fs::read(monitor_path).with_context(|| {
        format!(
            "Could not read Friday post-promotion monitor {}",
            monitor_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday post-promotion monitor {}",
            monitor_path.display()
        )
    })
}

fn active_promoted_record(
    ledger: &FridayReleasePromotionLedger,
) -> Option<&FridayReleasePromotionRecord> {
    ledger
        .records
        .iter()
        .rev()
        .find(|record| record.decision == FridayReleasePromotionDecision::Promoted)
}

fn promotion_ledger_check(
    path: &Path,
    ledger: Option<&FridayReleasePromotionLedger>,
) -> FridayReleasePostPromotionCheck {
    match ledger {
        Some(ledger) => source_check(
            "promotion-ledger",
            "Release promotion ledger",
            path,
            true,
            if ledger.promoted_count > 0 {
                FridayReleasePostPromotionCheckStatus::Passed
            } else {
                FridayReleasePostPromotionCheckStatus::Failed
            },
            format!(
                "{} promotion record(s), {} promoted candidate(s).",
                ledger.record_count, ledger.promoted_count
            ),
            "Record a promoted candidate before trusting post-promotion status.",
        ),
        None => source_check(
            "promotion-ledger",
            "Release promotion ledger",
            path,
            true,
            FridayReleasePostPromotionCheckStatus::Missing,
            "Promotion ledger JSON is missing or unreadable.",
            "Generate the release promotion ledger before post-promotion monitoring.",
        ),
    }
}

fn active_promotion_check(
    record: Option<&FridayReleasePromotionRecord>,
) -> FridayReleasePostPromotionCheck {
    match record {
        Some(record) => {
            let status = if record.post_promotion_missing_count > 0 {
                FridayReleasePostPromotionCheckStatus::Failed
            } else if !record.candidate_ready_to_deploy || record.candidate_blocker_count > 0 {
                FridayReleasePostPromotionCheckStatus::Warning
            } else {
                FridayReleasePostPromotionCheckStatus::Passed
            };
            inline_check(
                "active-promotion",
                "Active promoted candidate",
                status,
                true,
                format!(
                    "{} is promoted with {} candidate blocker(s) and {} missing post-promotion check(s).",
                    record.candidate_id,
                    record.candidate_blocker_count,
                    record.post_promotion_missing_count
                ),
                "Resolve every missing post-promotion check before marking the release stable.",
            )
        }
        None => inline_check(
            "active-promotion",
            "Active promoted candidate",
            FridayReleasePostPromotionCheckStatus::Missing,
            true,
            "No promoted candidate is active.",
            "Record a promoted candidate in the release promotion ledger.",
        ),
    }
}

fn rollback_reference_check(
    ledger: Option<&FridayReleasePromotionLedger>,
    record: Option<&FridayReleasePromotionRecord>,
) -> FridayReleasePostPromotionCheck {
    let rollback = record
        .map(|record| record.rollback_reference.as_str())
        .or_else(|| ledger.and_then(|ledger| ledger.active_rollback_reference.as_deref()))
        .unwrap_or_default();
    inline_check(
        "rollback-reference",
        "Rollback reference",
        if rollback.trim().is_empty() {
            FridayReleasePostPromotionCheckStatus::Failed
        } else {
            FridayReleasePostPromotionCheckStatus::Passed
        },
        true,
        if rollback.trim().is_empty() {
            "No rollback reference is attached to the active promotion.".to_string()
        } else {
            format!("Active rollback reference: {rollback}")
        },
        "Attach a rollback reference before marking the promoted candidate stable.",
    )
}

fn qa_check(
    path: &Path,
    qa: Option<&FridayReleaseQaCommandCenterReport>,
) -> FridayReleasePostPromotionCheck {
    match qa {
        Some(qa) => {
            let status = if qa.blocking_count > 0 {
                FridayReleasePostPromotionCheckStatus::Failed
            } else if qa.stale_count > 0 {
                FridayReleasePostPromotionCheckStatus::Stale
            } else if qa.warning_count > 0 || !qa.ready_to_ship {
                FridayReleasePostPromotionCheckStatus::Warning
            } else {
                FridayReleasePostPromotionCheckStatus::Passed
            };
            source_check(
                "release-qa",
                "Release QA command center",
                path,
                true,
                status,
                format!(
                    "QA score {} / 100 with {} blocking issue(s), {} warning(s), and {} stale check(s).",
                    qa.score_out_of_100, qa.blocking_count, qa.warning_count, qa.stale_count
                ),
                "Refresh release QA and clear every warning before stable promotion.",
            )
        }
        None => source_check(
            "release-qa",
            "Release QA command center",
            path,
            true,
            FridayReleasePostPromotionCheckStatus::Missing,
            "Release QA command center JSON is missing or unreadable.",
            "Generate release QA before post-promotion monitoring.",
        ),
    }
}

fn dashboard_smoke_check(path: &Path) -> FridayReleasePostPromotionCheck {
    match fs::metadata(path) {
        Ok(metadata) => {
            let modified_ms = metadata
                .modified()
                .ok()
                .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_millis())
                .unwrap_or_default();
            let stale = unix_ms().saturating_sub(modified_ms) > STALE_AFTER_MS;
            source_check(
                "dashboard-smoke",
                "Dashboard smoke result",
                path,
                true,
                if stale {
                    FridayReleasePostPromotionCheckStatus::Stale
                } else {
                    FridayReleasePostPromotionCheckStatus::Passed
                },
                format!(
                    "{} byte(s) captured from the latest dashboard smoke result.",
                    metadata.len()
                ),
                "Refresh dashboard smoke after post-promotion UI or dashboard changes.",
            )
        }
        Err(_) => source_check(
            "dashboard-smoke",
            "Dashboard smoke result",
            path,
            true,
            FridayReleasePostPromotionCheckStatus::Missing,
            "Dashboard smoke result file is missing.",
            "Run npm run smoke:dashboard and save the output before stable promotion.",
        ),
    }
}

fn incident_note(path: &str) -> FridayReleasePostPromotionIncidentNote {
    match fs::metadata(path) {
        Ok(metadata) => FridayReleasePostPromotionIncidentNote {
            id: incident_id(path),
            path: path_string(Path::new(path)),
            present: true,
            bytes: metadata.len(),
            summary: "Incident note evidence is present.".to_string(),
        },
        Err(_) => FridayReleasePostPromotionIncidentNote {
            id: incident_id(path),
            path: path_string(Path::new(path)),
            present: false,
            bytes: 0,
            summary: "Incident note evidence is missing.".to_string(),
        },
    }
}

fn incident_note_check(
    note: &FridayReleasePostPromotionIncidentNote,
) -> FridayReleasePostPromotionCheck {
    source_check(
        &format!("incident-note-{}", note.id),
        "Incident note",
        Path::new(&note.path),
        false,
        if note.present {
            FridayReleasePostPromotionCheckStatus::Passed
        } else {
            FridayReleasePostPromotionCheckStatus::Warning
        },
        &note.summary,
        "Attach the incident-note file when a post-promotion issue needs review.",
    )
}

fn source_check(
    id: &str,
    label: &str,
    source_path: &Path,
    required: bool,
    status: FridayReleasePostPromotionCheckStatus,
    summary: impl Into<String>,
    next_action: &str,
) -> FridayReleasePostPromotionCheck {
    let metadata = fs::metadata(source_path).ok();
    FridayReleasePostPromotionCheck {
        id: id.to_string(),
        label: label.to_string(),
        source_path: path_string(source_path),
        required,
        present: metadata.is_some() || source_path == Path::new("inline"),
        stale: status == FridayReleasePostPromotionCheckStatus::Stale,
        bytes: metadata.map(|metadata| metadata.len()).unwrap_or_default(),
        status,
        summary: summary.into(),
        next_action: next_action.to_string(),
    }
}

fn inline_check(
    id: &str,
    label: &str,
    status: FridayReleasePostPromotionCheckStatus,
    required: bool,
    summary: impl Into<String>,
    next_action: &str,
) -> FridayReleasePostPromotionCheck {
    FridayReleasePostPromotionCheck {
        id: id.to_string(),
        label: label.to_string(),
        source_path: "inline".to_string(),
        required,
        present: !matches!(status, FridayReleasePostPromotionCheckStatus::Missing),
        stale: false,
        bytes: 0,
        status,
        summary: summary.into(),
        next_action: next_action.to_string(),
    }
}

fn score_checks(checks: &[FridayReleasePostPromotionCheck]) -> u8 {
    let possible = checks.len() as f32;
    if possible <= f32::EPSILON {
        return 0;
    }
    let earned = checks
        .iter()
        .map(|check| check.status.score_multiplier())
        .sum::<f32>();
    ((earned / possible) * 100.0).round().clamp(0.0, 100.0) as u8
}

fn incident_id(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("incident-note")
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
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
