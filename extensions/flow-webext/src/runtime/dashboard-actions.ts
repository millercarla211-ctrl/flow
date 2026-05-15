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

export type FlowReleaseRecoveryRunbookPhaseKind =
  | "pause"
  | "diagnose"
  | "rollback"
  | "verify"
  | "resume"
  | "follow-up";

export type FlowReleaseRecoveryRunbookPhaseStatus =
  | "ready"
  | "requires-approval"
  | "blocked";

export interface FlowReleaseRecoveryRunbookPhase {
  kind: FlowReleaseRecoveryRunbookPhaseKind;
  order: number;
  label: string;
  status: FlowReleaseRecoveryRunbookPhaseStatus;
  approvalRequired: boolean;
  sourcePath: string;
  objective: string;
  command: string;
  verification: string;
  risks: string[];
  evidencePaths: string[];
  nextAction: string;
}

export interface FlowReleaseRecoveryRunbookApprovalGate {
  id: string;
  label: string;
  phase: FlowReleaseRecoveryRunbookPhaseKind;
  required: boolean;
  satisfied: boolean;
  summary: string;
  operatorAction: string;
}

export interface FlowReleaseRecoveryRunbookReport {
  runbookId: string;
  runbookJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  scoreOutOf100: number;
  readyForOperatorReview: boolean;
  readyToExecuteRecovery: boolean;
  activeCandidateId: string | null;
  activePromotionId: string | null;
  activeRollbackReference: string | null;
  latestPromotionDecision: FlowReleasePromotionDecision | null;
  stabilityBoardJson: string;
  rollbackDrillJson: string;
  promotionLedgerJson: string;
  postPromotionMonitorJson: string;
  phaseCount: number;
  blockedPhaseCount: number;
  approvalGateCount: number;
  unsatisfiedApprovalGateCount: number;
  commandCount: number;
  activeRisks: string[];
  phases: FlowReleaseRecoveryRunbookPhase[];
  approvalGates: FlowReleaseRecoveryRunbookApprovalGate[];
  recoveryCommands: string[];
  summary: string;
  commands: string[];
}

export type FlowReleaseIncidentSeverity = "info" | "watch" | "blocking" | "critical";

export type FlowReleaseIncidentOutcome =
  | "open"
  | "monitoring"
  | "resolved"
  | "rolled-back"
  | "prevented";

export interface FlowReleaseIncidentNote {
  id: string;
  path: string;
  present: boolean;
  bytes: number;
  summary: string;
}

export interface FlowReleaseIncidentArchiveEntry {
  incidentId: string;
  recordedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  severity: FlowReleaseIncidentSeverity;
  outcome: FlowReleaseIncidentOutcome;
  title: string;
  summary: string;
  recoveryRunbookId: string | null;
  recoveryRunbookJson: string;
  stabilityBoardJson: string;
  rollbackDrillJson: string;
  postPromotionMonitorJson: string;
  activeCandidateId: string | null;
  activePromotionId: string | null;
  activeRollbackReference: string | null;
  blockedPhaseCount: number;
  activeRiskCount: number;
  incidentNotes: FlowReleaseIncidentNote[];
  followUpActions: string[];
  preventionItems: string[];
  evidencePaths: string[];
}

export interface FlowReleaseIncidentArchive {
  archiveId: string;
  archiveJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  incidentCount: number;
  openCount: number;
  monitoringCount: number;
  resolvedCount: number;
  rolledBackCount: number;
  preventedCount: number;
  criticalCount: number;
  blockingCount: number;
  followUpCount: number;
  latestIncidentId: string | null;
  latestSeverity: FlowReleaseIncidentSeverity | null;
  latestOutcome: FlowReleaseIncidentOutcome | null;
  latestRollbackReference: string | null;
  entries: FlowReleaseIncidentArchiveEntry[];
  commands: string[];
}

export type FlowReleasePreventionFindingKind =
  | "critical-incident"
  | "repeated-failure-class"
  | "stale-evidence"
  | "missing-evidence"
  | "missing-incident-note"
  | "rollback-gap"
  | "stability-gate";

export type FlowReleasePreventionActionKind =
  | "refresh-evidence"
  | "harden-rollback"
  | "attach-incident-note"
  | "resolve-recurrence"
  | "review-release-gate";

export type FlowReleasePreventionActionStatus =
  | "owner-ready"
  | "needs-evidence"
  | "blocked";

export interface FlowReleasePreventionFinding {
  id: string;
  kind: FlowReleasePreventionFindingKind;
  severity: FlowReleaseIncidentSeverity;
  title: string;
  recurrenceCount: number;
  sourcePaths: string[];
  summary: string;
  nextAction: string;
  releaseGateBlocking: boolean;
}

export interface FlowReleasePreventionAction {
  id: string;
  kind: FlowReleasePreventionActionKind;
  status: FlowReleasePreventionActionStatus;
  owner: string;
  title: string;
  summary: string;
  sourcePath: string;
  evidencePath: string;
  command: string;
  required: boolean;
  releaseGateBlocking: boolean;
  nextAction: string;
}

export interface FlowReleasePreventionEvidenceLink {
  id: string;
  label: string;
  path: string;
  present: boolean;
}

export interface FlowReleasePreventionPlanReport {
  planId: string;
  planJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  scoreOutOf100: number;
  readyForNextCheckpoint: boolean;
  incidentArchiveJson: string;
  stabilityBoardJson: string;
  incidentCount: number;
  findingCount: number;
  recurringIssueCount: number;
  actionCount: number;
  ownerReadyCount: number;
  blockerCount: number;
  evidenceMissingCount: number;
  gateBlockingCount: number;
  latestIncidentId: string | null;
  activeRollbackReference: string | null;
  findings: FlowReleasePreventionFinding[];
  actions: FlowReleasePreventionAction[];
  evidenceLinks: FlowReleasePreventionEvidenceLink[];
  ownerReadyCopy: string;
  summary: string;
  commands: string[];
}

export type FlowReleaseOwnerFollowUpCompletionState =
  | "ready"
  | "needs-evidence"
  | "blocked"
  | "complete"
  | "overdue";

export type FlowReleaseOwnerFollowUpEvidenceState = "present" | "missing" | "not-required";

export interface FlowReleaseOwnerFollowUpRecord {
  id: string;
  actionId: string;
  owner: string;
  title: string;
  summary: string;
  completionState: FlowReleaseOwnerFollowUpCompletionState;
  evidenceState: FlowReleaseOwnerFollowUpEvidenceState;
  sourcePath: string;
  evidencePath: string;
  evidenceRequest: string;
  dueAfterUnixMs: string;
  dueBeforeUnixMs: string;
  overdue: boolean;
  required: boolean;
  releaseGateBlocking: boolean;
  command: string;
  assignmentCopy: string;
  nextAction: string;
}

export interface FlowReleaseOwnerFollowUpGroup {
  owner: string;
  recordCount: number;
  readyCount: number;
  waitingCount: number;
  blockedCount: number;
  overdueCount: number;
  completeCount: number;
  evidenceMissingCount: number;
  records: string[];
}

export interface FlowReleaseOwnerFollowUpBoardReport {
  boardId: string;
  boardJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  scoreOutOf100: number;
  readyForNextCheckpoint: boolean;
  preventionPlanJson: string;
  incidentArchiveJson: string;
  stabilityBoardJson: string;
  recordCount: number;
  ownerCount: number;
  readyCount: number;
  waitingCount: number;
  blockedCount: number;
  overdueCount: number;
  completeCount: number;
  evidenceMissingCount: number;
  gateBlockingCount: number;
  ownerGroups: FlowReleaseOwnerFollowUpGroup[];
  records: FlowReleaseOwnerFollowUpRecord[];
  assignmentCopy: string;
  summary: string;
  commands: string[];
}

export type FlowReleaseEvidenceSlaState =
  | "fresh"
  | "due-soon"
  | "overdue"
  | "missing"
  | "blocked"
  | "acknowledged";

export type FlowReleaseEvidenceEscalationLevel =
  | "none"
  | "owner"
  | "release-gate"
  | "checkpoint";

export type FlowReleaseEvidenceRequirementSource =
  | "owner-follow-up"
  | "prevention-plan"
  | "stability-board";

export interface FlowReleaseEvidenceSlaRequirement {
  id: string;
  source: FlowReleaseEvidenceRequirementSource;
  owner: string;
  title: string;
  state: FlowReleaseEvidenceSlaState;
  escalationLevel: FlowReleaseEvidenceEscalationLevel;
  evidencePath: string;
  evidencePresent: boolean;
  dueAfterUnixMs: string;
  dueBeforeUnixMs: string;
  slaWindowMs: string;
  ageMs: string;
  acknowledgementRequired: boolean;
  releaseGateBlocking: boolean;
  escalationCopy: string;
  nextAction: string;
}

export interface FlowReleaseEvidenceSlaOwnerGroup {
  owner: string;
  requirementCount: number;
  freshCount: number;
  dueSoonCount: number;
  overdueCount: number;
  missingCount: number;
  blockedCount: number;
  acknowledgedCount: number;
  escalationCount: number;
  releaseGateBlockingCount: number;
  requirements: string[];
}

export interface FlowReleaseEvidenceSlaMonitorReport {
  monitorId: string;
  monitorJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  scoreOutOf100: number;
  readyForNextCheckpoint: boolean;
  ownerFollowupBoardJson: string;
  preventionPlanJson: string;
  stabilityBoardJson: string;
  requirementCount: number;
  ownerCount: number;
  freshCount: number;
  dueSoonCount: number;
  overdueCount: number;
  missingCount: number;
  blockedCount: number;
  acknowledgedCount: number;
  escalationCount: number;
  gateBlockingCount: number;
  ownerGroups: FlowReleaseEvidenceSlaOwnerGroup[];
  requirements: FlowReleaseEvidenceSlaRequirement[];
  escalationCopy: string;
  summary: string;
  commands: string[];
}

export type FlowReleaseEscalationOwnerResponse =
  | "pending"
  | "acknowledged"
  | "resolved"
  | "rejected"
  | "carried-over";

export type FlowReleaseEscalationGateOutcome =
  | "blocked"
  | "carry-over"
  | "cleared"
  | "monitoring";

