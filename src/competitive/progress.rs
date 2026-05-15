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
            "release-handoff-governance-review-model",
            "Typed release handoff governance review model",
            20,
            CompletionItemStatus::Done,
            "`FridayReleaseHandoffGovernanceReview` consumes handoff audit trails and validates external handoff readiness",
            "open the next Friday release handoff dispatch checklist set",
        ),
        item(
            "release-handoff-governance-review-states",
            "Approved, held, needs-acknowledgement, stale-active-packet, and blocked-carryover states",
            20,
            CompletionItemStatus::Done,
            "governance findings preserve latest packet, active packet, acknowledgement gaps, stale/superseded/revoked warnings, and blocker carryover",
            "open the next Friday release handoff dispatch checklist set",
        ),
        item(
            "release-handoff-governance-review-cli",
            "Release handoff governance review CLI and JSON commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-release-handoff-governance-review` writes local review JSON without sending, deploying, building, or mutating external systems",
            "open the next Friday release handoff dispatch checklist set",
        ),
        item(
            "release-handoff-governance-review-dashboard",
            "Dashboard handoff governance review rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports governance reviews and renders approval state, latest/active packet, gate blockers, governance findings, command copy, and governance notes",
            "open the next Friday release handoff dispatch checklist set",
        ),
        item(
            "release-handoff-governance-review-coverage",
            "Release handoff governance review Rust and TypeScript coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage plus dashboard smoke checks verify review scoring, stale packet detection, acknowledgement enforcement, command safety, and dashboard rendering",
            "open the next Friday release handoff dispatch checklist set",
        ),
    ];

    CompletionSet {
        name: "Friday Release Handoff Governance Review".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Validate Friday release handoff audit trails before any release note, public handoff, deployment note, or external send.".to_string(),
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
    fn active_set_tracks_friday_release_handoff_governance_review_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Release Handoff Governance Review");
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
