use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::{
    FridayUiIntegrationStatus, FridayUiVisualViewport, FridayWorkspaceArea,
    default_friday_ui_integration_plan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayRouteVisualStatus {
    Passed,
    Warning,
    Failed,
}

impl FridayRouteVisualStatus {
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
pub struct FridayRouteVisualTarget {
    pub area: FridayWorkspaceArea,
    pub route: String,
    pub title: String,
    pub viewport: FridayUiVisualViewport,
    pub screenshot_path: String,
    pub metadata_path: String,
    pub source_file: String,
    pub capture_command: String,
    pub status: FridayRouteVisualStatus,
    pub evidence: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayRouteVisualReport {
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub loop_name: String,
    pub summary: String,
    pub score_out_of_100: u8,
    pub target_count: usize,
    pub passed_count: usize,
    pub warning_count: usize,
    pub blocking_count: usize,
    pub artifact_root: String,
    pub targets: Vec<FridayRouteVisualTarget>,
}

impl FridayRouteVisualReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_route_visual_report() -> FridayRouteVisualReport {
    friday_route_visual_report_for_root("tmp/friday-route-screenshots")
}

pub fn friday_route_visual_report_for_root(
    artifact_root: impl AsRef<Path>,
) -> FridayRouteVisualReport {
    let artifact_root = artifact_root.as_ref();
    let plan = default_friday_ui_integration_plan();
    let mut targets = Vec::new();

    for area in visual_route_areas() {
        let route = plan.route(area);
        for viewport in visual_viewports() {
            targets.push(visual_target(artifact_root, area, viewport, route));
        }
    }

    let passed_count = targets
        .iter()
        .filter(|target| target.status == FridayRouteVisualStatus::Passed)
        .count();
    let warning_count = targets
        .iter()
        .filter(|target| target.status == FridayRouteVisualStatus::Warning)
        .count();
    let blocking_count = targets
        .iter()
        .filter(|target| target.status == FridayRouteVisualStatus::Failed)
        .count();
    let score_out_of_100 = score_targets(&targets);

    FridayRouteVisualReport {
        generated_at_unix_ms: unix_ms(),
        product_name: "Friday".to_string(),
        loop_name: "Friday Live UI Execution".to_string(),
        summary: format!(
            "{passed_count}/{} screenshot route targets are configured; {warning_count} warning(s), {blocking_count} blocking issue(s).",
            targets.len()
        ),
        score_out_of_100,
        target_count: targets.len(),
        passed_count,
        warning_count,
        blocking_count,
        artifact_root: artifact_root.to_string_lossy().into_owned(),
        targets,
    }
}

fn visual_route_areas() -> [FridayWorkspaceArea; 5] {
    [
        FridayWorkspaceArea::Ask,
        FridayWorkspaceArea::Search,
        FridayWorkspaceArea::Research,
        FridayWorkspaceArea::Voice,
        FridayWorkspaceArea::Multimodal,
    ]
}

fn visual_viewports() -> Vec<FridayUiVisualViewport> {
    vec![
        viewport(
            "desktop",
            1440,
            900,
            "sidebar + route content + action rail",
        ),
        viewport(
            "mobile",
            390,
            844,
            "single-column route with sticky action controls",
        ),
    ]
}

fn visual_target(
    artifact_root: &Path,
    area: FridayWorkspaceArea,
    viewport: FridayUiVisualViewport,
    route: Option<&super::FridayUiRouteContract>,
) -> FridayRouteVisualTarget {
    let slug = area.label().to_ascii_lowercase().replace(' ', "-");
    let screenshot_path = artifact_root
        .join(format!("{}-{}.png", slug, viewport.id))
        .to_string_lossy()
        .replace('\\', "/");
    let screenshot_present = artifact_root
        .join(format!("{}-{}.png", slug, viewport.id))
        .exists();
    let metadata_path = artifact_root
        .join(format!("{}-{}.json", slug, viewport.id))
        .to_string_lossy()
        .replace('\\', "/");
    let source_file = visual_source_file(area).to_string();
    let source_ready = file_ready(&source_file);
    let route_ready = route
        .map(|route| {
            route.status == FridayUiIntegrationStatus::Wired
                && route.route == area.route()
                && !route.states.is_empty()
                && !route.primary_command.trim().is_empty()
        })
        .unwrap_or(false);
    let paths_ready = screenshot_path.ends_with(".png") && metadata_path.ends_with(".json");
    let status = if route_ready && source_ready && paths_ready {
        FridayRouteVisualStatus::Passed
    } else if source_ready && paths_ready {
        FridayRouteVisualStatus::Warning
    } else {
        FridayRouteVisualStatus::Failed
    };

    FridayRouteVisualTarget {
        area,
        route: area.route().to_string(),
        title: area.label().to_string(),
        capture_command: format!(
            "agent-browser screenshot {} {}x{} {}",
            area.route(),
            viewport.width,
            viewport.height,
            screenshot_path
        ),
        viewport,
        screenshot_path,
        metadata_path,
        source_file,
        status,
        evidence: vec![
            format!("route_ready={route_ready}"),
            format!("source_ready={source_ready}"),
            format!("artifact_paths_ready={paths_ready}"),
            format!("screenshot_present={screenshot_present}"),
        ],
        next_action: if status == FridayRouteVisualStatus::Passed {
            "Capture the route screenshot into the configured artifact path after UI changes."
                .to_string()
        } else {
            "Fix the route contract or tracked UI source before running screenshot capture."
                .to_string()
        },
    }
}

fn viewport(id: &str, width: u16, height: u16, expected_layout: &str) -> FridayUiVisualViewport {
    FridayUiVisualViewport {
        id: id.to_string(),
        width,
        height,
        expected_layout: expected_layout.to_string(),
    }
}

fn visual_source_file(area: FridayWorkspaceArea) -> &'static str {
    match area {
        FridayWorkspaceArea::Voice => "extensions/flow-webext/src/content/index.ts",
        FridayWorkspaceArea::Multimodal => {
            "extensions/flow-webext/src/runtime/transformers-runtime.ts"
        }
        _ => "extensions/flow-webext/src/ui/app.ts",
    }
}

fn file_ready(path: &str) -> bool {
    Path::new(path)
        .metadata()
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
}

fn score_targets(targets: &[FridayRouteVisualTarget]) -> u8 {
    if targets.is_empty() {
        return 0;
    }

    let earned = targets
        .iter()
        .map(|target| target.status.score())
        .sum::<f32>();
    ((earned / targets.len() as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
