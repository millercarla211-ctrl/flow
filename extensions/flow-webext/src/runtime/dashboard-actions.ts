import type { FlowDashboardProductUiActionBinding } from "./protocol";
import type { FlowDashboardActionKind } from "./protocol";

export type FlowDashboardCommandPermission = "allowed" | "confirmation-required" | "blocked";
export type FlowDashboardCommandStatus = "prepared" | "blocked" | "failed";

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

function stringValue(...values: unknown[]) {
  for (const value of values) {
    if (typeof value === "string") {
      return value;
    }
    if (typeof value === "number" && Number.isFinite(value)) {
      return String(value);
    }
  }
  return "";
}

function dashboardKind(value: string): FlowDashboardActionKind {
  if (value === "run-check" || value === "recover" || value === "capture" || value === "open") {
    return value;
  }
  return "open";
}
