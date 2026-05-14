use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardActionKind, FridayDashboardPanelStatus, FridayUiDataBinding,
    friday_dashboard_panel_from_export,
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
    pub disabled_reason: Option<String>,
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
                    disabled_reason: if action.enabled {
                        None
                    } else {
                        Some("Action command is empty.".to_string())
                    },
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
        ],
    })
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
