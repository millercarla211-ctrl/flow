import type { FlowDashboardProductUiActionBinding } from "./protocol";
import type { FlowDashboardActionKind } from "./protocol";

export type FlowDashboardCommandPermission = "allowed" | "confirmation-required" | "blocked";
export type FlowDashboardCommandStatus =
  | "prepared"
  | "blocked"
  | "failed"
  | "succeeded"
  | "timed-out"
  | "cancelled"
  | "denied";

export type FlowDashboardLiveRunnerStatus =
  | "pending"
  | "running"
  | "succeeded"
  | "failed"
  | "timed-out"
  | "cancelled"
  | "denied"
  | "stale";

export interface FlowDashboardCommandDispatchOptions {
  confirmed?: boolean;
  now?: string;
}

export interface FlowDashboardCommandResult {
  resultId: string;
  actionId: string;
  label: string;
  command: string;
  kind: FlowDashboardProductUiActionBinding["kind"];
  permission: FlowDashboardCommandPermission;
  status: FlowDashboardCommandStatus;
  message: string;
  nextStep: string;
  createdAt: string;
}

export interface FlowDashboardRunnerStatusSummary {
  status: FlowDashboardCommandStatus;
  count: number;
  title: string;
  description: string;
  tone: string;
}

export interface FlowDashboardRunnerAffordance {
  id: string;
  kind: "copy-command" | "retry" | "cancel" | string;
  actionId: string;
  status: FlowDashboardCommandStatus;
  label: string;
  command: string;
  detail: string;
  requiresApproval: boolean;
  disabled: boolean;
  disabledReason: string | null;
}

export interface FlowDashboardRunnerOperatorNote {
  id: string;
  label: string;
  detail: string;
  releaseReviewPath: string;
}

export interface FlowDashboardRunnerApprovalControl {
  id: string;
  kind: string;
  label: string;
  command: string;
  detail: string;
  ariaLabel: string;
  keyboardShortcut: {
    key: string;
    label: string;
    detail: string;
  } | null;
  requiresReason: boolean;
  requiresApproval: boolean;
  disabled: boolean;
  disabledReason: string | null;
}

export interface FlowDashboardRunnerApprovalUiReport {
  historyJson: string;
  resultCount: number;
  modalId: string;
  latestActionId: string | null;
  title: string;
  body: string;
  commandPreview: string;
  reasonLabel: string;
  reasonPlaceholder: string;
  auditReasonRequired: boolean;
  controls: FlowDashboardRunnerApprovalControl[];
  snoozeOptions: Array<{ id: string; label: string; durationSeconds: number }>;
  undoNote: string;
  releaseReviewPath: string;
}

export interface FlowDashboardRunnerUxReport {
  historyJson: string;
  resultCount: number;
  latestStatus: FlowDashboardCommandStatus | null;
  statusSummaries: FlowDashboardRunnerStatusSummary[];
  affordances: FlowDashboardRunnerAffordance[];
  operatorNotes: FlowDashboardRunnerOperatorNote[];
}

export interface FlowDashboardLiveRunnerRecord {
  jobId: string;
  actionId: string;
  label: string;
  command: string;
  status: FlowDashboardLiveRunnerStatus;
  message: string;
  localOnly: boolean;
  approved: boolean;
  timeoutMs: number;
  staleAfterMs: number;
  createdAtUnixMs: string;
  updatedAtUnixMs: string;
  finishedAtUnixMs: string | null;
  historyJson: string | null;
  recoveryCommand: string;
  cleanupCommand: string;
}

export interface FlowDashboardLiveRunnerState {
  stateJson: string;
  generatedAtUnixMs: string;
  recordCount: number;
  pendingCount: number;
  runningCount: number;
  finishedCount: number;
  staleCount: number;
  records: FlowDashboardLiveRunnerRecord[];
  staleRecoveryCopy: string;
}

export interface FlowDashboardRunnerCancellationControl {
  id: string;
  jobId: string;
  actionId: string;
  kind: string;
  label: string;
  command: string;
  detail: string;
  requiresReason: boolean;
  disabled: boolean;
  disabledReason: string | null;
}

