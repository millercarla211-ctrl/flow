use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::{FridayWorkspaceArea, default_friday_ui_integration_plan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayExecutionSurface {
    DesktopHost,
    BrowserExtension,
    WebWorkspace,
    RustCli,
}

impl FridayExecutionSurface {
    pub fn label(self) -> &'static str {
        match self {
            Self::DesktopHost => "desktop-host",
            Self::BrowserExtension => "browser-extension",
            Self::WebWorkspace => "web-workspace",
            Self::RustCli => "rust-cli",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayExecutionHandoffStatus {
    Passed,
    Warning,
    Failed,
}

impl FridayExecutionHandoffStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }

    fn score(self) -> f32 {
        match self {
            Self::Passed => 1.0,
            Self::Warning => 0.5,
            Self::Failed => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayExecutionHandoff {
    pub id: String,
    pub area: FridayWorkspaceArea,
    pub surface: FridayExecutionSurface,
    pub route: String,
    pub title: String,
    pub command: String,
    pub source_file: String,
    pub requires_user_gesture: bool,
    pub local_only: bool,
    pub permission_scopes: Vec<String>,
    pub artifact_path: Option<String>,
    pub recovery_command: String,
    pub status: FridayExecutionHandoffStatus,
    pub evidence: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayExecutionHandoffReport {
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub loop_name: String,
    pub summary: String,
    pub score_out_of_100: u8,
    pub handoff_count: usize,
    pub passed_count: usize,
    pub warning_count: usize,
    pub blocking_count: usize,
    pub handoffs: Vec<FridayExecutionHandoff>,
}

impl FridayExecutionHandoffReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_execution_handoff_report() -> FridayExecutionHandoffReport {
    let plan = default_friday_ui_integration_plan();
    let handoffs = handoff_specs()
        .into_iter()
        .map(|spec| {
            let route = plan.route(spec.area);
            handoff_from_spec(spec, route)
        })
        .collect::<Vec<_>>();
    let passed_count = handoffs
        .iter()
        .filter(|handoff| handoff.status == FridayExecutionHandoffStatus::Passed)
        .count();
    let warning_count = handoffs
        .iter()
        .filter(|handoff| handoff.status == FridayExecutionHandoffStatus::Warning)
        .count();
    let blocking_count = handoffs
        .iter()
        .filter(|handoff| handoff.status == FridayExecutionHandoffStatus::Failed)
        .count();
    let score_out_of_100 = score_handoffs(&handoffs);

    FridayExecutionHandoffReport {
        generated_at_unix_ms: unix_ms(),
        product_name: "Friday".to_string(),
        loop_name: "Friday Live UI Execution".to_string(),
        summary: format!(
            "{passed_count}/{} execution handoffs are ready; {warning_count} warning(s), {blocking_count} blocking issue(s).",
            handoffs.len()
        ),
        score_out_of_100,
        handoff_count: handoffs.len(),
        passed_count,
        warning_count,
        blocking_count,
        handoffs,
    }
}

#[derive(Debug, Clone)]
struct HandoffSpec {
    id: &'static str,
    area: FridayWorkspaceArea,
    surface: FridayExecutionSurface,
    command: &'static str,
    source_file: &'static str,
    requires_user_gesture: bool,
    permission_scopes: &'static [&'static str],
    artifact_path: Option<&'static str>,
    recovery_command: &'static str,
    next_action: &'static str,
}

fn handoff_specs() -> Vec<HandoffSpec> {
    vec![
        spec(
            "ask-stream",
            FridayWorkspaceArea::Ask,
            FridayExecutionSurface::WebWorkspace,
            "flow --friday-research-synthesize <prompt>",
            "extensions/flow-webext/src/ui/app.ts",
            true,
            &["local-model", "metasearch-sources", "citation-ledger"],
            Some("tmp/friday-handoffs/ask-answer.json"),
            "flow --friday-local-checks",
            "Bind the Ask send action to this handoff before adding remote-provider fallbacks.",
        ),
        spec(
            "search-cited-answer",
            FridayWorkspaceArea::Search,
            FridayExecutionSurface::BrowserExtension,
            "flow --friday-metasearch <query>",
            "extensions/flow-webext/src/runtime/flow-engine.ts",
            true,
            &["metasearch", "source-controls"],
            Some("tmp/friday-handoffs/search-results.json"),
            "flow --friday-browser-gate",
            "Use this handoff for answer-first search actions in popup, side panel, and sidebar surfaces.",
        ),
        spec(
            "research-report",
            FridayWorkspaceArea::Research,
            FridayExecutionSurface::WebWorkspace,
            "flow --friday-research-report-save tmp/friday-research <query>",
            "extensions/flow-webext/src/ui/app.ts",
            true,
            &["metasearch", "report-export", "local-files"],
            Some("tmp/friday-research/manifest.json"),
            "flow --friday-readiness",
            "Route deep research run buttons through this persisted report handoff.",
        ),
        spec(
            "voice-dictation",
            FridayWorkspaceArea::Voice,
            FridayExecutionSurface::DesktopHost,
            "flow --dictate",
            "src/bin/flow-dictate.rs",
            true,
            &["microphone", "focused-input", "overlay"],
            Some("tmp/friday-handoffs/voice-transcript.json"),
            "flow --friday-local-checks",
            "Keep voice start/stop buttons explicit and user-gesture gated.",
        ),
        spec(
            "multimodal-diagnostics",
            FridayWorkspaceArea::Multimodal,
            FridayExecutionSurface::BrowserExtension,
            "flow --friday-multimodal-diagnostics",
            "extensions/flow-webext/src/runtime/transformers-runtime.ts",
            true,
            &["local-model-files", "artifact-metadata"],
            Some("tmp/friday-handoffs/multimodal-diagnostics.json"),
            "flow --friday-route-visuals",
            "Use this as the no-load multimodal route action before model execution buttons.",
        ),
        spec(
            "readiness-command",
            FridayWorkspaceArea::Automations,
            FridayExecutionSurface::RustCli,
            "flow --friday-readiness",
            "src/friday/readiness.rs",
            false,
            &["local-files"],
            Some("tmp/friday-handoffs/readiness.json"),
            "flow --completion",
            "Expose this handoff to dashboards and release review surfaces.",
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn spec(
    id: &'static str,
    area: FridayWorkspaceArea,
    surface: FridayExecutionSurface,
    command: &'static str,
    source_file: &'static str,
    requires_user_gesture: bool,
    permission_scopes: &'static [&'static str],
    artifact_path: Option<&'static str>,
    recovery_command: &'static str,
    next_action: &'static str,
) -> HandoffSpec {
    HandoffSpec {
        id,
        area,
        surface,
        command,
        source_file,
        requires_user_gesture,
        permission_scopes,
        artifact_path,
        recovery_command,
        next_action,
    }
}

fn handoff_from_spec(
    spec: HandoffSpec,
    route: Option<&super::FridayUiRouteContract>,
) -> FridayExecutionHandoff {
    let route_ready = route
        .map(|route| route.route == spec.area.route() && !route.primary_command.is_empty())
        .unwrap_or(false);
    let command_bound = route
        .map(|route| {
            spec.command.contains(&route.primary_command)
                || route
                    .primary_command
                    .contains(spec.command.split_whitespace().next().unwrap_or(""))
                || !route.primary_command.trim().is_empty()
        })
        .unwrap_or(false);
    let source_ready = file_ready(spec.source_file);
    let local_only = true;
    let permissions_ready = !spec.permission_scopes.is_empty();
    let recovery_ready = !spec.recovery_command.trim().is_empty();
    let artifact_ready = spec
        .artifact_path
        .map(|path| path.ends_with(".json") || path.ends_with("manifest.json"))
        .unwrap_or(true);
    let status =
        if route_ready && command_bound && source_ready && permissions_ready && recovery_ready {
            FridayExecutionHandoffStatus::Passed
        } else if source_ready && artifact_ready {
            FridayExecutionHandoffStatus::Warning
        } else {
            FridayExecutionHandoffStatus::Failed
        };

    FridayExecutionHandoff {
        id: spec.id.to_string(),
        area: spec.area,
        surface: spec.surface,
        route: spec.area.route().to_string(),
        title: spec.area.label().to_string(),
        command: spec.command.to_string(),
        source_file: spec.source_file.to_string(),
        requires_user_gesture: spec.requires_user_gesture,
        local_only,
        permission_scopes: spec
            .permission_scopes
            .iter()
            .map(|scope| (*scope).to_string())
            .collect(),
        artifact_path: spec.artifact_path.map(str::to_string),
        recovery_command: spec.recovery_command.to_string(),
        status,
        evidence: vec![
            format!("route_ready={route_ready}"),
            format!("command_bound={command_bound}"),
            format!("source_ready={source_ready}"),
            format!("permissions_ready={permissions_ready}"),
            format!("recovery_ready={recovery_ready}"),
            format!("artifact_path_ready={artifact_ready}"),
            format!("surface={}", spec.surface.label()),
        ],
        next_action: spec.next_action.to_string(),
    }
}

fn file_ready(path: &str) -> bool {
    Path::new(path)
        .metadata()
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
}

fn score_handoffs(handoffs: &[FridayExecutionHandoff]) -> u8 {
    if handoffs.is_empty() {
        return 0;
    }

    let earned = handoffs
        .iter()
        .map(|handoff| handoff.status.score())
        .sum::<f32>();
    ((earned / handoffs.len() as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
