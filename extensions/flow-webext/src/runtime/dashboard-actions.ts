import type { FlowDashboardProductUiActionBinding } from "./protocol";
import type { FlowDashboardActionKind } from "./protocol";
import type { FlowDashboardPanelStatus } from "./protocol";

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

export interface FlowDashboardRunnerReviewFilter {
  status: FlowDashboardCommandStatus | "all";
  actionId: string | null;
  sinceUnixMs: string | null;
  untilUnixMs: string | null;
  limit: number;
}

export interface FlowDashboardRunnerReviewRecord {
  resultId: string;
  actionId: string;
  label: string;
  status: FlowDashboardCommandStatus;
  severity: string;
  command: string;
  summary: string;
  releaseGate: string;
  operatorReason: string | null;
  recordedAtUnixMs: string;
  durationMs: number;
  exitCode: number | null;
}

export interface FlowDashboardRunnerReleaseGateSummary {
  id: string;
  title: string;
  severity: string;
  count: number;
  detail: string;
  nextAction: string;
}

export interface FlowDashboardRunnerIncidentNote {
  id: string;
  actionId: string;
  status: FlowDashboardCommandStatus;
  severity: string;
  title: string;
  body: string;
  exportMarkdown: string;
  recordedAtUnixMs: string;
}

export interface FlowDashboardRunnerOperatorReviewReport {
  historyJson: string;
  reviewId: string;
  generatedAtUnixMs: string;
  filters: FlowDashboardRunnerReviewFilter;
  recordCount: number;
  matchedCount: number;
  readyCount: number;
  blockedCount: number;
  releaseGateStatus: string;
  releaseGateSummaries: FlowDashboardRunnerReleaseGateSummary[];
  incidentNotes: FlowDashboardRunnerIncidentNote[];
  records: FlowDashboardRunnerReviewRecord[];
}

export interface FlowDashboardRunnerReleaseEvidenceFile {
  id: string;
  label: string;
  kind: string;
  path: string;
  required: boolean;
  present: boolean;
  bytes: number;
  sha256: string | null;
  warning: string | null;
}

export interface FlowDashboardRunnerReleasePackageManifest {
  packageId: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  packageJson: string;
  dashboardExportDir: string;
  historyJson: string;
  liveStateJson: string;
  releaseReviewJson: string;
  dashboardIndexJson: string;
  evidenceCount: number;
  missingCount: number;
  warningCount: number;
  packageSignature: string;
  commands: string[];
  files: FlowDashboardRunnerReleaseEvidenceFile[];
}

export interface FlowDashboardRunnerReleasePackageReport {
  summary: string;
  readyToShip: boolean;
  warnings: string[];
  manifest: FlowDashboardRunnerReleasePackageManifest;
  operatorReview: FlowDashboardRunnerOperatorReviewReport | null;
  cancellationUx: FlowDashboardRunnerCancellationUxReport | null;
  liveState: FlowDashboardLiveRunnerState | null;
  incidentMarkdown: string;
}

export interface FlowDashboardRunnerReleaseTimelineEntry {
  packageId: string;
  packageJson: string;
  generatedAtUnixMs: string;
  readyToShip: boolean;
  evidenceCount: number;
  missingCount: number;
  warningCount: number;
  staleWarningCount: number;
  packageSignature: string;
  missingEvidenceIds: string[];
  summary: string;
}

export interface FlowDashboardRunnerReleaseTimelineDiff {
  fromPackageId: string;
  toPackageId: string;
  evidenceDelta: number;
  missingDelta: number;
  warningDelta: number;
  staleWarningDelta: number;
  signatureChanged: boolean;
  newMissingEvidenceIds: string[];
  resolvedMissingEvidenceIds: string[];
  regression: boolean;
  summary: string;
}

export interface FlowDashboardRunnerReleaseTimeline {
  timelineId: string;
  timelineJson: string;
  generatedAtUnixMs: string;
  localOnly: boolean;
  packageCount: number;
  readyCount: number;
  blockedCount: number;
  latestPackageId: string | null;
  latestPackageJson: string | null;
  missingEvidenceRegressions: number;
  warningRegressions: number;
  signatureChanges: number;
  warnings: string[];
  entries: FlowDashboardRunnerReleaseTimelineEntry[];
  diffs: FlowDashboardRunnerReleaseTimelineDiff[];
}

export interface FlowReleaseChecklistBlocker {
  id: string;
  category: string;
  severity: "warning" | "blocking";
  title: string;
  detail: string;
  sourcePath: string;
  nextAction: string;
}

export interface FlowReleaseChecklistItem {
  id: string;
  title: string;
  ready: boolean;
  detail: string;
  sourcePath: string;
}

export interface FlowReleaseChecklistSignoff {
  id: string;
  checklistId: string;
  operator: string;
  decision: "approved" | "needs-changes" | "blocked";
  reason: string;
  recordedAtUnixMs: string;
  localOnly: boolean;
}

export interface FlowReleaseOperatorChecklistReport {
  checklistId: string;
  checklistJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  readyToShip: boolean;
  summary: string;
  packageJson: string;
  timelineJson: string;
  dashboardExportDir: string;
  todoPath: string;
  changelogPath: string;
  signoffJson: string;
  readyCount: number;
  totalCount: number;
  warningCount: number;
  blockingCount: number;
  signoffRequired: boolean;
  signoffCount: number;
  latestSignoff: FlowReleaseChecklistSignoff | null;
  blockers: FlowReleaseChecklistBlocker[];
  checklist: FlowReleaseChecklistItem[];
  signoffs: FlowReleaseChecklistSignoff[];
  commands: string[];
}

export type FlowReleaseQaCheckStatus = "passed" | "warning" | "failed" | "missing" | "stale";

export interface FlowReleaseQaCheck {
  id: string;
  label: string;
  command: string;
  resultPath: string;
  required: boolean;
  present: boolean;
  stale: boolean;
  bytes: number;
  status: FlowReleaseQaCheckStatus;
  summary: string;
  nextAction: string;
}

export interface FlowReleaseQaCommandCenterReport {
  reportId: string;
  reportJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  scoreOutOf100: number;
  readyToShip: boolean;
  summary: string;
  checklistJson: string;
  packageJson: string;
  timelineJson: string;
  warningCount: number;
  blockingCount: number;
  staleCount: number;
  missingCount: number;
  checks: FlowReleaseQaCheck[];
  commands: string[];
}

export interface FlowReleaseEvidenceExportKitFile {
  id: string;
  label: string;
  kind: string;
  path: string;
  required: boolean;
  present: boolean;
  stale: boolean;
  bytes: number;
  sha256: string | null;
  warning: string | null;
}

export interface FlowReleaseEvidenceExportKitManifest {
  kitId: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  kitJson: string;
  exportDir: string;
  fileCount: number;
  requiredCount: number;
  missingCount: number;
  staleCount: number;
  warningCount: number;
  manifestSha256: string;
  commands: string[];
  files: FlowReleaseEvidenceExportKitFile[];
}

export interface FlowReleaseEvidenceExportKitReport {
  summary: string;
  readyToAttach: boolean;
  status: FlowDashboardPanelStatus;
  checklistReady: boolean | null;
  qaScoreOutOf100: number | null;
  qaReadyToShip: boolean | null;
  packageReadyToShip: boolean | null;
  timelinePackageCount: number | null;
  signoffCount: number;
  warnings: string[];
  operatorCopy: string;
  manifest: FlowReleaseEvidenceExportKitManifest;
}

export type FlowReleaseDeploymentGateDecision = "go" | "no-go" | "draft";
export type FlowReleaseDeploymentGateReasonCategory =
  | "missing-evidence"
  | "stale-checks"
  | "blocked-qa"
  | "unsigned-release"
  | "dashboard-state"
  | "target-mismatch";

export interface FlowReleaseDeploymentTarget {
  id: string;
  label: string;
  environment: string;
  provider: string;
  url: string | null;
  localOnlyRequired: boolean;
  requiresVercel: boolean;
  expectedProductName: string;
  rollbackNote: string;
}

export interface FlowReleaseDeploymentGateReason {
  id: string;
  category: FlowReleaseDeploymentGateReasonCategory;
  severity: "blocking" | "warning";
  title: string;
  detail: string;
  sourcePath: string;
  nextAction: string;
}

export interface FlowReleaseDeploymentGateChecklistItem {
  id: string;
  title: string;
  ready: boolean;
  detail: string;
  sourcePath: string;
}