export interface FlowDashboardRunnerCancellationDraft {
  storageKey: string;
  defaultReason: string;
  autosaveHint: string;
}

export interface FlowDashboardRunnerCancellationUxReport {
  stateJson: string;
  recordCount: number;
  activeCount: number;
  staleCount: number;
  denialCount: number;
  controls: FlowDashboardRunnerCancellationControl[];
  draft: FlowDashboardRunnerCancellationDraft;
  guidance: string[];
}

const RESULT_LIMIT = 8;
const RESULT_STORAGE_PREFIX = "flow.dashboard.actionResults.";

function resultId(action: FlowDashboardProductUiActionBinding, createdAt: string) {
  const compactTime = createdAt.replace(/[^0-9]/g, "").slice(0, 14);
  return `${action.actionId}.${compactTime}`;
}

export function dispatchDashboardCommand(
  action: FlowDashboardProductUiActionBinding,
  options: FlowDashboardCommandDispatchOptions = {},
): FlowDashboardCommandResult {
  const createdAt = options.now ?? new Date().toISOString();

  if (!action.localOnly) {
    return {
      resultId: resultId(action, createdAt),
      actionId: action.actionId,
      label: action.label,
      command: action.command,
      kind: action.kind,
      permission: "blocked",
      status: "blocked",
      message: "Remote dashboard actions are blocked in local-only mode.",
      nextStep: "Import a local dashboard JSON export or run this action from a trusted local host.",
      createdAt,
    };
  }

  if (!action.enabled || action.buttonState.disabled) {
    return {
      resultId: resultId(action, createdAt),
      actionId: action.actionId,
      label: action.label,
      command: action.command,
      kind: action.kind,
      permission: "blocked",
      status: "blocked",
      message: action.buttonState.disabledReason ?? "This dashboard action is disabled.",
      nextStep: "Resolve the blocked readiness item, then refresh the dashboard JSON.",
      createdAt,
    };
  }

  if (!action.command.trim()) {
    return {
      resultId: resultId(action, createdAt),
      actionId: action.actionId,
      label: action.label,
      command: action.command,
      kind: action.kind,
      permission: "blocked",
      status: "failed",
      message: "Dashboard action failed validation because no local command was provided.",
      nextStep: "Refresh the dashboard JSON from a trusted local export.",
      createdAt,
    };
  }

  if (action.buttonState.requiresConfirmation && !options.confirmed) {
    return {
      resultId: resultId(action, createdAt),
      actionId: action.actionId,
      label: action.label,
      command: action.command,
      kind: action.kind,
      permission: "confirmation-required",
      status: "blocked",
      message: "Operator confirmation is required before preparing this local command.",
      nextStep: "Confirm the action, then Friday will prepare the local command handoff.",
      createdAt,
    };
  }

  return {
    resultId: resultId(action, createdAt),
    actionId: action.actionId,
    label: action.label,
    command: action.command,
    kind: action.kind,
    permission: "allowed",
    status: "prepared",
    message: "Local command handoff prepared. Friday did not run anything silently.",
    nextStep: "Run the command locally from the trusted desktop host or terminal.",
    createdAt,
  };
}

export function dashboardCommandStorageKey(route: string) {
  return `${RESULT_STORAGE_PREFIX}${route.replace(/[^a-z0-9_-]/gi, "_")}`;
}

export function readDashboardCommandResults(route: string): FlowDashboardCommandResult[] {
  try {
    const raw = globalThis.localStorage?.getItem(dashboardCommandStorageKey(route));
    if (!raw) {
      return [];
    }

    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) {
      return [];
    }

    return parsed
      .filter((item): item is FlowDashboardCommandResult => {
        if (!item || typeof item !== "object") {
          return false;
        }
        const record = item as Record<string, unknown>;
        return (
          typeof record.resultId === "string" &&
          typeof record.actionId === "string" &&
          typeof record.command === "string" &&
          typeof record.message === "string" &&
          typeof record.createdAt === "string"
        );
      })
      .slice(0, RESULT_LIMIT);
  } catch {
    return [];
  }
}

