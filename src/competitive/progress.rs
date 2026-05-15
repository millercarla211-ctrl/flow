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
            "runner-release-package-report",
            "Typed trusted runner release package report",
            20,
            CompletionItemStatus::Done,
            "`FridayTrustedRunnerReleasePackageReport` merges runner history review, cancellation UX, live state, release-review links, and incident markdown",
            "open the next trusted runner evidence timeline set",
        ),
        item(
            "runner-release-package-manifest",
            "Local-only manifest with checksums",
            20,
            CompletionItemStatus::Done,
            "the release package manifest records local-only evidence paths, byte sizes, SHA-256 checksums, missing counts, warning counts, and a package signature",
            "open the next trusted runner evidence timeline set",
        ),
        item(
            "runner-release-package-cli",
            "CLI and JSON package commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-trusted-host-runner-release-package` writes the package and `--friday-trusted-host-runner-release-package-json` previews it without running host commands",
            "open the next trusted runner evidence timeline set",
        ),
        item(
            "runner-release-package-dashboard",
            "Dashboard package import rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports release packages, renders readiness, warnings, and evidence-file checksum rows",
            "open the next trusted runner evidence timeline set",
        ),
        item(
            "runner-release-package-tests",
            "Package completeness coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage and dashboard smoke checks verify package completeness, checksum presence, missing evidence, and stale warnings",
            "open the next trusted runner evidence timeline set",
        ),
    ];

    CompletionSet {
        name: "Friday Trusted Runner Release Package".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Package trusted-runner evidence into one signed, reviewable, local-only release handoff with manifest links, incident markdown, live-state freshness, and CLI/browser import guidance.".to_string(),
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
    fn active_set_tracks_friday_trusted_runner_release_package_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Trusted Runner Release Package");
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
