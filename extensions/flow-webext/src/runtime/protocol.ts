export type BrowserFlavor = "chromium" | "firefox" | "safari";
export type FlowSurface = "popup" | "options" | "sidepanel" | "sidebar";

export type FlowTask =
  | "rewrite-selection"
  | "summarize-page"
  | "compose-draft"
  | "explain-page"
  | "ocr-image"
  | "multimodal-ask";

export type FlowModality = "chat" | "ocr" | "vision-language";

export type FlowDeviceTarget = "webgpu" | "wasm";
export type FlowStorageBackend = "opfs" | "indexeddb" | "extension-storage";
export type FlowWorkspaceSection =
  | "overview"
  | "dashboard"
  | "workspace"
  | "packs"
  | "settings"
  | "delivery";

export interface BrowserCapabilityProfile {
  flavor: BrowserFlavor;
  webgpu: boolean;
  wasmThreads: boolean;
  crossOriginIsolated: boolean;
  opfs: boolean;
  indexeddb: boolean;
  sidePanel: boolean;
  sidebarAction: boolean;
  offscreenDocument: boolean;
  backgroundServiceWorker: boolean;
  notes: string[];
}

export interface BrowserPackFile {
  path: string;
  sourceUrl: string;
  purpose: string;
  contentType?: string;
  sha256?: string | null;
  sizeBytes?: number | null;
}

export interface BrowserPackManifest {
  version: number;
  packKey: string;
  modelKey: string;
  displayName: string;
  repoId: string;
  modality: FlowModality;
  backend: "transformersjs-onnx" | "webllm-mlc";
  quantization?: string | null;
  requiresWebgpu: boolean;
  files: BrowserPackFile[];
}

export type BrowserPackInstallState = "missing" | "partial" | "ready" | "corrupt";

export interface BrowserPackStatus {
  packKey: string;
  modelKey: string;
  displayName: string;
  state: BrowserPackInstallState;
  filesReady: number;
  filesTotal: number;
  storageBackend: FlowStorageBackend;
  lastUpdatedAt: number | null;
  lastError: string | null;
}

export interface FlowExecutionPlan {
  task: FlowTask;
  modality: FlowModality;
  selectedModel: string | null;
  packKey: string | null;
  storageBackend: FlowStorageBackend;
  deviceTarget: FlowDeviceTarget;
  localOnly: boolean;
  remoteAllowed: boolean;
  reasons: string[];
  unsupportedReason: string | null;
}

export interface FlowInferenceRequest {
  task: FlowTask;
  modality: FlowModality;
  prompt: string;
  selectionText?: string;
  pageText?: string;
  imageSources: string[];
  localOnly: boolean;
  preferredModel?: string;
}

export interface FlowUiSettings {
  localOnly: boolean;
  defaultTask: FlowTask;
  autoApplyRewrite: boolean;
  captureActiveTabContext: boolean;
  preferredChatModel: string;
  preferredOcrModel: string;
  preferredVisionLanguageModel: string;
}

export interface FlowWorkbenchDraft {
  task: FlowTask;
  prompt: string;
  imageSources: string;
}

export interface FlowRuntimeReadiness {
  capabilities: BrowserCapabilityProfile;
  storageBackend: FlowStorageBackend;
  packStatuses: BrowserPackStatus[];
  localOnly: boolean;
  textReady: boolean;
  ocrReady: boolean;
  multimodalReady: boolean;
  recommendedNextStep: string;
  notes: string[];
}

export interface QuickContextPayload {
  selectionText: string;
  pageText: string;
  title: string;
  url: string;
}

export type FlowDashboardPanelStatus = "ready" | "warning" | "blocked";
export type FlowDashboardActionKind = "open" | "run-check" | "recover" | "capture";

export interface FlowDashboardProductUiButtonState {
  disabled: boolean;
  disabledReason: string | null;
  idleLabel: string;
  loadingLabel: string;
  successLabel: string;
  errorLabel: string;
  ariaLabel: string;
  destructive: boolean;
  requiresConfirmation: boolean;
}

export interface FlowDashboardProductUiActionBinding {
  actionId: string;
  label: string;
  kind: FlowDashboardActionKind;
  command: string;
  localOnly: boolean;
  enabled: boolean;
  buttonState: FlowDashboardProductUiButtonState;
}

export interface FlowDashboardProductUiCardBinding {
  cardId: string;
  title: string;
  status: FlowDashboardPanelStatus;
  scoreOutOf100: number;
  primaryMetric: string;
  sourceJson: string;
  actionCount: number;
  actions: FlowDashboardProductUiActionBinding[];
}

export interface FlowDashboardProductUiHistoryBinding {
  recordCount: number;
  scoreDeltaFromPrevious: number;
  readinessDeltaFromPrevious: number;
  latestScoreOutOf100: number | null;
  previousScoreOutOf100: number | null;
  trendLabel: string;
}

export type FlowDashboardScreenshotStatus = "captured" | "missing" | "metadata-missing";

export interface FlowDashboardProductUiScreenshotPrompt {
  route: string;
  title: string;
  viewportId: string;
  status: FlowDashboardScreenshotStatus;
  prompt: string;
  captureCommand: string;
}

export interface FlowDashboardProductUiReleaseLink {
  id: string;
  label: string;
  kind: string;
  path: string;
  section: string;
  localOnly: boolean;
  buttonState: FlowDashboardProductUiButtonState;
}

export interface FlowDashboardProductUiBinding {
  productName: string;
  route: string;
  title: string;
  sourceFile: string;
  exportDir: string;
  status: FlowDashboardPanelStatus;
  scoreOutOf100: number;
  summary: string;
  panelJsonCommand: string;
  exportCommand: string;
  cardCount: number;
  boundCardCount: number;
  actionCount: number;
  warningCount: number;
  blockingCount: number;
  cards: FlowDashboardProductUiCardBinding[];
  history: FlowDashboardProductUiHistoryBinding;
  releaseLinks: FlowDashboardProductUiReleaseLink[];
  screenshotPrompts: FlowDashboardProductUiScreenshotPrompt[];
  nextActions: string[];
}
