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
            "release-checklist-model",
            "Typed release operator checklist",
            20,
            CompletionItemStatus::Done,
            "`FridayReleaseOperatorChecklistReport` consumes package, timeline, dashboard release review, TODO, changelog, and signoff evidence",
            "open the next release QA command center set",
        ),
        item(
            "release-checklist-blockers",
            "Release blocker categorization",
            20,
            CompletionItemStatus::Done,
            "the checklist classifies missing evidence, warning regressions, stale live state, pending runner work, release-review issues, and unreviewed TODO/changelog gaps",
            "open the next release QA command center set",
        ),
        item(
            "release-checklist-cli",
            "Checklist and signoff CLI commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-release-checklist`, `--friday-release-checklist-json`, `--friday-release-signoff`, and the JSON signoff command create review reports and append local signoffs",
            "open the next release QA command center set",
        ),
        item(
            "release-checklist-dashboard",
            "Dashboard checklist rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports release checklist JSON, renders blockers/items/freshness copy, captures a signoff reason, and copies the signoff command",
            "open the next release QA command center set",
        ),
        item(
            "release-checklist-tests",
            "Checklist and signoff coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage and dashboard smoke checks verify checklist generation, blocker copy, signoff persistence, and UI normalization",
            "open the next release QA command center set",
        ),
    ];

    CompletionSet {
        name: "Friday Release Operator Checklist".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Turn release package and evidence-timeline data into a concise operator checklist with explicit signoff, unresolved-blocker copy, and local-only audit history.".to_string(),
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
    fn active_set_tracks_friday_release_operator_checklist_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Release Operator Checklist");
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