export interface FlowReleaseDeploymentGateReport {
  gateId: string;
  gateJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  decision: FlowReleaseDeploymentGateDecision;
  readyToDeploy: boolean;
  scoreOutOf100: number;
  summary: string;
  target: FlowReleaseDeploymentTarget;
  exportKitJson: string;
  qaJson: string;
  checklistJson: string;
  packageJson: string;
  timelineJson: string;
  dashboardExportDir: string;
  noDeployReasonCount: number;
  warningCount: number;
  readyCount: number;
  totalCount: number;
  reasons: FlowReleaseDeploymentGateReason[];
  checklist: FlowReleaseDeploymentGateChecklistItem[];
  deployChecklist: string[];
  rollbackNote: string;
  operatorCopy: string;
  commands: string[];
}

export interface FlowReleaseCandidateArchiveEntry {
  candidateId: string;
  gateId: string;
  gateJson: string;
  exportKitJson: string;
  generatedAtUnixMs: string;
  productName: string;
  decision: FlowReleaseDeploymentGateDecision;
  scoreOutOf100: number;
  readyToDeploy: boolean;
  target: FlowReleaseDeploymentTarget;
  noDeployReasonCount: number;
  warningCount: number;
  reasonIds: string[];
  exportKitManifestSha256: string | null;
  rollbackNote: string;
  summary: string;
}

export interface FlowReleaseCandidateArchiveDiff {
  fromCandidateId: string;
  toCandidateId: string;
  scoreDelta: number;
  decisionChanged: boolean;
  targetChanged: boolean;
  evidenceChecksumChanged: boolean;
  newBlockerIds: string[];
  resolvedBlockerIds: string[];
  regression: boolean;
  summary: string;
}

export interface FlowReleaseCandidateArchive {
  archiveId: string;
  archiveJson: string;
  generatedAtUnixMs: string;
  localOnly: boolean;
  candidateCount: number;
  latestCandidateId: string | null;
  latestDecision: FlowReleaseDeploymentGateDecision | null;
  latestScoreOutOf100: number | null;
  goCount: number;
  noGoCount: number;
  draftCount: number;
  regressionCount: number;
  entries: FlowReleaseCandidateArchiveEntry[];
  diffs: FlowReleaseCandidateArchiveDiff[];
  commands: string[];
}

export type FlowReleasePromotionDecision =
  | "promoted"
  | "held"
  | "rolled-back"
  | "superseded"
  | "abandoned";

export interface FlowReleasePromotionPostCheck {
  id: string;
  label: string;
  resultPath: string;
  required: boolean;
  present: boolean;
  bytes: number;
  summary: string;
  nextAction: string;
}

export interface FlowReleasePromotionRecord {
  promotionId: string;
  candidateId: string;
  archiveJson: string;
  gateJson: string;
  exportKitJson: string;
  recordedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  decision: FlowReleasePromotionDecision;
  operator: string;
  reason: string;
  deploymentNote: string;
  target: FlowReleaseDeploymentTarget;
  rollbackReference: string;
  candidateScoreOutOf100: number;
  candidateReadyToDeploy: boolean;
  candidateBlockerCount: number;
  postPromotionRequiredCount: number;
  postPromotionMissingCount: number;
  postPromotionChecks: FlowReleasePromotionPostCheck[];
  summary: string;
}

export interface FlowReleasePromotionLedger {
  ledgerId: string;
  ledgerJson: string;
  generatedAtUnixMs: string;
  localOnly: boolean;
  recordCount: number;
  promotedCount: number;
  heldCount: number;
  rolledBackCount: number;
  supersededCount: number;
  abandonedCount: number;
  postPromotionMissingCount: number;
  activePromotionId: string | null;
  activeCandidateId: string | null;
  activeRollbackReference: string | null;
  latestDecision: FlowReleasePromotionDecision | null;
  latestDeploymentNote: string | null;
  warnings: string[];
  records: FlowReleasePromotionRecord[];
  commands: string[];
}

export type FlowReleasePostPromotionCheckStatus =
  | "passed"
  | "warning"
  | "failed"
  | "missing"
  | "stale";

export interface FlowReleasePostPromotionCheck {
  id: string;
  label: string;
  sourcePath: string;
  required: boolean;
  present: boolean;
  stale: boolean;
  bytes: number;
  status: FlowReleasePostPromotionCheckStatus;
  summary: string;
  nextAction: string;
}

export interface FlowReleasePostPromotionIncidentNote {
  id: string;
  path: string;
  present: boolean;
  bytes: number;
  summary: string;
}

export interface FlowReleasePostPromotionMonitorReport {
  monitorId: string;
  monitorJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  scoreOutOf100: number;
  readyForStable: boolean;
  promotionLedgerJson: string;
  qaJson: string;
  dashboardSmokeResultPath: string;
  activeCandidateId: string | null;
  activePromotionId: string | null;
  activeRollbackReference: string | null;
  latestDecision: FlowReleasePromotionDecision | null;
  promotedCount: number;
  incidentNoteCount: number;
  missingEvidenceCount: number;
  staleCount: number;
  warningCount: number;
  blockingCount: number;
  checks: FlowReleasePostPromotionCheck[];
  incidentNotes: FlowReleasePostPromotionIncidentNote[];
  warnings: string[];
  summary: string;
  commands: string[];
}

export type FlowReleaseRollbackDrillCheckStatus =
  | "passed"
  | "warning"
  | "failed"
  | "missing"
  | "stale";

export interface FlowReleaseRollbackDrillCheck {
  id: string;
  label: string;
  sourcePath: string;
  required: boolean;
  present: boolean;
  stale: boolean;
  bytes: number;
  status: FlowReleaseRollbackDrillCheckStatus;
  summary: string;
  nextAction: string;
}

export interface FlowReleaseRollbackDrillReport {
  drillId: string;
  drillJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  scoreOutOf100: number;
  readyToRollback: boolean;
  readyForStable: boolean;
  activeCandidateId: string | null;
  activePromotionId: string | null;
  activeRollbackReference: string | null;
  latestPromotionDecision: FlowReleasePromotionDecision | null;
  deploymentGateDecision: FlowReleaseDeploymentGateDecision | null;
  postPromotionMonitorJson: string;
  promotionLedgerJson: string;
  candidateArchiveJson: string;
  deploymentGateJson: string;
  rollbackCommand: string;
  dryRunCommand: string;
  operator: string;
  reason: string;
  blockingCount: number;
  warningCount: number;
  staleCount: number;
  missingEvidenceCount: number;
  checks: FlowReleaseRollbackDrillCheck[];
  blockedReasons: string[];
  summary: string;
  commands: string[];
}

export type FlowReleaseStabilityBoardCategory =
  | "deployment-readiness"
  | "qa-health"
  | "candidate-regression"
  | "promotion-state"
  | "post-promotion-freshness"
  | "rollback-recovery";

export type FlowReleaseStabilityBoardCheckStatus =
  | "passed"
  | "warning"
  | "failed"
  | "missing"
  | "stale";

export interface FlowReleaseStabilityBoardCheck {
  id: string;
  label: string;
  category: FlowReleaseStabilityBoardCategory;
  sourcePath: string;
  required: boolean;
  present: boolean;
  stale: boolean;
  bytes: number;
  status: FlowReleaseStabilityBoardCheckStatus;
  summary: string;
  nextAction: string;
}

export interface FlowReleaseStabilityBoardEvidenceLink {
  id: string;
  label: string;
  path: string;
  present: boolean;
}

