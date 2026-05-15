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
            "release-promotion-ledger-model",
            "Typed release promotion ledger model",
            20,
            CompletionItemStatus::Done,
            "`FridayReleasePromotionLedger` links release candidates to operator decisions, deployment notes, targets, rollback references, and post-promotion checks",
            "open the next release monitor set",
        ),
        item(
            "release-promotion-ledger-decisions",
            "Promotion decision categories",
            20,
            CompletionItemStatus::Done,
            "`FridayReleasePromotionDecision` covers promoted, held, rolled-back, superseded, and abandoned candidate states",
            "open the next release monitor set",
        ),
        item(
            "release-promotion-ledger-cli",
            "Promotion ledger CLI and JSON commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-release-promotion-ledger` and JSON mode record local promotion decisions without running deployments",
            "open the next release monitor set",
        ),
        item(
            "release-promotion-ledger-dashboard",
            "Dashboard promotion ledger rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports promotion ledger JSON and renders promotion history, active rollback reference, warnings, post-promotion checks, and copyable ledger command",
            "open the next release monitor set",
        ),
        item(
            "release-promotion-ledger-coverage",
            "Promotion ledger Rust and TypeScript coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage plus dashboard smoke checks verify ledger writes, rollback references, post-promotion checks, command copy, and visible dashboard import/rendering",
            "open the next release monitor set",
        ),
    ];

    CompletionSet {
        name: "Friday Release Promotion Ledger".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Track every Friday candidate promotion decision with deployment notes, rollback references, operator reasons, and post-promotion verification evidence.".to_string(),
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
    fn active_set_tracks_friday_release_promotion_ledger_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Release Promotion Ledger");
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
