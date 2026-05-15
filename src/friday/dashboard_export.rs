use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    FridayExecutionHandoffReport, FridayLiveUiRouteBindingReport, FridayOperatorReadinessReport,
    FridayOperatorReadinessStatus, FridayRouteVisualReport, FridayRouteVisualTarget,
    friday_execution_handoff_report, friday_live_ui_route_binding_report,
    friday_operator_readiness_report, friday_route_visual_report,
};
use crate::competitive::{CompletionSet, active_completion_set};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayDashboardPanelStatus {
    Ready,
    Warning,
    Blocked,
}

impl FridayDashboardPanelStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Warning => "warning",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardExportFile {
    pub path: String,
    pub kind: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayDashboardScreenshotStatus {
    Captured,
    Missing,
    MetadataMissing,
}

impl FridayDashboardScreenshotStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Captured => "captured",
            Self::Missing => "missing",
            Self::MetadataMissing => "metadata-missing",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardScreenshotRecord {
    pub route: String,
    pub title: String,
    pub viewport_id: String,
    pub viewport_width: u16,
    pub viewport_height: u16,
    pub screenshot_path: String,
    pub metadata_path: String,
    pub status: FridayDashboardScreenshotStatus,
    pub screenshot_bytes: u64,
    pub metadata_bytes: u64,
    pub captured_at_unix_ms: Option<u128>,
    pub capture_command: String,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardScreenshotHistory {
    pub total_targets: usize,
    pub captured_count: usize,
    pub missing_count: usize,
    pub metadata_missing_count: usize,
    pub records: Vec<FridayDashboardScreenshotRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardExportManifest {
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub loop_name: String,
    pub score_out_of_100: u8,
    pub export_dir: String,
    pub summary: String,
    pub manifest_json: String,
    pub readiness_json: String,
    pub route_bindings_json: String,
    pub route_visuals_json: String,
    pub execution_handoffs_json: String,
    pub completion_json: String,
    pub dashboard_index_json: String,
    #[serde(default)]
    pub dashboard_history_json: String,
    #[serde(default)]
    pub release_review_json: String,
    pub summary_markdown: String,
    pub commands: Vec<String>,
    pub files: Vec<FridayDashboardExportFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardHistoryRecord {
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub loop_name: String,
    pub score_out_of_100: u8,
    pub readiness_score_out_of_100: u8,
    pub readiness_warning_count: usize,
    pub readiness_blocking_count: usize,
    pub screenshot_captured_count: usize,
    pub screenshot_missing_count: usize,
    pub export_dir: String,
    pub manifest_json: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardExportHistory {
    pub history_json: String,
    pub record_count: usize,
    pub score_delta_from_previous: i16,
    pub readiness_delta_from_previous: i16,
    pub latest: Option<FridayDashboardHistoryRecord>,
    pub previous: Option<FridayDashboardHistoryRecord>,
    pub records: Vec<FridayDashboardHistoryRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardReleaseReviewItem {
    pub id: String,
    pub title: String,
    pub ready: bool,
    pub detail: String,
    pub source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardReleaseReviewLink {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardReleaseReviewHandoff {
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub loop_name: String,
    pub score_out_of_100: u8,
    pub status: FridayDashboardPanelStatus,
    pub summary: String,
    pub ready_count: usize,
    pub total_count: usize,
    pub export_file_count: usize,
    pub visual_target_count: usize,
    pub screenshot_missing_count: usize,
    pub checklist: Vec<FridayDashboardReleaseReviewItem>,
    pub links: Vec<FridayDashboardReleaseReviewLink>,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardExportBundle {
    pub manifest: FridayDashboardExportManifest,
    pub readiness: FridayOperatorReadinessReport,
    pub route_bindings: FridayLiveUiRouteBindingReport,
    pub route_visuals: FridayRouteVisualReport,
    pub execution_handoffs: FridayExecutionHandoffReport,
    pub completion: CompletionSet,
    pub export_history: FridayDashboardExportHistory,
    pub release_review: FridayDashboardReleaseReviewHandoff,
}

impl FridayDashboardExportBundle {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayDashboardActionKind {
    Open,
    RunCheck,
    Recover,
    Capture,
}

impl FridayDashboardActionKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::RunCheck => "run-check",
            Self::Recover => "recover",
            Self::Capture => "capture",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardAction {
    pub id: String,
    pub label: String,
    pub command: String,
    pub kind: FridayDashboardActionKind,
    pub source: String,
    pub local_only: bool,
    pub enabled: bool,
    pub destructive: bool,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardCard {
    pub id: String,
    pub title: String,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub primary_metric: String,
    pub detail: String,
    pub source_json: String,
    pub actions: Vec<FridayDashboardAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardPanel {
    pub product_name: String,
    pub loop_name: String,
    pub score_out_of_100: u8,
    pub export_dir: String,
    pub summary: String,
    pub status: FridayDashboardPanelStatus,
    pub cards: Vec<FridayDashboardCard>,
    pub screenshot_history: FridayDashboardScreenshotHistory,
    pub warnings: Vec<String>,
    pub blocking_count: usize,
    pub source_files: Vec<FridayDashboardExportFile>,
    pub export_history: FridayDashboardExportHistory,
    pub release_review: FridayDashboardReleaseReviewHandoff,
}

impl FridayDashboardPanel {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn export_friday_dashboard_bundle(
    output_dir: impl AsRef<Path>,
) -> Result<FridayDashboardExportBundle> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "Could not create Friday dashboard export directory {}",
            output_dir.display()
        )
    })?;

    let export_dir = output_dir
        .canonicalize()
        .unwrap_or_else(|_| output_dir.to_path_buf());
    let readiness = friday_operator_readiness_report();
    let route_bindings = friday_live_ui_route_binding_report();
    let route_visuals = friday_route_visual_report();
    let screenshot_history = friday_dashboard_screenshot_history(&route_visuals);
    let execution_handoffs = friday_execution_handoff_report();
    let completion = active_completion_set();
    let generated_at_unix_ms = unix_ms();

    let readiness_path = export_dir.join("readiness.json");
    let route_bindings_path = export_dir.join("route-bindings.json");
    let route_visuals_path = export_dir.join("route-visuals.json");
    let execution_handoffs_path = export_dir.join("execution-handoffs.json");
    let completion_path = export_dir.join("completion.json");
    let dashboard_index_path = export_dir.join("dashboard-index.json");
    let dashboard_history_path = export_dir.join("dashboard-history.json");
    let release_review_path = export_dir.join("release-review.json");
    let summary_path = export_dir.join("summary.md");

    let mut files = vec![
        write_json_file(&readiness_path, "operator-readiness", &readiness)?,
        write_json_file(&route_bindings_path, "route-bindings", &route_bindings)?,
        write_json_file(&route_visuals_path, "route-visuals", &route_visuals)?,
        write_json_file(
            &execution_handoffs_path,
            "execution-handoffs",
            &execution_handoffs,
        )?,
        write_json_file(&completion_path, "completion-loop", &completion)?,
    ];

    let dashboard_index = json!({
        "product_name": "Friday",
        "loop_name": &completion.name,
        "score_out_of_100": completion.current_score_out_of_100,
        "target_score_out_of_100": completion.target_score_out_of_100,
        "readiness": {
            "score_out_of_100": readiness.score_out_of_100,
            "passed": readiness.passed_count,
            "warnings": readiness.warning_count,
            "blocking": readiness.blocking_count,
        },
        "route_bindings": {
            "score_out_of_100": route_bindings.score_out_of_100,
            "routes": route_bindings.route_count,
            "blocking": route_bindings.blocking_count,
        },
        "route_visuals": {
            "score_out_of_100": route_visuals.score_out_of_100,
            "targets": route_visuals.target_count,
            "artifact_root": &route_visuals.artifact_root,
            "blocking": route_visuals.blocking_count,
        },
        "screenshot_history": {
            "captured": screenshot_history.captured_count,
            "missing": screenshot_history.missing_count,
            "metadata_missing": screenshot_history.metadata_missing_count,
            "targets": screenshot_history.total_targets,
        },
        "export_history": {
            "path": path_string(&dashboard_history_path),
        },
        "execution_handoffs": {
            "score_out_of_100": execution_handoffs.score_out_of_100,
            "handoffs": execution_handoffs.handoff_count,
            "blocking": execution_handoffs.blocking_count,
        },
        "commands": dashboard_commands(),
    });
    files.push(write_json_file(
        &dashboard_index_path,
        "dashboard-index",
        &dashboard_index,
    )?);

    let summary_markdown = dashboard_summary_markdown(
        &completion,
        &readiness,
        &route_bindings,
        &route_visuals,
        &execution_handoffs,
    );
    fs::write(&summary_path, summary_markdown.as_bytes()).with_context(|| {
        format!(
            "Could not write Friday dashboard summary to {}",
            summary_path.display()
        )
    })?;
    files.push(file_record(&summary_path, "summary-markdown")?);

    let manifest_path = export_dir.join("manifest.json");
    let manifest = FridayDashboardExportManifest {
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        loop_name: completion.name.clone(),
        score_out_of_100: completion.current_score_out_of_100,
        export_dir: path_string(&export_dir),
        summary: format!(
            "Friday dashboard export ready at {} / {} with {} readiness warning(s) and {} blocking issue(s).",
            completion.current_score_out_of_100,
            completion.target_score_out_of_100,
            readiness.warning_count,
            readiness.blocking_count
        ),
        manifest_json: path_string(&manifest_path),
        readiness_json: path_string(&readiness_path),
        route_bindings_json: path_string(&route_bindings_path),
        route_visuals_json: path_string(&route_visuals_path),
        execution_handoffs_json: path_string(&execution_handoffs_path),
        completion_json: path_string(&completion_path),
        dashboard_index_json: path_string(&dashboard_index_path),
        dashboard_history_json: path_string(&dashboard_history_path),
        release_review_json: path_string(&release_review_path),
        summary_markdown: path_string(&summary_path),
        commands: dashboard_commands(),
        files,
    };

    let mut manifest = manifest;
    let export_history = append_dashboard_export_history(
        &dashboard_history_path,
        &manifest,
        &readiness,
        &screenshot_history,
    )?;
    manifest
        .files
        .push(file_record(&dashboard_history_path, "dashboard-history")?);
    let mut release_review = dashboard_release_review_handoff(
        &manifest,
        &completion,
        &route_visuals,
        &screenshot_history,
        &export_history,
    );
    manifest.files.push(write_json_file(
        &release_review_path,
        "release-review",
        &release_review,
    )?);
    release_review = dashboard_release_review_handoff(
        &manifest,
        &completion,
        &route_visuals,
        &screenshot_history,
        &export_history,
    );
    write_json_file(&release_review_path, "release-review", &release_review)?;

    write_json_file(&manifest_path, "manifest", &manifest)?;

    Ok(FridayDashboardExportBundle {
        manifest,
        readiness,
        route_bindings,
        route_visuals,
        execution_handoffs,
        completion,
        export_history,
        release_review,
    })
}

pub fn friday_dashboard_panel_from_export(
    export_dir: impl AsRef<Path>,
) -> Result<FridayDashboardPanel> {
    let export_dir = export_dir.as_ref();
    let manifest_path = export_dir.join("manifest.json");
    let manifest: FridayDashboardExportManifest = read_json_file(&manifest_path)?;
    let readiness: FridayOperatorReadinessReport = read_json_file(&manifest.readiness_json)?;
    let route_bindings: FridayLiveUiRouteBindingReport =
        read_json_file(&manifest.route_bindings_json)?;
    let route_visuals: FridayRouteVisualReport = read_json_file(&manifest.route_visuals_json)?;
    let execution_handoffs: FridayExecutionHandoffReport =
        read_json_file(&manifest.execution_handoffs_json)?;
    let completion: CompletionSet = read_json_file(&manifest.completion_json)?;
    let screenshot_history = friday_dashboard_screenshot_history(&route_visuals);
    let export_history =
        read_dashboard_export_history(&dashboard_history_path(&manifest, export_dir))?;
    let release_review = read_release_review_handoff(
        &release_review_path(&manifest, export_dir),
        &manifest,
        &completion,
        &route_visuals,
        &screenshot_history,
        &export_history,
    )?;

    let cards = vec![
        completion_card(&manifest, &completion),
        export_history_card(&manifest, &export_history),
        release_review_card(&manifest, &release_review),
        readiness_card(&manifest, &readiness),
        route_bindings_card(&manifest, &route_bindings),
        screenshot_history_card(&manifest, &screenshot_history),
        route_visuals_card(&manifest, &route_visuals),
        execution_handoffs_card(&manifest, &execution_handoffs),
    ];
    let blocking_count = cards
        .iter()
        .filter(|card| card.status == FridayDashboardPanelStatus::Blocked)
        .count();
    let warnings = cards
        .iter()
        .filter(|card| card.status == FridayDashboardPanelStatus::Warning)
        .map(|card| format!("{} needs attention: {}", card.title, card.detail))
        .collect::<Vec<_>>();
    let status = if blocking_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if !warnings.is_empty() {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };

    Ok(FridayDashboardPanel {
        product_name: manifest.product_name.clone(),
        loop_name: manifest.loop_name.clone(),
        score_out_of_100: manifest.score_out_of_100,
        export_dir: manifest.export_dir.clone(),
        summary: manifest.summary.clone(),
        status,
        cards,
        screenshot_history,
        warnings,
        blocking_count,
        source_files: manifest.files.clone(),
        export_history,
        release_review,
    })
}

pub fn friday_dashboard_export_history_from_export(
    export_dir: impl AsRef<Path>,
) -> Result<FridayDashboardExportHistory> {
    let export_dir = export_dir.as_ref();
    let manifest_path = export_dir.join("manifest.json");
    let manifest: FridayDashboardExportManifest = read_json_file(&manifest_path)?;
    read_dashboard_export_history(&dashboard_history_path(&manifest, export_dir))
}

pub fn friday_dashboard_release_review_from_export(
    export_dir: impl AsRef<Path>,
) -> Result<FridayDashboardReleaseReviewHandoff> {
    let export_dir = export_dir.as_ref();
    let manifest_path = export_dir.join("manifest.json");
    let manifest: FridayDashboardExportManifest = read_json_file(&manifest_path)?;
    let route_visuals: FridayRouteVisualReport = read_json_file(&manifest.route_visuals_json)?;
    let completion: CompletionSet = read_json_file(&manifest.completion_json)?;
    let screenshot_history = friday_dashboard_screenshot_history(&route_visuals);
    let export_history =
        read_dashboard_export_history(&dashboard_history_path(&manifest, export_dir))?;
    read_release_review_handoff(
        &release_review_path(&manifest, export_dir),
        &manifest,
        &completion,
        &route_visuals,
        &screenshot_history,
        &export_history,
    )
}

pub fn friday_dashboard_screenshot_history(
    report: &FridayRouteVisualReport,
) -> FridayDashboardScreenshotHistory {
    let records = report
        .targets
        .iter()
        .map(screenshot_record)
        .collect::<Vec<_>>();
    let captured_count = records
        .iter()
        .filter(|record| record.status == FridayDashboardScreenshotStatus::Captured)
        .count();
    let missing_count = records
        .iter()
        .filter(|record| record.status == FridayDashboardScreenshotStatus::Missing)
        .count();
    let metadata_missing_count = records
        .iter()
        .filter(|record| record.status == FridayDashboardScreenshotStatus::MetadataMissing)
        .count();

    FridayDashboardScreenshotHistory {
        total_targets: records.len(),
        captured_count,
        missing_count,
        metadata_missing_count,
        records,
    }
}

fn dashboard_summary_markdown(
    completion: &CompletionSet,
    readiness: &FridayOperatorReadinessReport,
    route_bindings: &FridayLiveUiRouteBindingReport,
    route_visuals: &FridayRouteVisualReport,
    execution_handoffs: &FridayExecutionHandoffReport,
) -> String {
    format!(
        r#"# Friday Dashboard Readiness

Status: {score} / {target}

Active loop: {loop_name}

## Readiness

- Operator readiness: {readiness_score} / 100
- Readiness areas: {readiness_passed} passed, {readiness_warnings} warning, {readiness_blocking} blocking
- Route bindings: {routes_passed}/{routes_total} passed
- Route screenshot targets: {visuals_passed}/{visuals_total} configured
- Execution handoffs: {handoffs_passed}/{handoffs_total} ready

## Commands

- flow --friday-dashboard-export tmp/friday-dashboard
- flow --friday-readiness
- flow --friday-live-ui-routes
- flow --friday-route-visuals
- flow --friday-execution-handoffs
- flow --completion

## Next

Open the next 100-point set after this export is consumed by Friday and DX dashboards.
"#,
        score = completion.current_score_out_of_100,
        target = completion.target_score_out_of_100,
        loop_name = &completion.name,
        readiness_score = readiness.score_out_of_100,
        readiness_passed = readiness.passed_count,
        readiness_warnings = readiness.warning_count,
        readiness_blocking = readiness.blocking_count,
        routes_passed = route_bindings.passed_count,
        routes_total = route_bindings.route_count,
        visuals_passed = route_visuals.passed_count,
        visuals_total = route_visuals.target_count,
        handoffs_passed = execution_handoffs.passed_count,
        handoffs_total = execution_handoffs.handoff_count,
    )
}

fn completion_card(
    manifest: &FridayDashboardExportManifest,
    completion: &CompletionSet,
) -> FridayDashboardCard {
    let remaining = completion
        .items
        .iter()
        .filter(|item| item.status != crate::competitive::CompletionItemStatus::Done)
        .count();
    let blocked = completion
        .items
        .iter()
        .filter(|item| item.status == crate::competitive::CompletionItemStatus::Blocked)
        .count();
    card(
        "completion-loop",
        "Completion Loop",
        status_from_score_and_blocking(completion.current_score_out_of_100, blocked),
        completion.current_score_out_of_100,
        format!(
            "{} / {}",
            completion.current_score_out_of_100, completion.target_score_out_of_100
        ),
        if remaining == 0 {
            "All items in this loop are done.".to_string()
        } else {
            format!("{remaining} item(s) remain before the loop is done.")
        },
        &manifest.completion_json,
        vec![action(
            "completion-loop",
            "open-completion",
            "Open completion",
            "flow --completion",
            FridayDashboardActionKind::Open,
        )],
    )
}

fn export_history_card(
    manifest: &FridayDashboardExportManifest,
    history: &FridayDashboardExportHistory,
) -> FridayDashboardCard {
    let has_comparison = history.previous.is_some();
    card(
        "export-history",
        "Export History",
        if has_comparison {
            FridayDashboardPanelStatus::Ready
        } else {
            FridayDashboardPanelStatus::Warning
        },
        percentage(history.record_count.min(2), 2),
        format!("{} checkpoint(s)", history.record_count),
        if has_comparison {
            format!(
                "Score delta {:+}; readiness delta {:+}.",
                history.score_delta_from_previous, history.readiness_delta_from_previous
            )
        } else {
            "One checkpoint is stored; export again to compare readiness deltas.".to_string()
        },
        &manifest.dashboard_history_json,
        vec![action(
            "export-history",
            "refresh-export-history",
            "Refresh history",
            &format!("flow --friday-dashboard-export {}", manifest.export_dir),
            FridayDashboardActionKind::RunCheck,
        )],
    )
}

fn release_review_card(
    manifest: &FridayDashboardExportManifest,
    handoff: &FridayDashboardReleaseReviewHandoff,
) -> FridayDashboardCard {
    card(
        "release-review",
        "Release Review",
        handoff.status,
        percentage(handoff.ready_count, handoff.total_count),
        format!(
            "{}/{} checks ready",
            handoff.ready_count, handoff.total_count
        ),
        handoff.summary.clone(),
        &manifest.release_review_json,
        vec![action(
            "release-review",
            "open-release-review",
            "Open release review",
            &format!("flow --friday-dashboard-panel {}", manifest.export_dir),
            FridayDashboardActionKind::Open,
        )],
    )
}

fn readiness_card(
    manifest: &FridayDashboardExportManifest,
    readiness: &FridayOperatorReadinessReport,
) -> FridayDashboardCard {
    card(
        "operator-readiness",
        "Operator Readiness",
        status_from_counts(readiness.warning_count, readiness.blocking_count),
        readiness.score_out_of_100,
        format!(
            "{} passed / {} warning / {} blocking",
            readiness.passed_count, readiness.warning_count, readiness.blocking_count
        ),
        readiness.summary.clone(),
        &manifest.readiness_json,
        readiness_actions(readiness),
    )
}

fn route_bindings_card(
    manifest: &FridayDashboardExportManifest,
    report: &FridayLiveUiRouteBindingReport,
) -> FridayDashboardCard {
    card(
        "route-bindings",
        "Route Bindings",
        status_from_counts(report.warning_count, report.blocking_count),
        report.score_out_of_100,
        format!(
            "{}/{} routes bound",
            report.passed_count, report.route_count
        ),
        report.summary.clone(),
        &manifest.route_bindings_json,
        vec![action(
            "route-bindings",
            "check-route-bindings",
            "Check routes",
            "flow --friday-live-ui-routes",
            FridayDashboardActionKind::RunCheck,
        )],
    )
}

fn screenshot_history_card(
    manifest: &FridayDashboardExportManifest,
    history: &FridayDashboardScreenshotHistory,
) -> FridayDashboardCard {
    let missing_or_incomplete = history.missing_count + history.metadata_missing_count;
    card(
        "screenshot-history",
        "Screenshot History",
        status_from_counts(missing_or_incomplete, 0),
        percentage(history.captured_count, history.total_targets),
        format!(
            "{}/{} captures present",
            history.captured_count, history.total_targets
        ),
        if missing_or_incomplete == 0 {
            "All configured route screenshot targets have captures and metadata.".to_string()
        } else {
            format!(
                "{} screenshot target(s) need capture or metadata before the visual review is complete.",
                missing_or_incomplete
            )
        },
        &manifest.route_visuals_json,
        vec![action(
            "screenshot-history",
            "capture-route-screenshots",
            "Capture screenshots",
            "flow --friday-route-visuals",
            FridayDashboardActionKind::Capture,
        )],
    )
}

fn route_visuals_card(
    manifest: &FridayDashboardExportManifest,
    report: &FridayRouteVisualReport,
) -> FridayDashboardCard {
    card(
        "route-visuals",
        "Route Visuals",
        status_from_counts(report.warning_count, report.blocking_count),
        report.score_out_of_100,
        format!(
            "{}/{} targets configured",
            report.passed_count, report.target_count
        ),
        format!("{} Artifact root: {}", report.summary, report.artifact_root),
        &manifest.route_visuals_json,
        vec![action(
            "route-visuals",
            "check-route-visuals",
            "Check visuals",
            "flow --friday-route-visuals",
            FridayDashboardActionKind::RunCheck,
        )],
    )
}

fn execution_handoffs_card(
    manifest: &FridayDashboardExportManifest,
    report: &FridayExecutionHandoffReport,
) -> FridayDashboardCard {
    card(
        "execution-handoffs",
        "Execution Handoffs",
        status_from_counts(report.warning_count, report.blocking_count),
        report.score_out_of_100,
        format!(
            "{}/{} handoffs ready",
            report.passed_count, report.handoff_count
        ),
        report.summary.clone(),
        &manifest.execution_handoffs_json,
        vec![action(
            "execution-handoffs",
            "check-execution-handoffs",
            "Check handoffs",
            "flow --friday-execution-handoffs",
            FridayDashboardActionKind::RunCheck,
        )],
    )
}

fn readiness_actions(readiness: &FridayOperatorReadinessReport) -> Vec<FridayDashboardAction> {
    let mut actions = vec![action(
        "operator-readiness",
        "run-readiness",
        "Run readiness",
        "flow --friday-readiness",
        FridayDashboardActionKind::RunCheck,
    )];

    for item in readiness
        .items
        .iter()
        .filter(|item| item.status != FridayOperatorReadinessStatus::Passed)
    {
        actions.push(action(
            "operator-readiness",
            &format!("recover-{}", item.id),
            &format!("Recover {}", item.title),
            &item.command,
            FridayDashboardActionKind::Recover,
        ));
    }

    actions
}

fn screenshot_record(target: &FridayRouteVisualTarget) -> FridayDashboardScreenshotRecord {
    let screenshot_path = Path::new(&target.screenshot_path);
    let metadata_path = Path::new(&target.metadata_path);
    let screenshot_metadata = screenshot_path.metadata().ok();
    let metadata_metadata = metadata_path.metadata().ok();
    let screenshot_present = screenshot_metadata
        .as_ref()
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false);
    let metadata_present = metadata_metadata
        .as_ref()
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false);
    let status = if !screenshot_present {
        FridayDashboardScreenshotStatus::Missing
    } else if !metadata_present {
        FridayDashboardScreenshotStatus::MetadataMissing
    } else {
        FridayDashboardScreenshotStatus::Captured
    };

    FridayDashboardScreenshotRecord {
        route: target.route.clone(),
        title: target.title.clone(),
        viewport_id: target.viewport.id.clone(),
        viewport_width: target.viewport.width,
        viewport_height: target.viewport.height,
        screenshot_path: target.screenshot_path.clone(),
        metadata_path: target.metadata_path.clone(),
        status,
        screenshot_bytes: screenshot_metadata
            .as_ref()
            .map(|metadata| metadata.len())
            .unwrap_or(0),
        metadata_bytes: metadata_metadata
            .as_ref()
            .map(|metadata| metadata.len())
            .unwrap_or(0),
        captured_at_unix_ms: screenshot_metadata
            .and_then(|metadata| metadata.modified().ok())
            .and_then(system_time_to_unix_ms),
        capture_command: target.capture_command.clone(),
        prompt: screenshot_prompt(target, status),
    }
}

fn screenshot_prompt(
    target: &FridayRouteVisualTarget,
    status: FridayDashboardScreenshotStatus,
) -> String {
    match status {
        FridayDashboardScreenshotStatus::Captured => format!(
            "Review the captured {} {} screenshot before release.",
            target.title, target.viewport.id
        ),
        FridayDashboardScreenshotStatus::MetadataMissing => format!(
            "Add metadata for the {} {} screenshot at {}.",
            target.title, target.viewport.id, target.metadata_path
        ),
        FridayDashboardScreenshotStatus::Missing => format!(
            "Capture the {} {} route screenshot with `{}`.",
            target.title, target.viewport.id, target.capture_command
        ),
    }
}

fn card(
    id: &str,
    title: &str,
    status: FridayDashboardPanelStatus,
    score_out_of_100: u8,
    primary_metric: String,
    detail: String,
    source_json: &str,
    actions: Vec<FridayDashboardAction>,
) -> FridayDashboardCard {
    FridayDashboardCard {
        id: id.to_string(),
        title: title.to_string(),
        status,
        score_out_of_100,
        primary_metric,
        detail,
        source_json: source_json.to_string(),
        actions,
    }
}

fn action(
    source: &str,
    id: &str,
    label: &str,
    command: &str,
    kind: FridayDashboardActionKind,
) -> FridayDashboardAction {
    FridayDashboardAction {
        id: id.to_string(),
        label: label.to_string(),
        command: command.to_string(),
        kind,
        source: source.to_string(),
        local_only: true,
        enabled: !command.trim().is_empty(),
        destructive: false,
        requires_confirmation: false,
    }
}

fn status_from_score_and_blocking(
    score_out_of_100: u8,
    blocking_count: usize,
) -> FridayDashboardPanelStatus {
    if blocking_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if score_out_of_100 < 100 {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    }
}

fn status_from_counts(warning_count: usize, blocking_count: usize) -> FridayDashboardPanelStatus {
    if blocking_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if warning_count > 0 {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    }
}

fn percentage(numerator: usize, denominator: usize) -> u8 {
    if denominator == 0 {
        return 0;
    }

    ((numerator as f32 / denominator as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn dashboard_commands() -> Vec<String> {
    vec![
        "flow --friday-dashboard-export tmp/friday-dashboard".to_string(),
        "flow --friday-dashboard-export-json tmp/friday-dashboard".to_string(),
        "flow --friday-readiness".to_string(),
        "flow --friday-live-ui-routes".to_string(),
        "flow --friday-route-visuals".to_string(),
        "flow --friday-execution-handoffs".to_string(),
        "flow --completion".to_string(),
    ]
}

fn append_dashboard_export_history(
    history_path: &Path,
    manifest: &FridayDashboardExportManifest,
    readiness: &FridayOperatorReadinessReport,
    screenshots: &FridayDashboardScreenshotHistory,
) -> Result<FridayDashboardExportHistory> {
    let mut records = if history_path.exists() {
        let history: FridayDashboardExportHistory = read_json_file(history_path)?;
        history.records
    } else {
        Vec::new()
    };
    records.push(dashboard_history_record(manifest, readiness, screenshots));

    let history = dashboard_export_history_from_records(history_path, records);
    write_json_file(history_path, "dashboard-history", &history)?;
    Ok(history)
}

fn read_dashboard_export_history(history_path: &Path) -> Result<FridayDashboardExportHistory> {
    if history_path.exists() {
        read_json_file(history_path)
    } else {
        Ok(dashboard_export_history_from_records(
            history_path,
            Vec::new(),
        ))
    }
}

fn read_release_review_handoff(
    release_review_path: &Path,
    manifest: &FridayDashboardExportManifest,
    completion: &CompletionSet,
    route_visuals: &FridayRouteVisualReport,
    screenshots: &FridayDashboardScreenshotHistory,
    export_history: &FridayDashboardExportHistory,
) -> Result<FridayDashboardReleaseReviewHandoff> {
    if release_review_path.exists() {
        read_json_file(release_review_path)
    } else {
        Ok(dashboard_release_review_handoff(
            manifest,
            completion,
            route_visuals,
            screenshots,
            export_history,
        ))
    }
}

fn dashboard_history_path(manifest: &FridayDashboardExportManifest, export_dir: &Path) -> PathBuf {
    if manifest.dashboard_history_json.trim().is_empty() {
        export_dir.join("dashboard-history.json")
    } else {
        PathBuf::from(&manifest.dashboard_history_json)
    }
}

fn release_review_path(manifest: &FridayDashboardExportManifest, export_dir: &Path) -> PathBuf {
    if manifest.release_review_json.trim().is_empty() {
        export_dir.join("release-review.json")
    } else {
        PathBuf::from(&manifest.release_review_json)
    }
}

fn dashboard_history_record(
    manifest: &FridayDashboardExportManifest,
    readiness: &FridayOperatorReadinessReport,
    screenshots: &FridayDashboardScreenshotHistory,
) -> FridayDashboardHistoryRecord {
    FridayDashboardHistoryRecord {
        generated_at_unix_ms: manifest.generated_at_unix_ms,
        product_name: manifest.product_name.clone(),
        loop_name: manifest.loop_name.clone(),
        score_out_of_100: manifest.score_out_of_100,
        readiness_score_out_of_100: readiness.score_out_of_100,
        readiness_warning_count: readiness.warning_count,
        readiness_blocking_count: readiness.blocking_count,
        screenshot_captured_count: screenshots.captured_count,
        screenshot_missing_count: screenshots.missing_count,
        export_dir: manifest.export_dir.clone(),
        manifest_json: manifest.manifest_json.clone(),
        summary: manifest.summary.clone(),
    }
}

fn dashboard_release_review_handoff(
    manifest: &FridayDashboardExportManifest,
    completion: &CompletionSet,
    route_visuals: &FridayRouteVisualReport,
    screenshots: &FridayDashboardScreenshotHistory,
    export_history: &FridayDashboardExportHistory,
) -> FridayDashboardReleaseReviewHandoff {
    let checklist = release_review_checklist(
        manifest,
        completion,
        route_visuals,
        screenshots,
        export_history,
    );
    let ready_count = checklist.iter().filter(|item| item.ready).count();
    let total_count = checklist.len();
    let status = if ready_count == total_count {
        FridayDashboardPanelStatus::Ready
    } else {
        FridayDashboardPanelStatus::Warning
    };

    FridayDashboardReleaseReviewHandoff {
        generated_at_unix_ms: manifest.generated_at_unix_ms,
        product_name: manifest.product_name.clone(),
        loop_name: manifest.loop_name.clone(),
        score_out_of_100: manifest.score_out_of_100,
        status,
        summary: format!(
            "Release review handoff links {} checklist item(s), {} export file(s), and {} visual target(s).",
            total_count,
            manifest.files.len(),
            route_visuals.target_count
        ),
        ready_count,
        total_count,
        export_file_count: manifest.files.len(),
        visual_target_count: route_visuals.target_count,
        screenshot_missing_count: screenshots.missing_count + screenshots.metadata_missing_count,
        checklist,
        links: release_review_links(manifest),
        commands: vec![
            format!("flow --friday-dashboard-panel {}", manifest.export_dir),
            format!("flow --friday-dashboard-export {}", manifest.export_dir),
            "flow --completion".to_string(),
            "flow --friday-route-visuals".to_string(),
        ],
    }
}

fn release_review_checklist(
    manifest: &FridayDashboardExportManifest,
    completion: &CompletionSet,
    route_visuals: &FridayRouteVisualReport,
    screenshots: &FridayDashboardScreenshotHistory,
    export_history: &FridayDashboardExportHistory,
) -> Vec<FridayDashboardReleaseReviewItem> {
    vec![
        release_review_item(
            "completion-loop",
            "Completion loop",
            completion.current_score_out_of_100 == completion.target_score_out_of_100,
            format!(
                "{} / {} complete",
                completion.current_score_out_of_100, completion.target_score_out_of_100
            ),
            &manifest.completion_json,
        ),
        release_review_item(
            "todo",
            "TODO checkpoint",
            Path::new("TODO.md").exists(),
            "TODO.md is available for the release narrative.".to_string(),
            "TODO.md",
        ),
        release_review_item(
            "changelog",
            "Changelog checkpoint",
            Path::new("CHANGELOG.md").exists(),
            "CHANGELOG.md is available for the release narrative.".to_string(),
            "CHANGELOG.md",
        ),
        release_review_item(
            "visual-targets",
            "Visual target contract",
            route_visuals.blocking_count == 0 && route_visuals.target_count > 0,
            format!("{} visual target(s) configured", route_visuals.target_count),
            &manifest.route_visuals_json,
        ),
        release_review_item(
            "export-files",
            "Export files",
            manifest.files.iter().all(|file| file.bytes > 0),
            format!("{} export file(s) recorded", manifest.files.len()),
            &manifest.manifest_json,
        ),
        release_review_item(
            "export-history",
            "Export history",
            export_history.record_count > 0,
            format!("{} checkpoint(s) recorded", export_history.record_count),
            &manifest.dashboard_history_json,
        ),
        release_review_item(
            "screenshot-review",
            "Screenshot review",
            screenshots.missing_count + screenshots.metadata_missing_count == 0,
            format!(
                "{} missing, {} missing metadata",
                screenshots.missing_count, screenshots.metadata_missing_count
            ),
            &manifest.route_visuals_json,
        ),
    ]
}

fn release_review_item(
    id: &str,
    title: &str,
    ready: bool,
    detail: String,
    source_path: &str,
) -> FridayDashboardReleaseReviewItem {
    FridayDashboardReleaseReviewItem {
        id: id.to_string(),
        title: title.to_string(),
        ready,
        detail,
        source_path: source_path.to_string(),
    }
}

fn release_review_links(
    manifest: &FridayDashboardExportManifest,
) -> Vec<FridayDashboardReleaseReviewLink> {
    vec![
        release_review_link("manifest", "Manifest", "json", &manifest.manifest_json),
        release_review_link("summary", "Summary", "markdown", &manifest.summary_markdown),
        release_review_link("todo", "TODO", "markdown", "TODO.md"),
        release_review_link("changelog", "Changelog", "markdown", "CHANGELOG.md"),
        release_review_link(
            "completion",
            "Completion",
            "json",
            &manifest.completion_json,
        ),
        release_review_link(
            "route-visuals",
            "Route visuals",
            "json",
            &manifest.route_visuals_json,
        ),
        release_review_link(
            "dashboard-history",
            "Dashboard history",
            "json",
            &manifest.dashboard_history_json,
        ),
    ]
}

fn release_review_link(
    id: &str,
    label: &str,
    kind: &str,
    path: &str,
) -> FridayDashboardReleaseReviewLink {
    FridayDashboardReleaseReviewLink {
        id: id.to_string(),
        label: label.to_string(),
        kind: kind.to_string(),
        path: path.to_string(),
    }
}

fn dashboard_export_history_from_records(
    history_path: &Path,
    mut records: Vec<FridayDashboardHistoryRecord>,
) -> FridayDashboardExportHistory {
    records.sort_by_key(|record| record.generated_at_unix_ms);
    if records.len() > 50 {
        let overflow = records.len() - 50;
        records.drain(0..overflow);
    }

    let latest = records.last().cloned();
    let previous = records.iter().rev().nth(1).cloned();
    let score_delta_from_previous = match (&latest, &previous) {
        (Some(latest), Some(previous)) => {
            latest.score_out_of_100 as i16 - previous.score_out_of_100 as i16
        }
        _ => 0,
    };
    let readiness_delta_from_previous = match (&latest, &previous) {
        (Some(latest), Some(previous)) => {
            latest.readiness_score_out_of_100 as i16 - previous.readiness_score_out_of_100 as i16
        }
        _ => 0,
    };

    FridayDashboardExportHistory {
        history_json: path_string(history_path),
        record_count: records.len(),
        score_delta_from_previous,
        readiness_delta_from_previous,
        latest,
        previous,
        records,
    }
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let bytes = fs::read(path)
        .with_context(|| format!("Could not read Friday dashboard JSON {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("Could not parse Friday dashboard JSON {}", path.display()))
}

fn write_json_file<T: ?Sized + Serialize>(
    path: &Path,
    kind: &str,
    value: &T,
) -> Result<FridayDashboardExportFile> {
    let json = serde_json::to_string_pretty(value)
        .with_context(|| format!("Could not serialize Friday dashboard {}", kind))?;
    fs::write(path, json.as_bytes()).with_context(|| {
        format!(
            "Could not write Friday dashboard {} file to {}",
            kind,
            path.display()
        )
    })?;
    file_record(path, kind)
}

fn file_record(path: &Path, kind: &str) -> Result<FridayDashboardExportFile> {
    let bytes = path
        .metadata()
        .with_context(|| format!("Could not read metadata for {}", path.display()))?
        .len();

    Ok(FridayDashboardExportFile {
        path: path_string(path),
        kind: kind.to_string(),
        bytes,
    })
}

fn path_string(path: &Path) -> String {
    let normalized = normalize_path(path).to_string_lossy().replace('\\', "/");
    normalized
        .strip_prefix("//?/")
        .unwrap_or(&normalized)
        .to_string()
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn system_time_to_unix_ms(time: SystemTime) -> Option<u128> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis())
}