export interface FlowReleaseStabilityBoardReport {
  boardId: string;
  boardJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  scoreOutOf100: number;
  readyForCheckpoint: boolean;
  readyToDeploy: boolean;
  stableAfterPromotion: boolean;
  recoverable: boolean;
  activeCandidateId: string | null;
  activePromotionId: string | null;
  activeRollbackReference: string | null;
  latestPromotionDecision: FlowReleasePromotionDecision | null;
  deploymentGateDecision: FlowReleaseDeploymentGateDecision | null;
  qaJson: string;
  candidateArchiveJson: string;
  promotionLedgerJson: string;
  postPromotionMonitorJson: string;
  rollbackDrillJson: string;
  deploymentGateJson: string;
  blockingCount: number;
  warningCount: number;
  staleCount: number;
  missingEvidenceCount: number;
  checks: FlowReleaseStabilityBoardCheck[];
  evidenceLinks: FlowReleaseStabilityBoardEvidenceLink[];
  activeRisks: string[];
  nextActions: string[];
  summary: string;
  commands: string[];
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

export function normalizeTrustedHostRunnerOperatorReview(
  value: unknown,
): FlowDashboardRunnerOperatorReviewReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const report =
    root.operator_review && typeof root.operator_review === "object"
      ? (root.operator_review as Record<string, unknown>)
      : root.operatorReview && typeof root.operatorReview === "object"
        ? (root.operatorReview as Record<string, unknown>)
        : root;
  const summaries = arrayValue(report.release_gate_summaries, report.releaseGateSummaries)
    .map((item): FlowDashboardRunnerReleaseGateSummary | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const summary = item as Record<string, unknown>;
      const id = stringValue(summary.id);
      if (!id) {
        return null;
      }
      return {
        id,
        title: stringValue(summary.title) || id,
        severity: stringValue(summary.severity) || "watch",
        count: numberValue(summary.count),
        detail: stringValue(summary.detail),
        nextAction: stringValue(summary.next_action, summary.nextAction),
      };
    })
    .filter((item): item is FlowDashboardRunnerReleaseGateSummary => item !== null);
  const incidentNotes = arrayValue(report.incident_notes, report.incidentNotes)
    .map((item): FlowDashboardRunnerIncidentNote | null => {
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
        actionId: stringValue(note.action_id, note.actionId),
        status: runnerStatus(stringValue(note.status)),
        severity: stringValue(note.severity) || "watch",
        title: stringValue(note.title) || id,
        body: stringValue(note.body),
        exportMarkdown: stringValue(note.export_markdown, note.exportMarkdown),
        recordedAtUnixMs: stringValue(note.recorded_at_unix_ms, note.recordedAtUnixMs),
      };
    })
    .filter((item): item is FlowDashboardRunnerIncidentNote => item !== null);
  const records = arrayValue(report.records)
    .map((item): FlowDashboardRunnerReviewRecord | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const record = item as Record<string, unknown>;
      const resultId = stringValue(record.result_id, record.resultId);
      if (!resultId) {
        return null;
      }
      return {
        resultId,
        actionId: stringValue(record.action_id, record.actionId),
        label: stringValue(record.label),
        status: runnerStatus(stringValue(record.status)),
        severity: stringValue(record.severity),
        command: stringValue(record.command),
        summary: stringValue(record.summary),
        releaseGate: stringValue(record.release_gate, record.releaseGate),
        operatorReason: stringValue(record.operator_reason, record.operatorReason) || null,
        recordedAtUnixMs: stringValue(record.recorded_at_unix_ms, record.recordedAtUnixMs),
        durationMs: numberValue(record.duration_ms, record.durationMs),
        exitCode:
          stringValue(record.exit_code, record.exitCode) === ""
            ? null
            : numberValue(record.exit_code, record.exitCode),
      };
    })
    .filter((item): item is FlowDashboardRunnerReviewRecord => item !== null);

  if (summaries.length === 0 && incidentNotes.length === 0 && records.length === 0) {
    return null;
  }

  const filterRecord =
    report.filters && typeof report.filters === "object"
      ? (report.filters as Record<string, unknown>)
      : {};
  const status = stringValue(filterRecord.status);

  return {
    historyJson: stringValue(report.history_json, report.historyJson),
    reviewId: stringValue(report.review_id, report.reviewId),
    generatedAtUnixMs: stringValue(report.generated_at_unix_ms, report.generatedAtUnixMs),
    filters: {
      status: status ? runnerStatus(status) : "all",
      actionId: stringValue(filterRecord.action_id, filterRecord.actionId) || null,
      sinceUnixMs: stringValue(filterRecord.since_unix_ms, filterRecord.sinceUnixMs) || null,
      untilUnixMs: stringValue(filterRecord.until_unix_ms, filterRecord.untilUnixMs) || null,
      limit: numberValue(filterRecord.limit),
    },
    recordCount: numberValue(report.record_count, report.recordCount),
    matchedCount: numberValue(report.matched_count, report.matchedCount),
    readyCount: numberValue(report.ready_count, report.readyCount),
    blockedCount: numberValue(report.blocked_count, report.blockedCount),
    releaseGateStatus: stringValue(report.release_gate_status, report.releaseGateStatus),
    releaseGateSummaries: summaries,
    incidentNotes,
    records,
  };
}

export function normalizeTrustedRunnerReleasePackage(
  value: unknown,
): FlowDashboardRunnerReleasePackageReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const report =
    root.release_package && typeof root.release_package === "object"
      ? (root.release_package as Record<string, unknown>)
      : root.releasePackage && typeof root.releasePackage === "object"
        ? (root.releasePackage as Record<string, unknown>)
        : root;
  const manifestRecord =
    report.manifest && typeof report.manifest === "object"
      ? (report.manifest as Record<string, unknown>)
      : null;
  if (!manifestRecord) {
    return null;
  }
  const files = arrayValue(manifestRecord.files)
    .map((item): FlowDashboardRunnerReleaseEvidenceFile | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const file = item as Record<string, unknown>;
      const id = stringValue(file.id);
      if (!id) {
        return null;
      }
      return {
        id,
        label: stringValue(file.label) || id,
        kind: stringValue(file.kind),
        path: stringValue(file.path),
        required: booleanValue(file.required),
        present: booleanValue(file.present),
        bytes: numberValue(file.bytes),
        sha256: stringValue(file.sha256) || null,
        warning: stringValue(file.warning) || null,
      };
    })
    .filter((item): item is FlowDashboardRunnerReleaseEvidenceFile => item !== null);

  if (files.length === 0) {
    return null;
  }

  return {
    summary: stringValue(report.summary),
    readyToShip: booleanValue(report.ready_to_ship, report.readyToShip),
    warnings: arrayValue(report.warnings)
      .map((item) => stringValue(item))
      .filter(Boolean),
    manifest: {
      packageId: stringValue(manifestRecord.package_id, manifestRecord.packageId),
      generatedAtUnixMs: stringValue(
        manifestRecord.generated_at_unix_ms,
        manifestRecord.generatedAtUnixMs,
      ),
      productName: stringValue(manifestRecord.product_name, manifestRecord.productName),
      localOnly: booleanValue(manifestRecord.local_only, manifestRecord.localOnly),
      packageJson: stringValue(manifestRecord.package_json, manifestRecord.packageJson),
      dashboardExportDir: stringValue(
        manifestRecord.dashboard_export_dir,
        manifestRecord.dashboardExportDir,
      ),
      historyJson: stringValue(manifestRecord.history_json, manifestRecord.historyJson),
      liveStateJson: stringValue(manifestRecord.live_state_json, manifestRecord.liveStateJson),
      releaseReviewJson: stringValue(
        manifestRecord.release_review_json,
        manifestRecord.releaseReviewJson,
      ),
      dashboardIndexJson: stringValue(
        manifestRecord.dashboard_index_json,
        manifestRecord.dashboardIndexJson,
      ),
      evidenceCount: numberValue(manifestRecord.evidence_count, manifestRecord.evidenceCount),
      missingCount: numberValue(manifestRecord.missing_count, manifestRecord.missingCount),
      warningCount: numberValue(manifestRecord.warning_count, manifestRecord.warningCount),
      packageSignature: stringValue(
        manifestRecord.package_signature,
        manifestRecord.packageSignature,
      ),
      commands: arrayValue(manifestRecord.commands)
        .map((item) => stringValue(item))
        .filter(Boolean),
      files,
    },
    operatorReview: normalizeTrustedHostRunnerOperatorReview(report.operator_review ?? report.operatorReview),
    cancellationUx: normalizeTrustedHostRunnerCancellationUx(report.cancellation_ux ?? report.cancellationUx),
    liveState: normalizeTrustedHostLiveRunnerState(report.live_state ?? report.liveState),
    incidentMarkdown: stringValue(report.incident_markdown, report.incidentMarkdown),
  };
}

