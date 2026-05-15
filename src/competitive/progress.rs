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
            "runner-approval-modal-contract",
            "Approval modal contract",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerApprovalUiReport` exposes a typed approval dialog contract with reason fields, command preview, controls, snooze, undo, and release-review paths",
            "open the next live runner state set",
        ),
        item(
            "runner-keyboard-controls",
            "Keyboard-accessible controls",
            20,
            CompletionItemStatus::Done,
            "`extensions/flow-webext/src/ui/app.ts` renders approve, deny, copy, retry, cancel, snooze, and undo controls with keyboard shortcut metadata and button handlers",
            "open the next live runner state set",
        ),
        item(
            "runner-audit-reasons",
            "Approval and denial audit reasons",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerRequest` and `FridayTrustedHostRunnerResult` persist operator reasons for approved, denied, and cancelled runner records",
            "open the next live runner state set",
        ),
        item(
            "runner-undo-snooze-affordances",
            "Undo and snooze affordances",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedHostRunnerApprovalUiReport` includes snooze options and immutable-history undo copy for pending approval drafts",
            "open the next live runner state set",
        ),
        item(
            "runner-approval-smoke",
            "Approval modal smoke checks",
            20,
            CompletionItemStatus::Done,
            "`npm run smoke:dashboard` covers approval modal normalization, reason requirements, keyboard shortcuts, snooze, and undo controls",
            "open the next live runner state set",
        ),
    ];

    CompletionSet {
        name: "Friday Runner Approval UI".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Turn trusted runner UX metadata into a polished dashboard approval surface with keyboard-accessible controls, audit reasons, and live-safe affordances.".to_string(),
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
    fn active_set_tracks_friday_runner_approval_ui_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Runner Approval UI");
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
