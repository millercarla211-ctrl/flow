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
            "trusted-host-runner-contract",
            "Bounded trusted host runner",
            20,
            CompletionItemStatus::Done,
            "`src/friday/dashboard_host_runner.rs` executes only approved trusted host bridge records through a bounded runner",
            "add timeout, cancellation, stdout/stderr size limits, and process error mapping",
        ),
        item(
            "trusted-host-runner-bounds",
            "Runner timeout, cancellation, and output limits",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerRequest` carries approval, cancellation, timeout, and stdout/stderr byte limits",
            "persist host execution audit history separately from prepared handoff history",
        ),
        item(
            "trusted-host-runner-history",
            "Trusted runner audit history",
            20,
            CompletionItemStatus::Done,
            "`append_friday_trusted_host_runner_history` writes bounded trusted runner history separate from host bridge handoff history",
            "add tests for approved success, timeout, cancellation, and denied commands",
        ),
        item(
            "trusted-host-runner-tests",
            "Trusted runner status tests",
            20,
            CompletionItemStatus::Done,
            "`cargo test friday_dashboard -- --nocapture` covers approved success, timeout, cancellation, remote denial, malformed denial, and history persistence",
            "surface trusted runner status in the dashboard with non-blocking progress updates",
        ),
        item(
            "trusted-host-runner-visible-results",
            "Visible trusted runner results",
            20,
            CompletionItemStatus::Done,
            "`extensions/flow-webext/src/ui/app.ts` imports trusted runner JSON and renders non-blocking runner states in the dashboard result rail",
            "open the next durable desktop runner UI set",
        ),
    ];

    CompletionSet {
        name: "Friday Trusted Host Runner".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Execute approved dashboard host commands through a bounded trusted runner while preserving cancellation, auditability, and local-only safety.".to_string(),
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
    fn active_set_tracks_friday_trusted_host_runner_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Trusted Host Runner");
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
