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
            "extension-smoke-matrix",
            "Packaged extension smoke matrix",
            20,
            CompletionItemStatus::Done,
            "`flow --browser-extension-smoke` verifies Chromium, Edge, Firefox, and Safari package/dist readiness and detects installed browser targets without network access",
            "wire the smoke report into live launch automation for installed browsers",
        ),
        item(
            "live-browser-launch-smoke",
            "Live browser launch smoke",
            20,
            CompletionItemStatus::Done,
            "`flow --browser-extension-launch-smoke --execute` runs bounded temporary-profile launch smoke for installed Chromium-family targets and reports unsupported/missing browsers explicitly",
            "connect the launch report to release handoff exports and add Firefox-specific live smoke support when Firefox is installed",
        ),
        item(
            "offline-browser-pack-reuse",
            "Offline browser-pack reuse",
            20,
            CompletionItemStatus::Done,
            "`flow --browser-pack-reuse-smoke` simulates cached browser packs and verifies local-only execution planning with `remote_allowed=false` and local browserpack URLs",
            "connect this offline reuse contract to extension storage recovery fixtures",
        ),
        item(
            "browser-pack-recovery",
            "Browser-pack recovery",
            20,
            CompletionItemStatus::Planned,
            "The extension storage layer verifies hashes, but partial-download resume, hash rejection, and quota recovery need operator tests",
            "add corruption and partial-cache test fixtures for browser pack storage",
        ),
        item(
            "chromium-webllm-acceleration",
            "Chromium WebLLM acceleration",
            20,
            CompletionItemStatus::Planned,
            "Transformers.js browser packs are the validated baseline; optional WebLLM acceleration remains gated until Qwen packs are stable",
            "add a Chromium-only WebLLM plan after offline pack reuse is verified",
        ),
    ];

    CompletionSet {
        name: "Browser And Extension Core".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Harden Friday's browser extension release path with packaged smoke checks, installed-browser launch validation, offline browser-pack reuse, recovery handling, and optional Chromium acceleration without weakening local-first defaults.".to_string(),
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
    fn active_set_tracks_browser_extension_core_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Browser And Extension Core");
        assert_eq!(set.current_score_out_of_100, 60);
        assert!(
            set.items
                .iter()
                .any(|item| item.status == CompletionItemStatus::Planned)
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
