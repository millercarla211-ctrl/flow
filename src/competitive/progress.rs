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
            "release-publication-control-model",
            "Typed release publication control model",
            20,
            CompletionItemStatus::Done,
            "`FridayReleasePublicationControl` consumes completion ledgers and marks local-only publication readiness",
            "open the next Friday release outbound review set",
        ),
        item(
            "release-publication-control-states",
            "Draft, ready, held, blocked, published-manually, revoked, and superseded states",
            20,
            CompletionItemStatus::Done,
            "publication controls downgrade unsafe ready/published-manually requests to blocked when completion ledgers still carry blockers",
            "open the next Friday release outbound review set",
        ),
        item(
            "release-publication-control-cli",
            "Release publication control CLI and JSON commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-release-publication-control` prepares release notes, deployment notes, announcements, and send instructions without external publication",
            "open the next Friday release outbound review set",
        ),
        item(
            "release-publication-control-dashboard",
            "Dashboard release publication control rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports publication controls and renders readiness, blockers, latest completion, command copy, release notes, and send instructions",
            "open the next Friday release outbound review set",
        ),
        item(
            "release-publication-control-coverage",
            "Release publication control Rust and TypeScript coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage plus dashboard smoke checks verify publication readiness, blocked ledgers, manual-publish safety, and dashboard rendering",
            "open the next Friday release outbound review set",
        ),
    ];

    CompletionSet {
        name: "Friday Release Publication Control".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Prepare release notes, deployment notes, announcements, and external-send instructions locally without publishing or mutating external systems.".to_string(),
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
    fn active_set_tracks_friday_release_publication_control_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Release Publication Control");
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
