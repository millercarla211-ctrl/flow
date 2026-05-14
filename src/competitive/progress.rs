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
            "dashboard-export-panel",
            "Dashboard export panel",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-dashboard-panel` reads the exported manifest and JSON reports into typed dashboard cards/actions for Friday and DX UI surfaces",
            "wire the panel into the product UI and DX dashboard shell",
        ),
        item(
            "recent-screenshot-history",
            "Recent screenshot history",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-dashboard-panel` exposes screenshot capture history, missing-capture prompts, metadata gaps, and per-route capture commands for top Friday routes",
            "connect screenshot prompt actions to one-click capture controls in the dashboard shell",
        ),
        item(
            "dashboard-recovery-actions",
            "Dashboard recovery actions",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-dashboard-panel` exposes typed open, run-check, recover, and capture actions with local-only commands for warning readiness items",
            "wire these typed actions into the product UI/DX dashboard button handlers",
        ),
        item(
            "export-history-store",
            "Export history store",
            20,
            CompletionItemStatus::Planned,
            "not started",
            "persist dashboard export history for checkpoint-to-checkpoint readiness comparison",
        ),
        item(
            "release-review-handoff",
            "Release review handoff",
            20,
            CompletionItemStatus::Planned,
            "not started",
            "link completion, changelog, TODO, visual targets, and dashboard export files into a release-review handoff",
        ),
    ];

    CompletionSet {
        name: "Friday Dashboard Runtime Wiring".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Consume Friday readiness exports from product and DX dashboard surfaces, preserve local-first status visibility, and guide release work without manual CLI scraping.".to_string(),
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
    fn active_set_tracks_friday_dashboard_runtime_wiring_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Dashboard Runtime Wiring");
        assert_eq!(set.current_score_out_of_100, 60);
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
