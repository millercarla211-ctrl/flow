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
            "release-candidate-archive-model",
            "Typed release candidate archive model",
            20,
            CompletionItemStatus::Done,
            "`FridayReleaseCandidateArchive` stores deployment gate path, export kit path/checksum, target profile, score, decision, blocker ids, and rollback note",
            "add dashboard candidate-history rendering",
        ),
        item(
            "release-candidate-archive-diffs",
            "Candidate diff summaries",
            20,
            CompletionItemStatus::Done,
            "candidate diffs track score deltas, decision changes, target changes, evidence checksum changes, new blockers, resolved blockers, and regressions",
            "add dashboard candidate-history rendering",
        ),
        item(
            "release-candidate-archive-cli",
            "Candidate archive CLI and JSON commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-release-candidate-archive` appends gates and `--friday-release-candidate-archive-json` previews candidate history without running builds or deployments",
            "add dashboard candidate-history rendering",
        ),
        item(
            "release-candidate-archive-dashboard",
            "Dashboard candidate archive rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports candidate archive JSON and renders latest candidate, go/no-go counts, regressions, candidate cards, diff summaries, and copyable archive command",
            "open the next release promotion ledger set",
        ),
        item(
            "release-candidate-archive-coverage",
            "Candidate archive Rust and TypeScript coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage plus dashboard smoke checks verify archive writes, diff warnings, command copy, and visible dashboard import/rendering",
            "open the next release promotion ledger set",
        ),
    ];

    CompletionSet {
        name: "Friday Release Candidate Archive".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Preserve every Friday major-checkpoint candidate as comparable local records with deployment gates, evidence kits, target metadata, rollback notes, and promotion history.".to_string(),
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
    fn active_set_tracks_friday_release_candidate_archive_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Release Candidate Archive");
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
