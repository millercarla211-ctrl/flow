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
            "Trusted host command bridge contract",
            20,
            CompletionItemStatus::Done,
            "`src/friday/dashboard_host_bridge.rs` maps product UI dashboard actions into trusted host command bridge records",
            "require explicit operator approval before desktop/Tauri command execution",
        ),
        item(
            "dashboard-command-permissions",
            "Explicit host command approval",
            20,
            CompletionItemStatus::Done,
            "`FridayDashboardHostCommandRecord` sets `silent_execution_allowed=false` and `approval_state=required` for executable commands",
            "write command execution audit records with stdout/stderr summaries and duration",
        ),
        item(
            "dashboard-command-result-history",
            "Host command audit records",
            20,
            CompletionItemStatus::Done,
            "`FridayDashboardHostCommandAudit` records stdout/stderr summaries, duration, action id, event, and timestamp for every bridge record",
            "add blocked-command tests for remote, destructive, and malformed commands",
        ),
        item(
            "dashboard-command-dispatch-tests",
            "Host command blocked-state tests",
            20,
            CompletionItemStatus::Done,
            "`cargo test friday_dashboard -- --nocapture` covers local approval, remote, destructive, disabled, and malformed command states",
            "surface trusted host execution results in the dashboard without freezing the UI",
        ),
        item(
            "dashboard-command-visible-results",
            "Visible host bridge results",
            20,
            CompletionItemStatus::Done,
            "`extensions/flow-webext/src/ui/app.ts` imports host bridge JSON and renders the normalized command records in the dashboard result rail",
            "open the next trusted host runner implementation set",
        ),
    ];

    CompletionSet {
        name: "Friday Dashboard Host Command Bridge".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Bridge prepared dashboard command handoffs into trusted desktop execution with operator approval, audit logs, and safe failure recovery.".to_string(),
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
    fn active_set_tracks_friday_dashboard_host_command_bridge_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Dashboard Host Command Bridge");
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
