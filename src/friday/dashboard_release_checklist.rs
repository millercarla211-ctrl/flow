use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, friday_dashboard_release_review_from_export,
    read_friday_trusted_runner_release_package, read_friday_trusted_runner_release_timeline,
};
use crate::competitive::{CompletionItemStatus, active_completion_set};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseChecklistBlockerSeverity {
    Warning,
    Blocking,
}

impl FridayReleaseChecklistBlockerSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Blocking => "blocking",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseChecklistSignoffDecision {
    Approved,
    NeedsChanges,
    Blocked,
}

impl FridayReleaseChecklistSignoffDecision {
    pub fn label(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::NeedsChanges => "needs-changes",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseChecklistBlocker {
    pub id: String,
    pub category: String,
    pub severity: FridayReleaseChecklistBlockerSeverity,
    pub title: String,
    pub detail: String,
    pub source_path: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseChecklistItem {
    pub id: String,
    pub title: String,
    pub ready: bool,
    pub detail: String,
    pub source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseChecklistSignoff {
    pub id: String,
    pub checklist_id: String,
    pub operator: String,
    pub decision: FridayReleaseChecklistSignoffDecision,
    pub reason: String,
    pub recorded_at_unix_ms: u128,
    pub local_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseOperatorChecklistReport {
    pub checklist_id: String,
    pub checklist_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub ready_to_ship: bool,
    pub summary: String,
    pub package_json: String,
    pub timeline_json: String,
    pub dashboard_export_dir: String,
    pub todo_path: String,
    pub changelog_path: String,
    pub signoff_json: String,
    pub ready_count: usize,
    pub total_count: usize,
    pub warning_count: usize,
    pub blocking_count: usize,
    pub signoff_required: bool,
    pub signoff_count: usize,
    pub latest_signoff: Option<FridayReleaseChecklistSignoff>,
    pub blockers: Vec<FridayReleaseChecklistBlocker>,
    pub checklist: Vec<FridayReleaseChecklistItem>,
    pub signoffs: Vec<FridayReleaseChecklistSignoff>,
    pub commands: Vec<String>,
}

impl FridayReleaseOperatorChecklistReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_operator_checklist_report(
    checklist_path: impl AsRef<Path>,
    package_path: impl AsRef<Path>,
    timeline_path: impl AsRef<Path>,
    dashboard_export_dir: impl AsRef<Path>,
    todo_path: impl AsRef<Path>,
    changelog_path: impl AsRef<Path>,
    signoff_path: impl AsRef<Path>,
) -> FridayReleaseOperatorChecklistReport {
    let checklist_path = checklist_path.as_ref();
    let package_path = package_path.as_ref();
    let timeline_path = timeline_path.as_ref();
    let dashboard_export_dir = dashboard_export_dir.as_ref();
    let todo_path = todo_path.as_ref();
    let changelog_path = changelog_path.as_ref();
    let signoff_path = signoff_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let mut blockers = Vec::new();
    let mut checklist = Vec::new();

    let package = read_friday_trusted_runner_release_package(package_path).ok();
    match &package {
        Some(package) => {
            checklist.push(item(
                "release-package",
                "Trusted runner release package",
                package.ready_to_ship,
                format!(
                    "{} evidence item(s), {} missing, {} warning(s).",
                    package.manifest.evidence_count,
                    package.manifest.missing_count,
                    package.manifest.warning_count
                ),
                &package.manifest.package_json,
            ));
            if package.manifest.missing_count > 0 {
                blockers.push(blocker(
                    "missing-release-evidence",
                    "missing-evidence",
                    FridayReleaseChecklistBlockerSeverity::Blocking,
                    "Release package has missing evidence",
                    format!(
                        "{} required evidence item(s) are missing from the package.",
                        package.manifest.missing_count
                    ),
                    &package.manifest.package_json,
                    "Regenerate the trusted runner release package after producing the missing files.",
                ));
            }
            for warning in &package.warnings {
                let lower = warning.to_ascii_lowercase();
                let severity =
                    if lower.contains("stale") || lower.contains("pending") || lower.contains("running") {
                        FridayReleaseChecklistBlockerSeverity::Blocking
                    } else {
                        FridayReleaseChecklistBlockerSeverity::Warning
                    };
                blockers.push(blocker(
                    "release-package-warning",
                    if severity == FridayReleaseChecklistBlockerSeverity::Blocking {
                        "stale-live-state"
                    } else {
                        "package-warning"
                    },
                    severity,
                    "Release package warning",
                    warning.clone(),
                    &package.manifest.package_json,
                    "Review the package warning and refresh the package before signoff.",
                ));
            }
        }
        None => {
            checklist.push(item(
                "release-package",
                "Trusted runner release package",
                false,
                "Release package JSON is missing or unreadable.",
                &path_string(package_path),
            ));
            blockers.push(blocker(
                "missing-release-package",
                "missing-evidence",
                FridayReleaseChecklistBlockerSeverity::Blocking,
                "Release package is missing",
                "Friday cannot sign off a release without the trusted runner package.",
                &path_string(package_path),
                "Run `flow --friday-trusted-host-runner-release-package` first.",
            ));
        }
    }

    let timeline = read_friday_trusted_runner_release_timeline(timeline_path).ok();
    match &timeline {
        Some(timeline) => {
            let timeline_ready =
                timeline.missing_evidence_regressions == 0 && timeline.warning_regressions == 0;
            checklist.push(item(
                "release-timeline",
                "Trusted runner evidence timeline",
                timeline_ready,
                format!(
                    "{} package(s), {} missing-evidence regression(s), {} warning regression(s).",
                    timeline.package_count,
                    timeline.missing_evidence_regressions,
                    timeline.warning_regressions
                ),
                &timeline.timeline_json,
            ));
            if timeline.missing_evidence_regressions > 0 {
                blockers.push(blocker(
                    "timeline-missing-evidence-regression",
                    "warning-regression",
                    FridayReleaseChecklistBlockerSeverity::Blocking,
                    "Timeline introduced missing evidence",
                    format!(
                        "{} package comparison(s) introduced new missing evidence.",
                        timeline.missing_evidence_regressions
                    ),
                    &timeline.timeline_json,
                    "Compare the latest package against the previous package and restore the missing evidence.",
                ));
            }
            if timeline.warning_regressions > 0 {
                blockers.push(blocker(
                    "timeline-warning-regression",
                    "warning-regression",
                    FridayReleaseChecklistBlockerSeverity::Warning,
                    "Timeline warning count increased",
                    format!(
                        "{} package comparison(s) increased warning count.",
                        timeline.warning_regressions
                    ),
                    &timeline.timeline_json,
                    "Review the latest package warnings before operator signoff.",
                ));
            }
        }
        None => {
            checklist.push(item(
                "release-timeline",
                "Trusted runner evidence timeline",
                false,
                "Release timeline JSON is missing or unreadable.",
                &path_string(timeline_path),
            ));
            blockers.push(blocker(
                "missing-release-timeline",
                "missing-evidence",
                FridayReleaseChecklistBlockerSeverity::Blocking,
                "Release timeline is missing",
                "Friday cannot compare this package against previous evidence without a timeline.",
                &path_string(timeline_path),
                "Run `flow --friday-trusted-runner-release-timeline` or archive this package first.",
            ));
        }
    }

    let release_review = friday_dashboard_release_review_from_export(dashboard_export_dir).ok();
    match &release_review {
        Some(review) => {
            let ready = review.status == FridayDashboardPanelStatus::Ready;
            checklist.push(item(
                "dashboard-release-review",
                "Dashboard release review",
                ready,
                format!("{}/{} release-review item(s) ready.", review.ready_count, review.total_count),
                &path_string(&dashboard_export_dir.join("release-review.json")),
            ));
            if review.status == FridayDashboardPanelStatus::Blocked {
                blockers.push(blocker(
                    "dashboard-release-review-blocked",
                    "release-review",
                    FridayReleaseChecklistBlockerSeverity::Blocking,
                    "Dashboard release review is blocked",
                    review.summary.clone(),
                    &path_string(&dashboard_export_dir.join("release-review.json")),
                    "Refresh the dashboard export and resolve the blocked release-review items.",
                ));
            } else if review.status == FridayDashboardPanelStatus::Warning {
                blockers.push(blocker(
                    "dashboard-release-review-warning",
                    "release-review",
                    FridayReleaseChecklistBlockerSeverity::Warning,
                    "Dashboard release review needs attention",
                    review.summary.clone(),
                    &path_string(&dashboard_export_dir.join("release-review.json")),
                    "Review the dashboard release handoff before approving the checklist.",
                ));
            }
        }
        None => {
            checklist.push(item(
                "dashboard-release-review",
                "Dashboard release review",
                false,
                "Dashboard release-review JSON is missing or unreadable.",
                &path_string(&dashboard_export_dir.join("release-review.json")),
            ));
            blockers.push(blocker(
                "missing-dashboard-release-review",
                "release-review",
                FridayReleaseChecklistBlockerSeverity::Blocking,
                "Dashboard release review is missing",
                "The checklist needs dashboard release-review evidence before signoff.",
                &path_string(&dashboard_export_dir.join("release-review.json")),
                "Run `flow --friday-dashboard-export` before creating the checklist.",
            ));
        }
    }

    let completion = active_completion_set();
    let completion_ready = completion.current_score_out_of_100 == completion.target_score_out_of_100
        && completion
            .items
            .iter()
            .all(|entry| entry.status == CompletionItemStatus::Done);
    checklist.push(item(
        "active-completion-loop",
        "Active completion loop",
        completion_ready,
        format!(
            "{} is {} / {}.",
            completion.name, completion.current_score_out_of_100, completion.target_score_out_of_100
        ),
        "TODO.md",
    ));
    if !completion_ready {
        blockers.push(blocker(
            "active-loop-not-complete",
            "unreviewed-changes",
            FridayReleaseChecklistBlockerSeverity::Blocking,
            "Active TODO loop is not complete",
            format!(
                "{} is still at {} / {}.",
                completion.name, completion.current_score_out_of_100, completion.target_score_out_of_100
            ),
            "TODO.md",
            "Complete the current TODO loop or explicitly keep the release as a draft.",
        ));
    }

    checklist.push(file_item("todo-file", "TODO.md release plan", todo_path, &mut blockers));
    checklist.push(file_item(
        "changelog-file",
        "CHANGELOG.md release notes",
        changelog_path,
        &mut blockers,
    ));

    let signoffs = read_friday_release_operator_signoffs(signoff_path).unwrap_or_default();
    let latest_signoff = signoffs.last().cloned();
    let signed_off = latest_signoff
        .as_ref()
        .is_some_and(|signoff| signoff.decision == FridayReleaseChecklistSignoffDecision::Approved);
    checklist.push(item(
        "operator-signoff",
        "Operator signoff",
        signed_off,
        if let Some(signoff) = &latest_signoff {
            format!(
                "{} by {}: {}",
                signoff.decision.label(),
                signoff.operator,
                signoff.reason
            )
        } else {
            "No operator signoff has been recorded yet.".to_string()
        },
        &path_string(signoff_path),
    ));
    if !signed_off {
        blockers.push(blocker(
            "operator-signoff-missing",
            "signoff",
            FridayReleaseChecklistBlockerSeverity::Warning,
            "Operator signoff is missing",
            "Record a local signoff after reviewing the checklist blockers.",
            &path_string(signoff_path),
            "Use the signoff command from this checklist after adding a reason.",
        ));
    }

    let ready_count = checklist.iter().filter(|item| item.ready).count();
    let total_count = checklist.len();
    let warning_count = blockers
        .iter()
        .filter(|entry| entry.severity == FridayReleaseChecklistBlockerSeverity::Warning)
        .count();
    let blocking_count = blockers
        .iter()
        .filter(|entry| entry.severity == FridayReleaseChecklistBlockerSeverity::Blocking)
        .count();
    let ready_to_ship = blocking_count == 0 && signed_off;
    let status = if blocking_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if warning_count > 0 || !signed_off {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };
    let checklist_json = path_string(checklist_path);
    let signoff_json = path_string(signoff_path);

    FridayReleaseOperatorChecklistReport {
        checklist_id: format!("friday-release-checklist-{generated_at_unix_ms}"),
        checklist_json: checklist_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        ready_to_ship,
        summary: format!(
            "{ready_count}/{total_count} release checklist item(s) ready; {warning_count} warning(s), {blocking_count} blocking issue(s)."
        ),
        package_json: path_string(package_path),
        timeline_json: path_string(timeline_path),
        dashboard_export_dir: path_string(dashboard_export_dir),
        todo_path: path_string(todo_path),
        changelog_path: path_string(changelog_path),
        signoff_json: signoff_json.clone(),
        ready_count,
        total_count,
        warning_count,
        blocking_count,
        signoff_required: true,
        signoff_count: signoffs.len(),
        latest_signoff,
        blockers,
        checklist,
        signoffs,
        commands: vec![
            format!(
                "flow --friday-release-checklist --package {} --timeline {} --export-dir {} --output {} --signoffs {}",
                path_string(package_path),
                path_string(timeline_path),
                path_string(dashboard_export_dir),
                checklist_json,
                signoff_json
            ),
            format!(
                "flow --friday-release-signoff --checklist {} --signoffs {} --operator \"<operator>\" --decision approved --reason \"<signoff reason>\"",
                path_string(checklist_path),
                signoff_json
            ),
        ],
    }
}

pub fn write_friday_release_operator_checklist(
    checklist_path: impl AsRef<Path>,
    report: &FridayReleaseOperatorChecklistReport,
) -> Result<()> {
    let checklist_path = checklist_path.as_ref();
    if let Some(parent) = checklist_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release checklist directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(checklist_path, report.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release checklist {}",
            checklist_path.display()
        )
    })
}

pub fn read_friday_release_operator_checklist(
    checklist_path: impl AsRef<Path>,
) -> Result<FridayReleaseOperatorChecklistReport> {
    let checklist_path = checklist_path.as_ref();
    let bytes = fs::read(checklist_path).with_context(|| {
        format!(
            "Could not read Friday release checklist {}",
            checklist_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release checklist {}",
            checklist_path.display()
        )
    })
}

pub fn read_friday_release_operator_signoffs(
    signoff_path: impl AsRef<Path>,
) -> Result<Vec<FridayReleaseChecklistSignoff>> {
    let signoff_path = signoff_path.as_ref();
    if !signoff_path.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(signoff_path).with_context(|| {
        format!(
            "Could not read Friday release signoffs {}",
            signoff_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release signoffs {}",
            signoff_path.display()
        )
    })
}

pub fn append_friday_release_operator_signoff(
    checklist_path: impl AsRef<Path>,
    signoff_path: impl AsRef<Path>,
    operator: &str,
    decision: FridayReleaseChecklistSignoffDecision,
    reason: &str,
) -> Result<Vec<FridayReleaseChecklistSignoff>> {
    let checklist = read_friday_release_operator_checklist(checklist_path)?;
    let signoff_path = signoff_path.as_ref();
    let mut records = read_friday_release_operator_signoffs(signoff_path)?;
    records.push(FridayReleaseChecklistSignoff {
        id: format!("friday-release-signoff-{}", unix_ms()),
        checklist_id: checklist.checklist_id,
        operator: operator.trim().to_string(),
        decision,
        reason: reason.trim().to_string(),
        recorded_at_unix_ms: unix_ms(),
        local_only: true,
    });
    if let Some(parent) = signoff_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release signoff directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(signoff_path, serde_json::to_string_pretty(&records)?).with_context(|| {
        format!(
            "Could not write Friday release signoffs {}",
            signoff_path.display()
        )
    })?;
    Ok(records)
}

fn file_item(
    id: &str,
    title: &str,
    path: &Path,
    blockers: &mut Vec<FridayReleaseChecklistBlocker>,
) -> FridayReleaseChecklistItem {
    match fs::metadata(path) {
        Ok(metadata) if metadata.len() > 0 => item(
            id,
            title,
            true,
            format!("{} byte(s) available for release review.", metadata.len()),
            &path_string(path),
        ),
        Ok(_) => {
            blockers.push(blocker(
                &format!("{id}-empty"),
                "unreviewed-changes",
                FridayReleaseChecklistBlockerSeverity::Warning,
                format!("{title} is empty"),
                "The release checklist needs a populated planning/release-notes file.",
                &path_string(path),
                "Update this file before signoff.",
            ));
            item(id, title, false, "File exists but is empty.", &path_string(path))
        }
        Err(_) => {
            blockers.push(blocker(
                &format!("{id}-missing"),
                "unreviewed-changes",
                FridayReleaseChecklistBlockerSeverity::Blocking,
                format!("{title} is missing"),
                "The release checklist needs this file for operator review.",
                &path_string(path),
                "Restore or regenerate this file before signoff.",
            ));
            item(id, title, false, "File is missing.", &path_string(path))
        }
    }
}

fn item(
    id: &str,
    title: &str,
    ready: bool,
    detail: impl Into<String>,
    source_path: &str,
) -> FridayReleaseChecklistItem {
    FridayReleaseChecklistItem {
        id: id.to_string(),
        title: title.to_string(),
        ready,
        detail: detail.into(),
        source_path: source_path.to_string(),
    }
}

fn blocker(
    id: &str,
    category: &str,
    severity: FridayReleaseChecklistBlockerSeverity,
    title: impl Into<String>,
    detail: impl Into<String>,
    source_path: &str,
    next_action: &str,
) -> FridayReleaseChecklistBlocker {
    FridayReleaseChecklistBlocker {
        id: id.to_string(),
        category: category.to_string(),
        severity,
        title: title.into(),
        detail: detail.into(),
        source_path: source_path.to_string(),
        next_action: next_action.to_string(),
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
