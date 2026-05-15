use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardHostApprovalState, FridayDashboardHostCommandRecord,
    FridayDashboardHostCommandStatus,
};

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_OUTPUT_LIMIT_BYTES: usize = 4_096;
const HISTORY_LIMIT: usize = 50;
const LIVE_STATE_LIMIT: usize = 50;
const DEFAULT_STALE_AFTER_MS: u128 = 120_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayTrustedHostRunnerStatus {
    Succeeded,
    Failed,
    TimedOut,
    Cancelled,
    Denied,
}

impl FridayTrustedHostRunnerStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::TimedOut => "timed-out",
            Self::Cancelled => "cancelled",
            Self::Denied => "denied",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerRequest {
    pub approved: bool,
    pub cancel_requested: bool,
    pub timeout_ms: u64,
    pub stdout_limit_bytes: usize,
    pub stderr_limit_bytes: usize,
    pub operator_reason: Option<String>,
}

impl Default for FridayTrustedHostRunnerRequest {
    fn default() -> Self {
        Self {
            approved: false,
            cancel_requested: false,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            stdout_limit_bytes: DEFAULT_OUTPUT_LIMIT_BYTES,
            stderr_limit_bytes: DEFAULT_OUTPUT_LIMIT_BYTES,
            operator_reason: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostCommandRawOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub timed_out: bool,
}

pub trait FridayTrustedHostCommandExecutor {
    fn execute(
        &self,
        command: &str,
        timeout_ms: u64,
    ) -> std::result::Result<FridayTrustedHostCommandRawOutput, String>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FridayProcessTrustedHostCommandExecutor;

impl FridayTrustedHostCommandExecutor for FridayProcessTrustedHostCommandExecutor {
    fn execute(
        &self,
        command: &str,
        timeout_ms: u64,
    ) -> std::result::Result<FridayTrustedHostCommandRawOutput, String> {
        execute_local_flow_command(command, timeout_ms)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerResult {
    pub action_id: String,
    pub card_id: String,
    pub label: String,
    pub command: String,
    pub status: FridayTrustedHostRunnerStatus,
    pub exit_code: Option<i32>,
    pub stdout_summary: String,
    pub stderr_summary: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub duration_ms: u64,
    pub timeout_ms: u64,
    pub approved: bool,
    pub cancelled: bool,
    pub operator_reason: Option<String>,
    pub audit_event: String,
    pub recorded_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerHistory {
    pub history_json: String,
    pub result_count: usize,
    pub latest: Option<FridayTrustedHostRunnerResult>,
    pub records: Vec<FridayTrustedHostRunnerResult>,
}

impl FridayTrustedHostRunnerHistory {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerStatusSummary {
    pub status: FridayTrustedHostRunnerStatus,
    pub count: usize,
    pub title: String,
    pub description: String,
    pub tone: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerAffordance {
    pub id: String,
    pub kind: String,
    pub action_id: String,
    pub status: FridayTrustedHostRunnerStatus,
    pub label: String,
    pub command: String,
    pub detail: String,
    pub requires_approval: bool,
    pub disabled: bool,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerOperatorNote {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub release_review_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerUxReport {
    pub history_json: String,
    pub result_count: usize,
    pub latest_status: Option<FridayTrustedHostRunnerStatus>,
    pub status_summaries: Vec<FridayTrustedHostRunnerStatusSummary>,
    pub affordances: Vec<FridayTrustedHostRunnerAffordance>,
    pub operator_notes: Vec<FridayTrustedHostRunnerOperatorNote>,
}

impl FridayTrustedHostRunnerUxReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerKeyboardShortcut {
    pub key: String,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerApprovalControl {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub command: String,
    pub detail: String,
    pub aria_label: String,
    pub keyboard_shortcut: Option<FridayTrustedHostRunnerKeyboardShortcut>,
    pub requires_reason: bool,
    pub requires_approval: bool,
    pub disabled: bool,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerSnoozeOption {
    pub id: String,
    pub label: String,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerApprovalUiReport {
    pub history_json: String,
    pub result_count: usize,
    pub modal_id: String,
    pub latest_action_id: Option<String>,
    pub title: String,
    pub body: String,
    pub command_preview: String,
    pub reason_label: String,
    pub reason_placeholder: String,
    pub audit_reason_required: bool,
    pub controls: Vec<FridayTrustedHostRunnerApprovalControl>,
    pub snooze_options: Vec<FridayTrustedHostRunnerSnoozeOption>,
    pub undo_note: String,
    pub release_review_path: String,
}

impl FridayTrustedHostRunnerApprovalUiReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayTrustedHostLiveRunnerStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    TimedOut,
    Cancelled,
    Denied,
    Stale,
}

impl FridayTrustedHostLiveRunnerStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::TimedOut => "timed-out",
            Self::Cancelled => "cancelled",
            Self::Denied => "denied",
            Self::Stale => "stale",
        }
    }

    pub fn is_active(self) -> bool {
        matches!(self, Self::Pending | Self::Running)
    }

    pub fn is_finished(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::TimedOut | Self::Cancelled | Self::Denied
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostLiveRunnerRecord {
    pub job_id: String,
    pub action_id: String,
    pub label: String,
    pub command: String,
    pub status: FridayTrustedHostLiveRunnerStatus,
    pub message: String,
    pub local_only: bool,
    pub approved: bool,
    pub timeout_ms: u64,
    pub stale_after_ms: u128,
    pub created_at_unix_ms: u128,
    pub updated_at_unix_ms: u128,
    pub finished_at_unix_ms: Option<u128>,
    pub history_json: Option<String>,
    pub recovery_command: String,
    pub cleanup_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostLiveRunnerState {
    pub state_json: String,
    pub generated_at_unix_ms: u128,
    pub record_count: usize,
    pub pending_count: usize,
    pub running_count: usize,
    pub finished_count: usize,
    pub stale_count: usize,
    pub records: Vec<FridayTrustedHostLiveRunnerRecord>,
    pub stale_recovery_copy: String,
}

impl FridayTrustedHostLiveRunnerState {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerCancellationControl {
    pub id: String,
    pub job_id: String,
    pub action_id: String,
    pub kind: String,
    pub label: String,
    pub command: String,
    pub detail: String,
    pub requires_reason: bool,
    pub disabled: bool,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerCancellationDraft {
    pub storage_key: String,
    pub default_reason: String,
    pub autosave_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerCancellationUxReport {
    pub state_json: String,
    pub record_count: usize,
    pub active_count: usize,
    pub stale_count: usize,
    pub denial_count: usize,
    pub controls: Vec<FridayTrustedHostRunnerCancellationControl>,
    pub draft: FridayTrustedHostRunnerCancellationDraft,
    pub guidance: Vec<String>,
}

impl FridayTrustedHostRunnerCancellationUxReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerOperatorReviewFilter {
    pub status: Option<FridayTrustedHostRunnerStatus>,
    pub action_id: Option<String>,
    pub since_unix_ms: Option<u128>,
    pub until_unix_ms: Option<u128>,
    pub limit: usize,
}

impl Default for FridayTrustedHostRunnerOperatorReviewFilter {
    fn default() -> Self {
        Self {
            status: None,
            action_id: None,
            since_unix_ms: None,
            until_unix_ms: None,
            limit: HISTORY_LIMIT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerReviewRecord {
    pub result_id: String,
    pub action_id: String,
    pub label: String,
    pub status: FridayTrustedHostRunnerStatus,
    pub severity: String,
    pub command: String,
    pub summary: String,
    pub release_gate: String,
    pub operator_reason: Option<String>,
    pub recorded_at_unix_ms: u128,
    pub duration_ms: u64,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerReleaseGateSummary {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub count: usize,
    pub detail: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerIncidentNote {
    pub id: String,
    pub action_id: String,
    pub status: FridayTrustedHostRunnerStatus,
    pub severity: String,
    pub title: String,
    pub body: String,
    pub export_markdown: String,
    pub recorded_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerOperatorReviewReport {
    pub history_json: String,
    pub review_id: String,
    pub generated_at_unix_ms: u128,
    pub filters: FridayTrustedHostRunnerOperatorReviewFilter,
    pub record_count: usize,
    pub matched_count: usize,
    pub ready_count: usize,
    pub blocked_count: usize,
    pub release_gate_status: String,
    pub release_gate_summaries: Vec<FridayTrustedHostRunnerReleaseGateSummary>,
    pub incident_notes: Vec<FridayTrustedHostRunnerIncidentNote>,
    pub records: Vec<FridayTrustedHostRunnerReviewRecord>,
}

impl FridayTrustedHostRunnerOperatorReviewReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerCancellationToken {
    pub cancel_requested: bool,
    pub reason: Option<String>,
}

impl FridayTrustedHostRunnerCancellationToken {
    pub fn none() -> Self {
        Self {
            cancel_requested: false,
            reason: None,
        }
    }

    pub fn requested(reason: impl Into<String>) -> Self {
        Self {
            cancel_requested: true,
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerBridgeEvent {
    pub event_id: String,
    pub status: FridayTrustedHostLiveRunnerStatus,
    pub message: String,
    pub state_json: String,
    pub record: FridayTrustedHostLiveRunnerRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayTrustedHostRunnerBridgeReport {
    pub state_json: String,
    pub history_json: String,
    pub event_count: usize,
    pub dashboard_import_guidance: String,
    pub events: Vec<FridayTrustedHostRunnerBridgeEvent>,
    pub live_state: FridayTrustedHostLiveRunnerState,
    pub history: FridayTrustedHostRunnerHistory,
    pub result: FridayTrustedHostRunnerResult,
}

impl FridayTrustedHostRunnerBridgeReport {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_trusted_host_runner_ux_report(
    history: &FridayTrustedHostRunnerHistory,
    release_review_path: impl AsRef<Path>,
) -> FridayTrustedHostRunnerUxReport {
    let release_review_path = path_string(release_review_path.as_ref());
    let status_summaries = [
        FridayTrustedHostRunnerStatus::Succeeded,
        FridayTrustedHostRunnerStatus::Failed,
        FridayTrustedHostRunnerStatus::TimedOut,
        FridayTrustedHostRunnerStatus::Cancelled,
        FridayTrustedHostRunnerStatus::Denied,
    ]
    .into_iter()
    .map(|status| runner_status_summary(history, status))
    .collect::<Vec<_>>();
    let affordances = history
        .records
        .iter()
        .take(5)
        .flat_map(runner_affordances)
        .collect::<Vec<_>>();
    let operator_notes = vec![
        FridayTrustedHostRunnerOperatorNote {
            id: "release-review-link".to_string(),
            label: "Release review".to_string(),
            detail: format!(
                "Trusted runner history has {} record(s); attach this history to release review before shipping host execution changes.",
                history.result_count
            ),
            release_review_path: release_review_path.clone(),
        },
        FridayTrustedHostRunnerOperatorNote {
            id: "approval-boundary".to_string(),
            label: "Approval boundary".to_string(),
            detail: "Retry actions keep explicit approval requirements; the dashboard must not auto-run a retry."
                .to_string(),
            release_review_path,
        },
    ];

    FridayTrustedHostRunnerUxReport {
        history_json: history.history_json.clone(),
        result_count: history.result_count,
        latest_status: history.latest.as_ref().map(|result| result.status),
        status_summaries,
        affordances,
        operator_notes,
    }
}

pub fn friday_trusted_host_runner_ux_report_from_history_file(
    history_path: impl AsRef<Path>,
    release_review_path: impl AsRef<Path>,
) -> Result<FridayTrustedHostRunnerUxReport> {
    let history = read_friday_trusted_host_runner_history(history_path)?;
    Ok(friday_trusted_host_runner_ux_report(
        &history,
        release_review_path,
    ))
}

pub fn friday_trusted_host_runner_approval_ui_report(
    history: &FridayTrustedHostRunnerHistory,
    release_review_path: impl AsRef<Path>,
) -> FridayTrustedHostRunnerApprovalUiReport {
    let latest = history.latest.as_ref();
    let action_id = latest.map(|result| result.action_id.clone());
    let command = latest
        .map(|result| result.command.clone())
        .unwrap_or_else(|| "flow --completion".to_string());
    let disabled_reason = latest
        .is_none()
        .then(|| "No trusted runner history record is available yet.".to_string());
    let release_review_path = path_string(release_review_path.as_ref());

    FridayTrustedHostRunnerApprovalUiReport {
        history_json: history.history_json.clone(),
        result_count: history.result_count,
        modal_id: "trusted-runner-approval".to_string(),
        latest_action_id: action_id.clone(),
        title: "Approve trusted runner action".to_string(),
        body: "Review the local command, write a short audit reason, then choose the explicit action Friday should prepare."
            .to_string(),
        command_preview: command.clone(),
        reason_label: "Audit reason".to_string(),
        reason_placeholder: "Example: rerun readiness after dashboard export refresh".to_string(),
        audit_reason_required: true,
        controls: runner_approval_controls(action_id.as_deref(), &command, disabled_reason),
        snooze_options: vec![
            FridayTrustedHostRunnerSnoozeOption {
                id: "snooze-5m".to_string(),
                label: "Snooze 5 minutes".to_string(),
                duration_seconds: 300,
            },
            FridayTrustedHostRunnerSnoozeOption {
                id: "snooze-30m".to_string(),
                label: "Snooze 30 minutes".to_string(),
                duration_seconds: 1_800,
            },
        ],
        undo_note: "Undo only clears the dashboard approval draft; finished runner history remains immutable for audit."
            .to_string(),
        release_review_path,
    }
}

pub fn friday_trusted_host_runner_approval_ui_report_from_history_file(
    history_path: impl AsRef<Path>,
    release_review_path: impl AsRef<Path>,
) -> Result<FridayTrustedHostRunnerApprovalUiReport> {
    let history = read_friday_trusted_host_runner_history(history_path)?;
    Ok(friday_trusted_host_runner_approval_ui_report(
        &history,
        release_review_path,
    ))
}

pub fn friday_trusted_host_runner_cancellation_ux_report(
    state: &FridayTrustedHostLiveRunnerState,
) -> FridayTrustedHostRunnerCancellationUxReport {
    let active_count = state
        .records
        .iter()
        .filter(|record| record.status.is_active())
        .count();
    let stale_count = state
        .records
        .iter()
        .filter(|record| record.status == FridayTrustedHostLiveRunnerStatus::Stale)
        .count();
    let denial_count = state
        .records
        .iter()
        .filter(|record| record.status == FridayTrustedHostLiveRunnerStatus::Denied)
        .count();
    let controls = state
        .records
        .iter()
        .flat_map(|record| runner_cancellation_controls(state, record))
        .collect::<Vec<_>>();

    FridayTrustedHostRunnerCancellationUxReport {
        state_json: state.state_json.clone(),
        record_count: state.record_count,
        active_count,
        stale_count,
        denial_count,
        controls,
        draft: FridayTrustedHostRunnerCancellationDraft {
            storage_key: "flow.dashboard.runnerCancellationDrafts".to_string(),
            default_reason: "Operator reviewed live runner state".to_string(),
            autosave_hint:
                "Cancellation and retry reasons are remembered locally in this browser only."
                    .to_string(),
        },
        guidance: vec![
            "Cancel active live records before approving another command for the same action."
                .to_string(),
            "Clean up stale live records, then retry from a fresh bridge import if the action still matters."
                .to_string(),
            "Denial recovery always requires a short operator reason so the audit trail explains the correction."
                .to_string(),
        ],
    }
}

pub fn friday_trusted_host_runner_cancellation_ux_report_from_state_file(
    state_path: impl AsRef<Path>,
) -> Result<FridayTrustedHostRunnerCancellationUxReport> {
    let state = read_friday_trusted_host_live_runner_state(state_path)?;
    Ok(friday_trusted_host_runner_cancellation_ux_report(&state))
}

pub fn friday_trusted_host_runner_operator_review_report(
    history: &FridayTrustedHostRunnerHistory,
    filter: FridayTrustedHostRunnerOperatorReviewFilter,
) -> FridayTrustedHostRunnerOperatorReviewReport {
    let limit = filter.limit.clamp(1, HISTORY_LIMIT);
    let filtered = history
        .records
        .iter()
        .filter(|record| runner_review_matches_filter(record, &filter))
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    let records = filtered
        .iter()
        .map(runner_review_record)
        .collect::<Vec<_>>();
    let incident_notes = filtered
        .iter()
        .filter_map(runner_incident_note)
        .collect::<Vec<_>>();
    let release_gate_summaries = runner_release_gate_summaries(&filtered);
    let ready_count = filtered
        .iter()
        .filter(|record| record.status == FridayTrustedHostRunnerStatus::Succeeded)
        .count();
    let blocked_count = filtered
        .iter()
        .filter(|record| runner_release_gate_severity(record.status) == "blocked")
        .count();

    FridayTrustedHostRunnerOperatorReviewReport {
        history_json: history.history_json.clone(),
        review_id: format!("trusted-runner-review-{}", unix_ms()),
        generated_at_unix_ms: unix_ms(),
        filters: FridayTrustedHostRunnerOperatorReviewFilter { limit, ..filter },
        record_count: history.result_count,
        matched_count: filtered.len(),
        ready_count,
        blocked_count,
        release_gate_status: if blocked_count > 0 {
            "blocked".to_string()
        } else if filtered.is_empty() {
            "empty".to_string()
        } else {
            "ready".to_string()
        },
        release_gate_summaries,
        incident_notes,
        records,
    }
}

pub fn friday_trusted_host_runner_operator_review_report_from_history_file(
    history_path: impl AsRef<Path>,
    filter: FridayTrustedHostRunnerOperatorReviewFilter,
) -> Result<FridayTrustedHostRunnerOperatorReviewReport> {
    let history = read_friday_trusted_host_runner_history(history_path)?;
    Ok(friday_trusted_host_runner_operator_review_report(
        &history, filter,
    ))
}

pub fn read_friday_trusted_host_live_runner_state(
    state_path: impl AsRef<Path>,
) -> Result<FridayTrustedHostLiveRunnerState> {
    let state_path = state_path.as_ref();
    if !state_path.exists() {
        return Ok(live_state_from_records(state_path, Vec::new(), unix_ms()));
    }

    let text = fs::read_to_string(state_path).with_context(|| {
        format!(
            "Could not read trusted host live runner state {}",
            state_path.display()
        )
    })?;
    let state =
        serde_json::from_str::<FridayTrustedHostLiveRunnerState>(&text).with_context(|| {
            format!(
                "Could not parse trusted host live runner state {}",
                state_path.display()
            )
        })?;
    Ok(state)
}

pub fn write_friday_trusted_host_live_runner_state(
    state_path: impl AsRef<Path>,
    records: Vec<FridayTrustedHostLiveRunnerRecord>,
) -> Result<FridayTrustedHostLiveRunnerState> {
    let state_path = state_path.as_ref();
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create trusted host live runner state directory {}",
                parent.display()
            )
        })?;
    }
    let state = live_state_from_records(state_path, records, unix_ms());
    let json = serde_json::to_string_pretty(&state)?;
    fs::write(state_path, json).with_context(|| {
        format!(
            "Could not write trusted host live runner state {}",
            state_path.display()
        )
    })?;
    Ok(state)
}

pub fn friday_trusted_host_live_runner_state_from_history(
    history: &FridayTrustedHostRunnerHistory,
    state_path: impl AsRef<Path>,
) -> FridayTrustedHostLiveRunnerState {
    let now = unix_ms();
    let records = history
        .records
        .iter()
        .take(LIVE_STATE_LIMIT)
        .map(|result| live_record_from_result(result, history.history_json.clone(), now))
        .collect::<Vec<_>>();
    live_state_from_records(state_path.as_ref(), records, now)
}

pub fn friday_trusted_host_live_runner_state_from_history_file(
    history_path: impl AsRef<Path>,
    state_path: impl AsRef<Path>,
) -> Result<FridayTrustedHostLiveRunnerState> {
    let history = read_friday_trusted_host_runner_history(history_path)?;
    Ok(friday_trusted_host_live_runner_state_from_history(
        &history, state_path,
    ))
}

pub fn refresh_friday_trusted_host_live_runner_state(
    state: &FridayTrustedHostLiveRunnerState,
) -> FridayTrustedHostLiveRunnerState {
    let now = unix_ms();
    let records = state
        .records
        .iter()
        .cloned()
        .map(|record| mark_live_record_stale(record, now))
        .collect::<Vec<_>>();
    live_state_from_records(Path::new(&state.state_json), records, now)
}

pub fn run_friday_trusted_host_command(
    record: &FridayDashboardHostCommandRecord,
    request: &FridayTrustedHostRunnerRequest,
) -> FridayTrustedHostRunnerResult {
    run_friday_trusted_host_command_with_executor(
        record,
        request,
        &FridayProcessTrustedHostCommandExecutor,
    )
}

pub fn run_friday_trusted_host_command_bridge(
    record: &FridayDashboardHostCommandRecord,
    request: &FridayTrustedHostRunnerRequest,
    state_path: impl AsRef<Path>,
    history_path: impl AsRef<Path>,
    cancellation: &FridayTrustedHostRunnerCancellationToken,
) -> Result<FridayTrustedHostRunnerBridgeReport> {
    run_friday_trusted_host_command_bridge_with_executor(
        record,
        request,
        &FridayProcessTrustedHostCommandExecutor,
        state_path,
        history_path,
        cancellation,
    )
}

pub fn run_friday_trusted_host_command_bridge_with_executor(
    record: &FridayDashboardHostCommandRecord,
    request: &FridayTrustedHostRunnerRequest,
    executor: &impl FridayTrustedHostCommandExecutor,
    state_path: impl AsRef<Path>,
    history_path: impl AsRef<Path>,
    cancellation: &FridayTrustedHostRunnerCancellationToken,
) -> Result<FridayTrustedHostRunnerBridgeReport> {
    let state_path = state_path.as_ref();
    let history_path = history_path.as_ref();
    let mut events = Vec::new();

    let pending = live_record_from_bridge_record(
        record,
        request,
        FridayTrustedHostLiveRunnerStatus::Pending,
        "Waiting for explicit trusted host approval.",
    );
    let mut live_state =
        write_friday_trusted_host_live_runner_state(state_path, vec![pending.clone()])?;
    events.push(live_bridge_event("pending", &live_state, pending));

    let mut bridged_request = request.clone();
    if cancellation.cancel_requested {
        bridged_request.cancel_requested = true;
        if bridged_request.operator_reason.is_none() {
            bridged_request.operator_reason = cancellation.reason.clone();
        }
    }

    if bridged_request.approved && !bridged_request.cancel_requested {
        let running = live_record_from_bridge_record(
            record,
            &bridged_request,
            FridayTrustedHostLiveRunnerStatus::Running,
            "Trusted host runner is executing this bounded local command.",
        );
        live_state =
            write_friday_trusted_host_live_runner_state(state_path, vec![running.clone()])?;
        events.push(live_bridge_event("running", &live_state, running));
    }

    let result = run_friday_trusted_host_command_with_executor(record, &bridged_request, executor);
    let history = append_friday_trusted_host_runner_history(history_path, result.clone())?;
    let finished = live_record_from_result(&result, history.history_json.clone(), unix_ms());
    live_state = write_friday_trusted_host_live_runner_state(state_path, vec![finished.clone()])?;
    events.push(live_bridge_event(
        result.status.label(),
        &live_state,
        finished,
    ));

    Ok(FridayTrustedHostRunnerBridgeReport {
        state_json: path_string(state_path),
        history_json: path_string(history_path),
        event_count: events.len(),
        dashboard_import_guidance: "Import live-state JSON for current pending/running work; import runner history JSON only for immutable audit history."
            .to_string(),
        events,
        live_state,
        history,
        result,
    })
}

pub fn run_friday_trusted_host_command_with_executor(
    record: &FridayDashboardHostCommandRecord,
    request: &FridayTrustedHostRunnerRequest,
    executor: &impl FridayTrustedHostCommandExecutor,
) -> FridayTrustedHostRunnerResult {
    if !request.approved {
        return runner_result(
            record,
            request,
            FridayTrustedHostRunnerStatus::Denied,
            None,
            "",
            "Operator approval is required before running this command.",
            0,
        );
    }

    if request.cancel_requested {
        return runner_result(
            record,
            request,
            FridayTrustedHostRunnerStatus::Cancelled,
            None,
            "",
            "Execution was cancelled before the command started.",
            0,
        );
    }

    if let Some(reason) = runner_denial_reason(record) {
        return runner_result(
            record,
            request,
            FridayTrustedHostRunnerStatus::Denied,
            None,
            "",
            &reason,
            0,
        );
    }

    if let Some(reason) = command_allowlist_denial(&record.command) {
        return runner_result(
            record,
            request,
            FridayTrustedHostRunnerStatus::Denied,
            None,
            "",
            &reason,
            0,
        );
    }

    match executor.execute(&record.command, request.timeout_ms.max(1)) {
        Ok(raw) if raw.timed_out => runner_result(
            record,
            request,
            FridayTrustedHostRunnerStatus::TimedOut,
            raw.exit_code,
            &raw.stdout,
            &raw.stderr,
            raw.duration_ms,
        ),
        Ok(raw) if raw.exit_code == Some(0) => runner_result(
            record,
            request,
            FridayTrustedHostRunnerStatus::Succeeded,
            raw.exit_code,
            &raw.stdout,
            &raw.stderr,
            raw.duration_ms,
        ),
        Ok(raw) => runner_result(
            record,
            request,
            FridayTrustedHostRunnerStatus::Failed,
            raw.exit_code,
            &raw.stdout,
            &raw.stderr,
            raw.duration_ms,
        ),
        Err(error) => runner_result(
            record,
            request,
            FridayTrustedHostRunnerStatus::Failed,
            None,
            "",
            &error,
            0,
        ),
    }
}

pub fn append_friday_trusted_host_runner_history(
    history_path: impl AsRef<Path>,
    result: FridayTrustedHostRunnerResult,
) -> Result<FridayTrustedHostRunnerHistory> {
    let history_path = history_path.as_ref();
    let mut records = read_friday_trusted_host_runner_history(history_path)?.records;
    records.insert(0, result);
    records.truncate(HISTORY_LIMIT);
    write_friday_trusted_host_runner_history(history_path, records)
}

pub fn read_friday_trusted_host_runner_history(
    history_path: impl AsRef<Path>,
) -> Result<FridayTrustedHostRunnerHistory> {
    let history_path = history_path.as_ref();
    if !history_path.exists() {
        return Ok(history_from_records(history_path, Vec::new()));
    }

    let text = fs::read_to_string(history_path).with_context(|| {
        format!(
            "Could not read trusted host runner history {}",
            history_path.display()
        )
    })?;
    let history =
        serde_json::from_str::<FridayTrustedHostRunnerHistory>(&text).with_context(|| {
            format!(
                "Could not parse trusted host runner history {}",
                history_path.display()
            )
        })?;
    Ok(history)
}

fn write_friday_trusted_host_runner_history(
    history_path: &Path,
    records: Vec<FridayTrustedHostRunnerResult>,
) -> Result<FridayTrustedHostRunnerHistory> {
    if let Some(parent) = history_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create trusted host runner history directory {}",
                parent.display()
            )
        })?;
    }
    let history = history_from_records(history_path, records);
    let json = serde_json::to_string_pretty(&history)?;
    fs::write(history_path, json).with_context(|| {
        format!(
            "Could not write trusted host runner history {}",
            history_path.display()
        )
    })?;
    Ok(history)
}

fn history_from_records(
    history_path: &Path,
    records: Vec<FridayTrustedHostRunnerResult>,
) -> FridayTrustedHostRunnerHistory {
    FridayTrustedHostRunnerHistory {
        history_json: path_string(history_path),
        result_count: records.len(),
        latest: records.first().cloned(),
        records,
    }
}

fn live_state_from_records(
    state_path: &Path,
    records: Vec<FridayTrustedHostLiveRunnerRecord>,
    now: u128,
) -> FridayTrustedHostLiveRunnerState {
    let records = records
        .into_iter()
        .take(LIVE_STATE_LIMIT)
        .map(|record| mark_live_record_stale(record, now))
        .collect::<Vec<_>>();
    let pending_count = records
        .iter()
        .filter(|record| record.status == FridayTrustedHostLiveRunnerStatus::Pending)
        .count();
    let running_count = records
        .iter()
        .filter(|record| record.status == FridayTrustedHostLiveRunnerStatus::Running)
        .count();
    let finished_count = records
        .iter()
        .filter(|record| record.status.is_finished())
        .count();
    let stale_count = records
        .iter()
        .filter(|record| record.status == FridayTrustedHostLiveRunnerStatus::Stale)
        .count();

    FridayTrustedHostLiveRunnerState {
        state_json: path_string(state_path),
        generated_at_unix_ms: now,
        record_count: records.len(),
        pending_count,
        running_count,
        finished_count,
        stale_count,
        records,
        stale_recovery_copy: "Stale live runner records are not running anymore. Refresh the trusted host state, then clear stale drafts before approving another command."
            .to_string(),
    }
}

fn live_record_from_result(
    result: &FridayTrustedHostRunnerResult,
    history_json: String,
    now: u128,
) -> FridayTrustedHostLiveRunnerRecord {
    let status = live_status_from_runner_status(result.status);
    FridayTrustedHostLiveRunnerRecord {
        job_id: format!("runner-{}-{}", result.action_id, result.recorded_at_unix_ms),
        action_id: result.action_id.clone(),
        label: result.label.clone(),
        command: result.command.clone(),
        status,
        message: live_message_for_status(status),
        local_only: true,
        approved: result.approved,
        timeout_ms: result.timeout_ms,
        stale_after_ms: DEFAULT_STALE_AFTER_MS,
        created_at_unix_ms: result.recorded_at_unix_ms,
        updated_at_unix_ms: result.recorded_at_unix_ms,
        finished_at_unix_ms: Some(result.recorded_at_unix_ms.max(now.saturating_sub(1))),
        history_json: Some(history_json),
        recovery_command: live_recovery_command(&result.action_id),
        cleanup_command: live_cleanup_command(),
    }
}

fn live_record_from_bridge_record(
    record: &FridayDashboardHostCommandRecord,
    request: &FridayTrustedHostRunnerRequest,
    status: FridayTrustedHostLiveRunnerStatus,
    message: &str,
) -> FridayTrustedHostLiveRunnerRecord {
    let now = unix_ms();
    FridayTrustedHostLiveRunnerRecord {
        job_id: format!("runner-{}-{now}", record.action_id),
        action_id: record.action_id.clone(),
        label: record.label.clone(),
        command: record.command.clone(),
        status,
        message: message.to_string(),
        local_only: record.local_only,
        approved: request.approved,
        timeout_ms: request.timeout_ms,
        stale_after_ms: (request.timeout_ms as u128)
            .saturating_mul(2)
            .max(DEFAULT_STALE_AFTER_MS),
        created_at_unix_ms: now,
        updated_at_unix_ms: now,
        finished_at_unix_ms: status.is_finished().then_some(now),
        history_json: None,
        recovery_command: live_recovery_command(&record.action_id),
        cleanup_command: live_cleanup_command(),
    }
}

fn live_bridge_event(
    stage: &str,
    state: &FridayTrustedHostLiveRunnerState,
    record: FridayTrustedHostLiveRunnerRecord,
) -> FridayTrustedHostRunnerBridgeEvent {
    FridayTrustedHostRunnerBridgeEvent {
        event_id: format!("{}-{}", stage, record.job_id),
        status: record.status,
        message: record.message.clone(),
        state_json: state.state_json.clone(),
        record,
    }
}

fn mark_live_record_stale(
    mut record: FridayTrustedHostLiveRunnerRecord,
    now: u128,
) -> FridayTrustedHostLiveRunnerRecord {
    if record.status.is_active()
        && now.saturating_sub(record.updated_at_unix_ms) > record.stale_after_ms
    {
        record.status = FridayTrustedHostLiveRunnerStatus::Stale;
        record.message = live_message_for_status(record.status);
        record.finished_at_unix_ms = Some(now);
    }
    record
}

fn live_status_from_runner_status(
    status: FridayTrustedHostRunnerStatus,
) -> FridayTrustedHostLiveRunnerStatus {
    match status {
        FridayTrustedHostRunnerStatus::Succeeded => FridayTrustedHostLiveRunnerStatus::Succeeded,
        FridayTrustedHostRunnerStatus::Failed => FridayTrustedHostLiveRunnerStatus::Failed,
        FridayTrustedHostRunnerStatus::TimedOut => FridayTrustedHostLiveRunnerStatus::TimedOut,
        FridayTrustedHostRunnerStatus::Cancelled => FridayTrustedHostLiveRunnerStatus::Cancelled,
        FridayTrustedHostRunnerStatus::Denied => FridayTrustedHostLiveRunnerStatus::Denied,
    }
}

fn live_message_for_status(status: FridayTrustedHostLiveRunnerStatus) -> String {
    match status {
        FridayTrustedHostLiveRunnerStatus::Pending => {
            "Waiting for an explicit local approval command.".to_string()
        }
        FridayTrustedHostLiveRunnerStatus::Running => {
            "Trusted host runner is executing a bounded local command.".to_string()
        }
        FridayTrustedHostLiveRunnerStatus::Succeeded => {
            "Trusted host runner completed successfully.".to_string()
        }
        FridayTrustedHostLiveRunnerStatus::Failed => {
            "Trusted host runner finished with an error; inspect stderr before retrying.".to_string()
        }
        FridayTrustedHostLiveRunnerStatus::TimedOut => {
            "Trusted host runner timed out and stopped the local process.".to_string()
        }
        FridayTrustedHostLiveRunnerStatus::Cancelled => {
            "Trusted host runner was cancelled by the operator.".to_string()
        }
        FridayTrustedHostLiveRunnerStatus::Denied => {
            "Trusted host runner denied execution before starting a process.".to_string()
        }
        FridayTrustedHostLiveRunnerStatus::Stale => {
            "This live runner record is stale; refresh or clear it before trusting the dashboard state."
                .to_string()
        }
    }
}

fn live_recovery_command(action_id: &str) -> String {
    format!(
        "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id {action_id} --cancel --reason \"stale live runner cleanup\""
    )
}

fn live_cleanup_command() -> String {
    "flow --friday-trusted-host-live-state tmp/friday-dashboard/trusted-host-live-state.json"
        .to_string()
}

fn runner_cancellation_controls(
    state: &FridayTrustedHostLiveRunnerState,
    record: &FridayTrustedHostLiveRunnerRecord,
) -> Vec<FridayTrustedHostRunnerCancellationControl> {
    let mut controls = Vec::new();
    let input_dir = live_runner_input_dir(&state.state_json);
    let history_json = record
        .history_json
        .clone()
        .unwrap_or_else(|| format!("{input_dir}/trusted-host-runner-history.json"));
    let bridge_prefix = format!(
        "flow --friday-trusted-host-bridge-runner {input_dir} --action-id {} --state {} --history {history_json}",
        record.action_id, state.state_json
    );

    if record.status.is_active() {
        controls.push(FridayTrustedHostRunnerCancellationControl {
            id: format!("cancel-{}", record.job_id),
            job_id: record.job_id.clone(),
            action_id: record.action_id.clone(),
            kind: "cancel".to_string(),
            label: format!("Cancel {}", record.label),
            command: format!("{bridge_prefix} --cancel --reason \"<cancel reason>\""),
            detail: "Stops this live trusted runner before another command is approved."
                .to_string(),
            requires_reason: true,
            disabled: !record.local_only,
            disabled_reason: (!record.local_only)
                .then_some("Only local trusted runner records can be cancelled here.".to_string()),
        });
    }

    if record.status == FridayTrustedHostLiveRunnerStatus::Stale {
        controls.push(FridayTrustedHostRunnerCancellationControl {
            id: format!("cleanup-{}", record.job_id),
            job_id: record.job_id.clone(),
            action_id: record.action_id.clone(),
            kind: "cleanup-stale".to_string(),
            label: format!("Clean up {}", record.label),
            command: format!(
                "flow --friday-trusted-host-live-state {} --history {history_json}",
                state.state_json
            ),
            detail: "Refreshes the live state file so stale runner records stop looking active."
                .to_string(),
            requires_reason: false,
            disabled: false,
            disabled_reason: None,
        });
        controls.push(FridayTrustedHostRunnerCancellationControl {
            id: format!("retry-stale-{}", record.job_id),
            job_id: record.job_id.clone(),
            action_id: record.action_id.clone(),
            kind: "retry".to_string(),
            label: format!("Retry {}", record.label),
            command: format!("{bridge_prefix} --approve --execute --reason \"<retry reason>\""),
            detail: "Retries the action through the bridge after stale state has been reviewed."
                .to_string(),
            requires_reason: true,
            disabled: false,
            disabled_reason: None,
        });
    }

    if record.status == FridayTrustedHostLiveRunnerStatus::Denied {
        controls.push(FridayTrustedHostRunnerCancellationControl {
            id: format!("recover-denied-{}", record.job_id),
            job_id: record.job_id.clone(),
            action_id: record.action_id.clone(),
            kind: "denial-recovery".to_string(),
            label: format!("Recover {}", record.label),
            command: format!(
                "{bridge_prefix} --approve --execute --reason \"<denial recovery reason>\""
            ),
            detail: "Approves and reruns this denied command with an explicit correction reason."
                .to_string(),
            requires_reason: true,
            disabled: false,
            disabled_reason: None,
        });
    }

    controls
}

fn live_runner_input_dir(state_json: &str) -> String {
    Path::new(state_json)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(path_string)
        .unwrap_or_else(|| "tmp/friday-dashboard".to_string())
}

fn runner_result(
    record: &FridayDashboardHostCommandRecord,
    request: &FridayTrustedHostRunnerRequest,
    status: FridayTrustedHostRunnerStatus,
    exit_code: Option<i32>,
    stdout: &str,
    stderr: &str,
    duration_ms: u64,
) -> FridayTrustedHostRunnerResult {
    let (stdout_summary, stdout_truncated) = summarize_output(stdout, request.stdout_limit_bytes);
    let (stderr_summary, stderr_truncated) = summarize_output(stderr, request.stderr_limit_bytes);
    FridayTrustedHostRunnerResult {
        action_id: record.action_id.clone(),
        card_id: record.card_id.clone(),
        label: record.label.clone(),
        command: record.command.clone(),
        status,
        exit_code,
        stdout_summary,
        stderr_summary,
        stdout_truncated,
        stderr_truncated,
        duration_ms,
        timeout_ms: request.timeout_ms,
        approved: request.approved,
        cancelled: request.cancel_requested || status == FridayTrustedHostRunnerStatus::Cancelled,
        operator_reason: request
            .operator_reason
            .as_deref()
            .map(|reason| summarize_output(reason.trim(), 240).0)
            .filter(|reason| !reason.is_empty()),
        audit_event: format!("trusted-host-runner-{}", status.label()),
        recorded_at_unix_ms: unix_ms(),
    }
}

fn runner_review_matches_filter(
    record: &FridayTrustedHostRunnerResult,
    filter: &FridayTrustedHostRunnerOperatorReviewFilter,
) -> bool {
    if filter.status.is_some_and(|status| record.status != status) {
        return false;
    }
    if let Some(action_id) = filter.action_id.as_deref() {
        if !action_id.trim().is_empty() && !record.action_id.contains(action_id.trim()) {
            return false;
        }
    }
    if filter
        .since_unix_ms
        .is_some_and(|since| record.recorded_at_unix_ms < since)
    {
        return false;
    }
    if filter
        .until_unix_ms
        .is_some_and(|until| record.recorded_at_unix_ms > until)
    {
        return false;
    }
    true
}

fn runner_review_record(
    record: &FridayTrustedHostRunnerResult,
) -> FridayTrustedHostRunnerReviewRecord {
    FridayTrustedHostRunnerReviewRecord {
        result_id: format!("{}-{}", record.action_id, record.recorded_at_unix_ms),
        action_id: record.action_id.clone(),
        label: record.label.clone(),
        status: record.status,
        severity: runner_release_gate_severity(record.status).to_string(),
        command: record.command.clone(),
        summary: runner_review_summary(record),
        release_gate: runner_release_gate_detail(record.status).to_string(),
        operator_reason: record.operator_reason.clone(),
        recorded_at_unix_ms: record.recorded_at_unix_ms,
        duration_ms: record.duration_ms,
        exit_code: record.exit_code,
    }
}

fn runner_release_gate_summaries(
    records: &[FridayTrustedHostRunnerResult],
) -> Vec<FridayTrustedHostRunnerReleaseGateSummary> {
    let mut summaries = [
        FridayTrustedHostRunnerStatus::Succeeded,
        FridayTrustedHostRunnerStatus::Failed,
        FridayTrustedHostRunnerStatus::TimedOut,
        FridayTrustedHostRunnerStatus::Cancelled,
        FridayTrustedHostRunnerStatus::Denied,
    ]
    .into_iter()
    .map(|status| {
        let count = records
            .iter()
            .filter(|record| record.status == status)
            .count();
        FridayTrustedHostRunnerReleaseGateSummary {
            id: status.label().to_string(),
            title: runner_status_title(status).to_string(),
            severity: runner_release_gate_severity(status).to_string(),
            count,
            detail: runner_release_gate_detail(status).to_string(),
            next_action: runner_release_gate_next_action(status, count).to_string(),
        }
    })
    .collect::<Vec<_>>();
    summaries.push(FridayTrustedHostRunnerReleaseGateSummary {
        id: "stale-live-state".to_string(),
        title: "Stale live state".to_string(),
        severity: "watch".to_string(),
        count: 0,
        detail: "Immutable runner history cannot contain stale live records; import the live-state JSON to review stale in-flight work.".to_string(),
        next_action:
            "Open the live runner state card when stale cleanup controls are needed.".to_string(),
    });
    summaries
}

fn runner_incident_note(
    record: &FridayTrustedHostRunnerResult,
) -> Option<FridayTrustedHostRunnerIncidentNote> {
    if record.status == FridayTrustedHostRunnerStatus::Succeeded {
        return None;
    }
    let title = format!("{}: {}", runner_status_title(record.status), record.label);
    let body = format!(
        "{} Command `{}` recorded {} with exit code {}.",
        runner_release_gate_detail(record.status),
        record.command,
        record.recorded_at_unix_ms,
        record
            .exit_code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "n/a".to_string())
    );
    let export_markdown = format!(
        "### {title}\n\n- Action: `{}`\n- Status: `{}`\n- Severity: `{}`\n- Recorded: `{}`\n- Duration: `{}ms`\n- Exit code: `{}`\n- Operator reason: `{}`\n- Command: `{}`\n- Stdout: `{}`\n- Stderr: `{}`\n",
        record.action_id,
        record.status.label(),
        runner_release_gate_severity(record.status),
        record.recorded_at_unix_ms,
        record.duration_ms,
        record
            .exit_code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "n/a".to_string()),
        record.operator_reason.as_deref().unwrap_or("not recorded"),
        record.command,
        record.stdout_summary,
        record.stderr_summary,
    );
    Some(FridayTrustedHostRunnerIncidentNote {
        id: format!(
            "incident-{}-{}",
            record.action_id, record.recorded_at_unix_ms
        ),
        action_id: record.action_id.clone(),
        status: record.status,
        severity: runner_release_gate_severity(record.status).to_string(),
        title,
        body,
        export_markdown,
        recorded_at_unix_ms: record.recorded_at_unix_ms,
    })
}

fn runner_review_summary(record: &FridayTrustedHostRunnerResult) -> String {
    let output = if !record.stderr_summary.trim().is_empty() {
        &record.stderr_summary
    } else if !record.stdout_summary.trim().is_empty() {
        &record.stdout_summary
    } else {
        "no output"
    };
    format!(
        "{} in {}ms: {}",
        runner_status_title(record.status),
        record.duration_ms,
        output
    )
}

fn runner_release_gate_severity(status: FridayTrustedHostRunnerStatus) -> &'static str {
    match status {
        FridayTrustedHostRunnerStatus::Succeeded => "ready",
        FridayTrustedHostRunnerStatus::Failed
        | FridayTrustedHostRunnerStatus::TimedOut
        | FridayTrustedHostRunnerStatus::Denied => "blocked",
        FridayTrustedHostRunnerStatus::Cancelled => "watch",
    }
}

fn runner_release_gate_detail(status: FridayTrustedHostRunnerStatus) -> &'static str {
    match status {
        FridayTrustedHostRunnerStatus::Succeeded => {
            "This command completed successfully and does not block release review."
        }
        FridayTrustedHostRunnerStatus::Failed => {
            "This command failed and blocks release until stderr is reviewed or the action is rerun successfully."
        }
        FridayTrustedHostRunnerStatus::TimedOut => {
            "This command timed out and blocks release until the timeout cause is understood."
        }
        FridayTrustedHostRunnerStatus::Cancelled => {
            "This command was cancelled; confirm the cancellation was intentional before release."
        }
        FridayTrustedHostRunnerStatus::Denied => {
            "This command was blocked or denied before execution and needs an approval-policy note."
        }
    }
}

fn runner_release_gate_next_action(
    status: FridayTrustedHostRunnerStatus,
    count: usize,
) -> &'static str {
    if count == 0 {
        return "No action needed for this status.";
    }
    match status {
        FridayTrustedHostRunnerStatus::Succeeded => {
            "Attach successful checks to the release review package."
        }
        FridayTrustedHostRunnerStatus::Failed => {
            "Export incident notes, fix the failing command, and rerun through the trusted runner."
        }
        FridayTrustedHostRunnerStatus::TimedOut => {
            "Review timeout settings and rerun only after confirming the command remains safe."
        }
        FridayTrustedHostRunnerStatus::Cancelled => {
            "Record why cancellation was intentional or rerun with explicit approval."
        }
        FridayTrustedHostRunnerStatus::Denied => {
            "Resolve the approval-policy issue before preparing another execution."
        }
    }
}

fn runner_approval_controls(
    action_id: Option<&str>,
    command: &str,
    disabled_reason: Option<String>,
) -> Vec<FridayTrustedHostRunnerApprovalControl> {
    let action_id = action_id.unwrap_or("unknown-action");
    let disabled = disabled_reason.is_some();
    vec![
        approval_control(
            "approve",
            "approve",
            "Approve and run",
            &format!(
                "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id {action_id} --approve --execute --reason \"<audit reason>\""
            ),
            "Approve this local command and run it through the trusted host runner.",
            Some((
                "Ctrl+Enter",
                "Approve",
                "Approve and copy the approved runner command.",
            )),
            true,
            true,
            disabled,
            disabled_reason.clone(),
        ),
        approval_control(
            "deny",
            "deny",
            "Deny",
            &format!(
                "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id {action_id} --reason \"<denial reason>\""
            ),
            "Record a denial reason without executing the local command.",
            Some(("Esc", "Deny", "Deny this pending approval draft.")),
            true,
            false,
            disabled,
            disabled_reason.clone(),
        ),
        approval_control(
            "copy",
            "copy-command",
            "Copy command",
            command,
            "Copy the underlying local command without executing it.",
            Some(("Ctrl+C", "Copy", "Copy the current trusted runner command.")),
            false,
            false,
            disabled,
            disabled_reason.clone(),
        ),
        approval_control(
            "retry",
            "retry",
            "Retry with approval",
            &format!(
                "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id {action_id} --approve --execute --reason \"<audit reason>\""
            ),
            "Retry the action only after a fresh explicit approval reason.",
            Some(("Ctrl+R", "Retry", "Prepare a retry with explicit approval.")),
            true,
            true,
            disabled,
            disabled_reason.clone(),
        ),
        approval_control(
            "cancel",
            "cancel",
            "Cancel",
            &format!(
                "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id {action_id} --cancel --reason \"<cancel reason>\""
            ),
            "Cancel the pending runner action before execution.",
            Some((
                "Ctrl+Backspace",
                "Cancel",
                "Cancel this pending runner action.",
            )),
            true,
            false,
            disabled,
            disabled_reason.clone(),
        ),
        approval_control(
            "snooze",
            "snooze",
            "Snooze",
            "",
            "Hide the approval draft temporarily without changing immutable runner history.",
            Some(("Ctrl+S", "Snooze", "Snooze this pending approval draft.")),
            false,
            false,
            false,
            None,
        ),
        approval_control(
            "undo",
            "undo",
            "Undo draft",
            "",
            "Clear the dashboard approval draft without touching existing runner history.",
            Some(("Ctrl+Z", "Undo", "Undo the current approval draft.")),
            false,
            false,
            false,
            None,
        ),
    ]
}

fn approval_control(
    id: &str,
    kind: &str,
    label: &str,
    command: &str,
    detail: &str,
    keyboard_shortcut: Option<(&str, &str, &str)>,
    requires_reason: bool,
    requires_approval: bool,
    disabled: bool,
    disabled_reason: Option<String>,
) -> FridayTrustedHostRunnerApprovalControl {
    FridayTrustedHostRunnerApprovalControl {
        id: id.to_string(),
        kind: kind.to_string(),
        label: label.to_string(),
        command: command.to_string(),
        detail: detail.to_string(),
        aria_label: format!("{label}: {detail}"),
        keyboard_shortcut: keyboard_shortcut.map(|(key, label, detail)| {
            FridayTrustedHostRunnerKeyboardShortcut {
                key: key.to_string(),
                label: label.to_string(),
                detail: detail.to_string(),
            }
        }),
        requires_reason,
        requires_approval,
        disabled,
        disabled_reason,
    }
}

fn runner_status_summary(
    history: &FridayTrustedHostRunnerHistory,
    status: FridayTrustedHostRunnerStatus,
) -> FridayTrustedHostRunnerStatusSummary {
    let count = history
        .records
        .iter()
        .filter(|record| record.status == status)
        .count();
    FridayTrustedHostRunnerStatusSummary {
        status,
        count,
        title: runner_status_title(status).to_string(),
        description: runner_status_operator_copy(status).to_string(),
        tone: runner_status_tone(status).to_string(),
    }
}

fn runner_status_title(status: FridayTrustedHostRunnerStatus) -> &'static str {
    match status {
        FridayTrustedHostRunnerStatus::Succeeded => "Succeeded",
        FridayTrustedHostRunnerStatus::Failed => "Failed",
        FridayTrustedHostRunnerStatus::TimedOut => "Timed out",
        FridayTrustedHostRunnerStatus::Cancelled => "Cancelled",
        FridayTrustedHostRunnerStatus::Denied => "Denied",
    }
}

fn runner_status_operator_copy(status: FridayTrustedHostRunnerStatus) -> &'static str {
    match status {
        FridayTrustedHostRunnerStatus::Succeeded => {
            "Approved host commands completed successfully and are ready for release review."
        }
        FridayTrustedHostRunnerStatus::Failed => {
            "The approved command exited with an error; inspect stderr before retrying."
        }
        FridayTrustedHostRunnerStatus::TimedOut => {
            "The runner stopped the command after its timeout, so retry only after checking the command is still safe."
        }
        FridayTrustedHostRunnerStatus::Cancelled => {
            "The operator cancelled this run before or during execution; Friday must not retry it silently."
        }
        FridayTrustedHostRunnerStatus::Denied => {
            "Execution was denied before a process started because approval or policy requirements were not met."
        }
    }
}

fn runner_status_tone(status: FridayTrustedHostRunnerStatus) -> &'static str {
    match status {
        FridayTrustedHostRunnerStatus::Succeeded => "ready",
        FridayTrustedHostRunnerStatus::Failed => "bad",
        FridayTrustedHostRunnerStatus::TimedOut => "warn",
        FridayTrustedHostRunnerStatus::Cancelled => "muted",
        FridayTrustedHostRunnerStatus::Denied => "blocked",
    }
}

fn runner_affordances(
    result: &FridayTrustedHostRunnerResult,
) -> Vec<FridayTrustedHostRunnerAffordance> {
    let retry_disabled = result.status == FridayTrustedHostRunnerStatus::Succeeded;
    vec![
        FridayTrustedHostRunnerAffordance {
            id: format!("copy-command-{}", result.action_id),
            kind: "copy-command".to_string(),
            action_id: result.action_id.clone(),
            status: result.status,
            label: "Copy command".to_string(),
            command: result.command.clone(),
            detail: "Copy the original local command without executing it.".to_string(),
            requires_approval: false,
            disabled: false,
            disabled_reason: None,
        },
        FridayTrustedHostRunnerAffordance {
            id: format!("retry-with-approval-{}", result.action_id),
            kind: "retry".to_string(),
            action_id: result.action_id.clone(),
            status: result.status,
            label: "Retry with approval".to_string(),
            command: runner_retry_command(result),
            detail: "Prepare the same runner action again; explicit operator approval is still required."
                .to_string(),
            requires_approval: true,
            disabled: retry_disabled,
            disabled_reason: retry_disabled
                .then(|| "The latest run already succeeded; copy the command if you need it.".to_string()),
        },
        FridayTrustedHostRunnerAffordance {
            id: format!("cancel-pending-{}", result.action_id),
            kind: "cancel".to_string(),
            action_id: result.action_id.clone(),
            status: result.status,
            label: "Cancel pending run".to_string(),
            command: runner_cancel_command(result),
            detail: "Cancel a prepared runner action before it executes.".to_string(),
            requires_approval: false,
            disabled: true,
            disabled_reason: Some(
                "Imported history records are already finished; cancellation is only live before execution."
                    .to_string(),
            ),
        },
    ]
}

fn runner_retry_command(result: &FridayTrustedHostRunnerResult) -> String {
    format!(
        "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id {} --approve --execute",
        result.action_id
    )
}

fn runner_cancel_command(result: &FridayTrustedHostRunnerResult) -> String {
    format!(
        "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id {} --cancel",
        result.action_id
    )
}

fn runner_denial_reason(record: &FridayDashboardHostCommandRecord) -> Option<String> {
    if record.status != FridayDashboardHostCommandStatus::AwaitingApproval {
        return Some(
            record
                .blocked_reason
                .clone()
                .unwrap_or_else(|| "Host bridge record is not awaiting approval.".to_string()),
        );
    }
    if record.approval_state != FridayDashboardHostApprovalState::Required {
        return Some("Host bridge record does not request approval.".to_string());
    }
    if !record.can_execute_after_approval {
        return Some("Host bridge record cannot execute after approval.".to_string());
    }
    if record.silent_execution_allowed {
        return Some("Silent dashboard host execution is forbidden.".to_string());
    }
    if record.destructive {
        return Some("Destructive host command execution requires a separate runner.".to_string());
    }
    if !record.local_only {
        return Some("Remote host commands are blocked in local-only mode.".to_string());
    }
    None
}

fn command_allowlist_denial(command: &str) -> Option<String> {
    let parts = match parse_command_line(command) {
        Ok(parts) => parts,
        Err(error) => return Some(error),
    };
    let executable = parts.first().map(String::as_str).unwrap_or_default();
    let first_arg = parts.get(1).map(String::as_str).unwrap_or_default();
    if !matches!(executable, "flow" | "flow.exe") {
        return Some("Trusted host runner only executes the local flow binary.".to_string());
    }
    if !(first_arg.starts_with("--friday-")
        || matches!(first_arg, "--completion" | "--progress" | "--next-100"))
    {
        return Some(
            "Trusted host runner only accepts Friday dashboard or completion commands.".to_string(),
        );
    }
    None
}

fn execute_local_flow_command(
    command: &str,
    timeout_ms: u64,
) -> std::result::Result<FridayTrustedHostCommandRawOutput, String> {
    let parts = parse_command_line(command)?;
    let executable = if matches!(parts.first().map(String::as_str), Some("flow" | "flow.exe")) {
        std::env::current_exe().map_err(|error| error.to_string())?
    } else {
        PathBuf::from(parts.first().cloned().unwrap_or_default())
    };
    let started = Instant::now();
    let mut child = Command::new(executable)
        .args(parts.iter().skip(1))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let timeout = Duration::from_millis(timeout_ms.max(1));

    loop {
        match child.try_wait().map_err(|error| error.to_string())? {
            Some(_) => {
                let output = child
                    .wait_with_output()
                    .map_err(|error| error.to_string())?;
                return Ok(FridayTrustedHostCommandRawOutput {
                    exit_code: output.status.code(),
                    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                    duration_ms: started.elapsed().as_millis() as u64,
                    timed_out: false,
                });
            }
            None if started.elapsed() >= timeout => {
                let _ = child.kill();
                let output = child
                    .wait_with_output()
                    .map_err(|error| error.to_string())?;
                return Ok(FridayTrustedHostCommandRawOutput {
                    exit_code: output.status.code(),
                    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                    duration_ms: started.elapsed().as_millis() as u64,
                    timed_out: true,
                });
            }
            None => thread::sleep(Duration::from_millis(10)),
        }
    }
}

fn parse_command_line(command: &str) -> std::result::Result<Vec<String>, String> {
    if command
        .chars()
        .any(|ch| matches!(ch, ';' | '&' | '|' | '<' | '>' | '`'))
    {
        return Err("Trusted host runner rejects shell metacharacters.".to_string());
    }

    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in command.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ch if ch.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            ch => current.push(ch),
        }
    }

    if in_quotes {
        return Err("Trusted host runner rejects unterminated quotes.".to_string());
    }
    if !current.is_empty() {
        parts.push(current);
    }
    if parts.is_empty() {
        return Err("Trusted host runner command is empty.".to_string());
    }
    Ok(parts)
}

fn summarize_output(value: &str, limit: usize) -> (String, bool) {
    let limit = limit.max(1);
    if value.len() <= limit {
        return (value.to_string(), false);
    }

    let mut output = String::new();
    for ch in value.chars() {
        if output.len() + ch.len_utf8() > limit {
            break;
        }
        output.push(ch);
    }
    output.push_str("...");
    (output, true)
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