export function persistDashboardCommandResult(
  route: string,
  result: FlowDashboardCommandResult,
  previous: FlowDashboardCommandResult[],
) {
  const next = [result, ...previous.filter((item) => item.resultId !== result.resultId)].slice(
    0,
    RESULT_LIMIT,
  );

  try {
    globalThis.localStorage?.setItem(dashboardCommandStorageKey(route), JSON.stringify(next));
  } catch {
    // Local storage is a convenience cache here; command safety does not depend on it.
  }

  return next;
}

export function normalizeDashboardHostCommandResults(value: unknown): FlowDashboardCommandResult[] {
  if (!value || typeof value !== "object") {
    return [];
  }

  const records = (value as { records?: unknown }).records;
  if (!Array.isArray(records)) {
    return [];
  }

  return records
    .map((item): FlowDashboardCommandResult | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const record = item as Record<string, unknown>;
      const audit =
        record.audit && typeof record.audit === "object"
          ? (record.audit as Record<string, unknown>)
          : {};
      const actionId = stringValue(record.action_id, record.actionId);
      const command = stringValue(record.command);
      const status = stringValue(record.status);
      const approvalState = stringValue(record.approval_state, record.approvalState);
      const recordedAt = stringValue(audit.recorded_at_unix_ms, record.created_at, record.createdAt);

      if (!actionId || !command) {
        return null;
      }

      return {
        resultId: `host.${actionId}.${recordedAt || "imported"}`,
        actionId,
        label: stringValue(record.label) || actionId,
        command,
        kind: dashboardKind(stringValue(record.kind)),
        permission:
          approvalState === "required"
            ? "confirmation-required"
            : approvalState === "blocked"
              ? "blocked"
              : "allowed",
        status:
          status === "awaiting-approval"
            ? "prepared"
            : status === "blocked"
              ? "blocked"
              : "failed",
        message:
          stringValue(audit.stdout_summary, audit.stdoutSummary, record.summary) ||
          "Host bridge record imported.",
        nextStep:
          stringValue(record.blocked_reason, record.blockedReason, record.approval_prompt) ||
          "Review this host bridge record before execution.",
        createdAt: recordedAt || new Date().toISOString(),
      };
    })
    .filter((item): item is FlowDashboardCommandResult => item !== null);
}

export function normalizeTrustedHostRunnerResults(value: unknown): FlowDashboardCommandResult[] {
  const records = Array.isArray((value as { records?: unknown })?.records)
    ? ((value as { records: unknown[] }).records)
    : [value];

  return records
    .map((item): FlowDashboardCommandResult | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const record = item as Record<string, unknown>;
      const actionId = stringValue(record.action_id, record.actionId);
      const command = stringValue(record.command);
      const status = runnerStatus(stringValue(record.status));

      if (!actionId || !command) {
        return null;
      }

      return {
        resultId: `runner.${actionId}.${stringValue(record.recorded_at_unix_ms, record.recordedAtUnixMs) || "imported"}`,
        actionId,
        label: stringValue(record.label) || actionId,
        command,
        kind: "run-check",
        permission: stringValue(record.approved) === "true" ? "allowed" : "blocked",
        status,
        message:
          stringValue(record.stdout_summary, record.stdoutSummary) ||
          stringValue(record.stderr_summary, record.stderrSummary) ||
          "Trusted host runner result imported.",
        nextStep:
          status === "succeeded"
            ? "Review the persisted trusted host runner history."
            : "Review the failure, approval, timeout, or cancellation state before retrying.",
        createdAt: stringValue(record.recorded_at_unix_ms, record.recordedAtUnixMs) || new Date().toISOString(),
      };
    })
    .filter((item): item is FlowDashboardCommandResult => item !== null);
}

