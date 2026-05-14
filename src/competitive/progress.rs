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
            "checkpoint",
            "Professional git checkpoint before new work",
            10,
            CompletionItemStatus::Done,
            "tracked runtime/model-policy work was typechecked, committed, and pushed",
            "keep future commits focused and push at each healthy checkpoint",
        ),
        item(
            "roadmap",
            "Active TODO loop replaces static release-ready wording",
            15,
            CompletionItemStatus::Done,
            "TODO.md now names the current feature set, score, completed work, and next work",
            "keep scoring conservative and update it after every completed slice",
        ),
        item(
            "changelog",
            "Changelog captures product-loop progress",
            10,
            CompletionItemStatus::Done,
            "CHANGELOG.md has an Unreleased section for the completion-loop work",
            "record every shipped slice before committing",
        ),
        item(
            "completion-cli",
            "CLI exposes the current feature-set score",
            20,
            CompletionItemStatus::Done,
            "`flow --completion` prints the active set, score, evidence, and next actions",
            "expand the command with machine-readable output if downstream tooling needs it",
        ),
        item(
            "release-handoff",
            "Release summary includes the active completion loop",
            20,
            CompletionItemStatus::Done,
            "`flow --release-summary` carries active completion status into operator handoff",
            "include TODO and changelog snapshots in exported production bundles next",
        ),
        item(
            "bundle-handoff",
            "Production bundle includes TODO and changelog snapshots",
            15,
            CompletionItemStatus::Done,
            "production bundle exports copy TODO.md and CHANGELOG.md into handoff snapshots",
            "start the next feature set after this control loop reaches 100/100",
        ),
        item(
            "machine-export",
            "Completion loop has a machine-readable export",
            10,
            CompletionItemStatus::Done,
            "`flow --completion-json` prints the active completion loop for Friday/DX dashboards",
            "wire the JSON into downstream host dashboards when they consume Flow status",
        ),
        item(
            "guardrails",
            "Completion guardrails prevent accidental 100% claims",
            10,
            CompletionItemStatus::Done,
            "unit tests assert planned items keep a fixture set below 100/100",
            "open the next 100-point set instead of stretching this one",
        ),
    ];

    CompletionSet {
        name: "Completion Control Loop".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "When this set reaches 100/100, open the next set instead of declaring the whole product finished.".to_string(),
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
    fn active_set_is_not_prematurely_complete() {
        let set = active_completion_set();
        assert_eq!(
            set.current_score_out_of_100,
            set.target_score_out_of_100
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
