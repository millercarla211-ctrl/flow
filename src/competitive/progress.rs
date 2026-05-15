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
            "release-deployment-gate-model",
            "Typed release deployment-gate model",
            20,
            CompletionItemStatus::Done,
            "`FridayReleaseDeploymentGateReport` consumes export-kit, QA, checklist, package, timeline, dashboard state, and target metadata",
            "open the next release candidate archive set",
        ),
        item(
            "release-deployment-gate-reasons",
            "No-deploy reason categories",
            20,
            CompletionItemStatus::Done,
            "deployment gates classify missing evidence, stale checks, blocked QA, unsigned releases, dashboard state, and target mismatch as warning or blocking reasons",
            "open the next release candidate archive set",
        ),
        item(
            "release-deployment-gate-cli",
            "Deployment-gate CLI and JSON commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-release-deployment-gate` writes the go/no-go report and `--friday-release-deployment-gate-json` previews it without running builds or deployments",
            "open the next release candidate archive set",
        ),
        item(
            "release-deployment-gate-dashboard",
            "Dashboard deployment-gate rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports deployment-gate JSON, renders go/no-go status, target profile, rollback note, reasons, and deploy checklist",
            "open the next release candidate archive set",
        ),
        item(
            "release-deployment-gate-coverage",
            "Deployment-gate Rust and TypeScript coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage and dashboard smoke checks verify no-go scoring, target policy, operator copy, commands, and UI normalization",
            "open the next release candidate archive set",
        ),
    ];

    CompletionSet {
        name: "Friday Release Deployment Gate".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Turn release evidence, QA, checklist, dashboard state, and deployment target metadata into one explicit local-first go/no-go gate before major Friday checkpoints.".to_string(),
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
    fn active_set_tracks_friday_release_deployment_gate_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Release Deployment Gate");
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