export function normalizeTrustedHostRunnerUx(value: unknown): FlowDashboardRunnerUxReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const record = value as Record<string, unknown>;
  const summaries = arrayValue(record.status_summaries, record.statusSummaries)
    .map((item): FlowDashboardRunnerStatusSummary | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const summary = item as Record<string, unknown>;
      return {
        status: runnerStatus(stringValue(summary.status)),
        count: numberValue(summary.count),
        title: stringValue(summary.title) || stringValue(summary.status),
        description: stringValue(summary.description),
        tone: stringValue(summary.tone) || "muted",
      };
    })
    .filter((item): item is FlowDashboardRunnerStatusSummary => item !== null);

  const affordances = arrayValue(record.affordances)
    .map((item): FlowDashboardRunnerAffordance | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const affordance = item as Record<string, unknown>;
      const id = stringValue(affordance.id);
      const command = stringValue(affordance.command);
      if (!id || !command) {
        return null;
      }
      return {
        id,
        kind: stringValue(affordance.kind) || "copy-command",
        actionId: stringValue(affordance.action_id, affordance.actionId),
        status: runnerStatus(stringValue(affordance.status)),
        label: stringValue(affordance.label) || id,
        command,
        detail: stringValue(affordance.detail),
        requiresApproval: booleanValue(affordance.requires_approval, affordance.requiresApproval),
        disabled: booleanValue(affordance.disabled),
        disabledReason:
          stringValue(affordance.disabled_reason, affordance.disabledReason) || null,
      };
    })
    .filter((item): item is FlowDashboardRunnerAffordance => item !== null);

  const operatorNotes = arrayValue(record.operator_notes, record.operatorNotes)
    .map((item): FlowDashboardRunnerOperatorNote | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const note = item as Record<string, unknown>;
      const id = stringValue(note.id);
      if (!id) {
        return null;
      }
      return {
        id,
        label: stringValue(note.label) || id,
        detail: stringValue(note.detail),
        releaseReviewPath: stringValue(note.release_review_path, note.releaseReviewPath),
      };
    })
    .filter((item): item is FlowDashboardRunnerOperatorNote => item !== null);

  if (summaries.length === 0 && affordances.length === 0 && operatorNotes.length === 0) {
    return null;
  }

  const latestStatus = stringValue(record.latest_status, record.latestStatus);
  return {
    historyJson: stringValue(record.history_json, record.historyJson),
    resultCount: numberValue(record.result_count, record.resultCount),
    latestStatus: latestStatus ? runnerStatus(latestStatus) : null,
    statusSummaries: summaries,
    affordances,
    operatorNotes,
  };
}

