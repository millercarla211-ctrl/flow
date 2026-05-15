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
            "release-handoff-dispatch-checklist-model",
            "Typed release handoff dispatch checklist model",
            20,
            CompletionItemStatus::Done,
            "`FridayReleaseHandoffDispatchChecklist` consumes governance reviews and verifies local dispatch readiness",
            "open the next Friday release handoff dispatch audit set",
        ),
        item(
            "release-handoff-dispatch-checklist-states",
            "Ready, held, missing-recipient, missing-attachment, privacy-review, and blocked states",
            20,
            CompletionItemStatus::Done,
            "dispatch items preserve governance approval, recipient, attachment, note, privacy-boundary, and no-send safeguard readiness",
            "open the next Friday release handoff dispatch audit set",
        ),
        item(
            "release-handoff-dispatch-checklist-cli",
            "Release handoff dispatch checklist CLI and JSON commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-release-handoff-dispatch-checklist` writes local checklist JSON without sending, deploying, building, or mutating external systems",
            "open the next Friday release handoff dispatch audit set",
        ),
        item(
            "release-handoff-dispatch-checklist-dashboard",
            "Dashboard handoff dispatch checklist rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports dispatch checklists and renders recipients, attachments, privacy notes, no-send warnings, command copy, and checklist copy",
            "open the next Friday release handoff dispatch audit set",
        ),
        item(
            "release-handoff-dispatch-checklist-coverage",
            "Release handoff dispatch checklist Rust and TypeScript coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage plus dashboard smoke checks verify checklist readiness, privacy-boundary enforcement, missing attachment detection, command safety, and dashboard rendering",
            "open the next Friday release handoff dispatch audit set",
        ),
    ];

    CompletionSet {
        name: "Friday Release Handoff Dispatch Checklist".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Verify recipients, attachments, privacy boundaries, and no-send safeguards before any Friday release handoff dispatch.".to_string(),
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
    fn active_set_tracks_friday_release_handoff_dispatch_checklist_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Release Handoff Dispatch Checklist");
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
