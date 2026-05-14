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
            "multimodal-readiness-gate",
            "Local multimodal readiness gate",
            20,
            CompletionItemStatus::Done,
            "Friday local checks now verify OCR artifacts, STT/TTS readiness, metasearch wiring, and artifact preview contracts without loading heavy models",
            "connect these readiness checks to visible Multimodal and Voice route diagnostics",
        ),
        item(
            "ocr-execution-smoke",
            "OCR execution smoke path",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-ocr-smoke <dir> [image] [--execute]` writes a bounded OCR smoke markdown file, artifact JSON, checkpoint JSON, and report JSON without loading the model unless explicitly requested",
            "connect OCR smoke reports to visible Multimodal route diagnostics and richer artifact metadata",
        ),
        item(
            "vlm-screenshot-path",
            "VLM screenshot understanding path",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-vlm-contract <dir> [screenshot] [prompt]` writes a local-only screenshot understanding contract with explicit VLM model files, prompt, artifact, checkpoint, and missing-model findings",
            "surface VLM contract reports in the Multimodal route and connect them to real screenshot capture",
        ),
        item(
            "media-routing-policy",
            "Image, audio, and video routing policy",
            20,
            CompletionItemStatus::Planned,
            "The catalog contains multimodal candidates, but Friday still needs explicit local-first promotion, demotion, and missing-model handling for media tasks",
            "add model-role routing rules for image, audio, video, OCR, and VLM requests",
        ),
        item(
            "multimodal-artifact-metadata",
            "Publish-ready multimodal artifact metadata",
            20,
            CompletionItemStatus::Planned,
            "Artifact preview contracts exist, but multimodal outputs need richer metadata for source image/audio/video, model, prompt, timings, and confidence",
            "extend artifact records for OCR/VLM/media outputs and add tests for metadata round trips",
        ),
    ];

    CompletionSet {
        name: "Multimodal Local Core".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Finish Friday's local OCR, VLM, image, audio, and video foundations with explicit low-resource routing and artifact metadata before adding heavier UI polish.".to_string(),
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
        assert_eq!(set.name, "Multimodal Local Core");
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