export interface FlowReleaseEscalationLedgerEntry {
  escalationId: string;
  recordedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  monitorId: string | null;
  monitorJson: string;
  requirementId: string;
  source: FlowReleaseEvidenceRequirementSource;
  owner: string;
  title: string;
  slaState: FlowReleaseEvidenceSlaState;
  escalationLevel: FlowReleaseEvidenceEscalationLevel;
  ownerResponse: FlowReleaseEscalationOwnerResponse;
  gateOutcome: FlowReleaseEscalationGateOutcome;
  acknowledgementRequired: boolean;
  acknowledged: boolean;
  activeCarryover: boolean;
  releaseGateBlocking: boolean;
  evidencePath: string;
  escalationCopy: string;
  ownerResponseCopy: string;
  nextAction: string;
}

export interface FlowReleaseEscalationOwnerGroup {
  owner: string;
  entryCount: number;
  activeCount: number;
  acknowledgedCount: number;
  acknowledgementBlockerCount: number;
  carryoverCount: number;
  releaseGateBlockingCount: number;
  entries: string[];
}

export interface FlowReleaseEscalationLedger {
  ledgerId: string;
  ledgerJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  entryCount: number;
  activeCount: number;
  acknowledgedCount: number;
  responsePendingCount: number;
  rejectedCount: number;
  resolvedCount: number;
  carryoverCount: number;
  releaseGateBlockingCount: number;
  acknowledgementBlockerCount: number;
  ownerCount: number;
  latestEscalationId: string | null;
  latestGateOutcome: FlowReleaseEscalationGateOutcome | null;
  ownerGroups: FlowReleaseEscalationOwnerGroup[];
  entries: FlowReleaseEscalationLedgerEntry[];
  ownerResponseCopy: string;
  summary: string;
  commands: string[];
}

export type FlowReleaseCheckpointDecision =
  | "ready"
  | "hold"
  | "carry-over"
  | "needs-review";

export type FlowReleaseCheckpointReviewState =
  | "ready"
  | "hold"
  | "carry-over"
  | "review-required";

export type FlowReleaseCheckpointReviewSource =
  | "escalation-ledger"
  | "sla-monitor"
  | "owner-follow-up"
  | "prevention-plan"
  | "stability-board"
  | "missing-evidence";

export interface FlowReleaseCheckpointReviewItem {
  id: string;
  source: FlowReleaseCheckpointReviewSource;
  owner: string;
  title: string;
  state: FlowReleaseCheckpointReviewState;
  decision: FlowReleaseCheckpointDecision;
  acknowledgementRequired: boolean;
  acknowledged: boolean;
  activeCarryover: boolean;
  releaseGateBlocking: boolean;
  evidencePath: string;
  summary: string;
  nextAction: string;
}

export interface FlowReleaseCheckpointReviewOwnerGroup {
  owner: string;
  itemCount: number;
  holdCount: number;
  carryoverCount: number;
  reviewRequiredCount: number;
  acknowledgementBlockerCount: number;
  releaseGateBlockingCount: number;
  items: string[];
}

export interface FlowReleaseCheckpointReviewBoardReport {
  reviewId: string;
  reviewJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  scoreOutOf100: number;
  decision: FlowReleaseCheckpointDecision;
  readyForCheckpoint: boolean;
  itemCount: number;
  ownerCount: number;
  holdCount: number;
  carryoverCount: number;
  reviewRequiredCount: number;
  acknowledgementRequiredCount: number;
  acknowledgementBlockerCount: number;
  activeEscalationCount: number;
  releaseGateBlockingCount: number;
  escalationLedgerJson: string;
  slaMonitorJson: string;
  ownerFollowupBoardJson: string;
  preventionPlanJson: string;
  stabilityBoardJson: string;
  ownerGroups: FlowReleaseCheckpointReviewOwnerGroup[];
  items: FlowReleaseCheckpointReviewItem[];
  reviewNotesCopy: string;
  summary: string;
  commands: string[];
}

export type FlowReleaseCheckpointSignoffDecision =
  | "signed-off"
  | "held"
  | "carried-over"
  | "superseded"
  | "revoked";

export interface FlowReleaseCheckpointSignoffRecord {
  signoffId: string;
  reviewId: string;
  reviewJson: string;
  recordedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  decision: FlowReleaseCheckpointSignoffDecision;
  operator: string;
  reason: string;
  acknowledgementEvidencePath: string;
  acknowledgementEvidencePresent: boolean;
  acknowledgementEvidenceBytes: number;
  carryoverCommitment: string;
  reviewDecision: FlowReleaseCheckpointDecision;
  reviewScoreOutOf100: number;
  reviewReadyForCheckpoint: boolean;
  reviewHoldCount: number;
  reviewCarryoverCount: number;
  reviewAcknowledgementBlockerCount: number;
  releaseGateBlockingCount: number;
  activeHold: boolean;
  activeCarryover: boolean;
  releaseNotes: string;
  summary: string;
}

export interface FlowReleaseCheckpointSignoffLedger {
  ledgerId: string;
  ledgerJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  recordCount: number;
  signedOffCount: number;
  heldCount: number;
  carriedOverCount: number;
  supersededCount: number;
  revokedCount: number;
  activeSignoffId: string | null;
  activeReviewId: string | null;
  activeDecision: FlowReleaseCheckpointSignoffDecision | null;
  activeHoldCount: number;
  activeCarryoverCount: number;
  acknowledgementEvidenceMissingCount: number;
  releaseGateBlockingCount: number;
  records: FlowReleaseCheckpointSignoffRecord[];
  releaseNotesCopy: string;
  summary: string;
  commands: string[];
}

export type FlowReleaseCheckpointEvidenceVaultEntryKind =
  | "checkpoint-review-json"
  | "checkpoint-signoff-ledger-json"
  | "acknowledgement-evidence"
  | "carryover-commitment"
  | "release-notes";

export interface FlowReleaseCheckpointEvidenceVaultEntry {
  id: string;
  label: string;
  kind: FlowReleaseCheckpointEvidenceVaultEntryKind;
  path: string;
  required: boolean;
  present: boolean;
  bytes: number;
  sha256: string | null;
  sourceId: string;
  summary: string;
  warning: string | null;
}

export interface FlowReleaseCheckpointEvidenceVault {
  vaultId: string;
  vaultJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  readyToArchive: boolean;
  reviewId: string | null;
  reviewDecision: FlowReleaseCheckpointDecision | null;
  reviewScoreOutOf100: number | null;
  signoffLedgerId: string | null;
  activeSignoffId: string | null;
  activeDecision: FlowReleaseCheckpointSignoffDecision | null;
  entryCount: number;
  requiredCount: number;
  presentCount: number;
  missingCount: number;
  checksumCount: number;
  acknowledgementEvidenceMissingCount: number;
  activeHoldCount: number;
  activeCarryoverCount: number;
  releaseGateBlockingCount: number;
  manifestSha256: string;
  entries: FlowReleaseCheckpointEvidenceVaultEntry[];
  attachmentNotesCopy: string;
  summary: string;
  commands: string[];
}

export type FlowReleaseEvidenceAttachmentState =
  | "ready"
  | "missing"
  | "inline-only"
  | "checksum-missing"
  | "blocked";

export interface FlowReleaseEvidenceAttachmentReviewItem {
  id: string;
  vaultEntryId: string;
  label: string;
  kind: FlowReleaseCheckpointEvidenceVaultEntryKind;
  path: string;
  state: FlowReleaseEvidenceAttachmentState;
  required: boolean;
  present: boolean;
  attachable: boolean;
  bytes: number;
  sha256: string | null;
  sourceId: string;
  releaseGateBlocking: boolean;
  summary: string;
  nextAction: string;
}

export interface FlowReleaseEvidenceAttachmentReview {
  reviewId: string;
  reviewJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  readyForHandoff: boolean;
  vaultId: string;
  vaultJson: string;
  manifestSha256: string;
  itemCount: number;
  attachableCount: number;
  missingCount: number;
  inlineOnlyCount: number;
  checksumMissingCount: number;
  blockedCount: number;
  releaseGateBlockingCount: number;
  firstBlocker: string | null;
  items: FlowReleaseEvidenceAttachmentReviewItem[];
  handoffNotesCopy: string;
  summary: string;
  commands: string[];
}

export type FlowReleaseHandoffPacketSectionKind =
  | "operator-summary"
  | "attachable-file"
  | "inline-note"
  | "unresolved-blocker"
  | "manifest-checksum";

export interface FlowReleaseHandoffPacketSection {
  id: string;
  kind: FlowReleaseHandoffPacketSectionKind;
  title: string;
  body: string;
  path: string;
  sourceId: string;
  required: boolean;
  included: boolean;
  checksum: string | null;
  nextAction: string;
}

export interface FlowReleaseHandoffPacket {
  packetId: string;
  packetJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  status: FlowDashboardPanelStatus;
  readyToSend: boolean;
  attachmentReviewId: string;
  attachmentReviewJson: string;
  manifestSha256: string;
  sectionCount: number;
  includedCount: number;
  attachableFileCount: number;
  inlineNoteCount: number;
  unresolvedBlockerCount: number;
  checksumCount: number;
  missingCount: number;
  firstBlocker: string | null;
  sections: FlowReleaseHandoffPacketSection[];
  fileChecklistCopy: string;
  handoffPacketCopy: string;
  summary: string;
  commands: string[];
}

export type FlowReleaseHandoffAuditState =
  | "draft"
  | "ready"
  | "sent"
  | "superseded"
  | "revoked"
  | "blocked";

export interface FlowReleaseHandoffAuditRecord {
  auditId: string;
  packetId: string;
  packetJson: string;
  recordedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  state: FlowReleaseHandoffAuditState;
  operator: string;
  acknowledgementNote: string;
  supersedesPacketId: string | null;
  packetReadyToSend: boolean;
  packetStatus: FlowDashboardPanelStatus;
  packetSectionCount: number;
  attachableFileCount: number;
  inlineNoteCount: number;
  unresolvedBlockerCount: number;
  missingCount: number;
  manifestSha256: string;
  active: boolean;
  blockerCarryover: number;
  auditNotes: string;
  summary: string;
}

