use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompletionItemStatus {
    Done,
    InProgress,
    Planned,
    Blocked,
}

impl CompletionItemStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::InProgress => "in-progress",
            Self::Planned => "planned",
            Self::Blocked => "blocked",
        }
    }

    fn multiplier(self) -> f32 {
        match self {
            Self::Done => 1.0,
            Self::InProgress => 0.4,
            Self::Planned | Self::Blocked => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionItem {
    pub key: String,
    pub title: String,
    pub weight: u8,
    pub status: CompletionItemStatus,
    pub proof: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionSet {
    pub name: String,
    pub target_score_out_of_100: u8,
    pub current_score_out_of_100: u8,
    pub loop_rule: String,
    pub items: Vec<CompletionItem>,
}

pub fn active_completion_set() -> CompletionSet {
    let items = vec![
        item(
            "dashboard-panel-json-binding",
            "Dashboard panel JSON binding",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-dashboard-product-ui-json` maps the exported dashboard panel into a visible /dashboard route binding with data sources, cards, and local actions",
            "wire the typed dashboard binding into the visible web/desktop dashboard component",
        ),
        item(
            "dashboard-action-buttons",
            "Dashboard action buttons",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-dashboard-product-ui-json` exposes safe button state labels, disabled reasons, confirmation flags, and local-only action metadata",
            "render these button states in the visible dashboard controls",
        ),
        item(
            "dashboard-history-deltas",
            "Dashboard history deltas",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-dashboard-product-ui-json` exposes history trend labels plus missing screenshot prompts and capture commands for product UI rendering",
            "render these deltas and screenshot prompts in the visible dashboard",
        ),
        item(
            "release-review-links",
            "Release review links",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-dashboard-product-ui-json` exposes grouped release-review links with local open-button state for TODO, changelog, visual targets, and export artifacts",
            "render these release-review link groups in the visible dashboard",
        ),
        item(
            "dashboard-ui-smoke-contract",
            "Dashboard UI smoke contract",
            20,
            CompletionItemStatus::Planned,
            "not started",
            "add a small UI smoke contract proving the dashboard can load the exported panel",
        ),
    ];

    CompletionSet {
        name: "Friday Dashboard Product UI Wiring".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Consume typed Friday dashboard panel data from the visible product UI, keep all actions local-first and explicit, and avoid dummy dashboard copy.".to_string(),
        items,
    }
}

fn score_items(items: &[CompletionItem]) -> u8 {
    let earned = items
        .iter()
        .map(|item| item.weight as f32 * item.status.multiplier())
        .sum::<f32>();
    let possible = items.iter().map(|item| item.weight as f32).sum::<f32>();

    if possible <= f32::EPSILON {
        0
    } else {
        ((earned / possible) * 100.0).round().clamp(0.0, 100.0) as u8
    }
}

fn item(
    key: &str,
    title: &str,
    weight: u8,
    status: CompletionItemStatus,
    proof: &str,
    next_action: &str,
) -> CompletionItem {
    CompletionItem {
        key: key.to_string(),
        title: title.to_string(),
        weight,
        status,
        proof: proof.to_string(),
        next_action: next_action.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_set_tracks_friday_dashboard_product_ui_wiring_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Dashboard Product UI Wiring");
        assert_eq!(set.current_score_out_of_100, 80);
        assert!(
            set.items
                .iter()
                .any(|item| item.status == CompletionItemStatus::Planned)
        );
    }

    #[test]
    fn planned_items_keep_score_below_100() {
        let items = vec![
            item(
                "done",
                "Finished item",
                50,
                CompletionItemStatus::Done,
                "done",
                "none",
            ),
            item(
                "planned",
                "Planned item",
                50,
                CompletionItemStatus::Planned,
                "not done",
                "finish it",
            ),
        ];

        assert!(score_items(&items) < 100);
    }
}
