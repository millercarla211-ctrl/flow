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
            "tracked-route-file-bindings",
            "Tracked route file bindings",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-live-ui-routes` verifies every Friday route points at tracked UI/runtime files instead of contract-only placeholders",
            "add screenshot-backed checks for the most-used routes",
        ),
        item(
            "screenshot-backed-route-checks",
            "Screenshot-backed route checks",
            20,
            CompletionItemStatus::Planned,
            "not started",
            "capture and compare the default Friday, Voice, Search, Research, and Multimodal route screens",
        ),
        item(
            "operator-readiness-summary",
            "Operator readiness summary",
            20,
            CompletionItemStatus::Planned,
            "not started",
            "summarize local model, browser extension, desktop app, and route readiness in one operator report",
        ),
        item(
            "desktop-web-execution-handoff",
            "Desktop/web execution handoff",
            20,
            CompletionItemStatus::Planned,
            "not started",
            "add explicit contracts for launching live flows from desktop, web, and browser-extension UI surfaces",
        ),
        item(
            "release-dashboard-export",
            "Release dashboard export",
            20,
            CompletionItemStatus::Planned,
            "not started",
            "export the live UI readiness summary for Friday/DX dashboards",
        ),
    ];

    CompletionSet {
        name: "Friday Live UI Execution".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Connect Friday's Rust contracts to tracked desktop/web UI route files, screenshot-backed verification, and operator-facing readiness summaries without weakening local-first execution.".to_string(),
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
    fn active_set_tracks_friday_live_ui_execution_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Live UI Execution");
        assert_eq!(set.current_score_out_of_100, 20);
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
