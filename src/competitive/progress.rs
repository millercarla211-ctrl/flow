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
            "dashboard-command-dispatcher-contract",
            "Dashboard command dispatcher contract",
            20,
            CompletionItemStatus::Done,
            "`extensions/flow-webext/src/runtime/dashboard-actions.ts` defines typed local command dispatch results for dashboard actions",
            "add confirmation and permission states for risky dashboard commands",
        ),
        item(
            "dashboard-command-permissions",
            "Dashboard command confirmation and permissions",
            20,
            CompletionItemStatus::Done,
            "`dispatchDashboardCommand` blocks remote/disabled actions and requires explicit confirmation before risky local handoffs",
            "persist recent dashboard action results for operator review",
        ),
        item(
            "dashboard-command-result-history",
            "Dashboard command result history",
            20,
            CompletionItemStatus::Done,
            "`extensions/flow-webext/src/runtime/dashboard-actions.ts` persists recent command handoff results in a bounded local browser cache",
            "add focused tests for command dispatch success, failure, and blocked states",
        ),
        item(
            "dashboard-command-dispatch-tests",
            "Dashboard command dispatch smoke tests",
            20,
            CompletionItemStatus::Done,
            "`npm run smoke:dashboard` verifies prepared, confirmation-required, blocked, and failed dashboard command dispatch states",
            "surface command execution results in the visible dashboard without auto-running anything silently",
        ),
        item(
            "dashboard-command-visible-results",
            "Visible dashboard command results",
            20,
            CompletionItemStatus::Done,
            "`extensions/flow-webext/src/ui/app.ts` renders recent command handoff results and copies prepared local commands without silently running them",
            "open the next host-command bridge set for trusted desktop execution",
        ),
    ];

    CompletionSet {
        name: "Friday Dashboard Command Execution".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Execute dashboard actions through explicit local command handoffs while preserving user control, permissions, and low-resource behavior.".to_string(),
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
    fn active_set_tracks_friday_dashboard_command_execution_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Dashboard Command Execution");
        assert_eq!(set.current_score_out_of_100, 100);
        assert!(
            set.items
                .iter()
                .all(|item| item.status == CompletionItemStatus::Done)
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
