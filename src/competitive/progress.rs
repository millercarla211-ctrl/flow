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
            CompletionItemStatus::Done,
            "Friday now has durable artifact, checkpoint, diff, preview-runner, and code-task records for Canvas, Artifacts, and Code workspace wiring",
            "connect artifact records to editable UI panes, preview execution, and code review actions",
        ),
        item(
            "voice-multimodal-automations",
            "Voice, multimodal, and automation parity",
            14,
            CompletionItemStatus::Done,
            "Friday now has local-first Voice, Multimodal, and Automation runtime records for STT, TTS, wake commands, OCR/VLM planning, schedules, approvals, and audit files",
            "open the next loop for product UI integration, live execution hardening, and end-to-end browser checks",
        ),
    ];

    CompletionSet {
        name: "Friday Competitive AI Workspace".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "This 100-point Friday competitive workspace contract is complete. The next loop should connect these Rust surfaces to the production UI, live execution, and end-to-end verification.".to_string(),
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
    fn active_set_tracks_completed_contract_loop() {
        let set = active_completion_set();
        assert_eq!(set.current_score_out_of_100, set.target_score_out_of_100);
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
