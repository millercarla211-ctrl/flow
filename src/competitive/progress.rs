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
            "runner-review-filters",
            "Typed trusted runner review filters",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerOperatorReviewFilter` supports status, action, time-window, and limit filters over persisted runner history",
            "open the next trusted runner release packaging set",
        ),
        item(
            "runner-release-gate-summaries",
            "Release-gate summaries",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerOperatorReviewReport` groups succeeded, failed, timed-out, cancelled, denied, and stale-live-state review gates",
            "open the next trusted runner release packaging set",
        ),
        item(
            "runner-incident-notes",
            "Export-ready incident notes",
            20,
            CompletionItemStatus::Done,
            "failed, timed-out, cancelled, and denied runner records now produce Markdown incident notes for release review handoff",
            "open the next trusted runner release packaging set",
        ),
        item(
            "runner-review-dashboard",
            "Dashboard operator review rendering",
            20,
            CompletionItemStatus::Done,
            "the dashboard imports and renders operator review filters, release-gate cards, and copyable incident notes",
            "open the next trusted runner release packaging set",
        ),
        item(
            "runner-review-tests",
            "Review filter and gate tests",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage and browser-extension smoke checks verify review filtering, release-gate copy, and incident exports",
            "open the next trusted runner release packaging set",
        ),
    ];

    CompletionSet {
        name: "Friday Trusted Runner Operator Review".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Make trusted runner audit review useful after many local commands by giving operators filterable history, export-ready incident notes, release-gate summaries, and clear escalation paths.".to_string(),
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
    fn active_set_tracks_friday_trusted_runner_operator_review_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Trusted Runner Operator Review");
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
