use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::{FridayWorkspaceArea, default_friday_ui_integration_plan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayLiveUiBindingStatus {
    Passed,
    Warning,
    Failed,
}

impl FridayLiveUiBindingStatus {
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
pub struct FridayLiveUiFileBinding {
    pub path: String,
    pub role: String,
    pub required: bool,
    pub exists: bool,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayLiveUiRouteBinding {
    pub area: FridayWorkspaceArea,
    pub route: String,
    pub title: String,
    pub status: FridayLiveUiBindingStatus,
    pub primary_command: String,
    pub source_files: Vec<FridayLiveUiFileBinding>,
    pub evidence: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayLiveUiRouteBindingReport {
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub loop_name: String,
    pub summary: String,
    pub score_out_of_100: u8,
    pub route_count: usize,
    pub passed_count: usize,
    pub warning_count: usize,
    pub blocking_count: usize,
    pub routes: Vec<FridayLiveUiRouteBinding>,
}

impl FridayLiveUiRouteBindingReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_live_ui_route_binding_report() -> FridayLiveUiRouteBindingReport {
    friday_live_ui_route_binding_report_for_root(".")
}

pub fn friday_live_ui_route_binding_report_for_root(
    root: impl AsRef<Path>,
) -> FridayLiveUiRouteBindingReport {
    let root = root.as_ref();
    let plan = default_friday_ui_integration_plan();
    let routes = plan
        .routes
        .iter()
        .map(|route| {
            let source_files = route_file_bindings(root, route.area);
            let missing_required = source_files
                .iter()
                .filter(|file| file.required && !file.exists)
                .count();
            let empty_required = source_files
                .iter()
                .filter(|file| file.required && file.exists && file.bytes == 0)
                .count();
            let status = if missing_required > 0 {
                FridayLiveUiBindingStatus::Failed
            } else if empty_required > 0 {
                FridayLiveUiBindingStatus::Warning
            } else {
                FridayLiveUiBindingStatus::Passed
            };

            FridayLiveUiRouteBinding {
                area: route.area,
                route: route.route.clone(),
                title: route.title.clone(),
                status,
                primary_command: route.primary_command.clone(),
                evidence: vec![
                    format!("source_files={}", source_files.len()),
                    format!("missing_required={missing_required}"),
                    format!("empty_required={empty_required}"),
                    format!("primary_command={}", route.primary_command),
                    format!("route_contract_status={}", route.status.label()),
                ],
                next_action: if status == FridayLiveUiBindingStatus::Passed {
                    "Add screenshot-backed checks for this route in the next live UI slice."
                        .to_string()
                } else {
                    "Create or repair the missing tracked route files before claiming live UI wiring."
                        .to_string()
                },
                source_files,
            }
        })
        .collect::<Vec<_>>();
    let passed_count = routes
        .iter()
        .filter(|route| route.status == FridayLiveUiBindingStatus::Passed)
        .count();
    let warning_count = routes
        .iter()
        .filter(|route| route.status == FridayLiveUiBindingStatus::Warning)
        .count();
    let blocking_count = routes
        .iter()
        .filter(|route| route.status == FridayLiveUiBindingStatus::Failed)
        .count();
    let score_out_of_100 = score_routes(&routes);

    FridayLiveUiRouteBindingReport {
        generated_at_unix_ms: unix_ms(),
        product_name: "Friday".to_string(),
        loop_name: "Friday Live UI Execution".to_string(),
        summary: format!(
            "{passed_count}/{} Friday routes are bound to tracked UI/runtime files; {warning_count} warning(s), {blocking_count} blocking issue(s).",
            routes.len()
        ),
        score_out_of_100,
        route_count: routes.len(),
        passed_count,
        warning_count,
        blocking_count,
        routes,
    }
}

fn route_file_bindings(root: &Path, area: FridayWorkspaceArea) -> Vec<FridayLiveUiFileBinding> {
    let mut files = vec![
        file(root, "src/friday/ui.rs", "friday-route-contracts", true),
        file(
            root,
            "extensions/flow-webext/src/ui/app.ts",
            "shared-extension-workspace-shell",
            true,
        ),
        file(
            root,
            "extensions/flow-webext/src/runtime/protocol.ts",
            "browser-runtime-protocol",
            true,
        ),
    ];

    for (path, role) in route_specific_files(area) {
        files.push(file(root, path, role, true));
    }

    files
}

fn route_specific_files(area: FridayWorkspaceArea) -> &'static [(&'static str, &'static str)] {
    match area {
        FridayWorkspaceArea::Ask => &[
            ("src/friday/research.rs", "ask-research-synthesis-contract"),
            ("src/runtime/local.rs", "local-model-runtime"),
            (
                "extensions/flow-webext/src/runtime/flow-engine.ts",
                "browser-flow-engine",
            ),
        ],
        FridayWorkspaceArea::Search => &[
            ("src/friday/research.rs", "search-plan-contract"),
            ("src/search/metasearch_api.rs", "metasearch-api-client"),
        ],
        FridayWorkspaceArea::Research => &[
            ("src/friday/research.rs", "research-workflow-contract"),
            ("src/search/plan.rs", "metasearch-plan-source"),
        ],
        FridayWorkspaceArea::Agents => &[
            ("src/codex/adapter.rs", "agent-adapter-contract"),
            ("src/cli/commands.rs", "tool-agent-command-surface"),
        ],
        FridayWorkspaceArea::Canvas => &[
            ("src/friday/artifacts.rs", "canvas-artifact-store"),
            (
                "extensions/flow-webext/src/ui/options.ts",
                "settings-route-entry",
            ),
        ],
        FridayWorkspaceArea::Projects
        | FridayWorkspaceArea::Memory
        | FridayWorkspaceArea::Connectors => &[
            ("src/friday/workspace.rs", "workspace-store-contract"),
            ("src/friday/ui.rs", "workspace-route-contract"),
        ],
        FridayWorkspaceArea::Voice => &[
            ("src/friday/runtime.rs", "voice-runtime-store"),
            ("src/bin/flow-dictate.rs", "dictation-host-entry"),
            (
                "extensions/flow-webext/src/content/index.ts",
                "content-overlay-entry",
            ),
        ],
        FridayWorkspaceArea::Artifacts => &[
            ("src/friday/artifacts.rs", "artifact-store-contract"),
            (
                "extensions/flow-webext/src/ui/sidepanel.ts",
                "sidepanel-route-entry",
            ),
        ],
        FridayWorkspaceArea::Automations => &[
            ("src/friday/runtime.rs", "automation-runtime-store"),
            ("src/competitive/progress.rs", "completion-loop-source"),
        ],
        FridayWorkspaceArea::Code => &[
            ("src/friday/artifacts.rs", "code-artifact-store"),
            ("src/codex/adapter.rs", "code-agent-adapter"),
        ],
        FridayWorkspaceArea::Multimodal => &[
            ("src/friday/multimodal.rs", "multimodal-runtime-contract"),
            (
                "extensions/flow-webext/src/runtime/transformers-runtime.ts",
                "browser-model-runtime",
            ),
        ],
    }
}

fn file(root: &Path, path: &str, role: &str, required: bool) -> FridayLiveUiFileBinding {
    let full_path = normalize(root, path);
    let metadata = full_path.metadata().ok();

    FridayLiveUiFileBinding {
        path: path.to_string(),
        role: role.to_string(),
        required,
        exists: metadata.is_some(),
        bytes: metadata.map(|metadata| metadata.len()).unwrap_or(0),
    }
}

fn normalize(root: &Path, path: &str) -> PathBuf {
    let relative = path.replace('/', std::path::MAIN_SEPARATOR_STR);
    root.join(relative)
}

fn score_routes(routes: &[FridayLiveUiRouteBinding]) -> u8 {
    if routes.is_empty() {
        return 0;
    }

    let earned = routes.iter().map(|route| route.status.score()).sum::<f32>();
    ((earned / routes.len() as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
