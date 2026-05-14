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
            "automation-audit",
            "Host automation records read and replace actions",
            25,
            CompletionItemStatus::Done,
            "FlowAutomationEngine records ReadSelection and ReplaceSelection audit entries during rewrite-selection flows",
            "persist host audit summaries into release/runtime handoff surfaces",
        ),
        item(
            "accessibility-diagnostics",
            "Production-grade desktop accessibility diagnostics",
            25,
            CompletionItemStatus::Done,
            "`flow --accessibility [os] [--dry-run]` reports backend, mode, readiness, notes, and fix actions",
            "wire accessibility diagnostics into desktop host UI surfaces",
        ),
        item(
            "pause-resume-snooze",
            "Reliable pause, resume, and snooze controls for always-on hosts",
            20,
            CompletionItemStatus::Done,
            "FlowDefaultHostKit and FlowRuntimeSupervisor expose pause, snooze, resume, refresh, and pause snapshots",
            "wire pause controls into concrete desktop tray/overlay hosts",
        ),
        item(
            "global-dictation-host",
            "Harden Windows global dictation beyond CLI/demo paths",
            20,
            CompletionItemStatus::Planned,
            "dictation CLI and local runtime paths exist, but production host behavior needs more hardening",
            "connect host health, activation, overlay, and focused-input replacement into one checked path",
        ),
        item(
            "persistent-audit",
            "Persist native automation audit logs for host review",
            10,
            CompletionItemStatus::Planned,
            "session state stores approvals, but host audit review is still in-memory only",
            "export compact audit records through persistent state and release diagnostics",
        ),
    ];

    CompletionSet {
        name: "Host Autonomy Core".to_string(),
        target_score_out_of_100: 100,
        current_score_out_of_100: score_items(&items),
        loop_rule: "Complete the host autonomy set, then open the next 100-point set instead of declaring the whole product finished.".to_string(),
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
