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
            "runner-visible-cancel-controls",
            "Visible live runner cancellation controls",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerCancellationUxReport` exposes cancel controls for pending and running live records, and the dashboard renders copyable cancel commands",
            "open the next trusted runner operator review set",
        ),
        item(
            "runner-stale-cleanup-retry",
            "Stale cleanup and retry guidance",
            20,
            CompletionItemStatus::Done,
            "stale live records now surface cleanup and retry commands linked to the imported bridge state file and runner history",
            "open the next trusted runner operator review set",
        ),
        item(
            "runner-denial-recovery",
            "Denial recovery copy",
            20,
            CompletionItemStatus::Done,
            "denied live runner records produce an explicit denial-recovery command with a required operator reason placeholder",
            "open the next trusted runner operator review set",
        ),
        item(
            "runner-cancellation-drafts",
            "Dashboard-side cancellation drafts",
            20,
            CompletionItemStatus::Done,
            "the web dashboard persists per-control cancellation, retry, and denial-recovery reasons in local storage",
            "open the next trusted runner operator review set",
        ),
        item(
            "runner-cancellation-tests",
            "Cancellation and stale cleanup tests",
            20,
            CompletionItemStatus::Done,
            "Rust integration coverage and browser-extension dashboard smoke checks verify cancellation, stale cleanup, retry, and denial recovery contracts",
            "open the next trusted runner operator review set",
        ),
    ];

    CompletionSet {
        name: "Friday Desktop Runner Cancellation UX".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Make live trusted-runner cancellation and recovery obvious in the dashboard so operators can stop, clean up, retry, or recover denied work without guessing which JSON import is authoritative.".to_string(),
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
    fn active_set_tracks_friday_desktop_runner_cancellation_ux_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Desktop Runner Cancellation UX");
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
