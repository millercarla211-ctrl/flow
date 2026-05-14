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
