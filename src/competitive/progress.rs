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
            "release-external-receipt-model",
            "Typed release external receipt archive model",
            20,
            CompletionItemStatus::Done,
            "`FridayReleaseExternalReceiptArchive` consumes outbound review ledgers and preserves operator-owned receipt evidence",
            "open the next Friday release receipt review board set",
        ),
        item(
            "release-external-receipt-states",
            "Draft, attached, verified, stale, missing, revoked, superseded, and blocked states",
            20,
            CompletionItemStatus::Done,
            "receipt records downgrade unsafe attached/verified requests to blocked while outbound reviews still carry blockers, and missing evidence stays explicit",
            "open the next Friday release receipt review board set",
        ),
        item(
            "release-external-receipt-cli",
            "Release external receipt CLI, JSON, list, and export commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-release-external-receipt` appends receipt evidence without fetching, sending, publishing, deploying, uploading, or emailing",
            "open the next Friday release receipt review board set",
        ),
        item(
            "release-external-receipt-dashboard",
            "Dashboard release external receipt archive rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports receipt archives and renders attached/verified/stale/missing/blocked counts, evidence paths, command copy, and audit notes",
            "open the next Friday release receipt review board set",
        ),
        item(
            "release-external-receipt-coverage",
            "Release external receipt Rust and TypeScript coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage plus dashboard smoke checks verify blocked reviews, no-fetch/no-send copy, receipt archive import normalization, and dashboard rendering",
            "open the next Friday release receipt review board set",
        ),
    ];

    CompletionSet {
        name: "Friday Release External Receipt Archive".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Preserve human-owned publication, send, deploy, upload, or announcement receipts as local evidence without fetching or mutating external systems.".to_string(),
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
    fn active_set_tracks_friday_release_external_receipt_archive_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Release External Receipt Archive");
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
