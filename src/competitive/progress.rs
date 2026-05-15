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
            "live-runner-state-record",
            "Typed live runner state record",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostLiveRunnerRecord` separates pending/running/stale live state from immutable trusted runner history",
            "open the next desktop-host bridge set",
        ),
        item(
            "live-runner-state-store",
            "Local live state reader and writer",
            20,
            CompletionItemStatus::Done,
            "`read_friday_trusted_host_live_runner_state` and `write_friday_trusted_host_live_runner_state` persist local-only pending, running, finished, and stale records",
            "open the next desktop-host bridge set",
        ),
        item(
            "live-runner-dashboard-rendering",
            "Dashboard live progress rendering",
            20,
            CompletionItemStatus::Done,
            "`extensions/flow-webext/src/ui/app.ts` imports and renders live runner state separately from history and approval imports",
            "open the next desktop-host bridge set",
        ),
        item(
            "live-runner-stale-recovery",
            "Stale-state recovery copy",
            20,
            CompletionItemStatus::Done,
            "`refresh_friday_trusted_host_live_runner_state` marks old active records stale and exposes recovery and cleanup commands",
            "open the next desktop-host bridge set",
        ),
        item(
            "live-runner-state-tests",
            "Live state transition tests",
            20,
            CompletionItemStatus::Done,
            "`cargo test friday_dashboard -- --nocapture` and `npm run smoke:dashboard` cover live runner state normalization, stale handling, and dashboard rendering",
            "open the next desktop-host bridge set",
        ),
    ];

    CompletionSet {
        name: "Friday Live Runner State".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Connect trusted runner approval and history UX to live state transitions so the dashboard can show pending, running, completed, failed, timed-out, cancelled, and denied work without confusing stale imports for live execution.".to_string(),
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
    fn active_set_tracks_friday_live_runner_state_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Live Runner State");
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
