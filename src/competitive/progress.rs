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
            "dashboard-visible-card-rendering",
            "Visible dashboard card rendering",
            20,
            CompletionItemStatus::Done,
            "`extensions/flow-webext/src/ui/app.ts` renders typed dashboard cards from `FlowDashboardProductUiBinding` through `FlowBrowserEngine.dashboardBinding()`",
            "wire visible action buttons to the local command handoff metadata",
        ),
        item(
            "dashboard-visible-action-buttons",
            "Visible dashboard action buttons",
            20,
            CompletionItemStatus::Done,
            "`extensions/flow-webext/src/ui/app.ts` renders local dashboard action buttons with idle/loading/success/error labels from typed button state metadata",
            "render export history deltas, release-review links, and screenshot prompts in the dashboard",
        ),
        item(
            "dashboard-history-release-rail",
            "Dashboard history and release rail",
            20,
            CompletionItemStatus::Planned,
            "not started",
            "render export history deltas, release-review links, and screenshot prompts in the dashboard",
        ),
        item(
            "dashboard-typescript-smoke",
            "Dashboard TypeScript smoke path",
            20,
            CompletionItemStatus::Planned,
            "not started",
            "add a small TypeScript smoke/typecheck path for dashboard section rendering",
        ),
        item(
            "dashboard-local-only-no-dummy-copy",
            "Local-only dashboard copy cleanup",
            20,
            CompletionItemStatus::Planned,
            "not started",
            "keep local-only fallback behavior and remove dummy product copy from this dashboard surface",
        ),
    ];

    CompletionSet {
        name: "Friday Dashboard Visible UI Execution".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Render the typed Friday dashboard contract inside the visible browser/desktop UI, keep actions local-only, and remove dashboard dummy copy as the UI becomes real.".to_string(),
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
        assert_eq!(set.name, "Friday Dashboard Visible UI Execution");
        assert_eq!(set.current_score_out_of_100, 40);
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
