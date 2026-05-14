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
            "rewrite-explanations",
            "Deeper grammar, clarity, and rewrite explanations",
            30,
            CompletionItemStatus::Done,
            "Typing and text-command results now expose structured change explanations with grammar, tone, concision, terminology, snippet, domain, style, and formatting categories",
            "keep explanation contracts stable while adding citation, fact-check, and academic execution paths",
        ),
        item(
            "citation-factcheck",
            "Citation, fact-checking, and academic assistance paths",
            25,
            CompletionItemStatus::Done,
            "FlowProofingPlanner now exposes a local-first academic review report with citation needs, claim reviews, fact-check risk, supplied-source matching, and source-overlap warnings",
            "keep the academic review API local-first while adding durable style-guide and brand-tone policies",
        ),
        item(
            "style-guide-policy",
            "Style-guide and brand-tone policy enforcement",
            20,
            CompletionItemStatus::Planned,
            "style presets exist, but durable brand and team policy enforcement is not implemented",
            "add reusable style-guide policies with checked violations and suggested rewrites",
        ),
        item(
            "multilingual-writing",
            "Multilingual writing assistance",
            15,
            CompletionItemStatus::Planned,
            "workspace language fields exist, but multilingual rewrite and normalization are not first-class yet",
            "add language-aware dictation cleanup, rewrite, and dictionary normalization",
        ),
        item(
            "writing-host-surface",
            "Host-facing writing review surface",
            10,
            CompletionItemStatus::Planned,
            "host dictation insertion is checked, but writing review summaries need a host-friendly handoff",
            "surface writing issues, diffs, and accept/reject actions through the embedded host path",
        ),
    ];

    CompletionSet {
        name: "Writing Quality Core".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Complete the writing-quality set, then open the next 100-point set instead of declaring the whole product finished.".to_string(),
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
