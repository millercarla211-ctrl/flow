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
            "ask-search-research-ui",
            "Ask, Search, and Research UI wiring",
            20,
            CompletionItemStatus::Done,
            "Friday now has route-level UI contracts for Ask, Search, and Research with streaming flags, citations, source controls, saved reports, and local command/data bindings",
            "connect these contracts to the desktop/Next.js panes and interaction handlers",
        ),
        item(
            "store-backed-pages",
            "Store-backed Friday pages",
            25,
            CompletionItemStatus::Done,
            "Friday now has page-level UI bindings for Projects, Memory, Connectors, Agents, Canvas, Artifacts, Code, Voice, Multimodal, and Automations against the durable local stores",
            "connect these contracts to the visible desktop/Next.js page components",
        ),
        item(
            "local-execution-checks",
            "End-to-end local execution checks",
            20,
            CompletionItemStatus::Done,
            "Friday now exposes low-resource local execution checks for STT, TTS, OCR, metasearch request paths, artifact previews, and runtime records without loading heavy models",
            "connect the check report to the visible desktop diagnostics panel",
        ),
        item(
            "route-states",
            "Production route states",
            20,
            CompletionItemStatus::Done,
            "Every Friday route now exposes production empty, loading, error, permission, and ready state contracts with tone, visibility, blocking behavior, and recovery commands",
            "connect these route-state contracts to the visible desktop/Next.js components",
        ),
        item(
            "browser-verification-deploy-gate",
            "Browser verification and deploy gate",
            15,
            CompletionItemStatus::Planned,
            "The deploy rule exists in the TODO, but this loop still needs targeted browser verification and major-feature-only deploy discipline",
            "verify the visible product routes and deploy only after a major user-visible feature ships",
        ),
    ];

    CompletionSet {
        name: "Friday Product UI Integration".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Turn Friday's completed Rust contracts into polished product UI, verified live workflows, and production-ready route states without weakening local-first defaults.".to_string(),
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
    fn active_set_tracks_incomplete_ui_loop() {
        let set = active_completion_set();
        assert!(set.current_score_out_of_100 < set.target_score_out_of_100);
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