export function normalizeTrustedRunnerReleaseTimeline(
  value: unknown,
): FlowDashboardRunnerReleaseTimeline | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const timeline =
    root.release_timeline && typeof root.release_timeline === "object"
      ? (root.release_timeline as Record<string, unknown>)
      : root.releaseTimeline && typeof root.releaseTimeline === "object"
        ? (root.releaseTimeline as Record<string, unknown>)
        : root;
  const entries = arrayValue(timeline.entries)
    .map((item): FlowDashboardRunnerReleaseTimelineEntry | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const entry = item as Record<string, unknown>;
      const packageId = stringValue(entry.package_id, entry.packageId);
      if (!packageId) {
        return null;
      }
      return {
        packageId,
        packageJson: stringValue(entry.package_json, entry.packageJson),
        generatedAtUnixMs: stringValue(entry.generated_at_unix_ms, entry.generatedAtUnixMs),
        readyToShip: booleanValue(entry.ready_to_ship, entry.readyToShip),
        evidenceCount: numberValue(entry.evidence_count, entry.evidenceCount),
        missingCount: numberValue(entry.missing_count, entry.missingCount),
        warningCount: numberValue(entry.warning_count, entry.warningCount),
        staleWarningCount: numberValue(entry.stale_warning_count, entry.staleWarningCount),
        packageSignature: stringValue(entry.package_signature, entry.packageSignature),
        missingEvidenceIds: arrayValue(entry.missing_evidence_ids, entry.missingEvidenceIds)
          .map((id) => stringValue(id))
          .filter(Boolean),
        summary: stringValue(entry.summary),
      };
    })
    .filter((item): item is FlowDashboardRunnerReleaseTimelineEntry => item !== null);
  const diffs = arrayValue(timeline.diffs)
    .map((item): FlowDashboardRunnerReleaseTimelineDiff | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const diff = item as Record<string, unknown>;
      const fromPackageId = stringValue(diff.from_package_id, diff.fromPackageId);
      const toPackageId = stringValue(diff.to_package_id, diff.toPackageId);
      if (!fromPackageId || !toPackageId) {
        return null;
      }
      return {
        fromPackageId,
        toPackageId,
        evidenceDelta: numberValue(diff.evidence_delta, diff.evidenceDelta),
        missingDelta: numberValue(diff.missing_delta, diff.missingDelta),
        warningDelta: numberValue(diff.warning_delta, diff.warningDelta),
        staleWarningDelta: numberValue(diff.stale_warning_delta, diff.staleWarningDelta),
        signatureChanged: booleanValue(diff.signature_changed, diff.signatureChanged),
        newMissingEvidenceIds: arrayValue(
          diff.new_missing_evidence_ids,
          diff.newMissingEvidenceIds,
        )
          .map((id) => stringValue(id))
          .filter(Boolean),
        resolvedMissingEvidenceIds: arrayValue(
          diff.resolved_missing_evidence_ids,
          diff.resolvedMissingEvidenceIds,
        )
          .map((id) => stringValue(id))
          .filter(Boolean),
        regression: booleanValue(diff.regression),
        summary: stringValue(diff.summary),
      };
    })
    .filter((item): item is FlowDashboardRunnerReleaseTimelineDiff => item !== null);

  if (entries.length === 0 && diffs.length === 0) {
    return null;
  }

  return {
    timelineId: stringValue(timeline.timeline_id, timeline.timelineId),
    timelineJson: stringValue(timeline.timeline_json, timeline.timelineJson),
    generatedAtUnixMs: stringValue(timeline.generated_at_unix_ms, timeline.generatedAtUnixMs),
    localOnly: booleanValue(timeline.local_only, timeline.localOnly),
    packageCount: numberValue(timeline.package_count, timeline.packageCount),
    readyCount: numberValue(timeline.ready_count, timeline.readyCount),
    blockedCount: numberValue(timeline.blocked_count, timeline.blockedCount),
    latestPackageId: stringValue(timeline.latest_package_id, timeline.latestPackageId) || null,
    latestPackageJson:
      stringValue(timeline.latest_package_json, timeline.latestPackageJson) || null,
    missingEvidenceRegressions: numberValue(
      timeline.missing_evidence_regressions,
      timeline.missingEvidenceRegressions,
    ),
    warningRegressions: numberValue(timeline.warning_regressions, timeline.warningRegressions),
    signatureChanges: numberValue(timeline.signature_changes, timeline.signatureChanges),
    warnings: arrayValue(timeline.warnings)
      .map((warning) => stringValue(warning))
      .filter(Boolean),
    entries,
    diffs,
  };
}

