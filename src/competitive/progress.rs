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
            "release-post-promotion-monitor-model",
            "Typed post-promotion monitor model",
            20,
            CompletionItemStatus::Done,
            "`FridayReleasePostPromotionMonitorReport` consumes the promotion ledger, release QA, dashboard smoke output, and incident-note evidence",
            "open the next rollback drill set",
        ),
        item(
            "release-post-promotion-monitor-warnings",
            "Post-promotion stale and missing evidence warnings",
            20,
            CompletionItemStatus::Done,
            "monitor checks classify passed, warning, failed, missing, and stale evidence while preserving active rollback references",
            "open the next rollback drill set",
        ),
        item(
            "release-post-promotion-monitor-cli",
            "Post-promotion monitor CLI and JSON commands",
            20,
            CompletionItemStatus::Done,
            "`flow --friday-release-post-promotion-monitor` and JSON mode generate local-only monitor reports without running deployments",
            "open the next rollback drill set",
        ),
        item(
            "release-post-promotion-monitor-dashboard",
            "Dashboard post-promotion monitor rendering",
            20,
            CompletionItemStatus::Done,
            "the visible dashboard imports monitor JSON and renders stability status, rollback readiness, incident notes, stale checks, and copyable monitor command",
            "open the next rollback drill set",
        ),
        item(
            "release-post-promotion-monitor-coverage",
            "Post-promotion monitor Rust and TypeScript coverage",
            20,
            CompletionItemStatus::Done,
            "focused Rust integration coverage plus dashboard smoke checks verify monitor scoring, stale warnings, incident-note evidence, command copy, and visible dashboard import/rendering",
            "open the next rollback drill set",
        ),
    ];

    CompletionSet {
        name: "Friday Release Post-Promotion Monitor".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Make Friday's post-promotion status visible with verification freshness, rollback readiness, incident notes, and follow-up evidence.".to_string(),
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
    fn active_set_tracks_friday_release_post_promotion_monitor_loop() {
        let set = active_completion_set();
        assert_eq!(set.name, "Friday Release Post-Promotion Monitor");
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