export function normalizeTrustedHostRunnerApprovalUi(
  value: unknown,
): FlowDashboardRunnerApprovalUiReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const record = value as Record<string, unknown>;
  const controls = arrayValue(record.controls)
    .map((item): FlowDashboardRunnerApprovalControl | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const control = item as Record<string, unknown>;
      const shortcut =
        control.keyboard_shortcut && typeof control.keyboard_shortcut === "object"
          ? (control.keyboard_shortcut as Record<string, unknown>)
          : control.keyboardShortcut && typeof control.keyboardShortcut === "object"
            ? (control.keyboardShortcut as Record<string, unknown>)
            : null;
      const id = stringValue(control.id);
      if (!id) {
        return null;
      }
      return {
        id,
        kind: stringValue(control.kind),
        label: stringValue(control.label) || id,
        command: stringValue(control.command),
        detail: stringValue(control.detail),
        ariaLabel: stringValue(control.aria_label, control.ariaLabel),
        keyboardShortcut: shortcut
          ? {
              key: stringValue(shortcut.key),
              label: stringValue(shortcut.label),
              detail: stringValue(shortcut.detail),
            }
          : null,
        requiresReason: booleanValue(control.requires_reason, control.requiresReason),
        requiresApproval: booleanValue(control.requires_approval, control.requiresApproval),
        disabled: booleanValue(control.disabled),
        disabledReason: stringValue(control.disabled_reason, control.disabledReason) || null,
      };
    })
    .filter((item): item is FlowDashboardRunnerApprovalControl => item !== null);

  if (controls.length === 0) {
    return null;
  }

  return {
    historyJson: stringValue(record.history_json, record.historyJson),
    resultCount: numberValue(record.result_count, record.resultCount),
    modalId: stringValue(record.modal_id, record.modalId) || "trusted-runner-approval",
    latestActionId: stringValue(record.latest_action_id, record.latestActionId) || null,
    title: stringValue(record.title) || "Approve trusted runner action",
    body: stringValue(record.body),
    commandPreview: stringValue(record.command_preview, record.commandPreview),
    reasonLabel: stringValue(record.reason_label, record.reasonLabel) || "Audit reason",
    reasonPlaceholder: stringValue(record.reason_placeholder, record.reasonPlaceholder),
    auditReasonRequired: booleanValue(
      record.audit_reason_required,
      record.auditReasonRequired,
    ),
    controls,
    snoozeOptions: arrayValue(record.snooze_options, record.snoozeOptions)
      .map((item) => {
        if (!item || typeof item !== "object") {
          return null;
        }
        const option = item as Record<string, unknown>;
        const id = stringValue(option.id);
        if (!id) {
          return null;
        }
        return {
          id,
          label: stringValue(option.label) || id,
          durationSeconds: numberValue(option.duration_seconds, option.durationSeconds),
        };
      })
      .filter((item): item is { id: string; label: string; durationSeconds: number } => {
        return item !== null;
      }),
    undoNote: stringValue(record.undo_note, record.undoNote),
    releaseReviewPath: stringValue(record.release_review_path, record.releaseReviewPath),
  };
}

export function normalizeTrustedHostLiveRunnerState(
  value: unknown,
): FlowDashboardLiveRunnerState | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const state =
    root.live_state && typeof root.live_state === "object"
      ? (root.live_state as Record<string, unknown>)
      : root.liveState && typeof root.liveState === "object"
        ? (root.liveState as Record<string, unknown>)
        : root;
  const records = arrayValue(state.records)
    .map((item): FlowDashboardLiveRunnerRecord | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const record = item as Record<string, unknown>;
      const jobId = stringValue(record.job_id, record.jobId);
      const actionId = stringValue(record.action_id, record.actionId);
      if (!jobId || !actionId) {
        return null;
      }
      return {
        jobId,
        actionId,
        label: stringValue(record.label) || actionId,
        command: stringValue(record.command),
        status: liveRunnerStatus(stringValue(record.status)),
        message: stringValue(record.message),
        localOnly: booleanValue(record.local_only, record.localOnly),
        approved: booleanValue(record.approved),
        timeoutMs: numberValue(record.timeout_ms, record.timeoutMs),
        staleAfterMs: numberValue(record.stale_after_ms, record.staleAfterMs),
        createdAtUnixMs: stringValue(record.created_at_unix_ms, record.createdAtUnixMs),
        updatedAtUnixMs: stringValue(record.updated_at_unix_ms, record.updatedAtUnixMs),
        finishedAtUnixMs:
          stringValue(record.finished_at_unix_ms, record.finishedAtUnixMs) || null,
        historyJson: stringValue(record.history_json, record.historyJson) || null,
        recoveryCommand: stringValue(record.recovery_command, record.recoveryCommand),
        cleanupCommand: stringValue(record.cleanup_command, record.cleanupCommand),
      };
    })
    .filter((item): item is FlowDashboardLiveRunnerRecord => item !== null);

  if (records.length === 0 && !stringValue(state.state_json, state.stateJson)) {
    return null;
  }

  return {
    stateJson: stringValue(state.state_json, state.stateJson),
    generatedAtUnixMs: stringValue(state.generated_at_unix_ms, state.generatedAtUnixMs),
    recordCount: numberValue(state.record_count, state.recordCount),
    pendingCount: numberValue(state.pending_count, state.pendingCount),
    runningCount: numberValue(state.running_count, state.runningCount),
    finishedCount: numberValue(state.finished_count, state.finishedCount),
    staleCount: numberValue(state.stale_count, state.staleCount),
    records,
    staleRecoveryCopy: stringValue(state.stale_recovery_copy, state.staleRecoveryCopy),
  };
}