export interface FlowReleaseHandoffAuditTrail {
  trailId: string;
  trailJson: string;
  generatedAtUnixMs: string;
  productName: string;
  localOnly: boolean;
  recordCount: number;
  draftCount: number;
  readyCount: number;
  sentCount: number;
  supersededCount: number;
  revokedCount: number;
  blockedCount: number;
  activeAuditId: string | null;
  activePacketId: string | null;
  latestAuditId: string | null;
  latestPacketId: string | null;
  latestState: FlowReleaseHandoffAuditState | null;
  latestReadyToSend: boolean;
  unresolvedBlockerCount: number;
  blockerCarryoverCount: number;
  acknowledgementCount: number;
  records: FlowReleaseHandoffAuditRecord[];
  auditSummaryCopy: string;
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

export function normalizeReleaseRecoveryRunbook(
  value: unknown,
): FlowReleaseRecoveryRunbookReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const runbook =
    root.release_recovery_runbook && typeof root.release_recovery_runbook === "object"
      ? (root.release_recovery_runbook as Record<string, unknown>)
      : root.releaseRecoveryRunbook && typeof root.releaseRecoveryRunbook === "object"
        ? (root.releaseRecoveryRunbook as Record<string, unknown>)
        : root;
  const runbookId = stringValue(runbook.runbook_id, runbook.runbookId);
  const phases = arrayValue(runbook.phases)
    .map((item): FlowReleaseRecoveryRunbookPhase | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const phase = item as Record<string, unknown>;
      const label = stringValue(phase.label);
      return {
        kind: recoveryRunbookPhaseKind(stringValue(phase.kind)),
        order: numberValue(phase.order),
        label,
        status: recoveryRunbookPhaseStatus(stringValue(phase.status)),
        approvalRequired: booleanValue(phase.approval_required, phase.approvalRequired),
        sourcePath: stringValue(phase.source_path, phase.sourcePath),
        objective: stringValue(phase.objective),
        command: stringValue(phase.command),
        verification: stringValue(phase.verification),
        risks: arrayValue(phase.risks)
          .map((risk) => stringValue(risk))
          .filter(Boolean),
        evidencePaths: arrayValue(phase.evidence_paths, phase.evidencePaths)
          .map((path) => stringValue(path))
          .filter(Boolean),
        nextAction: stringValue(phase.next_action, phase.nextAction),
      };
    })
    .filter(
      (phase): phase is FlowReleaseRecoveryRunbookPhase =>
        phase !== null && phase.label.trim().length > 0,
    );
  const approvalGates = arrayValue(runbook.approval_gates, runbook.approvalGates)
    .map((item): FlowReleaseRecoveryRunbookApprovalGate | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const gate = item as Record<string, unknown>;
      const id = stringValue(gate.id);
      if (!id) {
        return null;
      }
      return {
        id,
        label: stringValue(gate.label),
        phase: recoveryRunbookPhaseKind(stringValue(gate.phase)),
        required: booleanValue(gate.required),
        satisfied: booleanValue(gate.satisfied),
        summary: stringValue(gate.summary),
        operatorAction: stringValue(gate.operator_action, gate.operatorAction),
      };
    })
    .filter((gate): gate is FlowReleaseRecoveryRunbookApprovalGate => gate !== null);

  if (!runbookId && phases.length === 0 && approvalGates.length === 0) {
    return null;
  }

  return {
    runbookId,
    runbookJson: stringValue(runbook.runbook_json, runbook.runbookJson),
    generatedAtUnixMs: stringValue(runbook.generated_at_unix_ms, runbook.generatedAtUnixMs),
    productName: stringValue(runbook.product_name, runbook.productName),
    localOnly: booleanValue(runbook.local_only, runbook.localOnly),
    status: panelStatus(stringValue(runbook.status)),
    scoreOutOf100: numberValue(runbook.score_out_of_100, runbook.scoreOutOf100),
    readyForOperatorReview: booleanValue(
      runbook.ready_for_operator_review,
      runbook.readyForOperatorReview,
    ),
    readyToExecuteRecovery: booleanValue(
      runbook.ready_to_execute_recovery,
      runbook.readyToExecuteRecovery,
    ),
    activeCandidateId:
      stringValue(runbook.active_candidate_id, runbook.activeCandidateId) || null,
    activePromotionId:
      stringValue(runbook.active_promotion_id, runbook.activePromotionId) || null,
    activeRollbackReference:
      stringValue(runbook.active_rollback_reference, runbook.activeRollbackReference) || null,
    latestPromotionDecision:
      runbook.latest_promotion_decision == null && runbook.latestPromotionDecision == null
        ? null
        : promotionDecision(
            stringValue(runbook.latest_promotion_decision, runbook.latestPromotionDecision),
          ),
    stabilityBoardJson: stringValue(
      runbook.stability_board_json,
      runbook.stabilityBoardJson,
    ),
    rollbackDrillJson: stringValue(runbook.rollback_drill_json, runbook.rollbackDrillJson),
    promotionLedgerJson: stringValue(
      runbook.promotion_ledger_json,
      runbook.promotionLedgerJson,
    ),
    postPromotionMonitorJson: stringValue(
      runbook.post_promotion_monitor_json,
      runbook.postPromotionMonitorJson,
    ),
    phaseCount: numberValue(runbook.phase_count, runbook.phaseCount),
    blockedPhaseCount: numberValue(runbook.blocked_phase_count, runbook.blockedPhaseCount),
    approvalGateCount: numberValue(runbook.approval_gate_count, runbook.approvalGateCount),
    unsatisfiedApprovalGateCount: numberValue(
      runbook.unsatisfied_approval_gate_count,
      runbook.unsatisfiedApprovalGateCount,
    ),
    commandCount: numberValue(runbook.command_count, runbook.commandCount),
    activeRisks: arrayValue(runbook.active_risks, runbook.activeRisks)
      .map((risk) => stringValue(risk))
      .filter(Boolean),
    phases,
    approvalGates,
    recoveryCommands: arrayValue(runbook.recovery_commands, runbook.recoveryCommands)
      .map((command) => stringValue(command))
      .filter(Boolean),
    summary: stringValue(runbook.summary),
    commands: arrayValue(runbook.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseIncidentArchive(value: unknown): FlowReleaseIncidentArchive | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const archive =
    root.release_incident_archive && typeof root.release_incident_archive === "object"
      ? (root.release_incident_archive as Record<string, unknown>)
      : root.releaseIncidentArchive && typeof root.releaseIncidentArchive === "object"
        ? (root.releaseIncidentArchive as Record<string, unknown>)
        : root;
  const archiveId = stringValue(archive.archive_id, archive.archiveId);
  const entries = arrayValue(archive.entries)
    .map((item): FlowReleaseIncidentArchiveEntry | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const entry = item as Record<string, unknown>;
      const incidentId = stringValue(entry.incident_id, entry.incidentId);
      if (!incidentId) {
        return null;
      }
      return {
        incidentId,
        recordedAtUnixMs: stringValue(entry.recorded_at_unix_ms, entry.recordedAtUnixMs),
        productName: stringValue(entry.product_name, entry.productName),
        localOnly: booleanValue(entry.local_only, entry.localOnly),
        severity: releaseIncidentSeverity(stringValue(entry.severity)),
        outcome: releaseIncidentOutcome(stringValue(entry.outcome)),
        title: stringValue(entry.title),
        summary: stringValue(entry.summary),
        recoveryRunbookId:
          stringValue(entry.recovery_runbook_id, entry.recoveryRunbookId) || null,
        recoveryRunbookJson: stringValue(
          entry.recovery_runbook_json,
          entry.recoveryRunbookJson,
        ),
        stabilityBoardJson: stringValue(entry.stability_board_json, entry.stabilityBoardJson),
        rollbackDrillJson: stringValue(entry.rollback_drill_json, entry.rollbackDrillJson),
        postPromotionMonitorJson: stringValue(
          entry.post_promotion_monitor_json,
          entry.postPromotionMonitorJson,
        ),
        activeCandidateId:
          stringValue(entry.active_candidate_id, entry.activeCandidateId) || null,
        activePromotionId:
          stringValue(entry.active_promotion_id, entry.activePromotionId) || null,
        activeRollbackReference:
          stringValue(entry.active_rollback_reference, entry.activeRollbackReference) || null,
        blockedPhaseCount: numberValue(entry.blocked_phase_count, entry.blockedPhaseCount),
        activeRiskCount: numberValue(entry.active_risk_count, entry.activeRiskCount),
        incidentNotes: arrayValue(entry.incident_notes, entry.incidentNotes)
          .map((note): FlowReleaseIncidentNote | null => {
            if (!note || typeof note !== "object") {
              return null;
            }
            const record = note as Record<string, unknown>;
            const id = stringValue(record.id);
            if (!id) {
              return null;
            }
            return {
              id,
              path: stringValue(record.path),
              present: booleanValue(record.present),
              bytes: numberValue(record.bytes),
              summary: stringValue(record.summary),
            };
          })
          .filter((note): note is FlowReleaseIncidentNote => note !== null),
        followUpActions: arrayValue(entry.follow_up_actions, entry.followUpActions)
          .map((action) => stringValue(action))
          .filter(Boolean),
        preventionItems: arrayValue(entry.prevention_items, entry.preventionItems)
          .map((item) => stringValue(item))
          .filter(Boolean),
        evidencePaths: arrayValue(entry.evidence_paths, entry.evidencePaths)
          .map((path) => stringValue(path))
          .filter(Boolean),
      };
    })
    .filter((entry): entry is FlowReleaseIncidentArchiveEntry => entry !== null);

  if (!archiveId && entries.length === 0) {
    return null;
  }

  return {
    archiveId,
    archiveJson: stringValue(archive.archive_json, archive.archiveJson),
    generatedAtUnixMs: stringValue(archive.generated_at_unix_ms, archive.generatedAtUnixMs),
    productName: stringValue(archive.product_name, archive.productName),
    localOnly: booleanValue(archive.local_only, archive.localOnly),
    incidentCount: numberValue(archive.incident_count, archive.incidentCount),
    openCount: numberValue(archive.open_count, archive.openCount),
    monitoringCount: numberValue(archive.monitoring_count, archive.monitoringCount),
    resolvedCount: numberValue(archive.resolved_count, archive.resolvedCount),
    rolledBackCount: numberValue(archive.rolled_back_count, archive.rolledBackCount),
    preventedCount: numberValue(archive.prevented_count, archive.preventedCount),
    criticalCount: numberValue(archive.critical_count, archive.criticalCount),
    blockingCount: numberValue(archive.blocking_count, archive.blockingCount),
    followUpCount: numberValue(archive.follow_up_count, archive.followUpCount),
    latestIncidentId:
      stringValue(archive.latest_incident_id, archive.latestIncidentId) || null,
    latestSeverity:
      archive.latest_severity == null && archive.latestSeverity == null
        ? null
        : releaseIncidentSeverity(stringValue(archive.latest_severity, archive.latestSeverity)),
    latestOutcome:
      archive.latest_outcome == null && archive.latestOutcome == null
        ? null
        : releaseIncidentOutcome(stringValue(archive.latest_outcome, archive.latestOutcome)),
    latestRollbackReference:
      stringValue(archive.latest_rollback_reference, archive.latestRollbackReference) || null,
    entries,
    commands: arrayValue(archive.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleasePreventionPlan(
  value: unknown,
): FlowReleasePreventionPlanReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const plan =
    root.release_prevention_plan && typeof root.release_prevention_plan === "object"
      ? (root.release_prevention_plan as Record<string, unknown>)
      : root.releasePreventionPlan && typeof root.releasePreventionPlan === "object"
        ? (root.releasePreventionPlan as Record<string, unknown>)
        : root;
  const planId = stringValue(plan.plan_id, plan.planId);
  const findings = arrayValue(plan.findings)
    .map((item): FlowReleasePreventionFinding | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const finding = item as Record<string, unknown>;
      const id = stringValue(finding.id);
      if (!id) {
        return null;
      }
      return {
        id,
        kind: releasePreventionFindingKind(stringValue(finding.kind)),
        severity: releaseIncidentSeverity(stringValue(finding.severity)),
        title: stringValue(finding.title),
        recurrenceCount: numberValue(finding.recurrence_count, finding.recurrenceCount),
        sourcePaths: arrayValue(finding.source_paths, finding.sourcePaths)
          .map((path) => stringValue(path))
          .filter(Boolean),
        summary: stringValue(finding.summary),
        nextAction: stringValue(finding.next_action, finding.nextAction),
        releaseGateBlocking: booleanValue(
          finding.release_gate_blocking,
          finding.releaseGateBlocking,
        ),
      };
    })
    .filter((finding): finding is FlowReleasePreventionFinding => finding !== null);
  const actions = arrayValue(plan.actions)
    .map((item): FlowReleasePreventionAction | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const action = item as Record<string, unknown>;
      const id = stringValue(action.id);
      if (!id) {
        return null;
      }
      return {
        id,
        kind: releasePreventionActionKind(stringValue(action.kind)),
        status: releasePreventionActionStatus(stringValue(action.status)),
        owner: stringValue(action.owner),
        title: stringValue(action.title),
        summary: stringValue(action.summary),
        sourcePath: stringValue(action.source_path, action.sourcePath),
        evidencePath: stringValue(action.evidence_path, action.evidencePath),
        command: stringValue(action.command),
        required: booleanValue(action.required),
        releaseGateBlocking: booleanValue(action.release_gate_blocking, action.releaseGateBlocking),
        nextAction: stringValue(action.next_action, action.nextAction),
      };
    })
    .filter((action): action is FlowReleasePreventionAction => action !== null);
  const evidenceLinks = arrayValue(plan.evidence_links, plan.evidenceLinks)
    .map((item): FlowReleasePreventionEvidenceLink | null => {
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
    .filter((link): link is FlowReleasePreventionEvidenceLink => link !== null);

  if (!planId && findings.length === 0 && actions.length === 0) {
    return null;
  }

  return {
    planId,
    planJson: stringValue(plan.plan_json, plan.planJson),
    generatedAtUnixMs: stringValue(plan.generated_at_unix_ms, plan.generatedAtUnixMs),
    productName: stringValue(plan.product_name, plan.productName),
    localOnly: booleanValue(plan.local_only, plan.localOnly),
    status: panelStatus(stringValue(plan.status)),
    scoreOutOf100: numberValue(plan.score_out_of_100, plan.scoreOutOf100),
    readyForNextCheckpoint: booleanValue(
      plan.ready_for_next_checkpoint,
      plan.readyForNextCheckpoint,
    ),
    incidentArchiveJson: stringValue(plan.incident_archive_json, plan.incidentArchiveJson),
    stabilityBoardJson: stringValue(plan.stability_board_json, plan.stabilityBoardJson),
    incidentCount: numberValue(plan.incident_count, plan.incidentCount),
    findingCount: numberValue(plan.finding_count, plan.findingCount),
    recurringIssueCount: numberValue(plan.recurring_issue_count, plan.recurringIssueCount),
    actionCount: numberValue(plan.action_count, plan.actionCount),
    ownerReadyCount: numberValue(plan.owner_ready_count, plan.ownerReadyCount),
    blockerCount: numberValue(plan.blocker_count, plan.blockerCount),
    evidenceMissingCount: numberValue(plan.evidence_missing_count, plan.evidenceMissingCount),
    gateBlockingCount: numberValue(plan.gate_blocking_count, plan.gateBlockingCount),
    latestIncidentId: stringValue(plan.latest_incident_id, plan.latestIncidentId) || null,
    activeRollbackReference:
      stringValue(plan.active_rollback_reference, plan.activeRollbackReference) || null,
    findings,
    actions,
    evidenceLinks,
    ownerReadyCopy: stringValue(plan.owner_ready_copy, plan.ownerReadyCopy),
    summary: stringValue(plan.summary),
    commands: arrayValue(plan.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseOwnerFollowUpBoard(
  value: unknown,
): FlowReleaseOwnerFollowUpBoardReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const board =
    root.release_owner_followup_board && typeof root.release_owner_followup_board === "object"
      ? (root.release_owner_followup_board as Record<string, unknown>)
      : root.releaseOwnerFollowUpBoard && typeof root.releaseOwnerFollowUpBoard === "object"
        ? (root.releaseOwnerFollowUpBoard as Record<string, unknown>)
        : root;
  const boardId = stringValue(board.board_id, board.boardId);
  const records = arrayValue(board.records)
    .map((item): FlowReleaseOwnerFollowUpRecord | null => {
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
        actionId: stringValue(record.action_id, record.actionId),
        owner: stringValue(record.owner),
        title: stringValue(record.title),
        summary: stringValue(record.summary),
        completionState: releaseOwnerFollowUpCompletionState(
          stringValue(record.completion_state, record.completionState),
        ),
        evidenceState: releaseOwnerFollowUpEvidenceState(
          stringValue(record.evidence_state, record.evidenceState),
        ),
        sourcePath: stringValue(record.source_path, record.sourcePath),
        evidencePath: stringValue(record.evidence_path, record.evidencePath),
        evidenceRequest: stringValue(record.evidence_request, record.evidenceRequest),
        dueAfterUnixMs: stringValue(record.due_after_unix_ms, record.dueAfterUnixMs),
        dueBeforeUnixMs: stringValue(record.due_before_unix_ms, record.dueBeforeUnixMs),
        overdue: booleanValue(record.overdue),
        required: booleanValue(record.required),
        releaseGateBlocking: booleanValue(record.release_gate_blocking, record.releaseGateBlocking),
        command: stringValue(record.command),
        assignmentCopy: stringValue(record.assignment_copy, record.assignmentCopy),
        nextAction: stringValue(record.next_action, record.nextAction),
      };
    })
    .filter((record): record is FlowReleaseOwnerFollowUpRecord => record !== null);
  const ownerGroups = arrayValue(board.owner_groups, board.ownerGroups)
    .map((item): FlowReleaseOwnerFollowUpGroup | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const group = item as Record<string, unknown>;
      const owner = stringValue(group.owner);
      if (!owner) {
        return null;
      }
      return {
        owner,
        recordCount: numberValue(group.record_count, group.recordCount),
        readyCount: numberValue(group.ready_count, group.readyCount),
        waitingCount: numberValue(group.waiting_count, group.waitingCount),
        blockedCount: numberValue(group.blocked_count, group.blockedCount),
        overdueCount: numberValue(group.overdue_count, group.overdueCount),
        completeCount: numberValue(group.complete_count, group.completeCount),
        evidenceMissingCount: numberValue(
          group.evidence_missing_count,
          group.evidenceMissingCount,
        ),
        records: arrayValue(group.records)
          .map((record) => stringValue(record))
          .filter(Boolean),
      };
    })
    .filter((group): group is FlowReleaseOwnerFollowUpGroup => group !== null);

  if (!boardId && records.length === 0) {
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
    readyForNextCheckpoint: booleanValue(
      board.ready_for_next_checkpoint,
      board.readyForNextCheckpoint,
    ),
    preventionPlanJson: stringValue(board.prevention_plan_json, board.preventionPlanJson),
    incidentArchiveJson: stringValue(board.incident_archive_json, board.incidentArchiveJson),
    stabilityBoardJson: stringValue(board.stability_board_json, board.stabilityBoardJson),
    recordCount: numberValue(board.record_count, board.recordCount),
    ownerCount: numberValue(board.owner_count, board.ownerCount),
    readyCount: numberValue(board.ready_count, board.readyCount),
    waitingCount: numberValue(board.waiting_count, board.waitingCount),
    blockedCount: numberValue(board.blocked_count, board.blockedCount),
    overdueCount: numberValue(board.overdue_count, board.overdueCount),
    completeCount: numberValue(board.complete_count, board.completeCount),
    evidenceMissingCount: numberValue(board.evidence_missing_count, board.evidenceMissingCount),
    gateBlockingCount: numberValue(board.gate_blocking_count, board.gateBlockingCount),
    ownerGroups,
    records,
    assignmentCopy: stringValue(board.assignment_copy, board.assignmentCopy),
    summary: stringValue(board.summary),
    commands: arrayValue(board.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseEvidenceSlaMonitor(
  value: unknown,
): FlowReleaseEvidenceSlaMonitorReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const monitor =
    root.release_evidence_sla_monitor &&
    typeof root.release_evidence_sla_monitor === "object"
      ? (root.release_evidence_sla_monitor as Record<string, unknown>)
      : root.releaseEvidenceSlaMonitor && typeof root.releaseEvidenceSlaMonitor === "object"
        ? (root.releaseEvidenceSlaMonitor as Record<string, unknown>)
        : root;
  const monitorId = stringValue(monitor.monitor_id, monitor.monitorId);
  const requirements = arrayValue(monitor.requirements)
    .map((item): FlowReleaseEvidenceSlaRequirement | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const requirement = item as Record<string, unknown>;
      const id = stringValue(requirement.id);
      if (!id) {
        return null;
      }
      return {
        id,
        source: releaseEvidenceRequirementSource(stringValue(requirement.source)),
        owner: stringValue(requirement.owner),
        title: stringValue(requirement.title),
        state: releaseEvidenceSlaState(stringValue(requirement.state)),
        escalationLevel: releaseEvidenceEscalationLevel(
          stringValue(requirement.escalation_level, requirement.escalationLevel),
        ),
        evidencePath: stringValue(requirement.evidence_path, requirement.evidencePath),
        evidencePresent: booleanValue(requirement.evidence_present, requirement.evidencePresent),
        dueAfterUnixMs: stringValue(requirement.due_after_unix_ms, requirement.dueAfterUnixMs),
        dueBeforeUnixMs: stringValue(requirement.due_before_unix_ms, requirement.dueBeforeUnixMs),
        slaWindowMs: stringValue(requirement.sla_window_ms, requirement.slaWindowMs),
        ageMs: stringValue(requirement.age_ms, requirement.ageMs),
        acknowledgementRequired: booleanValue(
          requirement.acknowledgement_required,
          requirement.acknowledgementRequired,
        ),
        releaseGateBlocking: booleanValue(
          requirement.release_gate_blocking,
          requirement.releaseGateBlocking,
        ),
        escalationCopy: stringValue(requirement.escalation_copy, requirement.escalationCopy),
        nextAction: stringValue(requirement.next_action, requirement.nextAction),
      };
    })
    .filter((requirement): requirement is FlowReleaseEvidenceSlaRequirement => requirement !== null);
  const ownerGroups = arrayValue(monitor.owner_groups, monitor.ownerGroups)
    .map((item): FlowReleaseEvidenceSlaOwnerGroup | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const group = item as Record<string, unknown>;
      const owner = stringValue(group.owner);
      if (!owner) {
        return null;
      }
      return {
        owner,
        requirementCount: numberValue(group.requirement_count, group.requirementCount),
        freshCount: numberValue(group.fresh_count, group.freshCount),
        dueSoonCount: numberValue(group.due_soon_count, group.dueSoonCount),
        overdueCount: numberValue(group.overdue_count, group.overdueCount),
        missingCount: numberValue(group.missing_count, group.missingCount),
        blockedCount: numberValue(group.blocked_count, group.blockedCount),
        acknowledgedCount: numberValue(group.acknowledged_count, group.acknowledgedCount),
        escalationCount: numberValue(group.escalation_count, group.escalationCount),
        releaseGateBlockingCount: numberValue(
          group.release_gate_blocking_count,
          group.releaseGateBlockingCount,
        ),
        requirements: arrayValue(group.requirements)
          .map((requirement) => stringValue(requirement))
          .filter(Boolean),
      };
    })
    .filter((group): group is FlowReleaseEvidenceSlaOwnerGroup => group !== null);

  if (!monitorId && requirements.length === 0) {
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
    readyForNextCheckpoint: booleanValue(
      monitor.ready_for_next_checkpoint,
      monitor.readyForNextCheckpoint,
    ),
    ownerFollowupBoardJson: stringValue(
      monitor.owner_followup_board_json,
      monitor.ownerFollowupBoardJson,
    ),
    preventionPlanJson: stringValue(monitor.prevention_plan_json, monitor.preventionPlanJson),
    stabilityBoardJson: stringValue(monitor.stability_board_json, monitor.stabilityBoardJson),
    requirementCount: numberValue(monitor.requirement_count, monitor.requirementCount),
    ownerCount: numberValue(monitor.owner_count, monitor.ownerCount),
    freshCount: numberValue(monitor.fresh_count, monitor.freshCount),
    dueSoonCount: numberValue(monitor.due_soon_count, monitor.dueSoonCount),
    overdueCount: numberValue(monitor.overdue_count, monitor.overdueCount),
    missingCount: numberValue(monitor.missing_count, monitor.missingCount),
    blockedCount: numberValue(monitor.blocked_count, monitor.blockedCount),
    acknowledgedCount: numberValue(monitor.acknowledged_count, monitor.acknowledgedCount),
    escalationCount: numberValue(monitor.escalation_count, monitor.escalationCount),
    gateBlockingCount: numberValue(monitor.gate_blocking_count, monitor.gateBlockingCount),
    ownerGroups,
    requirements,
    escalationCopy: stringValue(monitor.escalation_copy, monitor.escalationCopy),
    summary: stringValue(monitor.summary),
    commands: arrayValue(monitor.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseEscalationLedger(
  value: unknown,
): FlowReleaseEscalationLedger | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const ledger =
    root.release_escalation_ledger && typeof root.release_escalation_ledger === "object"
      ? (root.release_escalation_ledger as Record<string, unknown>)
      : root.releaseEscalationLedger && typeof root.releaseEscalationLedger === "object"
        ? (root.releaseEscalationLedger as Record<string, unknown>)
        : root;
  const ledgerId = stringValue(ledger.ledger_id, ledger.ledgerId);
  const entries = arrayValue(ledger.entries)
    .map((item): FlowReleaseEscalationLedgerEntry | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const entry = item as Record<string, unknown>;
      const escalationId = stringValue(entry.escalation_id, entry.escalationId);
      if (!escalationId) {
        return null;
      }
      return {
        escalationId,
        recordedAtUnixMs: stringValue(entry.recorded_at_unix_ms, entry.recordedAtUnixMs),
        productName: stringValue(entry.product_name, entry.productName),
        localOnly: booleanValue(entry.local_only, entry.localOnly),
        monitorId: stringValue(entry.monitor_id, entry.monitorId) || null,
        monitorJson: stringValue(entry.monitor_json, entry.monitorJson),
        requirementId: stringValue(entry.requirement_id, entry.requirementId),
        source: releaseEvidenceRequirementSource(stringValue(entry.source)),
        owner: stringValue(entry.owner),
        title: stringValue(entry.title),
        slaState: releaseEvidenceSlaState(stringValue(entry.sla_state, entry.slaState)),
        escalationLevel: releaseEvidenceEscalationLevel(
          stringValue(entry.escalation_level, entry.escalationLevel),
        ),
        ownerResponse: releaseEscalationOwnerResponse(
          stringValue(entry.owner_response, entry.ownerResponse),
        ),
        gateOutcome: releaseEscalationGateOutcome(
          stringValue(entry.gate_outcome, entry.gateOutcome),
        ),
        acknowledgementRequired: booleanValue(
          entry.acknowledgement_required,
          entry.acknowledgementRequired,
        ),
        acknowledged: booleanValue(entry.acknowledged),
        activeCarryover: booleanValue(entry.active_carryover, entry.activeCarryover),
        releaseGateBlocking: booleanValue(entry.release_gate_blocking, entry.releaseGateBlocking),
        evidencePath: stringValue(entry.evidence_path, entry.evidencePath),
        escalationCopy: stringValue(entry.escalation_copy, entry.escalationCopy),
        ownerResponseCopy: stringValue(entry.owner_response_copy, entry.ownerResponseCopy),
        nextAction: stringValue(entry.next_action, entry.nextAction),
      };
    })
    .filter((entry): entry is FlowReleaseEscalationLedgerEntry => entry !== null);
  const ownerGroups = arrayValue(ledger.owner_groups, ledger.ownerGroups)
    .map((item): FlowReleaseEscalationOwnerGroup | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const group = item as Record<string, unknown>;
      const owner = stringValue(group.owner);
      if (!owner) {
        return null;
      }
      return {
        owner,
        entryCount: numberValue(group.entry_count, group.entryCount),
        activeCount: numberValue(group.active_count, group.activeCount),
        acknowledgedCount: numberValue(group.acknowledged_count, group.acknowledgedCount),
        acknowledgementBlockerCount: numberValue(
          group.acknowledgement_blocker_count,
          group.acknowledgementBlockerCount,
        ),
        carryoverCount: numberValue(group.carryover_count, group.carryoverCount),
        releaseGateBlockingCount: numberValue(
          group.release_gate_blocking_count,
          group.releaseGateBlockingCount,
        ),
        entries: arrayValue(group.entries)
          .map((entry) => stringValue(entry))
          .filter(Boolean),
      };
    })
    .filter((group): group is FlowReleaseEscalationOwnerGroup => group !== null);

  if (!ledgerId && entries.length === 0) {
    return null;
  }

  return {
    ledgerId,
    ledgerJson: stringValue(ledger.ledger_json, ledger.ledgerJson),
    generatedAtUnixMs: stringValue(ledger.generated_at_unix_ms, ledger.generatedAtUnixMs),
    productName: stringValue(ledger.product_name, ledger.productName),
    localOnly: booleanValue(ledger.local_only, ledger.localOnly),
    entryCount: numberValue(ledger.entry_count, ledger.entryCount),
    activeCount: numberValue(ledger.active_count, ledger.activeCount),
    acknowledgedCount: numberValue(ledger.acknowledged_count, ledger.acknowledgedCount),
    responsePendingCount: numberValue(ledger.response_pending_count, ledger.responsePendingCount),
    rejectedCount: numberValue(ledger.rejected_count, ledger.rejectedCount),
    resolvedCount: numberValue(ledger.resolved_count, ledger.resolvedCount),
    carryoverCount: numberValue(ledger.carryover_count, ledger.carryoverCount),
    releaseGateBlockingCount: numberValue(
      ledger.release_gate_blocking_count,
      ledger.releaseGateBlockingCount,
    ),
    acknowledgementBlockerCount: numberValue(
      ledger.acknowledgement_blocker_count,
      ledger.acknowledgementBlockerCount,
    ),
    ownerCount: numberValue(ledger.owner_count, ledger.ownerCount),
    latestEscalationId: stringValue(ledger.latest_escalation_id, ledger.latestEscalationId) || null,
    latestGateOutcome:
      ledger.latest_gate_outcome == null && ledger.latestGateOutcome == null
        ? null
        : releaseEscalationGateOutcome(
            stringValue(ledger.latest_gate_outcome, ledger.latestGateOutcome),
          ),
    ownerGroups,
    entries,
    ownerResponseCopy: stringValue(ledger.owner_response_copy, ledger.ownerResponseCopy),
    summary: stringValue(ledger.summary),
    commands: arrayValue(ledger.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseCheckpointReview(
  value: unknown,
): FlowReleaseCheckpointReviewBoardReport | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const review =
    root.release_checkpoint_review && typeof root.release_checkpoint_review === "object"
      ? (root.release_checkpoint_review as Record<string, unknown>)
      : root.releaseCheckpointReview && typeof root.releaseCheckpointReview === "object"
        ? (root.releaseCheckpointReview as Record<string, unknown>)
        : root;
  const reviewId = stringValue(review.review_id, review.reviewId);
  const items = arrayValue(review.items)
    .map((item): FlowReleaseCheckpointReviewItem | null => {
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
        source: releaseCheckpointReviewSource(stringValue(record.source)),
        owner: stringValue(record.owner),
        title: stringValue(record.title),
        state: releaseCheckpointReviewState(stringValue(record.state)),
        decision: releaseCheckpointDecision(stringValue(record.decision)),
        acknowledgementRequired: booleanValue(
          record.acknowledgement_required,
          record.acknowledgementRequired,
        ),
        acknowledged: booleanValue(record.acknowledged),
        activeCarryover: booleanValue(record.active_carryover, record.activeCarryover),
        releaseGateBlocking: booleanValue(record.release_gate_blocking, record.releaseGateBlocking),
        evidencePath: stringValue(record.evidence_path, record.evidencePath),
        summary: stringValue(record.summary),
        nextAction: stringValue(record.next_action, record.nextAction),
      };
    })
    .filter((item): item is FlowReleaseCheckpointReviewItem => item !== null);
  const ownerGroups = arrayValue(review.owner_groups, review.ownerGroups)
    .map((item): FlowReleaseCheckpointReviewOwnerGroup | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const group = item as Record<string, unknown>;
      const owner = stringValue(group.owner);
      if (!owner) {
        return null;
      }
      return {
        owner,
        itemCount: numberValue(group.item_count, group.itemCount),
        holdCount: numberValue(group.hold_count, group.holdCount),
        carryoverCount: numberValue(group.carryover_count, group.carryoverCount),
        reviewRequiredCount: numberValue(
          group.review_required_count,
          group.reviewRequiredCount,
        ),
        acknowledgementBlockerCount: numberValue(
          group.acknowledgement_blocker_count,
          group.acknowledgementBlockerCount,
        ),
        releaseGateBlockingCount: numberValue(
          group.release_gate_blocking_count,
          group.releaseGateBlockingCount,
        ),
        items: arrayValue(group.items)
          .map((entry) => stringValue(entry))
          .filter(Boolean),
      };
    })
    .filter((group): group is FlowReleaseCheckpointReviewOwnerGroup => group !== null);

  if (!reviewId && items.length === 0) {
    return null;
  }

  return {
    reviewId,
    reviewJson: stringValue(review.review_json, review.reviewJson),
    generatedAtUnixMs: stringValue(review.generated_at_unix_ms, review.generatedAtUnixMs),
    productName: stringValue(review.product_name, review.productName),
    localOnly: booleanValue(review.local_only, review.localOnly),
    status: panelStatus(stringValue(review.status)),
    scoreOutOf100: numberValue(review.score_out_of_100, review.scoreOutOf100),
    decision: releaseCheckpointDecision(stringValue(review.decision)),
    readyForCheckpoint: booleanValue(review.ready_for_checkpoint, review.readyForCheckpoint),
    itemCount: numberValue(review.item_count, review.itemCount),
    ownerCount: numberValue(review.owner_count, review.ownerCount),
    holdCount: numberValue(review.hold_count, review.holdCount),
    carryoverCount: numberValue(review.carryover_count, review.carryoverCount),
    reviewRequiredCount: numberValue(review.review_required_count, review.reviewRequiredCount),
    acknowledgementRequiredCount: numberValue(
      review.acknowledgement_required_count,
      review.acknowledgementRequiredCount,
    ),
    acknowledgementBlockerCount: numberValue(
      review.acknowledgement_blocker_count,
      review.acknowledgementBlockerCount,
    ),
    activeEscalationCount: numberValue(
      review.active_escalation_count,
      review.activeEscalationCount,
    ),
    releaseGateBlockingCount: numberValue(
      review.release_gate_blocking_count,
      review.releaseGateBlockingCount,
    ),
    escalationLedgerJson: stringValue(
      review.escalation_ledger_json,
      review.escalationLedgerJson,
    ),
    slaMonitorJson: stringValue(review.sla_monitor_json, review.slaMonitorJson),
    ownerFollowupBoardJson: stringValue(
      review.owner_followup_board_json,
      review.ownerFollowupBoardJson,
    ),
    preventionPlanJson: stringValue(review.prevention_plan_json, review.preventionPlanJson),
    stabilityBoardJson: stringValue(review.stability_board_json, review.stabilityBoardJson),
    ownerGroups,
    items,
    reviewNotesCopy: stringValue(review.review_notes_copy, review.reviewNotesCopy),
    summary: stringValue(review.summary),
    commands: arrayValue(review.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseCheckpointSignoffLedger(
  value: unknown,
): FlowReleaseCheckpointSignoffLedger | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const ledger =
    root.release_checkpoint_signoff_ledger &&
    typeof root.release_checkpoint_signoff_ledger === "object"
      ? (root.release_checkpoint_signoff_ledger as Record<string, unknown>)
      : root.releaseCheckpointSignoffLedger &&
          typeof root.releaseCheckpointSignoffLedger === "object"
        ? (root.releaseCheckpointSignoffLedger as Record<string, unknown>)
        : root;
  const ledgerId = stringValue(ledger.ledger_id, ledger.ledgerId);
  const records = arrayValue(ledger.records)
    .map((item): FlowReleaseCheckpointSignoffRecord | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const record = item as Record<string, unknown>;
      const signoffId = stringValue(record.signoff_id, record.signoffId);
      if (!signoffId) {
        return null;
      }
      return {
        signoffId,
        reviewId: stringValue(record.review_id, record.reviewId),
        reviewJson: stringValue(record.review_json, record.reviewJson),
        recordedAtUnixMs: stringValue(record.recorded_at_unix_ms, record.recordedAtUnixMs),
        productName: stringValue(record.product_name, record.productName),
        localOnly: booleanValue(record.local_only, record.localOnly),
        decision: releaseCheckpointSignoffDecision(stringValue(record.decision)),
        operator: stringValue(record.operator),
        reason: stringValue(record.reason),
        acknowledgementEvidencePath: stringValue(
          record.acknowledgement_evidence_path,
          record.acknowledgementEvidencePath,
        ),
        acknowledgementEvidencePresent: booleanValue(
          record.acknowledgement_evidence_present,
          record.acknowledgementEvidencePresent,
        ),
        acknowledgementEvidenceBytes: numberValue(
          record.acknowledgement_evidence_bytes,
          record.acknowledgementEvidenceBytes,
        ),
        carryoverCommitment: stringValue(
          record.carryover_commitment,
          record.carryoverCommitment,
        ),
        reviewDecision: releaseCheckpointDecision(
          stringValue(record.review_decision, record.reviewDecision),
        ),
        reviewScoreOutOf100: numberValue(
          record.review_score_out_of_100,
          record.reviewScoreOutOf100,
        ),
        reviewReadyForCheckpoint: booleanValue(
          record.review_ready_for_checkpoint,
          record.reviewReadyForCheckpoint,
        ),
        reviewHoldCount: numberValue(record.review_hold_count, record.reviewHoldCount),
        reviewCarryoverCount: numberValue(
          record.review_carryover_count,
          record.reviewCarryoverCount,
        ),
        reviewAcknowledgementBlockerCount: numberValue(
          record.review_acknowledgement_blocker_count,
          record.reviewAcknowledgementBlockerCount,
        ),
        releaseGateBlockingCount: numberValue(
          record.release_gate_blocking_count,
          record.releaseGateBlockingCount,
        ),
        activeHold: booleanValue(record.active_hold, record.activeHold),
        activeCarryover: booleanValue(record.active_carryover, record.activeCarryover),
        releaseNotes: stringValue(record.release_notes, record.releaseNotes),
        summary: stringValue(record.summary),
      };
    })
    .filter((record): record is FlowReleaseCheckpointSignoffRecord => record !== null);

  if (!ledgerId && records.length === 0) {
    return null;
  }

  return {
    ledgerId,
    ledgerJson: stringValue(ledger.ledger_json, ledger.ledgerJson),
    generatedAtUnixMs: stringValue(ledger.generated_at_unix_ms, ledger.generatedAtUnixMs),
    productName: stringValue(ledger.product_name, ledger.productName),
    localOnly: booleanValue(ledger.local_only, ledger.localOnly),
    recordCount: numberValue(ledger.record_count, ledger.recordCount),
    signedOffCount: numberValue(ledger.signed_off_count, ledger.signedOffCount),
    heldCount: numberValue(ledger.held_count, ledger.heldCount),
    carriedOverCount: numberValue(ledger.carried_over_count, ledger.carriedOverCount),
    supersededCount: numberValue(ledger.superseded_count, ledger.supersededCount),
    revokedCount: numberValue(ledger.revoked_count, ledger.revokedCount),
    activeSignoffId: stringValue(ledger.active_signoff_id, ledger.activeSignoffId) || null,
    activeReviewId: stringValue(ledger.active_review_id, ledger.activeReviewId) || null,
    activeDecision:
      ledger.active_decision == null && ledger.activeDecision == null
        ? null
        : releaseCheckpointSignoffDecision(
            stringValue(ledger.active_decision, ledger.activeDecision),
          ),
    activeHoldCount: numberValue(ledger.active_hold_count, ledger.activeHoldCount),
    activeCarryoverCount: numberValue(
      ledger.active_carryover_count,
      ledger.activeCarryoverCount,
    ),
    acknowledgementEvidenceMissingCount: numberValue(
      ledger.acknowledgement_evidence_missing_count,
      ledger.acknowledgementEvidenceMissingCount,
    ),
    releaseGateBlockingCount: numberValue(
      ledger.release_gate_blocking_count,
      ledger.releaseGateBlockingCount,
    ),
    records,
    releaseNotesCopy: stringValue(ledger.release_notes_copy, ledger.releaseNotesCopy),
    summary: stringValue(ledger.summary),
    commands: arrayValue(ledger.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseCheckpointEvidenceVault(
  value: unknown,
): FlowReleaseCheckpointEvidenceVault | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const vault =
    root.release_checkpoint_evidence_vault &&
    typeof root.release_checkpoint_evidence_vault === "object"
      ? (root.release_checkpoint_evidence_vault as Record<string, unknown>)
      : root.releaseCheckpointEvidenceVault &&
          typeof root.releaseCheckpointEvidenceVault === "object"
        ? (root.releaseCheckpointEvidenceVault as Record<string, unknown>)
        : root;
  const vaultId = stringValue(vault.vault_id, vault.vaultId);
  const entries = arrayValue(vault.entries)
    .map((item): FlowReleaseCheckpointEvidenceVaultEntry | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const entry = item as Record<string, unknown>;
      const id = stringValue(entry.id);
      if (!id) {
        return null;
      }
      return {
        id,
        label: stringValue(entry.label),
        kind: releaseCheckpointEvidenceVaultEntryKind(stringValue(entry.kind)),
        path: stringValue(entry.path),
        required: booleanValue(entry.required),
        present: booleanValue(entry.present),
        bytes: numberValue(entry.bytes),
        sha256: stringValue(entry.sha256) || null,
        sourceId: stringValue(entry.source_id, entry.sourceId),
        summary: stringValue(entry.summary),
        warning: stringValue(entry.warning) || null,
      };
    })
    .filter((entry): entry is FlowReleaseCheckpointEvidenceVaultEntry => entry !== null);

  if (!vaultId && entries.length === 0) {
    return null;
  }

  return {
    vaultId,
    vaultJson: stringValue(vault.vault_json, vault.vaultJson),
    generatedAtUnixMs: stringValue(vault.generated_at_unix_ms, vault.generatedAtUnixMs),
    productName: stringValue(vault.product_name, vault.productName),
    localOnly: booleanValue(vault.local_only, vault.localOnly),
    status: panelStatus(stringValue(vault.status)),
    readyToArchive: booleanValue(vault.ready_to_archive, vault.readyToArchive),
    reviewId: stringValue(vault.review_id, vault.reviewId) || null,
    reviewDecision:
      vault.review_decision == null && vault.reviewDecision == null
        ? null
        : releaseCheckpointDecision(stringValue(vault.review_decision, vault.reviewDecision)),
    reviewScoreOutOf100: nullableNumber(
      vault.review_score_out_of_100,
      vault.reviewScoreOutOf100,
    ),
    signoffLedgerId: stringValue(vault.signoff_ledger_id, vault.signoffLedgerId) || null,
    activeSignoffId: stringValue(vault.active_signoff_id, vault.activeSignoffId) || null,
    activeDecision:
      vault.active_decision == null && vault.activeDecision == null
        ? null
        : releaseCheckpointSignoffDecision(
            stringValue(vault.active_decision, vault.activeDecision),
          ),
    entryCount: numberValue(vault.entry_count, vault.entryCount),
    requiredCount: numberValue(vault.required_count, vault.requiredCount),
    presentCount: numberValue(vault.present_count, vault.presentCount),
    missingCount: numberValue(vault.missing_count, vault.missingCount),
    checksumCount: numberValue(vault.checksum_count, vault.checksumCount),
    acknowledgementEvidenceMissingCount: numberValue(
      vault.acknowledgement_evidence_missing_count,
      vault.acknowledgementEvidenceMissingCount,
    ),
    activeHoldCount: numberValue(vault.active_hold_count, vault.activeHoldCount),
    activeCarryoverCount: numberValue(vault.active_carryover_count, vault.activeCarryoverCount),
    releaseGateBlockingCount: numberValue(
      vault.release_gate_blocking_count,
      vault.releaseGateBlockingCount,
    ),
    manifestSha256: stringValue(vault.manifest_sha256, vault.manifestSha256),
    entries,
    attachmentNotesCopy: stringValue(vault.attachment_notes_copy, vault.attachmentNotesCopy),
    summary: stringValue(vault.summary),
    commands: arrayValue(vault.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseEvidenceAttachmentReview(
  value: unknown,
): FlowReleaseEvidenceAttachmentReview | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const review =
    root.release_evidence_attachment_review &&
    typeof root.release_evidence_attachment_review === "object"
      ? (root.release_evidence_attachment_review as Record<string, unknown>)
      : root.releaseEvidenceAttachmentReview &&
          typeof root.releaseEvidenceAttachmentReview === "object"
        ? (root.releaseEvidenceAttachmentReview as Record<string, unknown>)
        : root;
  const reviewId = stringValue(review.review_id, review.reviewId);
  const items = arrayValue(review.items)
    .map((item): FlowReleaseEvidenceAttachmentReviewItem | null => {
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
        vaultEntryId: stringValue(record.vault_entry_id, record.vaultEntryId),
        label: stringValue(record.label),
        kind: releaseCheckpointEvidenceVaultEntryKind(stringValue(record.kind)),
        path: stringValue(record.path),
        state: releaseEvidenceAttachmentState(stringValue(record.state)),
        required: booleanValue(record.required),
        present: booleanValue(record.present),
        attachable: booleanValue(record.attachable),
        bytes: numberValue(record.bytes),
        sha256: stringValue(record.sha256) || null,
        sourceId: stringValue(record.source_id, record.sourceId),
        releaseGateBlocking: booleanValue(
          record.release_gate_blocking,
          record.releaseGateBlocking,
        ),
        summary: stringValue(record.summary),
        nextAction: stringValue(record.next_action, record.nextAction),
      };
    })
    .filter((item): item is FlowReleaseEvidenceAttachmentReviewItem => item !== null);

  if (!reviewId && items.length === 0) {
    return null;
  }

  return {
    reviewId,
    reviewJson: stringValue(review.review_json, review.reviewJson),
    generatedAtUnixMs: stringValue(review.generated_at_unix_ms, review.generatedAtUnixMs),
    productName: stringValue(review.product_name, review.productName),
    localOnly: booleanValue(review.local_only, review.localOnly),
    status: panelStatus(stringValue(review.status)),
    readyForHandoff: booleanValue(review.ready_for_handoff, review.readyForHandoff),
    vaultId: stringValue(review.vault_id, review.vaultId),
    vaultJson: stringValue(review.vault_json, review.vaultJson),
    manifestSha256: stringValue(review.manifest_sha256, review.manifestSha256),
    itemCount: numberValue(review.item_count, review.itemCount),
    attachableCount: numberValue(review.attachable_count, review.attachableCount),
    missingCount: numberValue(review.missing_count, review.missingCount),
    inlineOnlyCount: numberValue(review.inline_only_count, review.inlineOnlyCount),
    checksumMissingCount: numberValue(
      review.checksum_missing_count,
      review.checksumMissingCount,
    ),
    blockedCount: numberValue(review.blocked_count, review.blockedCount),
    releaseGateBlockingCount: numberValue(
      review.release_gate_blocking_count,
      review.releaseGateBlockingCount,
    ),
    firstBlocker: stringValue(review.first_blocker, review.firstBlocker) || null,
    items,
    handoffNotesCopy: stringValue(review.handoff_notes_copy, review.handoffNotesCopy),
    summary: stringValue(review.summary),
    commands: arrayValue(review.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseHandoffPacket(value: unknown): FlowReleaseHandoffPacket | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const packet =
    root.release_handoff_packet && typeof root.release_handoff_packet === "object"
      ? (root.release_handoff_packet as Record<string, unknown>)
      : root.releaseHandoffPacket && typeof root.releaseHandoffPacket === "object"
        ? (root.releaseHandoffPacket as Record<string, unknown>)
        : root;
  const packetId = stringValue(packet.packet_id, packet.packetId);
  const sections = arrayValue(packet.sections)
    .map((item): FlowReleaseHandoffPacketSection | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const section = item as Record<string, unknown>;
      const id = stringValue(section.id);
      if (!id) {
        return null;
      }
      return {
        id,
        kind: releaseHandoffPacketSectionKind(stringValue(section.kind)),
        title: stringValue(section.title),
        body: stringValue(section.body),
        path: stringValue(section.path),
        sourceId: stringValue(section.source_id, section.sourceId),
        required: booleanValue(section.required),
        included: booleanValue(section.included),
        checksum: stringValue(section.checksum) || null,
        nextAction: stringValue(section.next_action, section.nextAction),
      };
    })
    .filter((section): section is FlowReleaseHandoffPacketSection => section !== null);

  if (!packetId && sections.length === 0) {
    return null;
  }

  return {
    packetId,
    packetJson: stringValue(packet.packet_json, packet.packetJson),
    generatedAtUnixMs: stringValue(packet.generated_at_unix_ms, packet.generatedAtUnixMs),
    productName: stringValue(packet.product_name, packet.productName),
    localOnly: booleanValue(packet.local_only, packet.localOnly),
    status: panelStatus(stringValue(packet.status)),
    readyToSend: booleanValue(packet.ready_to_send, packet.readyToSend),
    attachmentReviewId: stringValue(packet.attachment_review_id, packet.attachmentReviewId),
    attachmentReviewJson: stringValue(
      packet.attachment_review_json,
      packet.attachmentReviewJson,
    ),
    manifestSha256: stringValue(packet.manifest_sha256, packet.manifestSha256),
    sectionCount: numberValue(packet.section_count, packet.sectionCount),
    includedCount: numberValue(packet.included_count, packet.includedCount),
    attachableFileCount: numberValue(
      packet.attachable_file_count,
      packet.attachableFileCount,
    ),
    inlineNoteCount: numberValue(packet.inline_note_count, packet.inlineNoteCount),
    unresolvedBlockerCount: numberValue(
      packet.unresolved_blocker_count,
      packet.unresolvedBlockerCount,
    ),
    checksumCount: numberValue(packet.checksum_count, packet.checksumCount),
    missingCount: numberValue(packet.missing_count, packet.missingCount),
    firstBlocker: stringValue(packet.first_blocker, packet.firstBlocker) || null,
    sections,
    fileChecklistCopy: stringValue(packet.file_checklist_copy, packet.fileChecklistCopy),
    handoffPacketCopy: stringValue(packet.handoff_packet_copy, packet.handoffPacketCopy),
    summary: stringValue(packet.summary),
    commands: arrayValue(packet.commands)
      .map((command) => stringValue(command))
      .filter(Boolean),
  };
}

export function normalizeReleaseHandoffAuditTrail(
  value: unknown,
): FlowReleaseHandoffAuditTrail | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const root = value as Record<string, unknown>;
  const trail =
    root.release_handoff_audit_trail && typeof root.release_handoff_audit_trail === "object"
      ? (root.release_handoff_audit_trail as Record<string, unknown>)
      : root.releaseHandoffAuditTrail && typeof root.releaseHandoffAuditTrail === "object"
        ? (root.releaseHandoffAuditTrail as Record<string, unknown>)
        : root;
  const trailId = stringValue(trail.trail_id, trail.trailId);
  const records = arrayValue(trail.records)
    .map((item): FlowReleaseHandoffAuditRecord | null => {
      if (!item || typeof item !== "object") {
        return null;
      }
      const record = item as Record<string, unknown>;
      const auditId = stringValue(record.audit_id, record.auditId);
      if (!auditId) {
        return null;
      }
      return {
        auditId,
        packetId: stringValue(record.packet_id, record.packetId),
        packetJson: stringValue(record.packet_json, record.packetJson),
        recordedAtUnixMs: stringValue(
          record.recorded_at_unix_ms,
          record.recordedAtUnixMs,
        ),
        productName: stringValue(record.product_name, record.productName),
        localOnly: booleanValue(record.local_only, record.localOnly),
        state: releaseHandoffAuditState(stringValue(record.state)),
        operator: stringValue(record.operator),
        acknowledgementNote: stringValue(
          record.acknowledgement_note,
          record.acknowledgementNote,
        ),
        supersedesPacketId:
          stringValue(record.supersedes_packet_id, record.supersedesPacketId) || null,
        packetReadyToSend: booleanValue(
          record.packet_ready_to_send,
          record.packetReadyToSend,
        ),
        packetStatus: panelStatus(stringValue(record.packet_status, record.packetStatus)),
        packetSectionCount: numberValue(
          record.packet_section_count,
          record.packetSectionCount,
        ),
        attachableFileCount: numberValue(
          record.attachable_file_count,
          record.attachableFileCount,
        ),
        inlineNoteCount: numberValue(record.inline_note_count, record.inlineNoteCount),
        unresolvedBlockerCount: numberValue(
          record.unresolved_blocker_count,
          record.unresolvedBlockerCount,
        ),
        missingCount: numberValue(record.missing_count, record.missingCount),
        manifestSha256: stringValue(record.manifest_sha256, record.manifestSha256),
        active: booleanValue(record.active),
        blockerCarryover: numberValue(record.blocker_carryover, record.blockerCarryover),
        auditNotes: stringValue(record.audit_notes, record.auditNotes),
        summary: stringValue(record.summary),
      };
    })
    .filter((record): record is FlowReleaseHandoffAuditRecord => record !== null);

  if (!trailId && records.length === 0) {
    return null;
  }

  return {
    trailId,
    trailJson: stringValue(trail.trail_json, trail.trailJson),
    generatedAtUnixMs: stringValue(trail.generated_at_unix_ms, trail.generatedAtUnixMs),
    productName: stringValue(trail.product_name, trail.productName),
    localOnly: booleanValue(trail.local_only, trail.localOnly),
    recordCount: numberValue(trail.record_count, trail.recordCount),
    draftCount: numberValue(trail.draft_count, trail.draftCount),
    readyCount: numberValue(trail.ready_count, trail.readyCount),
    sentCount: numberValue(trail.sent_count, trail.sentCount),
    supersededCount: numberValue(trail.superseded_count, trail.supersededCount),
    revokedCount: numberValue(trail.revoked_count, trail.revokedCount),
    blockedCount: numberValue(trail.blocked_count, trail.blockedCount),
    activeAuditId: stringValue(trail.active_audit_id, trail.activeAuditId) || null,
    activePacketId: stringValue(trail.active_packet_id, trail.activePacketId) || null,
    latestAuditId: stringValue(trail.latest_audit_id, trail.latestAuditId) || null,
    latestPacketId: stringValue(trail.latest_packet_id, trail.latestPacketId) || null,
    latestState: trail.latest_state || trail.latestState
      ? releaseHandoffAuditState(stringValue(trail.latest_state, trail.latestState))
      : null,
    latestReadyToSend: booleanValue(trail.latest_ready_to_send, trail.latestReadyToSend),
    unresolvedBlockerCount: numberValue(
      trail.unresolved_blocker_count,
      trail.unresolvedBlockerCount,
    ),
    blockerCarryoverCount: numberValue(
      trail.blocker_carryover_count,
      trail.blockerCarryoverCount,
    ),
    acknowledgementCount: numberValue(
      trail.acknowledgement_count,
      trail.acknowledgementCount,
    ),
    records,
    auditSummaryCopy: stringValue(trail.audit_summary_copy, trail.auditSummaryCopy),
    summary: stringValue(trail.summary),
    commands: arrayValue(trail.commands)
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

function recoveryRunbookPhaseKind(value: string): FlowReleaseRecoveryRunbookPhaseKind {
  if (
    value === "pause" ||
    value === "diagnose" ||
    value === "rollback" ||
    value === "verify" ||
    value === "resume" ||
    value === "follow-up"
  ) {
    return value;
  }
  return "diagnose";
}

function recoveryRunbookPhaseStatus(value: string): FlowReleaseRecoveryRunbookPhaseStatus {
  if (value === "ready" || value === "requires-approval" || value === "blocked") {
    return value;
  }
  return "blocked";
}

function releaseIncidentSeverity(value: string): FlowReleaseIncidentSeverity {
  if (value === "info" || value === "watch" || value === "blocking" || value === "critical") {
    return value;
  }
  return "watch";
}

function releaseIncidentOutcome(value: string): FlowReleaseIncidentOutcome {
  if (
    value === "open" ||
    value === "monitoring" ||
    value === "resolved" ||
    value === "rolled-back" ||
    value === "prevented"
  ) {
    return value;
  }
  return "open";
}

function releasePreventionFindingKind(value: string): FlowReleasePreventionFindingKind {
  if (
    value === "critical-incident" ||
    value === "repeated-failure-class" ||
    value === "stale-evidence" ||
    value === "missing-evidence" ||
    value === "missing-incident-note" ||
    value === "rollback-gap" ||
    value === "stability-gate"
  ) {
    return value;
  }
  return "stability-gate";
}

function releasePreventionActionKind(value: string): FlowReleasePreventionActionKind {
  if (
    value === "refresh-evidence" ||
    value === "harden-rollback" ||
    value === "attach-incident-note" ||
    value === "resolve-recurrence" ||
    value === "review-release-gate"
  ) {
    return value;
  }
  return "review-release-gate";
}

function releasePreventionActionStatus(value: string): FlowReleasePreventionActionStatus {
  if (value === "owner-ready" || value === "needs-evidence" || value === "blocked") {
    return value;
  }
  return "blocked";
}

function releaseOwnerFollowUpCompletionState(
  value: string,
): FlowReleaseOwnerFollowUpCompletionState {
  if (
    value === "ready" ||
    value === "needs-evidence" ||
    value === "blocked" ||
    value === "complete" ||
    value === "overdue"
  ) {
    return value;
  }
  return "blocked";
}

function releaseOwnerFollowUpEvidenceState(value: string): FlowReleaseOwnerFollowUpEvidenceState {
  if (value === "present" || value === "missing" || value === "not-required") {
    return value;
  }
  return "missing";
}

function releaseEvidenceSlaState(value: string): FlowReleaseEvidenceSlaState {
  if (
    value === "fresh" ||
    value === "due-soon" ||
    value === "overdue" ||
    value === "missing" ||
    value === "blocked" ||
    value === "acknowledged"
  ) {
    return value;
  }
  return "missing";
}

function releaseEvidenceEscalationLevel(value: string): FlowReleaseEvidenceEscalationLevel {
  if (
    value === "none" ||
    value === "owner" ||
    value === "release-gate" ||
    value === "checkpoint"
  ) {
    return value;
  }
  return "none";
}

function releaseEvidenceRequirementSource(value: string): FlowReleaseEvidenceRequirementSource {
  if (
    value === "owner-follow-up" ||
    value === "prevention-plan" ||
    value === "stability-board"
  ) {
    return value;
  }
  return "owner-follow-up";
}

function releaseEscalationOwnerResponse(value: string): FlowReleaseEscalationOwnerResponse {
  if (
    value === "pending" ||
    value === "acknowledged" ||
    value === "resolved" ||
    value === "rejected" ||
    value === "carried-over"
  ) {
    return value;
  }
  return "pending";
}

function releaseEscalationGateOutcome(value: string): FlowReleaseEscalationGateOutcome {
  if (
    value === "blocked" ||
    value === "carry-over" ||
    value === "cleared" ||
    value === "monitoring"
  ) {
    return value;
  }
  return "carry-over";
}

function releaseCheckpointDecision(value: string): FlowReleaseCheckpointDecision {
  if (
    value === "ready" ||
    value === "hold" ||
    value === "carry-over" ||
    value === "needs-review"
  ) {
    return value;
  }
  return "needs-review";
}

function releaseCheckpointReviewState(value: string): FlowReleaseCheckpointReviewState {
  if (
    value === "ready" ||
    value === "hold" ||
    value === "carry-over" ||
    value === "review-required"
  ) {
    return value;
  }
  return "review-required";
}

function releaseCheckpointReviewSource(value: string): FlowReleaseCheckpointReviewSource {
  if (
    value === "escalation-ledger" ||
    value === "sla-monitor" ||
    value === "owner-follow-up" ||
    value === "prevention-plan" ||
    value === "stability-board" ||
    value === "missing-evidence"
  ) {
    return value;
  }
  return "missing-evidence";
}

function releaseCheckpointSignoffDecision(
  value: string,
): FlowReleaseCheckpointSignoffDecision {
  if (
    value === "signed-off" ||
    value === "held" ||
    value === "carried-over" ||
    value === "superseded" ||
    value === "revoked"
  ) {
    return value;
  }
  return "held";
}

function releaseCheckpointEvidenceVaultEntryKind(
  value: string,
): FlowReleaseCheckpointEvidenceVaultEntryKind {
  if (
    value === "checkpoint-review-json" ||
    value === "checkpoint-signoff-ledger-json" ||
    value === "acknowledgement-evidence" ||
    value === "carryover-commitment" ||
    value === "release-notes"
  ) {
    return value;
  }
  return "acknowledgement-evidence";
}

function releaseEvidenceAttachmentState(value: string): FlowReleaseEvidenceAttachmentState {
  if (
    value === "ready" ||
    value === "missing" ||
    value === "inline-only" ||
    value === "checksum-missing" ||
    value === "blocked"
  ) {
    return value;
  }
  return "missing";
}

function releaseHandoffPacketSectionKind(value: string): FlowReleaseHandoffPacketSectionKind {
  if (
    value === "operator-summary" ||
    value === "attachable-file" ||
    value === "inline-note" ||
    value === "unresolved-blocker" ||
    value === "manifest-checksum"
  ) {
    return value;
  }
  return "unresolved-blocker";
}

function releaseHandoffAuditState(value: string): FlowReleaseHandoffAuditState {
  if (
    value === "draft" ||
    value === "ready" ||
    value === "sent" ||
    value === "superseded" ||
    value === "revoked" ||
    value === "blocked"
  ) {
    return value;
  }
  return "draft";
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
