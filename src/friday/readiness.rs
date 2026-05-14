use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::{
    FridayLocalCheckStatus, FridayMediaAffordanceStatus, FridayUiVisualCheckStatus,
    default_friday_browser_verification_report, default_friday_local_execution_checks,
    friday_execution_handoff_report, friday_live_ui_route_binding_report, friday_media_affordances,
    friday_multimodal_visual_check, friday_route_visual_report,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayOperatorReadinessStatus {
    Passed,
    Warning,
    Failed,
}

impl FridayOperatorReadinessStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }

    fn score_multiplier(self) -> f32 {
        match self {
            Self::Passed => 1.0,
            Self::Warning => 0.5,
            Self::Failed => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayOperatorReadinessItem {
    pub id: String,
    pub title: String,
    pub status: FridayOperatorReadinessStatus,
    pub score_out_of_100: u8,
    pub local_only: bool,
    pub command: String,
    pub evidence: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayOperatorReadinessReport {
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub loop_name: String,
    pub summary: String,
    pub score_out_of_100: u8,
    pub passed_count: usize,
    pub warning_count: usize,
    pub blocking_count: usize,
    pub items: Vec<FridayOperatorReadinessItem>,
}

impl FridayOperatorReadinessReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_operator_readiness_report() -> FridayOperatorReadinessReport {
    let items = vec![
        route_binding_item(),
        route_visual_item(),
        execution_handoff_item(),
        local_execution_item(),
        browser_gate_item(),
        desktop_host_item(),
        multimodal_visual_item(),
        media_affordance_item(),
        release_control_item(),
    ];
    let passed_count = items
        .iter()
        .filter(|item| item.status == FridayOperatorReadinessStatus::Passed)
        .count();
    let warning_count = items
        .iter()
        .filter(|item| item.status == FridayOperatorReadinessStatus::Warning)
        .count();
    let blocking_count = items
        .iter()
        .filter(|item| item.status == FridayOperatorReadinessStatus::Failed)
        .count();
    let score_out_of_100 = score_items(&items);

    FridayOperatorReadinessReport {
        generated_at_unix_ms: unix_ms(),
        product_name: "Friday".to_string(),
        loop_name: "Friday Live UI Execution".to_string(),
        summary: format!(
            "{passed_count}/{} readiness areas passed; {warning_count} warning(s), {blocking_count} blocking issue(s).",
            items.len()
        ),
        score_out_of_100,
        passed_count,
        warning_count,
        blocking_count,
        items,
    }
}

fn route_binding_item() -> FridayOperatorReadinessItem {
    let report = friday_live_ui_route_binding_report();
    item(
        "route-bindings",
        "Tracked route file bindings",
        if report.blocking_count == 0 {
            FridayOperatorReadinessStatus::Passed
        } else {
            FridayOperatorReadinessStatus::Failed
        },
        report.score_out_of_100,
        true,
        "flow --friday-live-ui-routes",
        vec![
            format!("routes={}", report.route_count),
            format!("passed={}", report.passed_count),
            format!("warnings={}", report.warning_count),
            format!("blocking={}", report.blocking_count),
        ],
        "Keep this command green whenever adding or moving Friday UI routes.",
    )
}

fn route_visual_item() -> FridayOperatorReadinessItem {
    let report = friday_route_visual_report();
    let status = if report.blocking_count > 0 {
        FridayOperatorReadinessStatus::Failed
    } else if report.warning_count > 0 {
        FridayOperatorReadinessStatus::Warning
    } else {
        FridayOperatorReadinessStatus::Passed
    };

    item(
        "route-visuals",
        "Route screenshot targets",
        status,
        report.score_out_of_100,
        true,
        "flow --friday-route-visuals",
        vec![
            format!("targets={}", report.target_count),
            format!("passed={}", report.passed_count),
            format!("warnings={}", report.warning_count),
            format!("blocking={}", report.blocking_count),
            format!("artifact_root={}", report.artifact_root),
        ],
        "Capture screenshots into the configured artifact paths after meaningful route UI edits.",
    )
}

fn execution_handoff_item() -> FridayOperatorReadinessItem {
    let report = friday_execution_handoff_report();
    let status = if report.blocking_count > 0 {
        FridayOperatorReadinessStatus::Failed
    } else if report.warning_count > 0 {
        FridayOperatorReadinessStatus::Warning
    } else {
        FridayOperatorReadinessStatus::Passed
    };

    item(
        "execution-handoffs",
        "Desktop/web execution handoffs",
        status,
        report.score_out_of_100,
        true,
        "flow --friday-execution-handoffs",
        vec![
            format!("handoffs={}", report.handoff_count),
            format!("passed={}", report.passed_count),
            format!("warnings={}", report.warning_count),
            format!("blocking={}", report.blocking_count),
        ],
        "Keep UI actions bound to explicit local commands, permissions, artifacts, and recovery commands.",
    )
}

fn local_execution_item() -> FridayOperatorReadinessItem {
    let report = default_friday_local_execution_checks();
    let score = local_execution_score(&report.checks);
    let status = if report.blocking_count() > 0 {
        FridayOperatorReadinessStatus::Failed
    } else if report.warning_count() > 0 {
        FridayOperatorReadinessStatus::Warning
    } else {
        FridayOperatorReadinessStatus::Passed
    };

    item(
        "local-execution",
        "Local model and runtime readiness",
        status,
        score,
        true,
        "flow --friday-local-checks",
        vec![
            format!("checks={}", report.checks.len()),
            format!("passed={}", report.passed_count()),
            format!("warnings={}", report.warning_count()),
            format!("blocking={}", report.blocking_count()),
        ],
        "Install or repair any missing local STT/TTS/OCR/runtime artifact before a release.",
    )
}

fn browser_gate_item() -> FridayOperatorReadinessItem {
    let report = default_friday_browser_verification_report();
    let status = if report.blocking_count() > 0 {
        FridayOperatorReadinessStatus::Failed
    } else if !report.deploy_gate.deployment_allowed {
        FridayOperatorReadinessStatus::Warning
    } else {
        FridayOperatorReadinessStatus::Passed
    };
    let score = percentage(report.passed_target_count(), report.targets.len());

    item(
        "browser-gate",
        "Browser extension and deploy gate",
        status,
        score,
        true,
        "flow --friday-browser-gate",
        vec![
            format!("targets={}", report.targets.len()),
            format!("passed={}", report.passed_target_count()),
            format!("blocking={}", report.blocking_count()),
            format!(
                "deployment_allowed={}",
                report.deploy_gate.deployment_allowed
            ),
        ],
        "Run the browser gate before packaging or deploying a major user-visible change.",
    )
}

fn desktop_host_item() -> FridayOperatorReadinessItem {
    let files = [
        "src/bin/flow-dictate.rs",
        "src/cli/commands.rs",
        "extensions/flow-webext/src/content/index.ts",
    ];
    let present = files.iter().filter(|path| Path::new(path).exists()).count();
    let status = if present == files.len() {
        FridayOperatorReadinessStatus::Passed
    } else {
        FridayOperatorReadinessStatus::Failed
    };

    item(
        "desktop-host",
        "Desktop voice host and overlay entries",
        status,
        percentage(present, files.len()),
        true,
        "flow --dictate",
        files
            .iter()
            .map(|path| format!("{path}={}", present_label(Path::new(path).exists())))
            .collect(),
        "Keep dictation and overlay entry points tracked before UI execution changes.",
    )
}

fn multimodal_visual_item() -> FridayOperatorReadinessItem {
    let report = friday_multimodal_visual_check();
    let status = match report.status {
        FridayUiVisualCheckStatus::Passed => FridayOperatorReadinessStatus::Passed,
        FridayUiVisualCheckStatus::Warning => FridayOperatorReadinessStatus::Warning,
        FridayUiVisualCheckStatus::Failed => FridayOperatorReadinessStatus::Failed,
    };

    item(
        "multimodal-visual",
        "Multimodal route visual contract",
        status,
        report.score_out_of_100,
        true,
        "flow --friday-multimodal-visual-check",
        vec![
            format!("route={}", report.route),
            format!("surface={}", report.target_surface),
            format!("requirements={}", report.requirements.len()),
            format!("blocking={}", report.blocking_count()),
        ],
        "Extend this from contract checks to screenshot captures for the most-used routes.",
    )
}

fn media_affordance_item() -> FridayOperatorReadinessItem {
    let affordances = friday_media_affordances();
    let ready = affordances
        .iter()
        .filter(|item| item.status == FridayMediaAffordanceStatus::Ready)
        .count();
    let needs_installer = affordances
        .iter()
        .filter(|item| item.status == FridayMediaAffordanceStatus::NeedsInstaller)
        .count();
    let planned = affordances
        .iter()
        .filter(|item| item.status == FridayMediaAffordanceStatus::Planned)
        .count();
    let status = if needs_installer > 0 || planned > 0 {
        FridayOperatorReadinessStatus::Warning
    } else {
        FridayOperatorReadinessStatus::Passed
    };

    item(
        "media-affordances",
        "Image and video local model actions",
        status,
        percentage(ready, affordances.len()),
        true,
        "flow --friday-media-affordances",
        vec![
            format!("actions={}", affordances.len()),
            format!("ready={ready}"),
            format!("needs_installer={needs_installer}"),
            format!("planned={planned}"),
        ],
        "Keep optional image/video actions visible but non-blocking until local installers are added.",
    )
}

fn release_control_item() -> FridayOperatorReadinessItem {
    let files = ["TODO.md", "CHANGELOG.md", "src/competitive/progress.rs"];
    let present = files.iter().filter(|path| Path::new(path).exists()).count();
    let status = if present == files.len() {
        FridayOperatorReadinessStatus::Passed
    } else {
        FridayOperatorReadinessStatus::Failed
    };

    item(
        "release-control",
        "Release loop and handoff records",
        status,
        percentage(present, files.len()),
        true,
        "flow --completion",
        files
            .iter()
            .map(|path| format!("{path}={}", present_label(Path::new(path).exists())))
            .collect(),
        "Update TODO, changelog, and active completion data at each coherent checkpoint.",
    )
}

#[allow(clippy::too_many_arguments)]
fn item(
    id: &str,
    title: &str,
    status: FridayOperatorReadinessStatus,
    score_out_of_100: u8,
    local_only: bool,
    command: &str,
    evidence: Vec<String>,
    next_action: &str,
) -> FridayOperatorReadinessItem {
    FridayOperatorReadinessItem {
        id: id.to_string(),
        title: title.to_string(),
        status,
        score_out_of_100,
        local_only,
        command: command.to_string(),
        evidence,
        next_action: next_action.to_string(),
    }
}

fn local_execution_score(checks: &[super::FridayLocalExecutionCheck]) -> u8 {
    if checks.is_empty() {
        return 0;
    }

    let earned = checks
        .iter()
        .map(|check| match check.status {
            FridayLocalCheckStatus::Passed => 1.0,
            FridayLocalCheckStatus::Warning | FridayLocalCheckStatus::Skipped => 0.5,
            FridayLocalCheckStatus::Failed => 0.0,
        })
        .sum::<f32>();

    ((earned / checks.len() as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn score_items(items: &[FridayOperatorReadinessItem]) -> u8 {
    if items.is_empty() {
        return 0;
    }

    let earned = items
        .iter()
        .map(|item| item.status.score_multiplier() * item.score_out_of_100 as f32)
        .sum::<f32>();
    (earned / items.len() as f32).round().clamp(0.0, 100.0) as u8
}

fn percentage(numerator: usize, denominator: usize) -> u8 {
    if denominator == 0 {
        return 0;
    }

    ((numerator as f32 / denominator as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn present_label(present: bool) -> &'static str {
    if present { "present" } else { "missing" }
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
