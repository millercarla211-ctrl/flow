use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardActionKind, FridayDashboardPanelStatus, FridayDashboardScreenshotStatus,
    FridayUiDataBinding, friday_dashboard_panel_from_export,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardProductUiCardBinding {
    pub card_id: String,
    pub title: String,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub primary_metric: String,
    pub source_json: String,
    pub action_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardProductUiActionBinding {
    pub card_id: String,
    pub action_id: String,
    pub label: String,
    pub kind: FridayDashboardActionKind,
    pub command: String,
    pub local_only: bool,
    pub enabled: bool,
    pub button_state: FridayDashboardProductUiButtonState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardProductUiButtonState {
    pub disabled: bool,
    pub disabled_reason: Option<String>,
    pub idle_label: String,
    pub loading_label: String,
    pub success_label: String,
    pub error_label: String,
    pub aria_label: String,
    pub destructive: bool,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardProductUiHistoryBinding {
    pub record_count: usize,
    pub score_delta_from_previous: i16,
    pub readiness_delta_from_previous: i16,
    pub latest_score_out_of_100: Option<u8>,
    pub previous_score_out_of_100: Option<u8>,
    pub trend_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardProductUiScreenshotPrompt {
    pub route: String,
    pub title: String,
    pub viewport_id: String,
    pub status: FridayDashboardScreenshotStatus,
    pub prompt: String,
    pub capture_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardProductUiReleaseLink {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub path: String,
    pub section: String,
    pub local_only: bool,
    pub button_state: FridayDashboardProductUiButtonState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardProductUiBinding {
    pub product_name: String,
    pub route: String,
    pub title: String,
    pub source_file: String,
    pub export_dir: String,
    pub status: FridayDashboardPanelStatus,
    pub score_out_of_100: u8,
    pub summary: String,
    pub panel_json_command: String,
    pub export_command: String,
    pub card_count: usize,
    pub bound_card_count: usize,
    pub action_count: usize,
    pub warning_count: usize,
    pub blocking_count: usize,
    pub data_bindings: Vec<FridayUiDataBinding>,
    pub cards: Vec<FridayDashboardProductUiCardBinding>,
    pub action_bindings: Vec<FridayDashboardProductUiActionBinding>,
    pub history: FridayDashboardProductUiHistoryBinding,
    pub screenshot_prompts: Vec<FridayDashboardProductUiScreenshotPrompt>,
    pub release_links: Vec<FridayDashboardProductUiReleaseLink>,
    pub next_actions: Vec<String>,
}

impl FridayDashboardProductUiBinding {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_dashboard_product_ui_binding_from_export(
    export_dir: impl AsRef<Path>,
) -> Result<FridayDashboardProductUiBinding> {
    let panel = friday_dashboard_panel_from_export(export_dir.as_ref())?;
    let panel_json_command = format!("flow --friday-dashboard-panel-json {}", panel.export_dir);
    let export_command = format!("flow --friday-dashboard-export {}", panel.export_dir);
    let cards = panel
        .cards
        .iter()
        .map(|card| FridayDashboardProductUiCardBinding {
            card_id: card.id.clone(),
            title: card.title.clone(),
            status: card.status,
            score_out_of_100: card.score_out_of_100,
            primary_metric: card.primary_metric.clone(),
            source_json: card.source_json.clone(),
            action_count: card.actions.len(),
        })
        .collect::<Vec<_>>();
    let action_bindings = panel
        .cards
        .iter()
        .flat_map(|card| {
            card.actions
                .iter()
                .map(|action| FridayDashboardProductUiActionBinding {
                    card_id: card.id.clone(),
                    action_id: action.id.clone(),
                    label: action.label.clone(),
                    kind: action.kind,
                    command: action.command.clone(),
                    local_only: action.local_only,
                    enabled: action.enabled,
                    button_state: action_button_state(
                        action.kind,
                        &action.label,
                        action.enabled,
                        action.destructive,
                        action.requires_confirmation,
                    ),
                })
        })
        .collect::<Vec<_>>();
    let warning_count = panel
        .cards
        .iter()
        .filter(|card| card.status == FridayDashboardPanelStatus::Warning)
        .count();
    let release_review_json = panel
        .cards
        .iter()
        .find(|card| card.id == "release-review")
        .map(|card| card.source_json.as_str())
        .unwrap_or("");
    let release_links = panel
        .release_review
        .links
        .iter()
        .map(release_link_binding)
        .collect::<Vec<_>>();

    Ok(FridayDashboardProductUiBinding {
        product_name: panel.product_name.clone(),
        route: "/dashboard".to_string(),
        title: "Friday Dashboard".to_string(),
        source_file: "extensions/flow-webext/src/ui/app.ts".to_string(),
        export_dir: panel.export_dir.clone(),
        status: panel.status,
        score_out_of_100: panel.score_out_of_100,
        summary: format!(
            "Bind {} dashboard card(s), {} action(s), export history, and release review state into the visible Friday dashboard.",
            panel.cards.len(),
            action_bindings.len()
        ),
        panel_json_command: panel_json_command.clone(),
        export_command: export_command.clone(),
        card_count: panel.cards.len(),
        bound_card_count: cards
            .iter()
            .filter(|card| !card.source_json.trim().is_empty())
            .count(),
        action_count: action_bindings.len(),
        warning_count,
        blocking_count: panel.blocking_count,
        history: dashboard_history_binding(&panel.export_history),
        screenshot_prompts: panel
            .screenshot_history
            .records
            .iter()
            .filter(|record| record.status != FridayDashboardScreenshotStatus::Captured)
            .map(|record| FridayDashboardProductUiScreenshotPrompt {
                route: record.route.clone(),
                title: record.title.clone(),
                viewport_id: record.viewport_id.clone(),
                status: record.status,
                prompt: record.prompt.clone(),
                capture_command: record.capture_command.clone(),
            })
            .collect(),
        release_links,
        data_bindings: dashboard_data_bindings(
            &panel_json_command,
            &export_command,
            &panel.export_history.history_json,
            release_review_json,
        ),
        cards,
        action_bindings,
        next_actions: vec![
            "Render these cards from the dashboard panel JSON instead of hard-coded product copy."
                .to_string(),
            "Connect enabled local-only actions to explicit UI buttons with loading and error states."
                .to_string(),
            "Show dashboard history deltas and release-review links in the visible dashboard."
                .to_string(),
            "Render release-review links as local artifact buttons grouped by release notes, visual review, and export artifacts."
                .to_string(),
        ],
    })
}

fn dashboard_history_binding(
    history: &super::FridayDashboardExportHistory,
) -> FridayDashboardProductUiHistoryBinding {
    let latest_score_out_of_100 = history
        .latest
        .as_ref()
        .map(|record| record.score_out_of_100);
    let previous_score_out_of_100 = history
        .previous
        .as_ref()
        .map(|record| record.score_out_of_100);

    FridayDashboardProductUiHistoryBinding {
        record_count: history.record_count,
        score_delta_from_previous: history.score_delta_from_previous,
        readiness_delta_from_previous: history.readiness_delta_from_previous,
        latest_score_out_of_100,
        previous_score_out_of_100,
        trend_label: if history.record_count < 2 {
            "not-enough-history".to_string()
        } else if history.score_delta_from_previous > 0 {
            "improving".to_string()
        } else if history.score_delta_from_previous < 0 {
            "regressed".to_string()
        } else {
            "steady".to_string()
        },
    }
}

fn release_link_binding(
    link: &super::FridayDashboardReleaseReviewLink,
) -> FridayDashboardProductUiReleaseLink {
    let label = format!("Open {}", link.label);
    FridayDashboardProductUiReleaseLink {
        id: link.id.clone(),
        label: link.label.clone(),
        kind: link.kind.clone(),
        path: link.path.clone(),
        section: release_link_section(&link.id).to_string(),
        local_only: true,
        button_state: action_button_state(
            FridayDashboardActionKind::Open,
            &label,
            !link.path.trim().is_empty(),
            false,
            false,
        ),
    }
}

fn release_link_section(id: &str) -> &'static str {
    match id {
        "todo" | "changelog" | "summary" => "release-notes",
        "route-visuals" => "visual-review",
        "manifest" | "completion" | "dashboard-history" => "export-artifacts",
        _ => "other",
    }
}

fn action_button_state(
    kind: FridayDashboardActionKind,
    label: &str,
    enabled: bool,
    destructive: bool,
    requires_confirmation: bool,
) -> FridayDashboardProductUiButtonState {
    let verb = match kind {
        FridayDashboardActionKind::Open => "Open",
        FridayDashboardActionKind::RunCheck => "Run",
        FridayDashboardActionKind::Recover => "Recover",
        FridayDashboardActionKind::Capture => "Capture",
    };
    let loading_label = match kind {
        FridayDashboardActionKind::Open => "Opening...".to_string(),
        FridayDashboardActionKind::RunCheck => "Running...".to_string(),
        FridayDashboardActionKind::Recover => "Recovering...".to_string(),
        FridayDashboardActionKind::Capture => "Capturing...".to_string(),
    };

    FridayDashboardProductUiButtonState {
        disabled: !enabled,
        disabled_reason: if enabled {
            None
        } else {
            Some("Action command is empty.".to_string())
        },
        idle_label: label.to_string(),
        loading_label,
        success_label: format!("{verb} complete"),
        error_label: format!("{verb} failed"),
        aria_label: format!("{verb} dashboard action: {label}"),
        destructive,
        requires_confirmation,
    }
}

fn dashboard_data_bindings(
    panel_json_command: &str,
    export_command: &str,
    history_json: &str,
    release_review_json: &str,
) -> Vec<FridayUiDataBinding> {
    vec![
        ui_binding(
            "dashboard-panel-json",
            "Friday dashboard panel JSON",
            panel_json_command,
            "Friday dashboard route state",
            "Loads typed dashboard cards, warnings, screenshots, history, and release-review state.",
        ),
        ui_binding(
            "dashboard-export-refresh",
            "Friday dashboard export refresh",
            export_command,
            "Friday dashboard export bundle",
            "Refreshes the local dashboard export before the product UI reads it.",
        ),
        ui_binding(
            "dashboard-history-json",
            "Friday dashboard history JSON",
            history_json,
            "Friday dashboard history strip",
            "Feeds latest/previous checkpoint deltas into the dashboard history UI.",
        ),
        ui_binding(
            "release-review-json",
            "Friday release-review JSON",
            release_review_json,
            "Friday release-review panel",
            "Feeds TODO, changelog, visual target, and artifact links into release review UI.",
        ),
    ]
}

fn ui_binding(
    id: &str,
    source: &str,
    command: &str,
    writes_to: &str,
    description: &str,
) -> FridayUiDataBinding {
    FridayUiDataBinding {
        id: id.to_string(),
        source: source.to_string(),
        command: command.to_string(),
        writes_to: writes_to.to_string(),
        local_only: true,
        description: description.to_string(),
    }
}