export function normalizeTrustedHostRunnerCancellationUx(
  value: unknown,
): FlowDashboardRunnerCancellationUxReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const report =
    root.cancellation_ux && typeof root.cancellation_ux === "object"
      ? (root.cancellation_ux as Record<string, unknown>)
      : root.cancellationUx && typeof root.cancellationUx === "object"
        ? (root.cancellationUx as Record<string, unknown>)
        : root;
  const controls = arrayValue(report.controls)
    .map((item): FlowDashboardRunnerCancellationControl | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const control = item as Record<string, unknown>;
      const id = stringValue(control.id);
      if (!id) {
        return null;
      }
      return {
        id,
        jobId: stringValue(control.job_id, control.jobId),
        actionId: stringValue(control.action_id, control.actionId),
        kind: stringValue(control.kind),
        label: stringValue(control.label) || id,
        command: stringValue(control.command),
        detail: stringValue(control.detail),
        requiresReason: booleanValue(control.requires_reason, control.requiresReason),
        disabled: booleanValue(control.disabled),
        disabledReason: stringValue(control.disabled_reason, control.disabledReason) || null,
      };
    })
    .filter((item): item is FlowDashboardRunnerCancellationControl => item !== null);

  if (controls.length === 0) {
    return null;
  }

  const draftRecord =
    report.draft && typeof report.draft === "object"
      ? (report.draft as Record<string, unknown>)
      : {};

  return {
    stateJson: stringValue(report.state_json, report.stateJson),
    recordCount: numberValue(report.record_count, report.recordCount),
    activeCount: numberValue(report.active_count, report.activeCount),
    staleCount: numberValue(report.stale_count, report.staleCount),
    denialCount: numberValue(report.denial_count, report.denialCount),
    controls,
    draft: {
      storageKey:
        stringValue(draftRecord.storage_key, draftRecord.storageKey) ||
        "flow.dashboard.runnerCancellationDrafts",
      defaultReason:
        stringValue(draftRecord.default_reason, draftRecord.defaultReason) ||
        "Operator reviewed live runner state",
      autosaveHint:
        stringValue(draftRecord.autosave_hint, draftRecord.autosaveHint) ||
        "Cancellation and retry reasons are remembered locally in this browser only.",
    },
    guidance: arrayValue(report.guidance)
      .map((item) => stringValue(item))
      .filter(Boolean),
  };
}

