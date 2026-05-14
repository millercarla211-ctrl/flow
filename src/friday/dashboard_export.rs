use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    FridayExecutionHandoffReport, FridayLiveUiRouteBindingReport, FridayOperatorReadinessReport,
    FridayRouteVisualReport, friday_execution_handoff_report, friday_live_ui_route_binding_report,
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
    pub summary_markdown: String,
    pub commands: Vec<String>,
    pub files: Vec<FridayDashboardExportFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardExportBundle {
    pub manifest: FridayDashboardExportManifest,
    pub readiness: FridayOperatorReadinessReport,
    pub route_bindings: FridayLiveUiRouteBindingReport,
    pub route_visuals: FridayRouteVisualReport,
    pub execution_handoffs: FridayExecutionHandoffReport,
    pub completion: CompletionSet,
}

impl FridayDashboardExportBundle {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardAction {
    pub id: String,
    pub label: String,
    pub command: String,
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
    pub warnings: Vec<String>,
    pub blocking_count: usize,
    pub source_files: Vec<FridayDashboardExportFile>,
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
    let execution_handoffs = friday_execution_handoff_report();
    let completion = active_completion_set();

    let readiness_path = export_dir.join("readiness.json");
    let route_bindings_path = export_dir.join("route-bindings.json");
    let route_visuals_path = export_dir.join("route-visuals.json");
    let execution_handoffs_path = export_dir.join("execution-handoffs.json");
    let completion_path = export_dir.join("completion.json");
    let dashboard_index_path = export_dir.join("dashboard-index.json");
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
        generated_at_unix_ms: unix_ms(),
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
        summary_markdown: path_string(&summary_path),
        commands: dashboard_commands(),
        files,
    };

    write_json_file(&manifest_path, "manifest", &manifest)?;

    Ok(FridayDashboardExportBundle {
        manifest,
        readiness,
        route_bindings,
        route_visuals,
        execution_handoffs,
        completion,
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

    let cards = vec![
        completion_card(&manifest, &completion),
        readiness_card(&manifest, &readiness),
        route_bindings_card(&manifest, &route_bindings),
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
        warnings,
        blocking_count,
        source_files: manifest.files.clone(),
    })
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
            "open-completion",
            "Open completion",
            "flow --completion",
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
        vec![action(
            "run-readiness",
            "Run readiness",
            "flow --friday-readiness",
        )],
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
            "check-route-bindings",
            "Check routes",
            "flow --friday-live-ui-routes",
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
            "check-route-visuals",
            "Check visuals",
            "flow --friday-route-visuals",
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
            "check-execution-handoffs",
            "Check handoffs",
            "flow --friday-execution-handoffs",
        )],
    )
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

fn action(id: &str, label: &str, command: &str) -> FridayDashboardAction {
    FridayDashboardAction {
        id: id.to_string(),
        label: label.to_string(),
        command: command.to_string(),
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
