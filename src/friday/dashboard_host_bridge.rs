use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardActionKind, FridayDashboardProductUiActionBinding,
    friday_dashboard_product_ui_binding_from_export,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayDashboardHostCommandStatus {
    AwaitingApproval,
    Blocked,
}

impl FridayDashboardHostCommandStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::AwaitingApproval => "awaiting-approval",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayDashboardHostApprovalState {
    Required,
    Blocked,
}

impl FridayDashboardHostApprovalState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardHostCommandAudit {
    pub action_id: String,
    pub event: String,
    pub stdout_summary: String,
    pub stderr_summary: String,
    pub duration_ms: u64,
    pub recorded_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardHostCommandRecord {
    pub action_id: String,
    pub card_id: String,
    pub label: String,
    pub kind: FridayDashboardActionKind,
    pub command: String,
    pub status: FridayDashboardHostCommandStatus,
    pub approval_state: FridayDashboardHostApprovalState,
    pub local_only: bool,
    pub destructive: bool,
    pub requires_confirmation: bool,
    pub silent_execution_allowed: bool,
    pub can_execute_after_approval: bool,
    pub blocked_reason: Option<String>,
    pub approval_prompt: String,
    pub audit: FridayDashboardHostCommandAudit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDashboardHostCommandBridgeReport {
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub route: String,
    pub export_dir: String,
    pub source_command: String,
    pub summary: String,
    pub command_count: usize,
    pub awaiting_approval_count: usize,
    pub blocked_count: usize,
    pub audit_count: usize,
    pub records: Vec<FridayDashboardHostCommandRecord>,
    pub next_actions: Vec<String>,
}

impl FridayDashboardHostCommandBridgeReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_dashboard_host_command_bridge_from_export(
    export_dir: impl AsRef<Path>,
) -> Result<FridayDashboardHostCommandBridgeReport> {
    let binding = friday_dashboard_product_ui_binding_from_export(export_dir.as_ref())?;
    let generated_at_unix_ms = unix_ms();
    let records = binding
        .action_bindings
        .iter()
        .map(|action| {
            friday_dashboard_host_command_record_from_action(action, generated_at_unix_ms)
        })
        .collect::<Vec<_>>();
    let awaiting_approval_count = records
        .iter()
        .filter(|record| record.status == FridayDashboardHostCommandStatus::AwaitingApproval)
        .count();
    let blocked_count = records
        .iter()
        .filter(|record| record.status == FridayDashboardHostCommandStatus::Blocked)
        .count();

    Ok(FridayDashboardHostCommandBridgeReport {
        generated_at_unix_ms,
        product_name: binding.product_name,
        route: binding.route,
        export_dir: binding.export_dir.clone(),
        source_command: format!(
            "flow --friday-dashboard-product-ui-json {}",
            binding.export_dir
        ),
        summary: format!(
            "{} dashboard command handoff(s) prepared for explicit host approval; {} blocked before execution.",
            awaiting_approval_count, blocked_count
        ),
        command_count: records.len(),
        awaiting_approval_count,
        blocked_count,
        audit_count: records.len(),
        records,
        next_actions: vec![
            "Connect these records to the trusted desktop/Tauri command runner.".to_string(),
            "Keep every command behind explicit operator approval before process execution."
                .to_string(),
            "Write real stdout/stderr and duration summaries after the trusted host runner executes an approved command."
                .to_string(),
        ],
    })
}

pub fn friday_dashboard_host_command_record_from_action(
    action: &FridayDashboardProductUiActionBinding,
    generated_at_unix_ms: u128,
) -> FridayDashboardHostCommandRecord {
    let blocked_reason = blocked_reason(action);
    let status = if blocked_reason.is_some() {
        FridayDashboardHostCommandStatus::Blocked
    } else {
        FridayDashboardHostCommandStatus::AwaitingApproval
    };
    let approval_state = if blocked_reason.is_some() {
        FridayDashboardHostApprovalState::Blocked
    } else {
        FridayDashboardHostApprovalState::Required
    };
    let event = match status {
        FridayDashboardHostCommandStatus::AwaitingApproval => "prepared-for-approval",
        FridayDashboardHostCommandStatus::Blocked => "blocked-before-approval",
    };
    let stdout_summary = match status {
        FridayDashboardHostCommandStatus::AwaitingApproval => {
            "not executed; waiting for operator approval".to_string()
        }
        FridayDashboardHostCommandStatus::Blocked => "not executed".to_string(),
    };
    let stderr_summary = blocked_reason.clone().unwrap_or_default();

    FridayDashboardHostCommandRecord {
        action_id: action.action_id.clone(),
        card_id: action.card_id.clone(),
        label: action.label.clone(),
        kind: action.kind,
        command: action.command.clone(),
        status,
        approval_state,
        local_only: action.local_only,
        destructive: action.button_state.destructive,
        requires_confirmation: action.button_state.requires_confirmation,
        silent_execution_allowed: false,
        can_execute_after_approval: status == FridayDashboardHostCommandStatus::AwaitingApproval,
        blocked_reason,
        approval_prompt: format!(
            "Approve trusted desktop execution for `{}`?",
            action.command
        ),
        audit: FridayDashboardHostCommandAudit {
            action_id: action.action_id.clone(),
            event: event.to_string(),
            stdout_summary,
            stderr_summary,
            duration_ms: 0,
            recorded_at_unix_ms: generated_at_unix_ms,
        },
    }
}

fn blocked_reason(action: &FridayDashboardProductUiActionBinding) -> Option<String> {
    if !action.local_only {
        return Some("Remote dashboard commands are not allowed in local-only mode.".to_string());
    }
    if action.button_state.destructive {
        return Some(
            "Destructive dashboard commands need a dedicated desktop approval flow.".to_string(),
        );
    }
    if !action.enabled || action.button_state.disabled {
        return Some(
            action
                .button_state
                .disabled_reason
                .clone()
                .unwrap_or_else(|| "Dashboard action is disabled.".to_string()),
        );
    }
    if action.command.trim().is_empty() {
        return Some("Dashboard action command is empty.".to_string());
    }
    None
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
