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
            "runner-release-timeline-model",
            "Typed trusted runner release timeline",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedRunnerReleaseTimeline` loads multiple package entries and keeps package count, latest package, readiness, blocker, and freshness metadata",
            "open the next release operator checklist set",
        ),
        item(
            "runner-release-timeline-diffs",
            "Package comparison and regression summaries",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedRunnerReleaseTimelineDiff` compares evidence count, missing evidence, warnings, stale warnings, signature changes, and regression state",
            "open the next release operator checklist set",
        ),
        item(
            "runner-release-timeline-cli",
            "Archive and timeline CLI commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-trusted-runner-release-archive`, `--friday-trusted-runner-release-timeline`, and the JSON timeline command append or review packages without host execution",
            "open the next release operator checklist set",
        ),
        item(
            "runner-release-timeline-dashboard",
            "Dashboard timeline import rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports timeline JSON and renders latest package, package counts, regression warnings, and recent package rows",
            "open the next release operator checklist set",
        ),
        item(
            "runner-release-timeline-tests",
            "Timeline diff and smoke coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage and dashboard smoke checks verify timeline loading, diff regressions, archive writes, and dashboard normalization",
            "open the next release operator checklist set",
        ),
    ];

    CompletionSet {
        name: "Friday Trusted Runner Evidence Timeline".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Make trusted-runner release packages comparable over time so operators can spot regressions, stale evidence, missing artifacts, and recurring command failures before shipping.".to_string(),
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
    fn active_set_tracks_friday_trusted_runner_evidence_timeline_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Trusted Runner Evidence Timeline");
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
