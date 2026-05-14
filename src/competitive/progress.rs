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
            "multimodal-ui-diagnostics",
            "Multimodal UI diagnostics",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-multimodal-diagnostics` connects OCR smoke reports, VLM contracts, routing policy, and metadata sidecars to the Multimodal UI contract",
            "render diagnostics in the desktop/Next.js Multimodal page and show latest artifact bundle links",
        ),
        item(
            "screenshot-capture-contract",
            "Screenshot capture command",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-screenshot-vlm <dir> <screenshot> [prompt]` validates a local screenshot file, records source metadata, and feeds the VLM contract without loading the model",
            "connect this handoff to real desktop/browser capture controls",
        ),
        item(
            "image-video-affordances",
            "Image and video install/run affordances",
            20,
            CompletionItemStatus::Planned,
            "Image and video routes are explicit but still planned, so operators need install/run affordances before the UI can expose them safely",
            "add explicit install/run affordances for image and video model candidates",
        ),
        item(
            "artifact-store-metadata",
            "Persist multimodal metadata in artifact store",
            20,
            CompletionItemStatus::Planned,
            "OCR and VLM commands write metadata sidecars, but the durable Friday artifact store does not yet index those sidecars for UI browsing",
            "persist multimodal artifact metadata in the Friday artifact store",
        ),
        item(
            "multimodal-visual-check",
            "Multimodal route visual checks",
            20,
            CompletionItemStatus::Planned,
            "Friday has browser-gate contracts, but the Multimodal route still needs a focused visual check after diagnostics are wired into the UI",
            "add browser or desktop visual checks for the Multimodal UI route",
        ),
    ];

    CompletionSet {
        name: "Multimodal Product Execution".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Connect Friday's multimodal contracts to UI capture, install/run affordances, persisted artifacts, and visual checks without changing the low-resource idle defaults.".to_string(),
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
        assert_eq!(set.name, "Multimodal Product Execution");
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
