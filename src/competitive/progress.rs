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
            "release-handoff-completion-ledger-model",
            "Typed release handoff completion ledger model",
            20,
            CompletionItemStatus::Done,
            "`FridayReleaseHandoffCompletionLedger` preserves governed local handoff outcomes from dispatch governance reviews",
            "open the next Friday release publication control set",
        ),
        item(
            "release-handoff-completion-ledger-states",
            "Draft, completed, manually-sent, held, revoked, superseded, and blocked states",
            20,
            CompletionItemStatus::Done,
            "completion records downgrade unsafe completed/manually-sent attempts to blocked when governance still carries blockers",
            "open the next Friday release publication control set",
        ),
        item(
            "release-handoff-completion-ledger-cli",
            "Release handoff completion ledger append/list/export/JSON commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-release-handoff-completion` records local completion outcomes without sending, deploying, building, uploading, or mutating external systems",
            "open the next Friday release publication control set",
        ),
        item(
            "release-handoff-completion-ledger-dashboard",
            "Dashboard handoff completion ledger rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports completion ledgers and renders latest outcome, governance state, blockers, command copy, and summary copy",
            "open the next Friday release publication control set",
        ),
        item(
            "release-handoff-completion-ledger-coverage",
            "Release handoff completion ledger Rust and TypeScript coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage plus dashboard smoke checks verify blocked completion downgrades, no external mutation, command safety, and dashboard rendering",
            "open the next Friday release publication control set",
        ),
    ];

    CompletionSet {
        name: "Friday Release Handoff Completion Ledger".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Preserve governed local handoff completion outcomes before any release note, deployment note, publication, or external send is considered done.".to_string(),
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
    fn active_set_tracks_friday_release_handoff_completion_ledger_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Release Handoff Completion Ledger");
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
