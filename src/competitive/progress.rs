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
            "desktop-runner-bridge-interface",
            "Desktop runner bridge interface",
            20,
            CompletionItemStatus::Done,
            "`run_friday_trusted_host_command_bridge_with_executor` emits pending, running, and finished bridge events around trusted local execution",
            "open the next desktop bridge cancellation UX set",
        ),
        item(
            "desktop-runner-state-emission",
            "Live state writes during execution",
            20,
            CompletionItemStatus::Done,
            "`run_friday_trusted_host_command_bridge_with_executor` writes live state before execution, during running, and after success, denial, timeout, cancellation, or failure",
            "open the next desktop bridge cancellation UX set",
        ),
        item(
            "desktop-runner-cancellation-token",
            "Cancellation token plumbing",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerCancellationToken` blocks execution before the running event and records cancellation reason metadata",
            "open the next desktop bridge cancellation UX set",
        ),
        item(
            "desktop-runner-import-guidance",
            "Live-state import guidance",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerBridgeReport` and dashboard normalizers distinguish live-state imports from immutable runner history JSON",
            "open the next desktop bridge cancellation UX set",
        ),
        item(
            "desktop-runner-bridge-tests",
            "Bridge event and cancellation tests",
            20,
            CompletionItemStatus::Done,
            "`cargo test friday_dashboard -- --nocapture` covers pending/running/finished bridge events and cancellation boundaries",
            "open the next desktop bridge cancellation UX set",
        ),
    ];

    CompletionSet {
        name: "Friday Desktop Runner Bridge".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Make the trusted desktop host produce live runner state updates during real approved command execution while keeping all host execution local-only, auditable, cancellable, and bounded.".to_string(),
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
    fn active_set_tracks_friday_desktop_runner_bridge_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Desktop Runner Bridge");
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