export function buildTrustedHostRunnerCancellationUx(
  liveState: FlowDashboardLiveRunnerState,
): FlowDashboardRunnerCancellationUxReport | null {
  const inputDir = liveState.stateJson.replace(/[\\/][^\\/]*$/, "") || "tmp/friday-dashboard";
  const controls = liveState.records.flatMap((record) => {
    const historyJson = record.historyJson || `${inputDir}/trusted-host-runner-history.json`;
    const bridgePrefix = `flow --friday-trusted-host-bridge-runner ${inputDir} --action-id ${record.actionId} --state ${liveState.stateJson} --history ${historyJson}`;
    const recordControls: FlowDashboardRunnerCancellationControl[] = [];

    if (record.status === "pending" || record.status === "running") {
      recordControls.push({
        id: `cancel-${record.jobId}`,
        jobId: record.jobId,
        actionId: record.actionId,
        kind: "cancel",
        label: `Cancel ${record.label}`,
        command: `${bridgePrefix} --cancel --reason "<cancel reason>"`,
        detail: "Stops this live trusted runner before another command is approved.",
        requiresReason: true,
        disabled: !record.localOnly,
        disabledReason: record.localOnly
          ? null
          : "Only local trusted runner records can be cancelled here.",
      });
    }

    if (record.status === "stale") {
      recordControls.push({
        id: `cleanup-${record.jobId}`,
        jobId: record.jobId,
        actionId: record.actionId,
        kind: "cleanup-stale",
        label: `Clean up ${record.label}`,
        command: `flow --friday-trusted-host-live-state ${liveState.stateJson} --history ${historyJson}`,
        detail: "Refreshes the live state file so stale runner records stop looking active.",
        requiresReason: false,
        disabled: false,
        disabledReason: null,
      });
      recordControls.push({
        id: `retry-stale-${record.jobId}`,
        jobId: record.jobId,
        actionId: record.actionId,
        kind: "retry",
        label: `Retry ${record.label}`,
        command: `${bridgePrefix} --approve --execute --reason "<retry reason>"`,
        detail: "Retries the action through the bridge after stale state has been reviewed.",
        requiresReason: true,
        disabled: false,
        disabledReason: null,
      });
    }

    if (record.status === "denied") {
      recordControls.push({
        id: `recover-denied-${record.jobId}`,
        jobId: record.jobId,
        actionId: record.actionId,
        kind: "denial-recovery",
        label: `Recover ${record.label}`,
        command: `${bridgePrefix} --approve --execute --reason "<denial recovery reason>"`,
        detail: "Approves and reruns this denied command with an explicit correction reason.",
        requiresReason: true,
        disabled: false,
        disabledReason: null,
      });
    }

    return recordControls;
  });

  if (controls.length === 0) {
    return null;
  }

  return {
    stateJson: liveState.stateJson,
    recordCount: liveState.recordCount,
    activeCount: liveState.pendingCount + liveState.runningCount,
    staleCount: liveState.staleCount,
    denialCount: liveState.records.filter((record) => record.status === "denied").length,
    controls,
    draft: {
      storageKey: "flow.dashboard.runnerCancellationDrafts",
      defaultReason: "Operator reviewed live runner state",
      autosaveHint: "Cancellation and retry reasons are remembered locally in this browser only.",
    },
    guidance: [
      "Cancel active live records before approving another command for the same action.",
      "Clean up stale live records, then retry from a fresh bridge import if the action still matters.",
      "Denial recovery always requires a short operator reason so the audit trail explains the correction.",
    ],
  };
}

function stringValue(...values: unknown[]) {
  for (const value of values) {
    if (typeof value === "string") {
      return value;
    }
    if (typeof value === "number" && Number.isFinite(value)) {
      return String(value);
    }
    if (typeof value === "boolean") {
      return String(value);
    }
  }
  return "";
}

function numberValue(...values: unknown[]) {
  for (const value of values) {
    if (typeof value === "number" && Number.isFinite(value)) {
      return value;
    }
    if (typeof value === "string" && value.trim()) {
      const parsed = Number(value);
      if (Number.isFinite(parsed)) {
        return parsed;
      }
    }
  }
  return 0;
}

function booleanValue(...values: unknown[]) {
  for (const value of values) {
    if (typeof value === "boolean") {
      return value;
    }
    if (typeof value === "string") {
      if (value === "true") {
        return true;
      }
      if (value === "false") {
        return false;
      }
    }
  }
  return false;
}

function arrayValue(...values: unknown[]) {
  for (const value of values) {
    if (Array.isArray(value)) {
      return value;
    }
  }
  return [];
}

function dashboardKind(value: string): FlowDashboardActionKind {
  if (value === "run-check" || value === "recover" || value === "capture" || value === "open") {
    return value;
  }
  return "open";
}

function runnerStatus(value: string): FlowDashboardCommandStatus {
  if (
    value === "succeeded" ||
    value === "timed-out" ||
    value === "cancelled" ||
    value === "denied" ||
    value === "failed"
  ) {
    return value;
  }
  return "failed";
}

function liveRunnerStatus(value: string): FlowDashboardLiveRunnerStatus {
  if (
    value === "pending" ||
    value === "running" ||
    value === "succeeded" ||
    value === "failed" ||
    value === "timed-out" ||
    value === "cancelled" ||
    value === "denied" ||
    value === "stale"
  ) {
    return value;
  }
  return "stale";
}