export function normalizeReleaseOperatorChecklist(
  value: unknown,
): FlowReleaseOperatorChecklistReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const report =
    root.release_checklist && typeof root.release_checklist === "object"
      ? (root.release_checklist as Record<string, unknown>)
      : root.releaseChecklist && typeof root.releaseChecklist === "object"
        ? (root.releaseChecklist as Record<string, unknown>)
        : root;
  const checklistId = stringValue(report.checklist_id, report.checklistId);
  const checklist = arrayValue(report.checklist)
    .map((item): FlowReleaseChecklistItem | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const record = item as Record<string, unknown>;
      const id = stringValue(record.id);
      if (!id) {
        return null;
      }
      return {
        id,
        title: stringValue(record.title) || id,
        ready: booleanValue(record.ready),
        detail: stringValue(record.detail),
        sourcePath: stringValue(record.source_path, record.sourcePath),
      };
    })
    .filter((item): item is FlowReleaseChecklistItem => item !== null);
  const blockers = arrayValue(report.blockers)
    .map((item): FlowReleaseChecklistBlocker | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const record = item as Record<string, unknown>;
      const id = stringValue(record.id);
      if (!id) {
        return null;
      }
      return {
        id,
        category: stringValue(record.category),
        severity: checklistSeverity(stringValue(record.severity)),
        title: stringValue(record.title) || id,
        detail: stringValue(record.detail),
        sourcePath: stringValue(record.source_path, record.sourcePath),
        nextAction: stringValue(record.next_action, record.nextAction),
      };
    })
    .filter((item): item is FlowReleaseChecklistBlocker => item !== null);
  const signoffs = arrayValue(report.signoffs)
    .map(normalizeReleaseSignoff)
    .filter((item): item is FlowReleaseChecklistSignoff => item !== null);

  if (!checklistId && checklist.length === 0 && blockers.length === 0) {
    return null;
  }

  return {
    checklistId,
    checklistJson: stringValue(report.checklist_json, report.checklistJson),
    generatedAtUnixMs: stringValue(report.generated_at_unix_ms, report.generatedAtUnixMs),
    productName: stringValue(report.product_name, report.productName),
    localOnly: booleanValue(report.local_only, report.localOnly),
    status: panelStatus(stringValue(report.status)),
    readyToShip: booleanValue(report.ready_to_ship, report.readyToShip),
    summary: stringValue(report.summary),
    packageJson: stringValue(report.package_json, report.packageJson),
    timelineJson: stringValue(report.timeline_json, report.timelineJson),
    dashboardExportDir: stringValue(report.dashboard_export_dir, report.dashboardExportDir),
    todoPath: stringValue(report.todo_path, report.todoPath),
    changelogPath: stringValue(report.changelog_path, report.changelogPath),
    signoffJson: stringValue(report.signoff_json, report.signoffJson),
    readyCount: numberValue(report.ready_count, report.readyCount),
    totalCount: numberValue(report.total_count, report.totalCount),
    warningCount: numberValue(report.warning_count, report.warningCount),
    blockingCount: numberValue(report.blocking_count, report.blockingCount),
    signoffRequired: booleanValue(report.signoff_required, report.signoffRequired),
    signoffCount: numberValue(report.signoff_count, report.signoffCount),
    latestSignoff: normalizeReleaseSignoff(report.latest_signoff ?? report.latestSignoff),
    blockers,
    checklist,
    signoffs,
    commands: arrayValue(report.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseQaCommandCenter(
  value: unknown,
): FlowReleaseQaCommandCenterReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const report =
    root.release_qa && typeof root.release_qa === "object"
      ? (root.release_qa as Record<string, unknown>)
      : root.releaseQa && typeof root.releaseQa === "object"
        ? (root.releaseQa as Record<string, unknown>)
        : root;
  const reportId = stringValue(report.report_id, report.reportId);
  const checks = arrayValue(report.checks)
    .map((item): FlowReleaseQaCheck | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const check = item as Record<string, unknown>;
      const id = stringValue(check.id);
      if (!id) {
        return null;
      }
      return {
        id,
        label: stringValue(check.label) || id,
        command: stringValue(check.command),
        resultPath: stringValue(check.result_path, check.resultPath),
        required: booleanValue(check.required),
        present: booleanValue(check.present),
        stale: booleanValue(check.stale),
        bytes: numberValue(check.bytes),
        status: qaCheckStatus(stringValue(check.status)),
        summary: stringValue(check.summary),
        nextAction: stringValue(check.next_action, check.nextAction),
      };
    })
    .filter((item): item is FlowReleaseQaCheck => item !== null);

  if (!reportId && checks.length === 0) {
    return null;
  }

  return {
    reportId,
    reportJson: stringValue(report.report_json, report.reportJson),
    generatedAtUnixMs: stringValue(report.generated_at_unix_ms, report.generatedAtUnixMs),
    productName: stringValue(report.product_name, report.productName),
    localOnly: booleanValue(report.local_only, report.localOnly),
    status: panelStatus(stringValue(report.status)),
    scoreOutOf100: numberValue(report.score_out_of_100, report.scoreOutOf100),
    readyToShip: booleanValue(report.ready_to_ship, report.readyToShip),
    summary: stringValue(report.summary),
    checklistJson: stringValue(report.checklist_json, report.checklistJson),
    packageJson: stringValue(report.package_json, report.packageJson),
    timelineJson: stringValue(report.timeline_json, report.timelineJson),
    warningCount: numberValue(report.warning_count, report.warningCount),
    blockingCount: numberValue(report.blocking_count, report.blockingCount),
    staleCount: numberValue(report.stale_count, report.staleCount),
    missingCount: numberValue(report.missing_count, report.missingCount),
    checks,
    commands: arrayValue(report.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseEvidenceExportKit(
  value: unknown,
): FlowReleaseEvidenceExportKitReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const report =
    root.release_export_kit && typeof root.release_export_kit === "object"
      ? (root.release_export_kit as Record<string, unknown>)
      : root.releaseExportKit && typeof root.releaseExportKit === "object"
        ? (root.releaseExportKit as Record<string, unknown>)
        : root;
  const manifestRecord =
    report.manifest && typeof report.manifest === "object"
      ? (report.manifest as Record<string, unknown>)
      : null;
  if (!manifestRecord) {
    return null;
  }

  const files = arrayValue(manifestRecord.files)
    .map((item): FlowReleaseEvidenceExportKitFile | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const file = item as Record<string, unknown>;
      const id = stringValue(file.id);
      if (!id) {
        return null;
      }
      return {
        id,
        label: stringValue(file.label) || id,
        kind: stringValue(file.kind),
        path: stringValue(file.path),
        required: booleanValue(file.required),
        present: booleanValue(file.present),
        stale: booleanValue(file.stale),
        bytes: numberValue(file.bytes),
        sha256: stringValue(file.sha256) || null,
        warning: stringValue(file.warning) || null,
      };
    })
    .filter((item): item is FlowReleaseEvidenceExportKitFile => item !== null);

  if (files.length === 0) {
    return null;
  }

  return {
    summary: stringValue(report.summary),
    readyToAttach: booleanValue(report.ready_to_attach, report.readyToAttach),
    status: panelStatus(stringValue(report.status)),
    checklistReady: nullableBoolean(report.checklist_ready, report.checklistReady),
    qaScoreOutOf100: nullableNumber(report.qa_score_out_of_100, report.qaScoreOutOf100),
    qaReadyToShip: nullableBoolean(report.qa_ready_to_ship, report.qaReadyToShip),
    packageReadyToShip: nullableBoolean(
      report.package_ready_to_ship,
      report.packageReadyToShip,
    ),
    timelinePackageCount: nullableNumber(
      report.timeline_package_count,
      report.timelinePackageCount,
    ),
    signoffCount: numberValue(report.signoff_count, report.signoffCount),
    warnings: arrayValue(report.warnings)
      .map((warning) => stringValue(warning))
      .filter(Boolean),
    operatorCopy: stringValue(report.operator_copy, report.operatorCopy),
    manifest: {
      kitId: stringValue(manifestRecord.kit_id, manifestRecord.kitId),
      generatedAtUnixMs: stringValue(
        manifestRecord.generated_at_unix_ms,
        manifestRecord.generatedAtUnixMs,
      ),
      productName: stringValue(manifestRecord.product_name, manifestRecord.productName),
      localOnly: booleanValue(manifestRecord.local_only, manifestRecord.localOnly),
      kitJson: stringValue(manifestRecord.kit_json, manifestRecord.kitJson),
      exportDir: stringValue(manifestRecord.export_dir, manifestRecord.exportDir),
      fileCount: numberValue(manifestRecord.file_count, manifestRecord.fileCount),
      requiredCount: numberValue(manifestRecord.required_count, manifestRecord.requiredCount),
      missingCount: numberValue(manifestRecord.missing_count, manifestRecord.missingCount),
      staleCount: numberValue(manifestRecord.stale_count, manifestRecord.staleCount),
      warningCount: numberValue(manifestRecord.warning_count, manifestRecord.warningCount),
      manifestSha256: stringValue(
        manifestRecord.manifest_sha256,
        manifestRecord.manifestSha256,
      ),
      commands: arrayValue(manifestRecord.commands)
        .map((command) => stringValue(command))
        .filter(Boolean),
      files,
    },
  };
}

export function normalizeReleaseDeploymentGate(
  value: unknown,
): FlowReleaseDeploymentGateReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const report =
    root.release_deployment_gate && typeof root.release_deployment_gate === "object"
      ? (root.release_deployment_gate as Record<string, unknown>)
      : root.releaseDeploymentGate && typeof root.releaseDeploymentGate === "object"
        ? (root.releaseDeploymentGate as Record<string, unknown>)
        : root;
  const gateId = stringValue(report.gate_id, report.gateId);
  const targetRecord =
    report.target && typeof report.target === "object"
      ? (report.target as Record<string, unknown>)
      : null;
  const reasons = arrayValue(report.reasons)
    .map((item): FlowReleaseDeploymentGateReason | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const reason = item as Record<string, unknown>;
      const id = stringValue(reason.id);
      if (!id) {
        return null;
      }
      return {
        id,
        category: deploymentReasonCategory(stringValue(reason.category)),
        severity: stringValue(reason.severity) === "warning" ? "warning" : "blocking",
        title: stringValue(reason.title) || id,
        detail: stringValue(reason.detail),
        sourcePath: stringValue(reason.source_path, reason.sourcePath),
        nextAction: stringValue(reason.next_action, reason.nextAction),
      };
    })
    .filter((item): item is FlowReleaseDeploymentGateReason => item !== null);
  const checklist = arrayValue(report.checklist)
    .map((item): FlowReleaseDeploymentGateChecklistItem | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const check = item as Record<string, unknown>;
      const id = stringValue(check.id);
      if (!id) {
        return null;
      }
      return {
        id,
        title: stringValue(check.title) || id,
        ready: booleanValue(check.ready),
        detail: stringValue(check.detail),
        sourcePath: stringValue(check.source_path, check.sourcePath),
      };
    })
    .filter((item): item is FlowReleaseDeploymentGateChecklistItem => item !== null);

  if (!gateId && !targetRecord && reasons.length === 0 && checklist.length === 0) {
    return null;
  }

  const target = targetRecord ?? {};

  return {
    gateId,
    gateJson: stringValue(report.gate_json, report.gateJson),
    generatedAtUnixMs: stringValue(report.generated_at_unix_ms, report.generatedAtUnixMs),
    productName: stringValue(report.product_name, report.productName),
    localOnly: booleanValue(report.local_only, report.localOnly),
    status: panelStatus(stringValue(report.status)),
    decision: deploymentDecision(stringValue(report.decision)),
    readyToDeploy: booleanValue(report.ready_to_deploy, report.readyToDeploy),
    scoreOutOf100: numberValue(report.score_out_of_100, report.scoreOutOf100),
    summary: stringValue(report.summary),
    target: {
      id: stringValue(target.id),
      label: stringValue(target.label),
      environment: stringValue(target.environment),
      provider: stringValue(target.provider),
      url: stringValue(target.url) || null,
      localOnlyRequired: booleanValue(target.local_only_required, target.localOnlyRequired),
      requiresVercel: booleanValue(target.requires_vercel, target.requiresVercel),
      expectedProductName: stringValue(
        target.expected_product_name,
        target.expectedProductName,
      ),
      rollbackNote: stringValue(target.rollback_note, target.rollbackNote),
    },
    exportKitJson: stringValue(report.export_kit_json, report.exportKitJson),
    qaJson: stringValue(report.qa_json, report.qaJson),
    checklistJson: stringValue(report.checklist_json, report.checklistJson),
    packageJson: stringValue(report.package_json, report.packageJson),
    timelineJson: stringValue(report.timeline_json, report.timelineJson),
    dashboardExportDir: stringValue(report.dashboard_export_dir, report.dashboardExportDir),
    noDeployReasonCount: numberValue(
      report.no_deploy_reason_count,
      report.noDeployReasonCount,
    ),
    warningCount: numberValue(report.warning_count, report.warningCount),
    readyCount: numberValue(report.ready_count, report.readyCount),
    totalCount: numberValue(report.total_count, report.totalCount),
    reasons,
    checklist,
    deployChecklist: arrayValue(report.deploy_checklist, report.deployChecklist)
      .map((item) => stringValue(item))
      .filter(Boolean),
    rollbackNote: stringValue(report.rollback_note, report.rollbackNote),
    operatorCopy: stringValue(report.operator_copy, report.operatorCopy),
    commands: arrayValue(report.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseCandidateArchive(
  value: unknown,
): FlowReleaseCandidateArchive | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const archive =
    root.release_candidate_archive && typeof root.release_candidate_archive === "object"
      ? (root.release_candidate_archive as Record<string, unknown>)
      : root.releaseCandidateArchive && typeof root.releaseCandidateArchive === "object"
        ? (root.releaseCandidateArchive as Record<string, unknown>)
        : root;
  const archiveId = stringValue(archive.archive_id, archive.archiveId);
  const entries = arrayValue(archive.entries)
    .map((item): FlowReleaseCandidateArchiveEntry | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const entry = item as Record<string, unknown>;
      const candidateId = stringValue(entry.candidate_id, entry.candidateId);
      if (!candidateId) {
        return null;
      }
      const target =
        entry.target && typeof entry.target === "object"
          ? (entry.target as Record<string, unknown>)
          : {};
      return {
        candidateId,
        gateId: stringValue(entry.gate_id, entry.gateId),
        gateJson: stringValue(entry.gate_json, entry.gateJson),
        exportKitJson: stringValue(entry.export_kit_json, entry.exportKitJson),
        generatedAtUnixMs: stringValue(entry.generated_at_unix_ms, entry.generatedAtUnixMs),
        productName: stringValue(entry.product_name, entry.productName),
        decision: deploymentDecision(stringValue(entry.decision)),
        scoreOutOf100: numberValue(entry.score_out_of_100, entry.scoreOutOf100),
        readyToDeploy: booleanValue(entry.ready_to_deploy, entry.readyToDeploy),
        target: releaseDeploymentTarget(target),
        noDeployReasonCount: numberValue(
          entry.no_deploy_reason_count,
          entry.noDeployReasonCount,
        ),
        warningCount: numberValue(entry.warning_count, entry.warningCount),
        reasonIds: arrayValue(entry.reason_ids, entry.reasonIds)
          .map((id) => stringValue(id))
          .filter(Boolean),
        exportKitManifestSha256:
          stringValue(entry.export_kit_manifest_sha256, entry.exportKitManifestSha256) || null,
        rollbackNote: stringValue(entry.rollback_note, entry.rollbackNote),
        summary: stringValue(entry.summary),
      };
    })
    .filter((item): item is FlowReleaseCandidateArchiveEntry => item !== null);
  const diffs = arrayValue(archive.diffs)
    .map((item): FlowReleaseCandidateArchiveDiff | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const diff = item as Record<string, unknown>;
      const fromCandidateId = stringValue(diff.from_candidate_id, diff.fromCandidateId);
      const toCandidateId = stringValue(diff.to_candidate_id, diff.toCandidateId);
      if (!fromCandidateId || !toCandidateId) {
        return null;
      }
      return {
        fromCandidateId,
        toCandidateId,
        scoreDelta: numberValue(diff.score_delta, diff.scoreDelta),
        decisionChanged: booleanValue(diff.decision_changed, diff.decisionChanged),
        targetChanged: booleanValue(diff.target_changed, diff.targetChanged),
        evidenceChecksumChanged: booleanValue(
          diff.evidence_checksum_changed,
          diff.evidenceChecksumChanged,
        ),
        newBlockerIds: arrayValue(diff.new_blocker_ids, diff.newBlockerIds)
          .map((id) => stringValue(id))
          .filter(Boolean),
        resolvedBlockerIds: arrayValue(diff.resolved_blocker_ids, diff.resolvedBlockerIds)
          .map((id) => stringValue(id))
          .filter(Boolean),
        regression: booleanValue(diff.regression),
        summary: stringValue(diff.summary),
      };
    })
    .filter((item): item is FlowReleaseCandidateArchiveDiff => item !== null);

  if (!archiveId && entries.length === 0 && diffs.length === 0) {
    return null;
  }

  return {
    archiveId,
    archiveJson: stringValue(archive.archive_json, archive.archiveJson),
    generatedAtUnixMs: stringValue(archive.generated_at_unix_ms, archive.generatedAtUnixMs),
    localOnly: booleanValue(archive.local_only, archive.localOnly),
    candidateCount: numberValue(archive.candidate_count, archive.candidateCount),
    latestCandidateId:
      stringValue(archive.latest_candidate_id, archive.latestCandidateId) || null,
    latestDecision:
      archive.latest_decision == null && archive.latestDecision == null
        ? null
        : deploymentDecision(stringValue(archive.latest_decision, archive.latestDecision)),
    latestScoreOutOf100: nullableNumber(
      archive.latest_score_out_of_100,
      archive.latestScoreOutOf100,
    ),
    goCount: numberValue(archive.go_count, archive.goCount),
    noGoCount: numberValue(archive.no_go_count, archive.noGoCount),
    draftCount: numberValue(archive.draft_count, archive.draftCount),
    regressionCount: numberValue(archive.regression_count, archive.regressionCount),
    entries,
    diffs,
    commands: arrayValue(archive.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleasePromotionLedger(
  value: unknown,
): FlowReleasePromotionLedger | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const ledger =
    root.release_promotion_ledger && typeof root.release_promotion_ledger === "object"
      ? (root.release_promotion_ledger as Record<string, unknown>)
      : root.releasePromotionLedger && typeof root.releasePromotionLedger === "object"
        ? (root.releasePromotionLedger as Record<string, unknown>)
        : root;
  const ledgerId = stringValue(ledger.ledger_id, ledger.ledgerId);
  const records = arrayValue(ledger.records)
    .map((item): FlowReleasePromotionRecord | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const record = item as Record<string, unknown>;
      const promotionId = stringValue(record.promotion_id, record.promotionId);
      const candidateId = stringValue(record.candidate_id, record.candidateId);
      if (!promotionId || !candidateId) {
        return null;
      }
      const target =
        record.target && typeof record.target === "object"
          ? (record.target as Record<string, unknown>)
          : {};
      const postPromotionChecks = arrayValue(
        record.post_promotion_checks,
        record.postPromotionChecks,
      )
        .map((check): FlowReleasePromotionPostCheck | null => {
          if (!check || typeof check !== "object") {
            return null;
          }
          const postCheck = check as Record<string, unknown>;
          const id = stringValue(postCheck.id);
          if (!id) {
            return null;
          }
          return {
            id,
            label: stringValue(postCheck.label),
            resultPath: stringValue(postCheck.result_path, postCheck.resultPath),
            required: booleanValue(postCheck.required),
            present: booleanValue(postCheck.present),
            bytes: numberValue(postCheck.bytes),
            summary: stringValue(postCheck.summary),
            nextAction: stringValue(postCheck.next_action, postCheck.nextAction),
          };
        })
        .filter((check): check is FlowReleasePromotionPostCheck => check !== null);

      return {
        promotionId,
        candidateId,
        archiveJson: stringValue(record.archive_json, record.archiveJson),
        gateJson: stringValue(record.gate_json, record.gateJson),
        exportKitJson: stringValue(record.export_kit_json, record.exportKitJson),
        recordedAtUnixMs: stringValue(record.recorded_at_unix_ms, record.recordedAtUnixMs),
        productName: stringValue(record.product_name, record.productName),
        localOnly: booleanValue(record.local_only, record.localOnly),
        decision: promotionDecision(stringValue(record.decision)),
        operator: stringValue(record.operator),
        reason: stringValue(record.reason),
        deploymentNote: stringValue(record.deployment_note, record.deploymentNote),
        target: releaseDeploymentTarget(target),
        rollbackReference: stringValue(record.rollback_reference, record.rollbackReference),
        candidateScoreOutOf100: numberValue(
          record.candidate_score_out_of_100,
          record.candidateScoreOutOf100,
        ),
        candidateReadyToDeploy: booleanValue(
          record.candidate_ready_to_deploy,
          record.candidateReadyToDeploy,
        ),
        candidateBlockerCount: numberValue(
          record.candidate_blocker_count,
          record.candidateBlockerCount,
        ),
        postPromotionRequiredCount: numberValue(
          record.post_promotion_required_count,
          record.postPromotionRequiredCount,
        ),
        postPromotionMissingCount: numberValue(
          record.post_promotion_missing_count,
          record.postPromotionMissingCount,
        ),
        postPromotionChecks,
        summary: stringValue(record.summary),
      };
    })
    .filter((record): record is FlowReleasePromotionRecord => record !== null);

  if (!ledgerId && records.length === 0) {
    return null;
  }

  return {
    ledgerId,
    ledgerJson: stringValue(ledger.ledger_json, ledger.ledgerJson),
    generatedAtUnixMs: stringValue(ledger.generated_at_unix_ms, ledger.generatedAtUnixMs),
    localOnly: booleanValue(ledger.local_only, ledger.localOnly),
    recordCount: numberValue(ledger.record_count, ledger.recordCount),
    promotedCount: numberValue(ledger.promoted_count, ledger.promotedCount),
    heldCount: numberValue(ledger.held_count, ledger.heldCount),
    rolledBackCount: numberValue(ledger.rolled_back_count, ledger.rolledBackCount),
    supersededCount: numberValue(ledger.superseded_count, ledger.supersededCount),
    abandonedCount: numberValue(ledger.abandoned_count, ledger.abandonedCount),
    postPromotionMissingCount: numberValue(
      ledger.post_promotion_missing_count,
      ledger.postPromotionMissingCount,
    ),
    activePromotionId:
      stringValue(ledger.active_promotion_id, ledger.activePromotionId) || null,
    activeCandidateId:
      stringValue(ledger.active_candidate_id, ledger.activeCandidateId) || null,
    activeRollbackReference:
      stringValue(ledger.active_rollback_reference, ledger.activeRollbackReference) || null,
    latestDecision:
      ledger.latest_decision == null && ledger.latestDecision == null
        ? null
        : promotionDecision(stringValue(ledger.latest_decision, ledger.latestDecision)),
    latestDeploymentNote:
      stringValue(ledger.latest_deployment_note, ledger.latestDeploymentNote) || null,
    warnings: arrayValue(ledger.warnings)
      .map((warning) => stringValue(warning))
      .filter(Boolean),
    records,
    commands: arrayValue(ledger.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleasePostPromotionMonitor(
  value: unknown,
): FlowReleasePostPromotionMonitorReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const monitor =
    root.release_post_promotion_monitor &&
    typeof root.release_post_promotion_monitor === "object"
      ? (root.release_post_promotion_monitor as Record<string, unknown>)
      : root.releasePostPromotionMonitor &&
          typeof root.releasePostPromotionMonitor === "object"
        ? (root.releasePostPromotionMonitor as Record<string, unknown>)
        : root;
  const monitorId = stringValue(monitor.monitor_id, monitor.monitorId);
  const checks = arrayValue(monitor.checks)
    .map((item): FlowReleasePostPromotionCheck | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const check = item as Record<string, unknown>;
      const id = stringValue(check.id);
      if (!id) {
        return null;
      }
      return {
        id,
        label: stringValue(check.label),
        sourcePath: stringValue(check.source_path, check.sourcePath),
        required: booleanValue(check.required),
        present: booleanValue(check.present),
        stale: booleanValue(check.stale),
        bytes: numberValue(check.bytes),
        status: postPromotionCheckStatus(stringValue(check.status)),
        summary: stringValue(check.summary),
        nextAction: stringValue(check.next_action, check.nextAction),
      };
    })
    .filter((check): check is FlowReleasePostPromotionCheck => check !== null);
  const incidentNotes = arrayValue(monitor.incident_notes, monitor.incidentNotes)
    .map((item): FlowReleasePostPromotionIncidentNote | null => {
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
        path: stringValue(note.path),
        present: booleanValue(note.present),
        bytes: numberValue(note.bytes),
        summary: stringValue(note.summary),
      };
    })
    .filter((note): note is FlowReleasePostPromotionIncidentNote => note !== null);

  if (!monitorId && checks.length === 0 && incidentNotes.length === 0) {
    return null;
  }

  return {
    monitorId,
    monitorJson: stringValue(monitor.monitor_json, monitor.monitorJson),
    generatedAtUnixMs: stringValue(monitor.generated_at_unix_ms, monitor.generatedAtUnixMs),
    productName: stringValue(monitor.product_name, monitor.productName),
    localOnly: booleanValue(monitor.local_only, monitor.localOnly),
    status: panelStatus(stringValue(monitor.status)),
    scoreOutOf100: numberValue(monitor.score_out_of_100, monitor.scoreOutOf100),
    readyForStable: booleanValue(monitor.ready_for_stable, monitor.readyForStable),
    promotionLedgerJson: stringValue(
      monitor.promotion_ledger_json,
      monitor.promotionLedgerJson,
    ),
    qaJson: stringValue(monitor.qa_json, monitor.qaJson),
    dashboardSmokeResultPath: stringValue(
      monitor.dashboard_smoke_result_path,
      monitor.dashboardSmokeResultPath,
    ),
    activeCandidateId:
      stringValue(monitor.active_candidate_id, monitor.activeCandidateId) || null,
    activePromotionId:
      stringValue(monitor.active_promotion_id, monitor.activePromotionId) || null,
    activeRollbackReference:
      stringValue(monitor.active_rollback_reference, monitor.activeRollbackReference) || null,
    latestDecision:
      monitor.latest_decision == null && monitor.latestDecision == null
        ? null
        : promotionDecision(stringValue(monitor.latest_decision, monitor.latestDecision)),
    promotedCount: numberValue(monitor.promoted_count, monitor.promotedCount),
    incidentNoteCount: numberValue(monitor.incident_note_count, monitor.incidentNoteCount),
    missingEvidenceCount: numberValue(
      monitor.missing_evidence_count,
      monitor.missingEvidenceCount,
    ),
    staleCount: numberValue(monitor.stale_count, monitor.staleCount),
    warningCount: numberValue(monitor.warning_count, monitor.warningCount),
    blockingCount: numberValue(monitor.blocking_count, monitor.blockingCount),
    checks,
    incidentNotes,
    warnings: arrayValue(monitor.warnings)
      .map((warning) => stringValue(warning))
      .filter(Boolean),
    summary: stringValue(monitor.summary),
    commands: arrayValue(monitor.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseRollbackDrill(
  value: unknown,
): FlowReleaseRollbackDrillReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const drill =
    root.release_rollback_drill && typeof root.release_rollback_drill === "object"
      ? (root.release_rollback_drill as Record<string, unknown>)
      : root.releaseRollbackDrill && typeof root.releaseRollbackDrill === "object"
        ? (root.releaseRollbackDrill as Record<string, unknown>)
        : root;
  const drillId = stringValue(drill.drill_id, drill.drillId);
  const checks = arrayValue(drill.checks)
    .map((item): FlowReleaseRollbackDrillCheck | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const check = item as Record<string, unknown>;
      const id = stringValue(check.id);
      if (!id) {
        return null;
      }
      return {
        id,
        label: stringValue(check.label),
        sourcePath: stringValue(check.source_path, check.sourcePath),
        required: booleanValue(check.required),
        present: booleanValue(check.present),
        stale: booleanValue(check.stale),
        bytes: numberValue(check.bytes),
        status: rollbackDrillCheckStatus(stringValue(check.status)),
        summary: stringValue(check.summary),
        nextAction: stringValue(check.next_action, check.nextAction),
      };
    })
    .filter((check): check is FlowReleaseRollbackDrillCheck => check !== null);

  if (!drillId && checks.length === 0) {
    return null;
  }

  return {
    drillId,
    drillJson: stringValue(drill.drill_json, drill.drillJson),
    generatedAtUnixMs: stringValue(drill.generated_at_unix_ms, drill.generatedAtUnixMs),
    productName: stringValue(drill.product_name, drill.productName),
    localOnly: booleanValue(drill.local_only, drill.localOnly),
    status: panelStatus(stringValue(drill.status)),
    scoreOutOf100: numberValue(drill.score_out_of_100, drill.scoreOutOf100),
    readyToRollback: booleanValue(drill.ready_to_rollback, drill.readyToRollback),
    readyForStable: booleanValue(drill.ready_for_stable, drill.readyForStable),
    activeCandidateId: stringValue(drill.active_candidate_id, drill.activeCandidateId) || null,
    activePromotionId: stringValue(drill.active_promotion_id, drill.activePromotionId) || null,
    activeRollbackReference:
      stringValue(drill.active_rollback_reference, drill.activeRollbackReference) || null,
    latestPromotionDecision:
      drill.latest_promotion_decision == null && drill.latestPromotionDecision == null
        ? null
        : promotionDecision(
            stringValue(drill.latest_promotion_decision, drill.latestPromotionDecision),
          ),
    deploymentGateDecision:
      drill.deployment_gate_decision == null && drill.deploymentGateDecision == null
        ? null
        : deploymentDecision(
            stringValue(drill.deployment_gate_decision, drill.deploymentGateDecision),
          ),
    postPromotionMonitorJson: stringValue(
      drill.post_promotion_monitor_json,
      drill.postPromotionMonitorJson,
    ),
    promotionLedgerJson: stringValue(drill.promotion_ledger_json, drill.promotionLedgerJson),
    candidateArchiveJson: stringValue(drill.candidate_archive_json, drill.candidateArchiveJson),
    deploymentGateJson: stringValue(drill.deployment_gate_json, drill.deploymentGateJson),
    rollbackCommand: stringValue(drill.rollback_command, drill.rollbackCommand),
    dryRunCommand: stringValue(drill.dry_run_command, drill.dryRunCommand),
    operator: stringValue(drill.operator),
    reason: stringValue(drill.reason),
    blockingCount: numberValue(drill.blocking_count, drill.blockingCount),
    warningCount: numberValue(drill.warning_count, drill.warningCount),
    staleCount: numberValue(drill.stale_count, drill.staleCount),
    missingEvidenceCount: numberValue(
      drill.missing_evidence_count,
      drill.missingEvidenceCount,
    ),
    checks,
    blockedReasons: arrayValue(drill.blocked_reasons, drill.blockedReasons)
      .map((reason) => stringValue(reason))
      .filter(Boolean),
    summary: stringValue(drill.summary),
    commands: arrayValue(drill.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseStabilityBoard(
  value: unknown,
): FlowReleaseStabilityBoardReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const board =
    root.release_stability_board && typeof root.release_stability_board === "object"
      ? (root.release_stability_board as Record<string, unknown>)
      : root.releaseStabilityBoard && typeof root.releaseStabilityBoard === "object"
        ? (root.releaseStabilityBoard as Record<string, unknown>)
        : root;
  const boardId = stringValue(board.board_id, board.boardId);
  const checks = arrayValue(board.checks)
    .map((item): FlowReleaseStabilityBoardCheck | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const check = item as Record<string, unknown>;
      const id = stringValue(check.id);
      if (!id) {
        return null;
      }
      return {
        id,
        label: stringValue(check.label),
        category: stabilityBoardCategory(stringValue(check.category)),
        sourcePath: stringValue(check.source_path, check.sourcePath),
        required: booleanValue(check.required),
        present: booleanValue(check.present),
        stale: booleanValue(check.stale),
        bytes: numberValue(check.bytes),
        status: stabilityBoardCheckStatus(stringValue(check.status)),
        summary: stringValue(check.summary),
        nextAction: stringValue(check.next_action, check.nextAction),
      };
    })
    .filter((check): check is FlowReleaseStabilityBoardCheck => check !== null);
  const evidenceLinks = arrayValue(board.evidence_links, board.evidenceLinks)
    .map((item): FlowReleaseStabilityBoardEvidenceLink | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const link = item as Record<string, unknown>;
      const id = stringValue(link.id);
      if (!id) {
        return null;
      }
      return {
        id,
        label: stringValue(link.label),
        path: stringValue(link.path),
        present: booleanValue(link.present),
      };
    })
    .filter((link): link is FlowReleaseStabilityBoardEvidenceLink => link !== null);

  if (!boardId && checks.length === 0 && evidenceLinks.length === 0) {
    return null;
  }

  return {
    boardId,
    boardJson: stringValue(board.board_json, board.boardJson),
    generatedAtUnixMs: stringValue(board.generated_at_unix_ms, board.generatedAtUnixMs),
    productName: stringValue(board.product_name, board.productName),
    localOnly: booleanValue(board.local_only, board.localOnly),
    status: panelStatus(stringValue(board.status)),
    scoreOutOf100: numberValue(board.score_out_of_100, board.scoreOutOf100),
    readyForCheckpoint: booleanValue(
      board.ready_for_checkpoint,
      board.readyForCheckpoint,
    ),
    readyToDeploy: booleanValue(board.ready_to_deploy, board.readyToDeploy),
    stableAfterPromotion: booleanValue(
      board.stable_after_promotion,
      board.stableAfterPromotion,
    ),
    recoverable: booleanValue(board.recoverable),
    activeCandidateId: stringValue(board.active_candidate_id, board.activeCandidateId) || null,
    activePromotionId: stringValue(board.active_promotion_id, board.activePromotionId) || null,
    activeRollbackReference:
      stringValue(board.active_rollback_reference, board.activeRollbackReference) || null,
    latestPromotionDecision:
      board.latest_promotion_decision == null && board.latestPromotionDecision == null
        ? null
        : promotionDecision(
            stringValue(board.latest_promotion_decision, board.latestPromotionDecision),
          ),
    deploymentGateDecision:
      board.deployment_gate_decision == null && board.deploymentGateDecision == null
        ? null
        : deploymentDecision(
            stringValue(board.deployment_gate_decision, board.deploymentGateDecision),
          ),
    qaJson: stringValue(board.qa_json, board.qaJson),
    candidateArchiveJson: stringValue(board.candidate_archive_json, board.candidateArchiveJson),
    promotionLedgerJson: stringValue(board.promotion_ledger_json, board.promotionLedgerJson),
    postPromotionMonitorJson: stringValue(
      board.post_promotion_monitor_json,
      board.postPromotionMonitorJson,
    ),
    rollbackDrillJson: stringValue(board.rollback_drill_json, board.rollbackDrillJson),
    deploymentGateJson: stringValue(board.deployment_gate_json, board.deploymentGateJson),
    blockingCount: numberValue(board.blocking_count, board.blockingCount),
    warningCount: numberValue(board.warning_count, board.warningCount),
    staleCount: numberValue(board.stale_count, board.staleCount),
    missingEvidenceCount: numberValue(
      board.missing_evidence_count,
      board.missingEvidenceCount,
    ),
    checks,
    evidenceLinks,
    activeRisks: arrayValue(board.active_risks, board.activeRisks)
      .map((risk) => stringValue(risk))
      .filter(Boolean),
    nextActions: arrayValue(board.next_actions, board.nextActions)
      .map((action) => stringValue(action))
      .filter(Boolean),
    summary: stringValue(board.summary),
    commands: arrayValue(board.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

function normalizeReleaseSignoff(value: unknown): FlowReleaseChecklistSignoff | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const record = value as Record<string, unknown>;
  const id = stringValue(record.id);
  if (!id) {
    return null;
  }
  return {
    id,
    checklistId: stringValue(record.checklist_id, record.checklistId),
    operator: stringValue(record.operator),
    decision: signoffDecision(stringValue(record.decision)),
    reason: stringValue(record.reason),
    recordedAtUnixMs: stringValue(record.recorded_at_unix_ms, record.recordedAtUnixMs),
    localOnly: booleanValue(record.local_only, record.localOnly),
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

function nullableNumber(...values: unknown[]): number | null {
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
  return null;
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

function nullableBoolean(...values: unknown[]): boolean | null {
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
  return null;
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

function panelStatus(value: string): FlowDashboardPanelStatus {
  if (value === "ready" || value === "warning" || value === "blocked") {
    return value;
  }
  return "warning";
}

function checklistSeverity(value: string): FlowReleaseChecklistBlocker["severity"] {
  return value === "blocking" ? "blocking" : "warning";
}

function signoffDecision(value: string): FlowReleaseChecklistSignoff["decision"] {
  if (value === "approved" || value === "needs-changes" || value === "blocked") {
    return value;
  }
  return "needs-changes";
}

function qaCheckStatus(value: string): FlowReleaseQaCheckStatus {
  if (
    value === "passed" ||
    value === "warning" ||
    value === "failed" ||
    value === "missing" ||
    value === "stale"
  ) {
    return value;
  }
  return "warning";
}

function postPromotionCheckStatus(value: string): FlowReleasePostPromotionCheckStatus {
  if (
    value === "passed" ||
    value === "warning" ||
    value === "failed" ||
    value === "missing" ||
    value === "stale"
  ) {
    return value;
  }
  return "warning";
}

function rollbackDrillCheckStatus(value: string): FlowReleaseRollbackDrillCheckStatus {
  if (
    value === "passed" ||
    value === "warning" ||
    value === "failed" ||
    value === "missing" ||
    value === "stale"
  ) {
    return value;
  }
  return "warning";
}

function stabilityBoardCheckStatus(value: string): FlowReleaseStabilityBoardCheckStatus {
  if (
    value === "passed" ||
    value === "warning" ||
    value === "failed" ||
    value === "missing" ||
    value === "stale"
  ) {
    return value;
  }
  return "warning";
}

function stabilityBoardCategory(value: string): FlowReleaseStabilityBoardCategory {
  if (
    value === "deployment-readiness" ||
    value === "qa-health" ||
    value === "candidate-regression" ||
    value === "promotion-state" ||
    value === "post-promotion-freshness" ||
    value === "rollback-recovery"
  ) {
    return value;
  }
  return "deployment-readiness";
}

function deploymentDecision(value: string): FlowReleaseDeploymentGateDecision {
  if (value === "go" || value === "no-go" || value === "draft") {
    return value;
  }
  return "draft";
}

function promotionDecision(value: string): FlowReleasePromotionDecision {
  if (
    value === "promoted" ||
    value === "held" ||
    value === "rolled-back" ||
    value === "superseded" ||
    value === "abandoned"
  ) {
    return value;
  }
  return "held";
}

function releaseDeploymentTarget(target: Record<string, unknown>): FlowReleaseDeploymentTarget {
  return {
    id: stringValue(target.id),
    label: stringValue(target.label),
    environment: stringValue(target.environment),
    provider: stringValue(target.provider),
    url: stringValue(target.url) || null,
    localOnlyRequired: booleanValue(target.local_only_required, target.localOnlyRequired),
    requiresVercel: booleanValue(target.requires_vercel, target.requiresVercel),
    expectedProductName: stringValue(target.expected_product_name, target.expectedProductName),
    rollbackNote: stringValue(target.rollback_note, target.rollbackNote),
  };
}

function deploymentReasonCategory(value: string): FlowReleaseDeploymentGateReasonCategory {
  if (
    value === "missing-evidence" ||
    value === "stale-checks" ||
    value === "blocked-qa" ||
    value === "unsigned-release" ||
    value === "dashboard-state" ||
    value === "target-mismatch"
  ) {
    return value;
  }
  return "missing-evidence";
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
