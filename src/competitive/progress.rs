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
            "runner-status-groups",
            "Grouped runner history summaries",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerUxReport` groups trusted runner history into succeeded, failed, timed-out, cancelled, and denied summaries",
            "open the next dashboard approval UI set",
        ),
        item(
            "runner-retry-copy-affordances",
            "Retry and copy affordances",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerAffordance` exposes copy-command, retry, and cancel metadata while keeping retry approval explicit",
            "open the next dashboard approval UI set",
        ),
        item(
            "runner-clear-status-copy",
            "Cancellation and timeout copy",
            20,
            CompletionItemStatus::Done,
            "`runner_status_operator_copy` gives clear operator-facing copy for timeout, cancelled, denied, failed, and succeeded states",
            "open the next dashboard approval UI set",
        ),
        item(
            "runner-ui-smoke",
            "Runner UX smoke checks",
            20,
            CompletionItemStatus::Done,
            "`npm run smoke:dashboard` covers trusted runner UX grouping and retry/copy affordance normalization",
            "open the next dashboard approval UI set",
        ),
        item(
            "runner-release-review-notes",
            "Operator release-review notes",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerOperatorNote` links runner history back to release-review artifacts for shipping checks",
            "open the next dashboard approval UI set",
        ),
    ];

    CompletionSet {
        name: "Friday Dashboard Runner UX".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Make trusted runner history and live progress easy to understand, retry, cancel, and audit from the dashboard without clutter or blocking the UI.".to_string(),
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
    fn active_set_tracks_friday_dashboard_runner_ux_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Dashboard Runner UX");
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
