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
            "feature-map",
            "Competitive Friday feature inventory",
            19,
            CompletionItemStatus::Done,
            "Friday now has a Rust capability map covering ChatGPT, Gemini, Perplexity, Grok, and Claude feature targets with routes, statuses, and local-first boundaries",
            "keep the matrix updated as features ship so progress stays honest",
        ),
        item(
            "metasearch-search-policy",
            "Metasearch-first cited search and research policy",
            20,
            CompletionItemStatus::Done,
            "Friday search and deep-research plans explicitly route through the adjacent metasearch Rust crate and forbid Perplexity Computer as a dependency",
            "wire the plan to the real metasearch execution API and citation store",
        ),
        item(
            "research-source-runtime",
            "Research source, citation, and export runtime",
            12,
            CompletionItemStatus::Done,
            "Friday now has a metasearch-backed research workflow, local metasearch API client, source groups, citation ledgers, progress events, markdown reports, and persisted report bundles",
            "connect this persisted contract to the Friday UI Research surface",
        ),
        item(
            "ask-research-streaming",
            "Ask and Research local-first streaming synthesis",
            8,
            CompletionItemStatus::Done,
            "Friday can gather sources, build a citation-aware synthesis prompt, run the local quality-chat model, and expose answer deltas with citation references",
            "connect the synthesis deltas to the Friday Ask and Research UI surfaces",
        ),
        item(
            "projects-memory-connectors",
            "Projects, memory, and connector workspace state",
            13,
            CompletionItemStatus::Done,
            "Friday now has durable local project, memory, and connector store records with permission findings and separate JSON persistence for UI consumption",
            "connect the workspace store to the Friday sidebar pages and auth/provider settings",
        ),
        item(
            "canvas-artifacts-code",
            "Canvas, Artifacts, and coding-agent workspace",
            14,
            CompletionItemStatus::Planned,
            "Codex/Zed/ZeroClaw adapters and UI generation paths exist, but Friday needs an editable artifact canvas, previews, diffs, and checkpoints",
            "build artifact storage plus editable docs/code/UI previews and code-agent task checkpoints",
        ),
        item(
            "voice-multimodal-automations",
            "Voice, multimodal, and automation parity",
            14,
            CompletionItemStatus::Planned,
            "WhisperFlow-beater STT/TTS/wake foundations and OCR planning exist, but duplex voice, multimodal execution, and scheduled Friday jobs need product wiring",
            "connect Voice, Multimodal, and Automations pages to local runtime, OCR/VLM plans, scheduler, and audit records",
        ),
    ];

    CompletionSet {
        name: "Friday Competitive AI Workspace".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Beat the major AI assistant surfaces by shipping Friday features in local-first slices: Ask, Search, Research, Agents, Canvas, Projects, Memory, Connectors, Voice, Artifacts, Automations, Code, and Multimodal.".to_string(),
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
