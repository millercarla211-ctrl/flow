import { requestQuickContext, replaceSelection, toggleOverlay } from "../runtime/browser-api";
import {
  buildTrustedHostRunnerCancellationUx,
  dispatchDashboardCommand,
  normalizeReleaseCandidateArchive,
  normalizeReleaseCheckpointEvidenceVault,
  normalizeReleaseCheckpointReview,
  normalizeReleaseCheckpointSignoffLedger,
  normalizeReleaseDeploymentGate,
  normalizeReleaseEscalationLedger,
  normalizeReleaseEvidenceAttachmentReview,
  normalizeReleaseEvidenceSlaMonitor,
  normalizeReleaseEvidenceExportKit,
  normalizeReleaseExternalReceiptArchive,
  normalizeReleaseClosureLedger,
  normalizeReleaseReceiptReviewBoard,
  normalizeReleaseHandoffAuditTrail,
  normalizeReleaseHandoffDispatchAuditTrail,
  normalizeReleaseHandoffDispatchChecklist,
  normalizeReleaseHandoffCompletionLedger,
  normalizeReleaseHandoffDispatchGovernanceReview,
  normalizeReleaseHandoffGovernanceReview,
  normalizeReleaseHandoffPacket,
  normalizeReleaseIncidentArchive,
  normalizeReleaseOperatorChecklist,
  normalizeReleaseOwnerFollowUpBoard,
  normalizeReleaseOutboundReviewLedger,
  normalizeReleasePostPromotionMonitor,
  normalizeReleasePreventionPlan,
  normalizeReleasePromotionLedger,
  normalizeReleasePublicationControl,
  normalizeReleaseQaCommandCenter,
  normalizeReleaseRecoveryRunbook,
  normalizeReleaseRollbackDrill,
  normalizeReleaseStabilityBoard,
  normalizeDashboardHostCommandResults,
  normalizeTrustedHostLiveRunnerState,
  normalizeTrustedHostRunnerCancellationUx,
  normalizeTrustedHostRunnerOperatorReview,
  normalizeTrustedHostRunnerApprovalUi,
  normalizeTrustedHostRunnerResults,
  normalizeTrustedHostRunnerUx,
  normalizeTrustedRunnerReleasePackage,
  normalizeTrustedRunnerReleaseTimeline,
  persistDashboardCommandResult,
  readDashboardCommandResults,
  type FlowDashboardCommandResult,
  type FlowDashboardLiveRunnerState,
  type FlowDashboardRunnerApprovalControl,
  type FlowDashboardRunnerApprovalUiReport,
  type FlowDashboardRunnerCancellationControl,
  type FlowDashboardRunnerCancellationUxReport,
  type FlowDashboardRunnerOperatorReviewReport,
  type FlowDashboardRunnerReleasePackageReport,
  type FlowDashboardRunnerReleaseTimeline,
  type FlowDashboardRunnerUxReport,
  type FlowDashboardCommandStatus,
  type FlowReleaseCandidateArchive,
  type FlowReleaseCheckpointEvidenceVault,
  type FlowReleaseCheckpointReviewBoardReport,
  type FlowReleaseCheckpointSignoffLedger,
  type FlowReleaseDeploymentGateReport,
  type FlowReleaseEscalationLedger,
  type FlowReleaseEvidenceAttachmentReview,
  type FlowReleaseEvidenceExportKitReport,
  type FlowReleaseEvidenceSlaMonitorReport,
  type FlowReleaseExternalReceiptArchive,
  type FlowReleaseClosureLedger,
  type FlowReleaseReceiptReviewBoardReport,
  type FlowReleaseHandoffAuditTrail,
  type FlowReleaseHandoffDispatchAuditTrail,
  type FlowReleaseHandoffDispatchChecklist,
  type FlowReleaseHandoffCompletionLedger,
  type FlowReleaseHandoffDispatchGovernanceReview,
  type FlowReleaseHandoffGovernanceReview,
  type FlowReleaseHandoffPacket,
  type FlowReleaseIncidentArchive,
  type FlowReleaseOperatorChecklistReport,
  type FlowReleaseOwnerFollowUpBoardReport,
  type FlowReleaseOutboundReviewLedger,
  type FlowReleasePostPromotionMonitorReport,
  type FlowReleasePreventionPlanReport,
  type FlowReleasePromotionLedger,
  type FlowReleasePublicationControl,
  type FlowReleaseQaCommandCenterReport,
  type FlowReleaseRecoveryRunbookReport,
  type FlowReleaseRollbackDrillReport,
  type FlowReleaseStabilityBoardReport,
} from "../runtime/dashboard-actions";
import { normalizeFridayDashboardBinding } from "../runtime/dashboard-binding";
import { FlowBrowserEngine } from "../runtime/flow-engine";
import type {
  BrowserPackManifest,
  BrowserPackStatus,
  FlowDashboardProductUiBinding,
  FlowDashboardProductUiCardBinding,
  FlowExecutionPlan,
  FlowInferenceRequest,
  FlowRuntimeReadiness,
  FlowSurface,
  FlowTask,
  FlowUiSettings,
  FlowWorkbenchDraft,
  FlowWorkspaceSection,
  QuickContextPayload,
} from "../runtime/protocol";

type UiState = {
  settings: FlowUiSettings;
  draft: FlowWorkbenchDraft;
  readiness: FlowRuntimeReadiness;
  quickContext: QuickContextPayload | null;
  dashboardBinding: FlowDashboardProductUiBinding;
  activeSection: FlowWorkspaceSection;
  status: string;
  output: string;
  running: boolean;
  lastPlan: FlowExecutionPlan | null;
  lastPack: BrowserPackManifest | null;
  dashboardActionStates: Record<string, "idle" | "loading" | "success" | "error" | "blocked">;
  dashboardActionResults: FlowDashboardCommandResult[];
  dashboardRunnerUx: FlowDashboardRunnerUxReport | null;
  dashboardRunnerApprovalUi: FlowDashboardRunnerApprovalUiReport | null;
  dashboardRunnerApprovalReason: string;
  dashboardLiveRunnerState: FlowDashboardLiveRunnerState | null;
  dashboardRunnerCancellationUx: FlowDashboardRunnerCancellationUxReport | null;
  dashboardRunnerCancellationDrafts: Record<string, string>;
  dashboardRunnerOperatorReview: FlowDashboardRunnerOperatorReviewReport | null;
  dashboardRunnerReleasePackage: FlowDashboardRunnerReleasePackageReport | null;
  dashboardRunnerReleaseTimeline: FlowDashboardRunnerReleaseTimeline | null;
  dashboardReleaseChecklist: FlowReleaseOperatorChecklistReport | null;
  dashboardReleaseChecklistReason: string;
  dashboardReleaseQa: FlowReleaseQaCommandCenterReport | null;
  dashboardReleaseExportKit: FlowReleaseEvidenceExportKitReport | null;
  dashboardReleaseDeploymentGate: FlowReleaseDeploymentGateReport | null;
  dashboardReleaseCandidateArchive: FlowReleaseCandidateArchive | null;
  dashboardReleasePromotionLedger: FlowReleasePromotionLedger | null;
  dashboardReleasePostPromotionMonitor: FlowReleasePostPromotionMonitorReport | null;
  dashboardReleaseRollbackDrill: FlowReleaseRollbackDrillReport | null;
  dashboardReleaseStabilityBoard: FlowReleaseStabilityBoardReport | null;
  dashboardReleaseRecoveryRunbook: FlowReleaseRecoveryRunbookReport | null;
  dashboardReleaseIncidentArchive: FlowReleaseIncidentArchive | null;
  dashboardReleasePreventionPlan: FlowReleasePreventionPlanReport | null;
  dashboardReleaseOwnerFollowUpBoard: FlowReleaseOwnerFollowUpBoardReport | null;
  dashboardReleaseEvidenceSlaMonitor: FlowReleaseEvidenceSlaMonitorReport | null;
  dashboardReleaseEscalationLedger: FlowReleaseEscalationLedger | null;
  dashboardReleaseCheckpointReview: FlowReleaseCheckpointReviewBoardReport | null;
  dashboardReleaseCheckpointSignoffLedger: FlowReleaseCheckpointSignoffLedger | null;
  dashboardReleaseCheckpointEvidenceVault: FlowReleaseCheckpointEvidenceVault | null;
  dashboardReleaseEvidenceAttachmentReview: FlowReleaseEvidenceAttachmentReview | null;
  dashboardReleaseHandoffPacket: FlowReleaseHandoffPacket | null;
  dashboardReleaseHandoffAuditTrail: FlowReleaseHandoffAuditTrail | null;
  dashboardReleaseHandoffGovernanceReview: FlowReleaseHandoffGovernanceReview | null;
  dashboardReleaseHandoffDispatchChecklist: FlowReleaseHandoffDispatchChecklist | null;
  dashboardReleaseHandoffDispatchAuditTrail: FlowReleaseHandoffDispatchAuditTrail | null;
  dashboardReleaseHandoffDispatchGovernanceReview: FlowReleaseHandoffDispatchGovernanceReview | null;
  dashboardReleaseHandoffCompletionLedger: FlowReleaseHandoffCompletionLedger | null;
  dashboardReleasePublicationControl: FlowReleasePublicationControl | null;
  dashboardReleaseOutboundReviewLedger: FlowReleaseOutboundReviewLedger | null;
  dashboardReleaseExternalReceiptArchive: FlowReleaseExternalReceiptArchive | null;
  dashboardReleaseReceiptReviewBoard: FlowReleaseReceiptReviewBoardReport | null;
  dashboardReleaseClosureLedger: FlowReleaseClosureLedger | null;
};

const RUNNER_CANCELLATION_DRAFT_KEY = "flow.dashboard.runnerCancellationDrafts";

function readRunnerCancellationDrafts(): Record<string, string> {
  try {
    const raw = globalThis.localStorage?.getItem(RUNNER_CANCELLATION_DRAFT_KEY);
    if (!raw) {
      return {};
    }
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return {};
    }
    return Object.fromEntries(
      Object.entries(parsed as Record<string, unknown>).filter(
        (entry): entry is [string, string] => typeof entry[1] === "string",
      ),
    );
  } catch {
    return {};
  }
}

function writeRunnerCancellationDrafts(drafts: Record<string, string>) {
  try {
    globalThis.localStorage?.setItem(RUNNER_CANCELLATION_DRAFT_KEY, JSON.stringify(drafts));
  } catch {
    // Draft persistence is a dashboard convenience; trusted execution still requires local commands.
  }
}

const TASK_OPTIONS: Array<{ task: FlowTask; label: string; detail: string }> = [
  {
    task: "rewrite-selection",
    label: "Rewrite selection",
    detail: "Tighten highlighted text and optionally apply it back into the page.",
  },
  {
    task: "summarize-page",
    label: "Summarize page",
    detail: "Turn the active tab into key points, risks, and next steps.",
  },
  {
    task: "compose-draft",
    label: "Compose draft",
    detail: "Write an email, reply, or post using the current page context.",
  },
  {
    task: "explain-page",
    label: "Explain page",
    detail: "Explain the current page in plain language for the user.",
  },
  {
    task: "ocr-image",
    label: "OCR image",
    detail: "Extract readable text from an image URL or screenshot asset.",
  },
  {
    task: "multimodal-ask",
    label: "Multimodal ask",
    detail: "Ask about an image or document when a WebGPU browser is available.",
  },
];

const SECTION_LABELS: Record<FlowWorkspaceSection, string> = {
  overview: "Overview",
  dashboard: "Dashboard",
  workspace: "Workbench",
  packs: "Model Packs",
  settings: "Settings",
  delivery: "Delivery",
};

const SURFACE_COPY: Record<
  FlowSurface,
  { title: string; eyebrow: string; intro: string; sections: FlowWorkspaceSection[] }
> = {
  popup: {
    title: "Flow Quick Panel",
    eyebrow: "Fast local actions",
    intro:
      "Handle rewrites, page summaries, and draft replies directly inside the browser with local models.",
    sections: ["overview", "dashboard", "workspace", "packs"],
  },
  sidepanel: {
    title: "Flow Side Panel",
    eyebrow: "Persistent browser workspace",
    intro:
      "Keep the full local workbench open while researching, drafting, and applying edits back to production apps.",
    sections: ["overview", "dashboard", "workspace", "packs", "settings", "delivery"],
  },
  sidebar: {
    title: "Flow Sidebar",
    eyebrow: "Firefox local workspace",
    intro:
      "Use the same local-first Flow runtime in Firefox with pack management, workbench tools, and delivery checks.",
    sections: ["overview", "dashboard", "workspace", "packs", "settings", "delivery"],
  },
  options: {
    title: "Flow Setup Console",
    eyebrow: "Client handoff controls",
    intro:
      "Configure local-only behavior, verify model packs, and review the project state before handing the build to the client.",
    sections: ["overview", "dashboard", "packs", "settings", "delivery"],
  },
};

function normalizeSurface(surface: string): FlowSurface {
  switch (surface) {
    case "options":
    case "sidepanel":
    case "sidebar":
      return surface;
    default:
      return "popup";
  }
}

function defaultSection(surface: FlowSurface): FlowWorkspaceSection {
  switch (surface) {
    case "popup":
      return "overview";
    case "options":
      return "settings";
    default:
      return "workspace";
  }
}

function badgeTone(value: string) {
  if (value === "ready" || value === "on" || value === "enabled") {
    return "good";
  }
  if (value === "partial" || value === "pending" || value === "optional" || value === "warning") {
    return "warn";
  }
  if (value === "corrupt" || value === "blocked" || value === "off") {
    return "bad";
  }
  return "muted";
}

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function clipText(value: string, max = 180) {
  if (!value.trim()) {
    return "";
  }

  return value.length > max ? `${value.slice(0, max - 1)}…` : value;
}

function taskOption(task: FlowTask) {
  return TASK_OPTIONS.find((item) => item.task === task) ?? TASK_OPTIONS[0];
}

function packStatusFor(
  readiness: FlowRuntimeReadiness,
  modelKey: string,
): BrowserPackStatus | null {
  return readiness.packStatuses.find((status) => status.modelKey === modelKey) ?? null;
}

function packActionLabel(status: BrowserPackStatus) {
  switch (status.state) {
    case "partial":
      return "Resume download";
    case "corrupt":
      return "Repair pack";
    case "missing":
      return "Install pack";
    default:
      return "Verify pack";
  }
}

function packSummaryText(status: BrowserPackStatus) {
  switch (status.state) {
    case "ready":
      return "Cached and verified for offline local use.";
    case "partial":
      return "Some files are cached. Resume the download to finish verification.";
    case "corrupt":
      return "At least one cached file failed integrity checks. Repair this pack before use.";
    default:
      return "This pack is not cached yet.";
  }
}

function renderPackCard(
  pack: BrowserPackManifest,
  status: BrowserPackStatus,
  readiness: FlowRuntimeReadiness,
) {
  const isRequired = pack.modelKey === "qwen3-0.6b";
  const needsWebgpu = pack.requiresWebgpu;
  const blocked = needsWebgpu && !readiness.capabilities.webgpu;

  return `
    <article class="feature-card pack-card ${status.state}">
      <div class="card-topline">
        <span class="eyebrow">${pack.modality}</span>
        <span class="badge ${badgeTone(status.state)}">${status.state}</span>
      </div>
      <h3>${escapeHtml(pack.displayName)}</h3>
      <p>${escapeHtml(packSummaryText(status))}</p>
      <div class="meta-list">
        <span><strong>Model</strong> ${escapeHtml(pack.modelKey)}</span>
        <span><strong>Backend</strong> ${escapeHtml(pack.backend)}</span>
        <span><strong>Quant</strong> ${escapeHtml(pack.quantization ?? "default")}</span>
        <span><strong>Files</strong> ${status.filesReady}/${status.filesTotal}</span>
        <span><strong>Storage</strong> ${escapeHtml(status.storageBackend)}</span>
        <span><strong>Role</strong> ${isRequired ? "Required" : "Optional"}</span>
      </div>
      <div class="hint ${blocked ? "bad" : "muted"}">
        ${
          blocked
            ? "This pack is capability-gated because WebGPU is unavailable in the current browser."
            : needsWebgpu
              ? "This pack unlocks multimodal image and document reasoning on WebGPU browsers."
              : "This pack runs on the cross-browser local baseline without a remote dependency."
        }
      </div>
      <div class="actions">
        <button type="button" data-action="download-pack" data-model-key="${escapeHtml(pack.modelKey)}">
          ${escapeHtml(packActionLabel(status))}
        </button>
        <button
          type="button"
          class="secondary"
          data-action="remove-pack"
          data-model-key="${escapeHtml(pack.modelKey)}"
          ${status.filesReady === 0 ? "disabled" : ""}
        >
          Remove cached files
        </button>
      </div>
    </article>
  `;
}

function renderOverview(surface: FlowSurface, state: UiState, engine: FlowBrowserEngine) {
  const textStatus = packStatusFor(state.readiness, state.settings.preferredChatModel);
  const ocrStatus = packStatusFor(state.readiness, state.settings.preferredOcrModel);
  const multimodalStatus = packStatusFor(
    state.readiness,
    state.settings.preferredVisionLanguageModel,
  );

  return `
    <section class="section-stack">
      <div class="hero-card">
        <div class="card-topline">
          <span class="eyebrow">${escapeHtml(SURFACE_COPY[surface].eyebrow)}</span>
          <span class="badge ${badgeTone(state.settings.localOnly ? "on" : "optional")}">
            ${state.settings.localOnly ? "Local only" : "Remote optional"}
          </span>
        </div>
        <h2>${escapeHtml(SURFACE_COPY[surface].title)}</h2>
        <p>${escapeHtml(SURFACE_COPY[surface].intro)}</p>
        <div class="hero-facts">
          <span class="pill">${escapeHtml(state.readiness.capabilities.flavor)}</span>
          <span class="pill">${escapeHtml(state.readiness.storageBackend)}</span>
          <span class="pill">${state.readiness.capabilities.webgpu ? "WebGPU ready" : "WASM fallback"}</span>
          <span class="pill">${textStatus?.state === "ready" ? "Qwen3 ready" : "Qwen3 pending"}</span>
        </div>
      </div>

      <div class="card-grid metrics-grid">
        <article class="metric-card">
          <span class="eyebrow">Text runtime</span>
          <strong>${state.readiness.textReady ? "Ready" : "Install required"}</strong>
          <p>${escapeHtml(textStatus?.displayName ?? "Qwen3 0.6B Browser Pack")}</p>
        </article>
        <article class="metric-card">
          <span class="eyebrow">OCR runtime</span>
          <strong>${state.readiness.ocrReady ? "Ready" : "Optional"}</strong>
          <p>${escapeHtml(ocrStatus?.displayName ?? "TrOCR Small Printed Browser Pack")}</p>
        </article>
        <article class="metric-card">
          <span class="eyebrow">Multimodal</span>
          <strong>${
            state.readiness.capabilities.webgpu
              ? state.readiness.multimodalReady
                ? "Ready"
                : "Install optional"
              : "Capability gated"
          }</strong>
          <p>${escapeHtml(multimodalStatus?.displayName ?? "Qwen3.5 0.8B Browser Pack")}</p>
        </article>
        <article class="metric-card">
          <span class="eyebrow">Client handoff</span>
          <strong>${surface === "options" ? "Control surface" : "Ready for review"}</strong>
          <p>Firebase remains the only intended external wiring gap.</p>
        </article>
      </div>

      <article class="feature-card callout-card">
        <div class="card-topline">
          <span class="eyebrow">Next step</span>
          <span class="badge ${badgeTone("optional")}">Delivery focus</span>
        </div>
        <h3>What Flow still needs on this machine</h3>
        <p>${escapeHtml(state.readiness.recommendedNextStep)}</p>
        <div class="note-list">
          ${state.readiness.notes.map((note) => `<span>${escapeHtml(note)}</span>`).join("")}
        </div>
      </article>

      <div class="section-header">
        <div>
          <h3>Quick actions</h3>
          <p>Run the highest-value local tasks without leaving the page.</p>
        </div>
        <div class="actions">
          <button type="button" class="secondary" data-action="refresh-context">Refresh tab context</button>
          <button type="button" class="secondary" data-action="toggle-overlay">Toggle page overlay</button>
        </div>
      </div>
      <div class="card-grid action-grid">
        ${TASK_OPTIONS.slice(0, 4)
          .map(
            (item) => `
              <button
                type="button"
                class="quick-action"
                data-action="quick-run"
                data-task="${item.task}"
              >
                <strong>${escapeHtml(item.label)}</strong>
                <span>${escapeHtml(item.detail)}</span>
              </button>
            `,
          )
          .join("")}
      </div>

      <article class="feature-card">
        <div class="section-header">
          <div>
            <h3>Active tab context</h3>
            <p>Flow can use the active page, selection, and title to produce better local output.</p>
          </div>
          <button type="button" class="secondary" data-action="open-workspace">
            Open workbench
          </button>
        </div>
        ${
          state.quickContext
            ? `
              <div class="context-grid">
                <div class="context-card">
                  <span class="eyebrow">Title</span>
                  <strong>${escapeHtml(state.quickContext.title || "Untitled page")}</strong>
                  <p>${escapeHtml(clipText(state.quickContext.url, 90))}</p>
                </div>
                <div class="context-card">
                  <span class="eyebrow">Selection</span>
                  <p>${escapeHtml(clipText(state.quickContext.selectionText || "No text selected.", 160))}</p>
                </div>
                <div class="context-card context-wide">
                  <span class="eyebrow">Page excerpt</span>
                  <p>${escapeHtml(clipText(state.quickContext.pageText || "No page text captured.", 260))}</p>
                </div>
              </div>
            `
            : `
              <div class="empty-state">
                <strong>No active tab context cached yet.</strong>
                <p>Use Flow on a supported page, then refresh the tab context to pull the current selection and page text.</p>
              </div>
            `
        }
      </article>
    </section>
  `;
}

function renderWorkspace(state: UiState, engine: FlowBrowserEngine) {
  const plan = engine.planExecution(
    state.draft.task,
    engine.modalityForTask(state.draft.task),
    state.settings.localOnly,
    engine.preferredModelForTask(state.draft.task, state.settings),
  );
  const task = taskOption(state.draft.task);

  return `
    <section class="section-stack">
      <div class="section-header">
        <div>
          <h2>Local workbench</h2>
          <p>Prepare requests, run them locally, and apply results back into the page.</p>
        </div>
        <span class="badge ${badgeTone(plan.unsupportedReason ? "blocked" : "ready")}">
          ${plan.unsupportedReason ? "Blocked" : "Runnable"}
        </span>
      </div>

      <div class="card-grid workspace-grid">
        <article class="feature-card">
          <div class="grid two">
            <label>
              Task
              <select id="flow-task">
                ${TASK_OPTIONS.map(
                  (option) => `
                    <option value="${option.task}" ${option.task === state.draft.task ? "selected" : ""}>
                      ${escapeHtml(option.label)}
                    </option>
                  `,
                ).join("")}
              </select>
            </label>
            <label>
              Target model
              <select id="flow-model">
                ${engine
                  .packCatalog()
                  .filter((pack) => pack.modality === engine.modalityForTask(state.draft.task))
                  .map((pack) => {
                    const selectedModel = engine.preferredModelForTask(
                      state.draft.task,
                      state.settings,
                    );
                    return `
                      <option value="${escapeHtml(pack.modelKey)}" ${
                        pack.modelKey === selectedModel ? "selected" : ""
                      }>
                        ${escapeHtml(pack.displayName)}
                      </option>
                    `;
                  })
                  .join("")}
              </select>
            </label>
          </div>
          <div class="hint muted">${escapeHtml(task.detail)}</div>
          <label>
            Prompt
            <textarea id="flow-prompt">${escapeHtml(state.draft.prompt)}</textarea>
          </label>
          <label>
            Image URL(s)
            <input
              id="flow-images"
              type="text"
              value="${escapeHtml(state.draft.imageSources)}"
              placeholder="https://example.com/image.png, data:image/png;base64,..."
            />
          </label>
          <div class="actions">
            <button type="button" class="secondary" data-action="refresh-context">Refresh tab context</button>
            <button type="button" class="secondary" data-action="clear-context">Clear context</button>
            <button type="button" data-action="run-flow">${state.running ? "Running..." : "Run local flow"}</button>
          </div>
        </article>

        <article class="feature-card">
          <div class="card-topline">
            <span class="eyebrow">Execution plan</span>
            <span class="badge ${badgeTone(plan.unsupportedReason ? "blocked" : "ready")}">
              ${plan.deviceTarget}
            </span>
          </div>
          <h3>${escapeHtml(task.label)}</h3>
          <div class="meta-list">
            <span><strong>Model</strong> ${escapeHtml(plan.selectedModel ?? "none")}</span>
            <span><strong>Pack</strong> ${escapeHtml(plan.packKey ?? "none")}</span>
            <span><strong>Storage</strong> ${escapeHtml(plan.storageBackend)}</span>
            <span><strong>Remote</strong> ${plan.remoteAllowed ? "Allowed later" : "Disabled"}</span>
          </div>
          ${
            plan.unsupportedReason
              ? `<div class="hint bad">${escapeHtml(plan.unsupportedReason)}</div>`
              : `<div class="note-list">${plan.reasons
                  .map((reason) => `<span>${escapeHtml(reason)}</span>`)
                  .join("")}</div>`
          }
          <div class="divider"></div>
          <div class="status-card">
            <span class="eyebrow">Status</span>
            <pre id="flow-status">${escapeHtml(state.status)}</pre>
          </div>
        </article>
      </div>

      <article class="feature-card">
        <div class="section-header">
          <div>
            <h3>Captured context</h3>
            <p>Selection and page data stay local to the extension unless you later wire a remote backend.</p>
          </div>
        </div>
        ${
          state.quickContext
            ? `
              <div class="context-grid">
                <div class="context-card">
                  <span class="eyebrow">Title</span>
                  <strong>${escapeHtml(state.quickContext.title || "Untitled page")}</strong>
                  <p>${escapeHtml(clipText(state.quickContext.url, 90))}</p>
                </div>
                <div class="context-card">
                  <span class="eyebrow">Selection</span>
                  <p>${escapeHtml(clipText(state.quickContext.selectionText || "No selection captured.", 180))}</p>
                </div>
                <div class="context-card context-wide">
                  <span class="eyebrow">Page excerpt</span>
                  <p>${escapeHtml(clipText(state.quickContext.pageText || "No page text captured.", 260))}</p>
                </div>
              </div>
            `
            : `
              <div class="empty-state">
                <strong>No active tab context cached.</strong>
                <p>Refresh the tab context to pull the current selection and page text before running a local task.</p>
              </div>
            `
        }
      </article>

      <article class="feature-card">
        <div class="section-header">
          <div>
            <h3>Output</h3>
            <p>Copy the result, push a rewrite back into the page, or keep refining the prompt.</p>
          </div>
          <div class="actions">
            <button type="button" class="secondary" data-action="copy-output" ${
              !state.output.trim() ? "disabled" : ""
            }>Copy</button>
            <button type="button" class="secondary" data-action="apply-output" ${
              !state.output.trim() ? "disabled" : ""
            }>Apply to page</button>
            <button type="button" class="secondary" data-action="clear-output" ${
              !state.output.trim() ? "disabled" : ""
            }>Clear</button>
          </div>
        </div>
        <div class="output-shell">
          <pre id="flow-output">${escapeHtml(state.output || "No local output yet.")}</pre>
        </div>
        ${
          state.lastPlan && state.lastPack
            ? `
              <div class="meta-list">
                <span><strong>Last model</strong> ${escapeHtml(state.lastPlan.selectedModel ?? "none")}</span>
                <span><strong>Last pack</strong> ${escapeHtml(state.lastPack.displayName)}</span>
                <span><strong>Target</strong> ${escapeHtml(state.lastPlan.deviceTarget)}</span>
              </div>
            `
            : ""
        }
      </article>
    </section>
  `;
}

function renderSettings(state: UiState, engine: FlowBrowserEngine) {
  const chatPacks = engine.packCatalog().filter((pack) => pack.modality === "chat");
  const ocrPacks = engine.packCatalog().filter((pack) => pack.modality === "ocr");
  const vlmPacks = engine
    .packCatalog()
    .filter((pack) => pack.modality === "vision-language");

  return `
    <section class="section-stack">
      <div class="section-header">
        <div>
          <h2>Behavior and delivery settings</h2>
          <p>Keep the browser experience local-first and ready for the client handoff.</p>
        </div>
      </div>

      <article class="feature-card">
        <div class="toggle-list">
          <label class="toggle-row">
            <input id="setting-local-only" type="checkbox" ${
              state.settings.localOnly ? "checked" : ""
            } />
            <span>
              <strong>Local-only mode</strong>
              <small>Prevent silent remote fallback. Firebase can wire remote providers later.</small>
            </span>
          </label>
          <label class="toggle-row">
            <input id="setting-auto-apply" type="checkbox" ${
              state.settings.autoApplyRewrite ? "checked" : ""
            } />
            <span>
              <strong>Auto-apply rewrites</strong>
              <small>When rewrite selection succeeds, push the updated text back into the active page.</small>
            </span>
          </label>
          <label class="toggle-row">
            <input id="setting-capture-context" type="checkbox" ${
              state.settings.captureActiveTabContext ? "checked" : ""
            } />
            <span>
              <strong>Capture active tab context before runs</strong>
              <small>Refresh title, selection, and page text automatically before local execution.</small>
            </span>
          </label>
        </div>
      </article>

      <article class="feature-card">
        <div class="grid two">
          <label>
            Default task
            <select id="setting-default-task">
              ${TASK_OPTIONS.map(
                (option) => `
                  <option value="${option.task}" ${
                    option.task === state.settings.defaultTask ? "selected" : ""
                  }>
                    ${escapeHtml(option.label)}
                  </option>
                `,
              ).join("")}
            </select>
          </label>
          <label>
            Preferred text model
            <select id="setting-chat-model">
              ${chatPacks.map(
                (pack) => `
                  <option value="${escapeHtml(pack.modelKey)}" ${
                    pack.modelKey === state.settings.preferredChatModel ? "selected" : ""
                  }>
                    ${escapeHtml(pack.displayName)}
                  </option>
                `,
              ).join("")}
            </select>
          </label>
          <label>
            Preferred OCR model
            <select id="setting-ocr-model">
              ${ocrPacks.map(
                (pack) => `
                  <option value="${escapeHtml(pack.modelKey)}" ${
                    pack.modelKey === state.settings.preferredOcrModel ? "selected" : ""
                  }>
                    ${escapeHtml(pack.displayName)}
                  </option>
                `,
              ).join("")}
            </select>
          </label>
          <label>
            Preferred multimodal model
            <select id="setting-vlm-model">
              ${vlmPacks.map(
                (pack) => `
                  <option value="${escapeHtml(pack.modelKey)}" ${
                    pack.modelKey === state.settings.preferredVisionLanguageModel
                      ? "selected"
                      : ""
                  }>
                    ${escapeHtml(pack.displayName)}
                  </option>
                `,
              ).join("")}
            </select>
          </label>
        </div>
      </article>

      <article class="feature-card">
        <div class="section-header">
          <div>
            <h3>User-facing controls</h3>
            <p>These flows are ready to demo or hand off to the client.</p>
          </div>
        </div>
        <div class="note-list">
          <span>Keyboard shortcut support is already wired through the extension command surface.</span>
          <span>Context menu actions can open Flow or toggle the in-page overlay.</span>
          <span>Popup, side panel, sidebar, and options all share the same local browser runtime.</span>
          <span>Qwen3 remains the low-end default for cross-browser local text execution.</span>
        </div>
      </article>
    </section>
  `;
}

function renderDelivery(surface: FlowSurface, state: UiState) {
  const textStatus = state.readiness.textReady ? "complete" : "install text pack";
  const ocrStatus = state.readiness.ocrReady ? "complete" : "optional";
  const multimodalStatus = state.readiness.capabilities.webgpu
    ? state.readiness.multimodalReady
      ? "complete"
      : "optional"
    : "capability gated";

  return `
    <section class="section-stack">
      <div class="section-header">
        <div>
          <h2>Client delivery checklist</h2>
          <p>Use this surface as the final review while Firebase is still being wired.</p>
        </div>
        <span class="badge ${badgeTone("ready")}">Flow surface: ${escapeHtml(surface)}</span>
      </div>

      <div class="card-grid metrics-grid">
        <article class="metric-card">
          <span class="eyebrow">Local text</span>
          <strong>${escapeHtml(textStatus)}</strong>
          <p>Qwen3 offline chat, rewrite, compose, and explain flows.</p>
        </article>
        <article class="metric-card">
          <span class="eyebrow">OCR</span>
          <strong>${escapeHtml(ocrStatus)}</strong>
          <p>Screenshot and image text extraction stays local after pack install.</p>
        </article>
        <article class="metric-card">
          <span class="eyebrow">Multimodal</span>
          <strong>${escapeHtml(multimodalStatus)}</strong>
          <p>WebGPU browsers can unlock image and document reasoning locally.</p>
        </article>
        <article class="metric-card">
          <span class="eyebrow">Remote wiring</span>
          <strong>Firebase pending</strong>
          <p>Everything else in this browser surface is already prepared locally.</p>
        </article>
      </div>

      <article class="feature-card">
        <div class="section-header">
          <div>
            <h3>What is already complete</h3>
            <p>The client can review these features without waiting for remote auth.</p>
          </div>
        </div>
        <div class="note-list">
          <span>Cross-browser popup, side panel, sidebar, and options screens are implemented.</span>
          <span>Local pack download, verification, removal, and partial-download recovery are in place.</span>
          <span>Page context capture, quick actions, rewrite apply-back, and clipboard actions are implemented.</span>
          <span>Local-only privacy defaults are enforced unless you deliberately enable remote wiring later.</span>
          <span>DX / Zed native Rust integration can reuse the same local-first model policy outside the browser.</span>
        </div>
      </article>

      <article class="feature-card">
        <div class="card-topline">
          <span class="eyebrow">Final handoff note</span>
          <span class="badge ${badgeTone("optional")}">External dependency</span>
        </div>
        <h3>Remaining external work</h3>
        <p>
          Firebase environment wiring remains the last outside dependency. This extension surface,
          its local model logic, and the user-facing screens are otherwise ready to deliver.
        </p>
      </article>
    </section>
  `;
}

function renderPacks(state: UiState, engine: FlowBrowserEngine) {
  return `
    <section class="section-stack">
      <div class="section-header">
        <div>
          <h2>Browser model packs</h2>
          <p>Install only the packs the client needs. Qwen3 text is the low-end baseline.</p>
        </div>
        <div class="actions">
          <button type="button" class="secondary" data-action="refresh-runtime">Refresh pack status</button>
        </div>
      </div>
      <div class="card-grid pack-grid">
        ${engine
          .packCatalog()
          .map((pack) =>
            renderPackCard(
              pack,
              packStatusFor(state.readiness, pack.modelKey) ?? {
                packKey: pack.packKey,
                modelKey: pack.modelKey,
                displayName: pack.displayName,
                state: "missing",
                filesReady: 0,
                filesTotal: pack.files.length,
                storageBackend: state.readiness.storageBackend,
                lastUpdatedAt: null,
                lastError: null,
              },
              state.readiness,
            ),
          )
          .join("")}
      </div>
    </section>
  `;
}

function dashboardActionLabel(
  action: FlowDashboardProductUiCardBinding["actions"][number],
  state: UiState["dashboardActionStates"][string] = "idle",
) {
  if (state === "loading") {
    return action.buttonState.loadingLabel;
  }
  if (state === "success") {
    return action.buttonState.successLabel;
  }
  if (state === "error") {
    return action.buttonState.errorLabel;
  }
  if (state === "blocked") {
    return "Blocked";
  }
  return action.buttonState.idleLabel;
}

function dashboardResultTone(status: FlowDashboardCommandStatus) {
  if (status === "prepared") {
    return "ready";
  }
  if (status === "succeeded") {
    return "ready";
  }
  if (status === "timed-out" || status === "cancelled" || status === "denied") {
    return "blocked";
  }
  return status === "failed" ? "off" : "blocked";
}

function renderDashboardActionResults(
  results: FlowDashboardCommandResult[],
  runnerUx: FlowDashboardRunnerUxReport | null,
) {
  return `
    <article class="feature-card dashboard-command-results">
      <div class="card-topline">
        <span class="eyebrow">Command results</span>
        <span class="badge ${badgeTone(results.length > 0 ? "ready" : "pending")}">
          ${results.length} recent
        </span>
      </div>
      ${
        results.length === 0
          ? "<p>No dashboard command handoffs have been prepared in this browser yet.</p>"
          : `<div class="note-list">
              ${results
                .map(
                  (result) => `
                    <span>
                      <strong>${escapeHtml(result.label)}</strong>
                      <em class="badge ${badgeTone(dashboardResultTone(result.status))}">
                        ${escapeHtml(result.permission)}
                      </em>
                      ${escapeHtml(result.message)}
                      <code>${escapeHtml(result.command)}</code>
                    </span>
                  `,
                )
                .join("")}
            </div>`
      }
      ${
        runnerUx
          ? `
            <div class="runner-ux-panel">
              <div class="card-topline">
                <span class="eyebrow">Trusted runner history</span>
                <span class="badge ${badgeTone(runnerUx.latestStatus ? dashboardResultTone(runnerUx.latestStatus) : "pending")}">
                  ${runnerUx.resultCount} result${runnerUx.resultCount === 1 ? "" : "s"}
                </span>
              </div>
              <div class="meta-list">
                ${runnerUx.statusSummaries
                  .map(
                    (summary) => `
                      <span>
                        <strong>${escapeHtml(summary.title)}</strong>
                        ${summary.count}
                        <small>${escapeHtml(summary.description)}</small>
                      </span>
                    `,
                  )
                  .join("")}
              </div>
              <div class="actions dashboard-actions">
                ${runnerUx.affordances
                  .map(
                    (affordance) => `
                      <button
                        type="button"
                        class="secondary dashboard-runner-affordance"
                        data-action="dashboard-runner-affordance"
                        data-runner-affordance-id="${escapeHtml(affordance.id)}"
                        title="${escapeHtml(affordance.detail)}"
                        ${affordance.disabled ? "disabled" : ""}
                      >
                        ${escapeHtml(affordance.label)}
                      </button>
                    `,
                  )
                  .join("")}
              </div>
              <div class="note-list">
                ${runnerUx.operatorNotes
                  .map(
                    (note) => `
                      <span>
                        <strong>${escapeHtml(note.label)}</strong>
                        ${escapeHtml(note.detail)}
                        <code>${escapeHtml(note.releaseReviewPath)}</code>
                      </span>
                    `,
                  )
                  .join("")}
              </div>
            </div>
          `
          : ""
      }
    </article>
  `;
}

function renderRunnerApprovalModal(
  approvalUi: FlowDashboardRunnerApprovalUiReport | null,
  reason: string,
) {
  if (!approvalUi) {
    return "";
  }

  return `
    <article
      class="feature-card dashboard-runner-approval"
      role="dialog"
      aria-modal="false"
      aria-labelledby="${escapeHtml(approvalUi.modalId)}-title"
    >
      <div class="card-topline">
        <span class="eyebrow">Trusted runner approval</span>
        <span class="badge ${badgeTone(approvalUi.latestActionId ? "pending" : "blocked")}">
          ${approvalUi.resultCount} history
        </span>
      </div>
      <h3 id="${escapeHtml(approvalUi.modalId)}-title">${escapeHtml(approvalUi.title)}</h3>
      <p>${escapeHtml(approvalUi.body)}</p>
      <code>${escapeHtml(approvalUi.commandPreview)}</code>
      <label class="runner-reason">
        ${escapeHtml(approvalUi.reasonLabel)}
        <textarea
          id="dashboard-runner-approval-reason"
          rows="3"
          placeholder="${escapeHtml(approvalUi.reasonPlaceholder)}"
        >${escapeHtml(reason)}</textarea>
      </label>
      <div class="actions dashboard-actions">
        ${approvalUi.controls
          .map(
            (control) => `
              <button
                type="button"
                class="secondary dashboard-runner-approval-control"
                data-action="dashboard-runner-approval-control"
                data-runner-approval-control-id="${escapeHtml(control.id)}"
                aria-label="${escapeHtml(control.ariaLabel || control.label)}"
                title="${escapeHtml(control.detail)}"
                ${control.disabled ? "disabled" : ""}
              >
                ${escapeHtml(control.label)}
                ${
                  control.keyboardShortcut?.key
                    ? `<small>${escapeHtml(control.keyboardShortcut.key)}</small>`
                    : ""
                }
              </button>
            `,
          )
          .join("")}
      </div>
      <div class="meta-list">
        ${approvalUi.snoozeOptions
          .map(
            (option) => `
              <span><strong>${escapeHtml(option.label)}</strong> ${option.durationSeconds}s</span>
            `,
          )
          .join("")}
        <span><strong>Undo</strong> ${escapeHtml(approvalUi.undoNote)}</span>
        <span><strong>Review</strong> ${escapeHtml(approvalUi.releaseReviewPath)}</span>
      </div>
    </article>
  `;
}

function renderRunnerCancellationControls(
  cancellationUx: FlowDashboardRunnerCancellationUxReport | null,
  drafts: Record<string, string>,
) {
  if (!cancellationUx) {
    return "";
  }

  return `
    <div class="runner-cancellation-panel">
      <div class="card-topline">
        <span class="eyebrow">Cancellation and recovery</span>
        <span class="badge ${badgeTone(cancellationUx.activeCount > 0 ? "pending" : cancellationUx.staleCount > 0 ? "blocked" : "ready")}">
          ${cancellationUx.controls.length} control${cancellationUx.controls.length === 1 ? "" : "s"}
        </span>
      </div>
      <div class="meta-list">
        <span><strong>${cancellationUx.activeCount}</strong> active</span>
        <span><strong>${cancellationUx.staleCount}</strong> stale</span>
        <span><strong>${cancellationUx.denialCount}</strong> denied</span>
      </div>
      <div class="note-list">
        ${cancellationUx.guidance.map((item) => `<span>${escapeHtml(item)}</span>`).join("")}
      </div>
      <p>${escapeHtml(cancellationUx.draft.autosaveHint)}</p>
      <div class="runner-cancellation-controls">
        ${cancellationUx.controls
          .map((control) => {
            const draft = drafts[control.id] ?? cancellationUx.draft.defaultReason;
            return `
              <div class="runner-cancellation-control">
                <div>
                  <strong>${escapeHtml(control.label)}</strong>
                  <small>${escapeHtml(control.detail)}</small>
                </div>
                ${
                  control.requiresReason
                    ? `
                      <textarea
                        data-runner-cancellation-reason-id="${escapeHtml(control.id)}"
                        aria-label="${escapeHtml(`${control.label} reason`)}"
                        placeholder="Short operator reason"
                      >${escapeHtml(draft)}</textarea>
                    `
                    : ""
                }
                <button
                  type="button"
                  class="secondary"
                  data-action="dashboard-runner-cancellation-control"
                  data-runner-cancellation-control-id="${escapeHtml(control.id)}"
                  ${control.disabled ? "disabled" : ""}
                  title="${escapeHtml(control.command)}"
                >
                  ${escapeHtml(controlLabel(control))}
                </button>
              </div>
            `;
          })
          .join("")}
      </div>
    </div>
  `;
}

function controlLabel(control: FlowDashboardRunnerCancellationControl) {
  if (control.kind === "cleanup-stale") {
    return "Copy cleanup";
  }
  if (control.kind === "retry") {
    return "Copy retry";
  }
  if (control.kind === "denial-recovery") {
    return "Copy recovery";
  }
  if (control.kind === "cancel") {
    return "Copy cancel";
  }
  return "Copy command";
}

function renderLiveRunnerState(
  liveState: FlowDashboardLiveRunnerState | null,
  cancellationUx: FlowDashboardRunnerCancellationUxReport | null,
  drafts: Record<string, string>,
) {
  if (!liveState) {
    return "";
  }

  return `
    <article class="feature-card dashboard-live-runner-state">
      <div class="card-topline">
        <span class="eyebrow">Live runner state</span>
        <span class="badge ${badgeTone(liveState.staleCount > 0 ? "blocked" : "ready")}">
          ${liveState.recordCount} tracked
        </span>
      </div>
      <div class="dashboard-history-metrics">
        <span><strong>${liveState.pendingCount}</strong><small>pending</small></span>
        <span><strong>${liveState.runningCount}</strong><small>running</small></span>
        <span><strong>${liveState.finishedCount}</strong><small>finished</small></span>
        <span><strong>${liveState.staleCount}</strong><small>stale</small></span>
      </div>
      ${
        liveState.staleCount > 0
          ? `<p>${escapeHtml(liveState.staleRecoveryCopy)}</p>`
          : "<p>Live runner state is current. Imported history is shown separately from active host work.</p>"
      }
      <div class="note-list">
        ${liveState.records
          .map(
            (record) => `
              <span>
                <strong>${escapeHtml(record.label)}</strong>
                <em class="badge ${badgeTone(record.status === "stale" ? "blocked" : record.status === "running" ? "pending" : "ready")}">
                  ${escapeHtml(record.status)}
                </em>
                ${escapeHtml(record.message)}
                <code>${escapeHtml(record.command)}</code>
                ${
                  record.status === "stale"
                    ? `<code>${escapeHtml(record.cleanupCommand || record.recoveryCommand)}</code>`
                    : ""
                }
              </span>
            `,
          )
          .join("")}
      </div>
      ${renderRunnerCancellationControls(cancellationUx, drafts)}
    </article>
  `;
}

function renderRunnerOperatorReview(review: FlowDashboardRunnerOperatorReviewReport | null) {
  if (!review) {
    return "";
  }

  return `
    <article class="feature-card dashboard-runner-review">
      <div class="card-topline">
        <span class="eyebrow">Trusted runner review</span>
        <span class="badge ${badgeTone(review.releaseGateStatus === "blocked" ? "blocked" : review.releaseGateStatus === "ready" ? "ready" : "pending")}">
          ${escapeHtml(review.releaseGateStatus || "review")}
        </span>
      </div>
      <div class="dashboard-history-metrics">
        <span><strong>${review.matchedCount}</strong><small>matched</small></span>
        <span><strong>${review.blockedCount}</strong><small>blocked</small></span>
        <span><strong>${review.readyCount}</strong><small>ready</small></span>
      </div>
      <div class="meta-list">
        <span><strong>History</strong> ${escapeHtml(review.historyJson)}</span>
        <span><strong>Status</strong> ${escapeHtml(review.filters.status)}</span>
        <span><strong>Action</strong> ${escapeHtml(review.filters.actionId ?? "all")}</span>
        <span><strong>Limit</strong> ${review.filters.limit}</span>
      </div>
      <div class="runner-review-grid">
        <div>
          <span class="eyebrow">Release gate</span>
          <div class="note-list">
            ${review.releaseGateSummaries
              .map(
                (summary) => `
                  <span>
                    <strong>${escapeHtml(summary.title)}</strong>
                    <em class="badge ${badgeTone(summary.severity === "blocked" ? "blocked" : summary.severity === "ready" ? "ready" : "pending")}">${summary.count}</em>
                    ${escapeHtml(summary.detail)}
                    <small>${escapeHtml(summary.nextAction)}</small>
                  </span>
                `,
              )
              .join("")}
          </div>
        </div>
        <div>
          <span class="eyebrow">Incident notes</span>
          <div class="runner-incident-list">
            ${
              review.incidentNotes.length
                ? review.incidentNotes
                    .map(
                      (note) => `
                        <div class="runner-incident">
                          <strong>${escapeHtml(note.title)}</strong>
                          <small>${escapeHtml(note.body)}</small>
                          <button
                            type="button"
                            class="secondary"
                            data-action="dashboard-runner-incident-copy"
                            data-runner-incident-id="${escapeHtml(note.id)}"
                          >
                            Copy note
                          </button>
                        </div>
                      `,
                    )
                    .join("")
                : "<p>No incident notes for the current filter.</p>"
            }
          </div>
        </div>
      </div>
    </article>
  `;
}

function renderRunnerReleasePackage(pkg: FlowDashboardRunnerReleasePackageReport | null) {
  if (!pkg) {
    return "";
  }

  return `
    <article class="feature-card dashboard-runner-package">
      <div class="card-topline">
        <span class="eyebrow">Trusted runner release package</span>
        <span class="badge ${badgeTone(pkg.readyToShip ? "ready" : "blocked")}">
          ${pkg.readyToShip ? "ready" : "review"}
        </span>
      </div>
      <p>${escapeHtml(pkg.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${pkg.manifest.evidenceCount}</strong><small>evidence</small></span>
        <span><strong>${pkg.manifest.missingCount}</strong><small>missing</small></span>
        <span><strong>${pkg.manifest.warningCount}</strong><small>warnings</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Package</strong> ${escapeHtml(pkg.manifest.packageJson)}</span>
        <span><strong>Signature</strong> ${escapeHtml(pkg.manifest.packageSignature.slice(0, 16))}</span>
        <span><strong>Local</strong> ${pkg.manifest.localOnly ? "yes" : "no"}</span>
      </div>
      ${
        pkg.warnings.length
          ? `<div class="note-list">${pkg.warnings
              .map((warning) => `<span>${escapeHtml(warning)}</span>`)
              .join("")}</div>`
          : "<p>No missing evidence or stale runner warnings in this package.</p>"
      }
      <div class="runner-package-files">
        ${pkg.manifest.files
          .map(
            (file) => `
              <div class="runner-package-file ${file.present ? "present" : "missing"}">
                <strong>${escapeHtml(file.label)}</strong>
                <small>${escapeHtml(file.kind)} - ${file.bytes} bytes</small>
                <code>${escapeHtml(file.path)}</code>
                <code>${escapeHtml(file.sha256 ?? "missing checksum")}</code>
              </div>
            `,
          )
          .join("")}
      </div>
    </article>
  `;
}

function renderRunnerReleaseTimeline(timeline: FlowDashboardRunnerReleaseTimeline | null) {
  if (!timeline) {
    return "";
  }

  return `
    <article class="feature-card dashboard-runner-timeline">
      <div class="card-topline">
        <span class="eyebrow">Trusted runner evidence timeline</span>
        <span class="badge ${badgeTone(timeline.missingEvidenceRegressions ? "blocked" : "ready")}">
          ${timeline.missingEvidenceRegressions ? "regression" : "stable"}
        </span>
      </div>
      <p>Compare release packages before shipping. Latest package: ${escapeHtml(
        timeline.latestPackageId ?? "none",
      )}.</p>
      <div class="dashboard-history-metrics">
        <span><strong>${timeline.packageCount}</strong><small>packages</small></span>
        <span><strong>${timeline.missingEvidenceRegressions}</strong><small>missing regressions</small></span>
        <span><strong>${timeline.warningRegressions}</strong><small>warning regressions</small></span>
      </div>
      ${
        timeline.warnings.length
          ? `<div class="note-list">${timeline.warnings
              .map((warning) => `<span>${escapeHtml(warning)}</span>`)
              .join("")}</div>`
          : "<p>No timeline regressions detected.</p>"
      }
      <div class="runner-package-files">
        ${timeline.entries
          .slice(-4)
          .map(
            (entry) => `
              <div class="runner-package-file ${entry.readyToShip ? "present" : "missing"}">
                <strong>${escapeHtml(entry.packageId)}</strong>
                <small>${entry.readyToShip ? "ready" : "review"} - ${entry.warningCount} warning(s)</small>
                <code>${escapeHtml(entry.packageJson)}</code>
                <code>${escapeHtml(entry.packageSignature)}</code>
              </div>
            `,
          )
          .join("")}
      </div>
      ${
        timeline.diffs.length
          ? `<div class="note-list">${timeline.diffs
              .slice(-3)
              .map((diff) => `<span>${escapeHtml(diff.summary)}</span>`)
              .join("")}</div>`
          : ""
      }
    </article>
  `;
}

function renderReleaseChecklist(
  checklist: FlowReleaseOperatorChecklistReport | null,
  reason: string,
) {
  if (!checklist) {
    return "";
  }
  const signoffCommand =
    checklist.commands.find((command) => command.includes("--friday-release-signoff")) ?? "";

  return `
    <article class="feature-card dashboard-release-checklist">
      <div class="card-topline">
        <span class="eyebrow">Release operator checklist</span>
        <span class="badge ${badgeTone(checklist.status)}">${escapeHtml(checklist.status)}</span>
      </div>
      <p>${escapeHtml(checklist.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${checklist.readyCount}/${checklist.totalCount}</strong><small>ready</small></span>
        <span><strong>${checklist.blockingCount}</strong><small>blocking</small></span>
        <span><strong>${checklist.signoffCount}</strong><small>signoffs</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Checklist</strong> ${escapeHtml(checklist.checklistJson)}</span>
        <span><strong>Signoffs</strong> ${escapeHtml(checklist.signoffJson)}</span>
        <span><strong>Ready</strong> ${checklist.readyToShip ? "yes" : "no"}</span>
      </div>
      <div class="note-list">
        ${
          checklist.blockers.length
            ? checklist.blockers
                .slice(0, 5)
                .map(
                  (blocker) =>
                    `<span>${escapeHtml(blocker.title)}: ${escapeHtml(blocker.detail)}</span>`,
                )
                .join("")
            : "<span>No blockers in the imported checklist.</span>"
        }
      </div>
      <div class="runner-package-files">
        ${checklist.checklist
          .map(
            (item) => `
              <div class="runner-package-file ${item.ready ? "present" : "missing"}">
                <strong>${escapeHtml(item.title)}</strong>
                <small>${item.ready ? "ready" : "needs review"}</small>
                <code>${escapeHtml(item.sourcePath)}</code>
                <span>${escapeHtml(item.detail)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <label class="field-label" for="dashboard-release-checklist-reason">
        Signoff reason
        <textarea
          id="dashboard-release-checklist-reason"
          rows="3"
          placeholder="Example: reviewed package, timeline, release review, TODO, and changelog."
        >${escapeHtml(reason)}</textarea>
      </label>
      <div class="actions">
        <button
          type="button"
          class="secondary"
          data-action="dashboard-release-checklist-signoff"
          ${signoffCommand ? "" : "disabled"}
        >
          Copy signoff command
        </button>
      </div>
    </article>
  `;
}

function renderReleaseQa(qa: FlowReleaseQaCommandCenterReport | null) {
  if (!qa) {
    return "";
  }

  return `
    <article class="feature-card dashboard-release-qa">
      <div class="card-topline">
        <span class="eyebrow">Release QA command center</span>
        <span class="badge ${badgeTone(qa.status)}">${escapeHtml(qa.status)}</span>
      </div>
      <p>${escapeHtml(qa.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${qa.scoreOutOf100}</strong><small>score</small></span>
        <span><strong>${qa.blockingCount}</strong><small>blocking</small></span>
        <span><strong>${qa.staleCount}</strong><small>stale</small></span>
      </div>
      <div class="runner-package-files">
        ${qa.checks
          .map(
            (check) => `
              <div class="runner-package-file ${
                check.status === "passed" ? "present" : "missing"
              }">
                <strong>${escapeHtml(check.label)}</strong>
                <small>${escapeHtml(check.status)} - ${check.bytes} bytes</small>
                <code>${escapeHtml(check.resultPath)}</code>
                <span>${escapeHtml(check.summary)}</span>
                <button
                  type="button"
                  class="secondary"
                  data-action="dashboard-release-qa-command"
                  data-release-qa-command-id="${escapeHtml(check.id)}"
                >
                  Copy command
                </button>
              </div>
            `,
          )
          .join("")}
      </div>
    </article>
  `;
}

function renderReleaseExportKit(kit: FlowReleaseEvidenceExportKitReport | null) {
  if (!kit) {
    return "";
  }

  return `
    <article class="feature-card dashboard-release-export-kit">
      <div class="card-topline">
        <span class="eyebrow">Release evidence export kit</span>
        <span class="badge ${badgeTone(kit.status)}">${escapeHtml(kit.status)}</span>
      </div>
      <p>${escapeHtml(kit.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${kit.manifest.fileCount}</strong><small>files</small></span>
        <span><strong>${kit.manifest.missingCount}</strong><small>missing</small></span>
        <span><strong>${kit.manifest.staleCount}</strong><small>stale</small></span>
        <span><strong>${kit.signoffCount}</strong><small>signoffs</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Kit</strong> ${escapeHtml(kit.manifest.kitJson)}</span>
        <span><strong>Checksum</strong> ${escapeHtml(kit.manifest.manifestSha256.slice(0, 16))}</span>
        <span><strong>QA</strong> ${kit.qaScoreOutOf100 ?? 0}/100</span>
      </div>
      ${
        kit.warnings.length
          ? `<div class="note-list">${kit.warnings
              .slice(0, 4)
              .map((warning) => `<span>${escapeHtml(warning)}</span>`)
              .join("")}</div>`
          : ""
      }
      <div class="runner-package-files">
        ${kit.manifest.files
          .map(
            (file) => `
              <div class="runner-package-file ${
                file.present && !file.stale ? "present" : "missing"
              }">
                <strong>${escapeHtml(file.label)}</strong>
                <small>${file.present ? "present" : "missing"} - ${file.bytes} bytes</small>
                <code>${escapeHtml(file.path)}</code>
                <span>${escapeHtml(
                  file.sha256
                    ? `sha256 ${file.sha256.slice(0, 16)}`
                    : (file.warning ?? "No checksum"),
                )}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-export-kit-copy">
          Copy export note
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-export-kit-command">
          Copy kit command
        </button>
      </div>
    </article>
  `;
}

function renderReleaseDeploymentGate(gate: FlowReleaseDeploymentGateReport | null) {
  if (!gate) {
    return "";
  }

  return `
    <article class="feature-card dashboard-release-deployment-gate ${gate.decision}">
      <div class="card-topline">
        <span class="eyebrow">Release deployment gate</span>
        <span class="badge ${badgeTone(gate.status)}">${escapeHtml(gate.decision)}</span>
      </div>
      <h3>${gate.readyToDeploy ? "Ready to deploy" : "Do not deploy yet"}</h3>
      <p>${escapeHtml(gate.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${gate.scoreOutOf100}</strong><small>score</small></span>
        <span><strong>${gate.noDeployReasonCount}</strong><small>blocking</small></span>
        <span><strong>${gate.warningCount}</strong><small>warnings</small></span>
        <span><strong>${gate.readyCount}/${gate.totalCount}</strong><small>ready</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Target</strong> ${escapeHtml(gate.target.label)}</span>
        <span><strong>Provider</strong> ${escapeHtml(gate.target.provider)}</span>
        <span><strong>Environment</strong> ${escapeHtml(gate.target.environment)}</span>
        <span><strong>Gate</strong> ${escapeHtml(gate.gateJson)}</span>
      </div>
      ${
        gate.reasons.length
          ? `<div class="runner-package-files">${gate.reasons
              .slice(0, 5)
              .map(
                (reason) => `
                  <div class="runner-package-file ${
                    reason.severity === "warning" ? "present" : "missing"
                  }">
                    <strong>${escapeHtml(reason.title)}</strong>
                    <small>${escapeHtml(reason.category)} - ${escapeHtml(reason.severity)}</small>
                    <code>${escapeHtml(reason.sourcePath)}</code>
                    <span>${escapeHtml(reason.nextAction)}</span>
                  </div>
                `,
              )
              .join("")}</div>`
          : `<div class="note-list"><span>All deployment-gate reasons are clear.</span></div>`
      }
      <div class="note-list">
        ${gate.deployChecklist
          .slice(0, 4)
          .map((item) => `<span>${escapeHtml(item)}</span>`)
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-deployment-gate-copy">
          Copy gate note
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-deployment-gate-command">
          Copy gate command
        </button>
      </div>
    </article>
  `;
}

function renderReleaseCandidateArchive(archive: FlowReleaseCandidateArchive | null) {
  if (!archive) {
    return "";
  }

  const latest = archive.entries[archive.entries.length - 1] ?? null;

  return `
    <article class="feature-card dashboard-release-candidate-archive">
      <div class="card-topline">
        <span class="eyebrow">Release candidate archive</span>
        <span class="badge ${badgeTone(archive.regressionCount > 0 ? "warning" : "ready")}">
          ${archive.candidateCount} candidates
        </span>
      </div>
      <h3>${latest ? escapeHtml(latest.target.label) : "No candidates yet"}</h3>
      <p>
        ${latest
          ? `Latest ${escapeHtml(latest.decision)} at ${latest.scoreOutOf100}/100 with ${latest.noDeployReasonCount} blocker(s).`
          : "Append deployment gates to preserve release history before major checkpoints."}
      </p>
      <div class="dashboard-history-metrics">
        <span><strong>${archive.goCount}</strong><small>go</small></span>
        <span><strong>${archive.noGoCount}</strong><small>no-go</small></span>
        <span><strong>${archive.draftCount}</strong><small>draft</small></span>
        <span><strong>${archive.regressionCount}</strong><small>regressions</small></span>
      </div>
      <div class="runner-package-files">
        ${archive.entries
          .slice(-4)
          .reverse()
          .map(
            (entry) => `
              <div class="runner-package-file ${entry.readyToDeploy ? "present" : "missing"}">
                <strong>${escapeHtml(entry.candidateId)}</strong>
                <small>${escapeHtml(entry.decision)} - ${entry.scoreOutOf100}/100</small>
                <code>${escapeHtml(entry.gateJson)}</code>
                <span>${escapeHtml(entry.rollbackNote)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      ${
        archive.diffs.length
          ? `<div class="note-list">${archive.diffs
              .slice(-3)
              .reverse()
              .map((diff) => `<span>${escapeHtml(diff.summary)}</span>`)
              .join("")}</div>`
          : ""
      }
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-candidate-archive-command">
          Copy archive command
        </button>
      </div>
    </article>
  `;
}

function renderReleasePromotionLedger(ledger: FlowReleasePromotionLedger | null) {
  if (!ledger) {
    return "";
  }

  const latest = ledger.records[ledger.records.length - 1] ?? null;

  return `
    <article class="feature-card dashboard-release-promotion-ledger">
      <div class="card-topline">
        <span class="eyebrow">Release promotion ledger</span>
        <span class="badge ${badgeTone(ledger.postPromotionMissingCount > 0 ? "warning" : "ready")}">
          ${ledger.recordCount} records
        </span>
      </div>
      <h3>${latest ? escapeHtml(latest.candidateId) : "No promotion decisions yet"}</h3>
      <p>
        ${latest
          ? `Latest ${escapeHtml(latest.decision)} decision by ${escapeHtml(latest.operator)} with ${latest.postPromotionMissingCount} missing post-promotion check(s).`
          : "Record held, promoted, rolled-back, superseded, or abandoned candidate decisions before major deploys."}
      </p>
      <div class="dashboard-history-metrics">
        <span><strong>${ledger.promotedCount}</strong><small>promoted</small></span>
        <span><strong>${ledger.heldCount}</strong><small>held</small></span>
        <span><strong>${ledger.rolledBackCount}</strong><small>rolled back</small></span>
        <span><strong>${ledger.postPromotionMissingCount}</strong><small>missing</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Active rollback</strong> ${escapeHtml(ledger.activeRollbackReference ?? "not recorded")}</span>
        <span><strong>Ledger</strong> ${escapeHtml(ledger.ledgerJson)}</span>
        <span><strong>Latest note</strong> ${escapeHtml(ledger.latestDeploymentNote ?? "none")}</span>
      </div>
      <div class="runner-package-files">
        ${ledger.records
          .slice(-4)
          .reverse()
          .map(
            (record) => `
              <div class="runner-package-file ${
                record.postPromotionMissingCount === 0 ? "present" : "missing"
              }">
                <strong>${escapeHtml(record.promotionId)}</strong>
                <small>${escapeHtml(record.decision)} - ${record.candidateScoreOutOf100}/100</small>
                <code>${escapeHtml(record.archiveJson)}</code>
                <span>${escapeHtml(record.reason)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      ${
        latest?.postPromotionChecks.length
          ? `<div class="note-list">${latest.postPromotionChecks
              .slice(0, 5)
              .map(
                (check) =>
                  `<span>${escapeHtml(check.label)}: ${check.present ? "present" : "missing"} - ${escapeHtml(check.nextAction)}</span>`,
              )
              .join("")}</div>`
          : ""
      }
      ${
        ledger.warnings.length
          ? `<div class="note-list">${ledger.warnings
              .slice(0, 4)
              .map((warning) => `<span>${escapeHtml(warning)}</span>`)
              .join("")}</div>`
          : ""
      }
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-promotion-ledger-command">
          Copy ledger command
        </button>
      </div>
    </article>
  `;
}

function renderReleasePostPromotionMonitor(
  monitor: FlowReleasePostPromotionMonitorReport | null,
) {
  if (!monitor) {
    return "";
  }

  return `
    <article class="feature-card dashboard-release-post-promotion-monitor">
      <div class="card-topline">
        <span class="eyebrow">Post-promotion monitor</span>
        <span class="badge ${badgeTone(monitor.status)}">${monitor.scoreOutOf100} / 100</span>
      </div>
      <h3>${monitor.activeCandidateId ? escapeHtml(monitor.activeCandidateId) : "No promoted candidate"}</h3>
      <p>${escapeHtml(monitor.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${monitor.blockingCount}</strong><small>blocking</small></span>
        <span><strong>${monitor.warningCount}</strong><small>warnings</small></span>
        <span><strong>${monitor.staleCount}</strong><small>stale</small></span>
        <span><strong>${monitor.incidentNoteCount}</strong><small>incidents</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Stable</strong> ${monitor.readyForStable ? "ready" : "not ready"}</span>
        <span><strong>Rollback</strong> ${escapeHtml(monitor.activeRollbackReference ?? "not recorded")}</span>
        <span><strong>Monitor</strong> ${escapeHtml(monitor.monitorJson)}</span>
        <span><strong>QA</strong> ${escapeHtml(monitor.qaJson)}</span>
      </div>
      <div class="runner-package-files">
        ${monitor.checks
          .slice(0, 6)
          .map(
            (check) => `
              <div class="runner-package-file ${
                check.status === "passed" ? "present" : "missing"
              }">
                <strong>${escapeHtml(check.label)}</strong>
                <small>${escapeHtml(check.status)} - ${check.required ? "required" : "optional"}</small>
                <code>${escapeHtml(check.sourcePath)}</code>
                <span>${escapeHtml(check.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      ${
        monitor.incidentNotes.length
          ? `<div class="note-list">${monitor.incidentNotes
              .slice(0, 4)
              .map(
                (note) =>
                  `<span>${escapeHtml(note.id)}: ${note.present ? "present" : "missing"} - ${escapeHtml(note.path)}</span>`,
              )
              .join("")}</div>`
          : ""
      }
      ${
        monitor.warnings.length
          ? `<div class="note-list">${monitor.warnings
              .slice(0, 4)
              .map((warning) => `<span>${escapeHtml(warning)}</span>`)
              .join("")}</div>`
          : ""
      }
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-post-promotion-monitor-command">
          Copy monitor command
        </button>
      </div>
    </article>
  `;
}

function renderReleaseRollbackDrill(drill: FlowReleaseRollbackDrillReport | null) {
  if (!drill) {
    return "";
  }

  return `
    <article class="feature-card dashboard-release-rollback-drill">
      <div class="card-topline">
        <span class="eyebrow">Rollback drill</span>
        <span class="badge ${badgeTone(drill.status)}">${drill.scoreOutOf100} / 100</span>
      </div>
      <h3>${drill.activeRollbackReference ? escapeHtml(drill.activeRollbackReference) : "Rollback not ready"}</h3>
      <p>${escapeHtml(drill.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${drill.blockingCount}</strong><small>blocking</small></span>
        <span><strong>${drill.warningCount}</strong><small>warnings</small></span>
        <span><strong>${drill.staleCount}</strong><small>stale</small></span>
        <span><strong>${drill.missingEvidenceCount}</strong><small>missing</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Rollback</strong> ${drill.readyToRollback ? "ready" : "blocked"}</span>
        <span><strong>Stable</strong> ${drill.readyForStable ? "ready" : "not ready"}</span>
        <span><strong>Candidate</strong> ${escapeHtml(drill.activeCandidateId ?? "not recorded")}</span>
        <span><strong>Operator</strong> ${escapeHtml(drill.operator)}</span>
      </div>
      <div class="runner-package-files">
        ${drill.checks
          .slice(0, 6)
          .map(
            (check) => `
              <div class="runner-package-file ${
                check.status === "passed" ? "present" : "missing"
              }">
                <strong>${escapeHtml(check.label)}</strong>
                <small>${escapeHtml(check.status)} - ${check.required ? "required" : "optional"}</small>
                <code>${escapeHtml(check.sourcePath)}</code>
                <span>${escapeHtml(check.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      ${
        drill.blockedReasons.length
          ? `<div class="note-list">${drill.blockedReasons
              .slice(0, 5)
              .map((reason) => `<span>${escapeHtml(reason)}</span>`)
              .join("")}</div>`
          : ""
      }
      <div class="command-preview">${escapeHtml(drill.dryRunCommand)}</div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-rollback-drill-command">
          Copy drill command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-rollback-drill-dry-run">
          Copy dry run
        </button>
      </div>
    </article>
  `;
}

function renderReleaseStabilityBoard(board: FlowReleaseStabilityBoardReport | null) {
  if (!board) {
    return "";
  }

  return `
    <article class="feature-card dashboard-release-stability-board">
      <div class="card-topline">
        <span class="eyebrow">Stability board</span>
        <span class="badge ${badgeTone(board.status)}">${board.scoreOutOf100} / 100</span>
      </div>
      <h3>${board.activeCandidateId ? escapeHtml(board.activeCandidateId) : "No active candidate"}</h3>
      <p>${escapeHtml(board.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${board.blockingCount}</strong><small>blocking</small></span>
        <span><strong>${board.warningCount}</strong><small>warnings</small></span>
        <span><strong>${board.staleCount}</strong><small>stale</small></span>
        <span><strong>${board.missingEvidenceCount}</strong><small>missing</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Checkpoint</strong> ${board.readyForCheckpoint ? "ready" : "blocked"}</span>
        <span><strong>Deploy</strong> ${board.readyToDeploy ? "ready" : "blocked"}</span>
        <span><strong>Stable</strong> ${board.stableAfterPromotion ? "ready" : "not ready"}</span>
        <span><strong>Recoverable</strong> ${board.recoverable ? "ready" : "blocked"}</span>
      </div>
      <div class="runner-package-files">
        ${board.checks
          .slice(0, 6)
          .map(
            (check) => `
              <div class="runner-package-file ${
                check.status === "passed" ? "present" : "missing"
              }">
                <strong>${escapeHtml(check.label)}</strong>
                <small>${escapeHtml(check.category)} - ${escapeHtml(check.status)}</small>
                <code>${escapeHtml(check.sourcePath)}</code>
                <span>${escapeHtml(check.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      ${
        board.activeRisks.length
          ? `<div class="note-list">${board.activeRisks
              .slice(0, 5)
              .map((risk) => `<span>${escapeHtml(risk)}</span>`)
              .join("")}</div>`
          : ""
      }
      <div class="note-list">
        ${board.evidenceLinks
          .slice(0, 6)
          .map(
            (link) =>
              `<span>${escapeHtml(link.label)}: ${link.present ? "present" : "missing"} - ${escapeHtml(link.path)}</span>`,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-stability-board-command">
          Copy board command
        </button>
      </div>
    </article>
  `;
}

function renderReleaseRecoveryRunbook(runbook: FlowReleaseRecoveryRunbookReport | null) {
  if (!runbook) {
    return "";
  }

  return `
    <article class="feature-card dashboard-release-recovery-runbook">
      <div class="card-topline">
        <span class="eyebrow">Recovery runbook</span>
        <span class="badge ${badgeTone(runbook.status)}">${runbook.scoreOutOf100} / 100</span>
      </div>
      <h3>${runbook.activeRollbackReference ? escapeHtml(runbook.activeRollbackReference) : "Recovery needs approval"}</h3>
      <p>${escapeHtml(runbook.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${runbook.phaseCount}</strong><small>phases</small></span>
        <span><strong>${runbook.blockedPhaseCount}</strong><small>blocked</small></span>
        <span><strong>${runbook.unsatisfiedApprovalGateCount}</strong><small>approvals</small></span>
        <span><strong>${runbook.commandCount}</strong><small>commands</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Review</strong> ${runbook.readyForOperatorReview ? "ready" : "blocked"}</span>
        <span><strong>Execute</strong> ${runbook.readyToExecuteRecovery ? "ready" : "blocked"}</span>
        <span><strong>Candidate</strong> ${escapeHtml(runbook.activeCandidateId ?? "not recorded")}</span>
        <span><strong>Promotion</strong> ${escapeHtml(runbook.latestPromotionDecision ?? "not recorded")}</span>
      </div>
      <div class="runner-package-files">
        ${runbook.phases
          .slice()
          .sort((left, right) => left.order - right.order)
          .map(
            (phase) => `
              <div class="runner-package-file ${
                phase.status === "ready" ? "present" : "missing"
              }">
                <strong>${phase.order}. ${escapeHtml(phase.label)}</strong>
                <small>${escapeHtml(phase.kind)} - ${escapeHtml(phase.status)}</small>
                <code>${escapeHtml(phase.command)}</code>
                <span>${escapeHtml(phase.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      ${
        runbook.activeRisks.length
          ? `<div class="note-list">${runbook.activeRisks
              .slice(0, 5)
              .map((risk) => `<span>${escapeHtml(risk)}</span>`)
              .join("")}</div>`
          : ""
      }
      ${
        runbook.approvalGates.length
          ? `<div class="note-list">${runbook.approvalGates
              .slice(0, 4)
              .map(
                (gate) =>
                  `<span>${escapeHtml(gate.label)}: ${gate.satisfied ? "satisfied" : "pending"} - ${escapeHtml(gate.summary)}</span>`,
              )
              .join("")}</div>`
          : ""
      }
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-recovery-runbook-command">
          Copy runbook command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-recovery-runbook-phase">
          Copy first phase
        </button>
      </div>
    </article>
  `;
}

function renderReleaseIncidentArchive(archive: FlowReleaseIncidentArchive | null) {
  if (!archive) {
    return "";
  }

  const latest = archive.entries[archive.entries.length - 1] ?? null;

  return `
    <article class="feature-card dashboard-release-incident-archive">
      <div class="card-topline">
        <span class="eyebrow">Incident archive</span>
        <span class="badge ${badgeTone(archive.blockingCount ? "blocked" : "ready")}">
          ${archive.incidentCount} incident${archive.incidentCount === 1 ? "" : "s"}
        </span>
      </div>
      <h3>${latest ? escapeHtml(latest.title) : "No incidents archived yet"}</h3>
      <p>${latest ? escapeHtml(latest.summary) : "Local release recovery history is ready for the next incident record."}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${archive.openCount}</strong><small>open</small></span>
        <span><strong>${archive.monitoringCount}</strong><small>monitoring</small></span>
        <span><strong>${archive.resolvedCount}</strong><small>resolved</small></span>
        <span><strong>${archive.followUpCount}</strong><small>follow-ups</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Critical</strong> ${archive.criticalCount}</span>
        <span><strong>Blocking</strong> ${archive.blockingCount}</span>
        <span><strong>Rollback</strong> ${escapeHtml(archive.latestRollbackReference ?? "not recorded")}</span>
        <span><strong>Archive</strong> ${escapeHtml(archive.archiveJson)}</span>
      </div>
      <div class="runner-package-files">
        ${archive.entries
          .slice(-4)
          .reverse()
          .map(
            (entry) => `
              <div class="runner-package-file ${
                entry.severity === "critical" || entry.severity === "blocking"
                  ? "missing"
                  : "present"
              }">
                <strong>${escapeHtml(entry.title)}</strong>
                <small>${escapeHtml(entry.severity)} - ${escapeHtml(entry.outcome)}</small>
                <code>${escapeHtml(entry.recoveryRunbookJson)}</code>
                <span>${escapeHtml(entry.followUpActions[0] ?? entry.summary)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-incident-archive-command">
          Copy archive command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-incident-archive-follow-up">
          Copy latest follow-up
        </button>
      </div>
    </article>
  `;
}

function renderReleasePreventionPlan(plan: FlowReleasePreventionPlanReport | null) {
  if (!plan) {
    return "";
  }

  return `
    <article class="feature-card dashboard-release-prevention-plan">
      <div class="card-topline">
        <span class="eyebrow">Prevention plan</span>
        <span class="badge ${badgeTone(plan.status)}">${plan.scoreOutOf100} / 100</span>
      </div>
      <h3>${plan.readyForNextCheckpoint ? "Next checkpoint is clear" : "Next checkpoint needs prevention evidence"}</h3>
      <p>${escapeHtml(plan.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${plan.findingCount}</strong><small>findings</small></span>
        <span><strong>${plan.recurringIssueCount}</strong><small>recurring</small></span>
        <span><strong>${plan.actionCount}</strong><small>actions</small></span>
        <span><strong>${plan.gateBlockingCount}</strong><small>gate blocks</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Owner ready</strong> ${plan.ownerReadyCount}</span>
        <span><strong>Evidence missing</strong> ${plan.evidenceMissingCount}</span>
        <span><strong>Incidents</strong> ${plan.incidentCount}</span>
        <span><strong>Rollback</strong> ${escapeHtml(plan.activeRollbackReference ?? "not recorded")}</span>
      </div>
      <div class="runner-package-files">
        ${plan.actions
          .slice(0, 6)
          .map(
            (action) => `
              <div class="runner-package-file ${
                action.status === "owner-ready" ? "present" : "missing"
              }">
                <strong>${escapeHtml(action.title)}</strong>
                <small>${escapeHtml(action.kind)} - ${escapeHtml(action.status)}</small>
                <code>${escapeHtml(action.command)}</code>
                <span>${escapeHtml(action.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      ${
        plan.findings.length
          ? `<div class="note-list">${plan.findings
              .slice(0, 5)
              .map(
                (finding) =>
                  `<span>${escapeHtml(finding.title)}: ${escapeHtml(finding.nextAction)}</span>`,
              )
              .join("")}</div>`
          : ""
      }
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-prevention-plan-command">
          Copy plan command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-prevention-plan-owner-copy">
          Copy owner actions
        </button>
      </div>
    </article>
  `;
}

function renderReleaseOwnerFollowUpBoard(board: FlowReleaseOwnerFollowUpBoardReport | null) {
  if (!board) {
    return "";
  }

  return `
    <article class="feature-card dashboard-release-owner-followup-board">
      <div class="card-topline">
        <span class="eyebrow">Owner follow-up</span>
        <span class="badge ${badgeTone(board.status)}">${board.scoreOutOf100} / 100</span>
      </div>
      <h3>${board.readyForNextCheckpoint ? "Owner evidence is clear" : "Owner evidence is still required"}</h3>
      <p>${escapeHtml(board.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${board.ownerCount}</strong><small>owners</small></span>
        <span><strong>${board.recordCount}</strong><small>records</small></span>
        <span><strong>${board.overdueCount}</strong><small>overdue</small></span>
        <span><strong>${board.gateBlockingCount}</strong><small>gate blocks</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Ready</strong> ${board.readyCount}</span>
        <span><strong>Waiting</strong> ${board.waitingCount}</span>
        <span><strong>Missing evidence</strong> ${board.evidenceMissingCount}</span>
        <span><strong>Plan</strong> ${escapeHtml(board.preventionPlanJson)}</span>
      </div>
      <div class="runner-package-files">
        ${board.records
          .slice(0, 6)
          .map(
            (record) => `
              <div class="runner-package-file ${
                record.completionState === "complete" || record.completionState === "ready"
                  ? "present"
                  : "missing"
              }">
                <strong>@${escapeHtml(record.owner)} - ${escapeHtml(record.title)}</strong>
                <small>${escapeHtml(record.completionState)} - ${escapeHtml(record.evidenceState)}</small>
                <code>${escapeHtml(record.command)}</code>
                <span>${escapeHtml(record.evidenceRequest)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="note-list">
        ${board.ownerGroups
          .slice(0, 4)
          .map(
            (group) =>
              `<span>@${escapeHtml(group.owner)}: ${group.recordCount} item(s), ${group.evidenceMissingCount} evidence request(s), ${group.overdueCount} overdue</span>`,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-owner-followup-board-command">
          Copy board command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-owner-followup-board-assignment">
          Copy assignments
        </button>
      </div>
    </article>
  `;
}

function renderReleaseEvidenceSlaMonitor(monitor: FlowReleaseEvidenceSlaMonitorReport | null) {
  if (!monitor) {
    return "";
  }

  return `
    <article class="feature-card dashboard-release-evidence-sla-monitor">
      <div class="card-topline">
        <span class="eyebrow">Evidence SLA</span>
        <span class="badge ${badgeTone(monitor.status)}">${monitor.scoreOutOf100} / 100</span>
      </div>
      <h3>${monitor.readyForNextCheckpoint ? "Release evidence is inside SLA" : "Release evidence needs escalation"}</h3>
      <p>${escapeHtml(monitor.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${monitor.requirementCount}</strong><small>requirements</small></span>
        <span><strong>${monitor.overdueCount}</strong><small>overdue</small></span>
        <span><strong>${monitor.missingCount}</strong><small>missing</small></span>
        <span><strong>${monitor.escalationCount}</strong><small>escalations</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Fresh</strong> ${monitor.freshCount}</span>
        <span><strong>Due soon</strong> ${monitor.dueSoonCount}</span>
        <span><strong>Gate blocks</strong> ${monitor.gateBlockingCount}</span>
        <span><strong>Owner board</strong> ${escapeHtml(monitor.ownerFollowupBoardJson)}</span>
      </div>
      <div class="runner-package-files">
        ${monitor.requirements
          .slice(0, 6)
          .map(
            (requirement) => `
              <div class="runner-package-file ${
                requirement.state === "fresh" || requirement.state === "acknowledged"
                  ? "present"
                  : "missing"
              }">
                <strong>@${escapeHtml(requirement.owner)} - ${escapeHtml(requirement.title)}</strong>
                <small>${escapeHtml(requirement.state)} - ${escapeHtml(requirement.escalationLevel)}</small>
                <code>${escapeHtml(requirement.evidencePath)}</code>
                <span>${escapeHtml(requirement.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="note-list">
        ${monitor.ownerGroups
          .slice(0, 4)
          .map(
            (group) =>
              `<span>@${escapeHtml(group.owner)}: ${group.requirementCount} requirement(s), ${group.overdueCount} overdue, ${group.escalationCount} escalation(s)</span>`,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-evidence-sla-monitor-command">
          Copy SLA command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-evidence-sla-monitor-escalation">
          Copy escalations
        </button>
      </div>
    </article>
  `;
}

function renderReleaseEscalationLedger(ledger: FlowReleaseEscalationLedger | null) {
  if (!ledger) {
    return "";
  }

  const status = ledger.activeCount > 0 || ledger.acknowledgementBlockerCount > 0
    ? "blocked"
    : "ready";

  return `
    <article class="feature-card dashboard-release-escalation-ledger">
      <div class="card-topline">
        <span class="eyebrow">Escalation ledger</span>
        <span class="badge ${badgeTone(status)}">${ledger.activeCount} active</span>
      </div>
      <h3>${ledger.activeCount > 0 ? "Escalations are carrying forward" : "No active escalation carryovers"}</h3>
      <p>${escapeHtml(ledger.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${ledger.entryCount}</strong><small>records</small></span>
        <span><strong>${ledger.carryoverCount}</strong><small>carryovers</small></span>
        <span><strong>${ledger.acknowledgementBlockerCount}</strong><small>ack blockers</small></span>
        <span><strong>${ledger.releaseGateBlockingCount}</strong><small>gate blocks</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Owners</strong> ${ledger.ownerCount}</span>
        <span><strong>Pending</strong> ${ledger.responsePendingCount}</span>
        <span><strong>Resolved</strong> ${ledger.resolvedCount}</span>
        <span><strong>Ledger</strong> ${escapeHtml(ledger.ledgerJson)}</span>
      </div>
      <div class="runner-package-files">
        ${ledger.entries
          .slice(0, 6)
          .map(
            (entry) => `
              <div class="runner-package-file ${
                entry.acknowledged && !entry.activeCarryover ? "present" : "missing"
              }">
                <strong>@${escapeHtml(entry.owner)} - ${escapeHtml(entry.title)}</strong>
                <small>${escapeHtml(entry.ownerResponse)} - ${escapeHtml(entry.gateOutcome)}</small>
                <code>${escapeHtml(entry.evidencePath)}</code>
                <span>${escapeHtml(entry.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="note-list">
        ${ledger.ownerGroups
          .slice(0, 4)
          .map(
            (group) =>
              `<span>@${escapeHtml(group.owner)}: ${group.entryCount} record(s), ${group.activeCount} active, ${group.acknowledgementBlockerCount} acknowledgement blocker(s)</span>`,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-escalation-ledger-command">
          Copy ledger command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-escalation-ledger-owner-response">
          Copy owner responses
        </button>
      </div>
    </article>
  `;
}

function renderReleaseCheckpointReview(review: FlowReleaseCheckpointReviewBoardReport | null) {
  if (!review) {
    return "";
  }

  return `
    <article class="feature-card dashboard-release-checkpoint-review">
      <div class="card-topline">
        <span class="eyebrow">Checkpoint review</span>
        <span class="badge ${badgeTone(review.status)}">${escapeHtml(review.decision)}</span>
      </div>
      <h3>${review.readyForCheckpoint ? "Checkpoint can move forward" : "Checkpoint is held for review"}</h3>
      <p>${escapeHtml(review.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${review.scoreOutOf100}</strong><small>score</small></span>
        <span><strong>${review.holdCount}</strong><small>holds</small></span>
        <span><strong>${review.carryoverCount}</strong><small>carryovers</small></span>
        <span><strong>${review.acknowledgementBlockerCount}</strong><small>ack blockers</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Items</strong> ${review.itemCount}</span>
        <span><strong>Owners</strong> ${review.ownerCount}</span>
        <span><strong>Gate blocks</strong> ${review.releaseGateBlockingCount}</span>
        <span><strong>Review</strong> ${escapeHtml(review.reviewJson)}</span>
      </div>
      <div class="runner-package-files">
        ${review.items
          .slice(0, 6)
          .map(
            (item) => `
              <div class="runner-package-file ${item.state === "ready" ? "present" : "missing"}">
                <strong>@${escapeHtml(item.owner)} - ${escapeHtml(item.title)}</strong>
                <small>${escapeHtml(item.source)} - ${escapeHtml(item.state)}</small>
                <code>${escapeHtml(item.evidencePath)}</code>
                <span>${escapeHtml(item.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="note-list">
        ${review.ownerGroups
          .slice(0, 4)
          .map(
            (group) =>
              `<span>@${escapeHtml(group.owner)}: ${group.itemCount} item(s), ${group.holdCount} hold, ${group.acknowledgementBlockerCount} acknowledgement blocker(s)</span>`,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-checkpoint-review-command">
          Copy review command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-checkpoint-review-notes">
          Copy review notes
        </button>
      </div>
    </article>
  `;
}

function renderReleaseCheckpointSignoffLedger(
  ledger: FlowReleaseCheckpointSignoffLedger | null,
) {
  if (!ledger) {
    return "";
  }

  const status =
    ledger.activeHoldCount > 0 ||
    ledger.acknowledgementEvidenceMissingCount > 0 ||
    ledger.releaseGateBlockingCount > 0
      ? "blocked"
      : ledger.activeCarryoverCount > 0
        ? "warning"
        : "ready";

  return `
    <article class="feature-card dashboard-release-checkpoint-signoff">
      <div class="card-topline">
        <span class="eyebrow">Checkpoint signoff</span>
        <span class="badge ${badgeTone(status)}">${escapeHtml(ledger.activeDecision ?? "none")}</span>
      </div>
      <h3>${ledger.activeHoldCount > 0 ? "Checkpoint signoff is holding release" : "Checkpoint signoffs are recorded"}</h3>
      <p>${escapeHtml(ledger.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${ledger.recordCount}</strong><small>records</small></span>
        <span><strong>${ledger.signedOffCount}</strong><small>signed off</small></span>
        <span><strong>${ledger.heldCount}</strong><small>held</small></span>
        <span><strong>${ledger.carriedOverCount}</strong><small>carryovers</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Missing ack evidence</strong> ${ledger.acknowledgementEvidenceMissingCount}</span>
        <span><strong>Gate blocks</strong> ${ledger.releaseGateBlockingCount}</span>
        <span><strong>Active review</strong> ${escapeHtml(ledger.activeReviewId ?? "none")}</span>
        <span><strong>Ledger</strong> ${escapeHtml(ledger.ledgerJson)}</span>
      </div>
      <div class="runner-package-files">
        ${ledger.records
          .slice(-6)
          .reverse()
          .map(
            (record) => `
              <div class="runner-package-file ${
                record.activeHold || record.activeCarryover ? "missing" : "present"
              }">
                <strong>${escapeHtml(record.operator)} - ${escapeHtml(record.reviewId)}</strong>
                <small>${escapeHtml(record.decision)} - ${record.reviewScoreOutOf100}/100</small>
                <code>${escapeHtml(record.acknowledgementEvidencePath || record.reviewJson)}</code>
                <span>${escapeHtml(record.reason)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-checkpoint-signoff-command">
          Copy signoff command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-checkpoint-signoff-notes">
          Copy release notes
        </button>
      </div>
    </article>
  `;
}

function renderReleaseCheckpointEvidenceVault(
  vault: FlowReleaseCheckpointEvidenceVault | null,
) {
  if (!vault) {
    return "";
  }

  const status =
    !vault.readyToArchive ||
    vault.missingCount > 0 ||
    vault.releaseGateBlockingCount > 0 ||
    vault.activeHoldCount > 0
      ? "blocked"
      : vault.activeCarryoverCount > 0
        ? "warning"
        : "ready";

  return `
    <article class="feature-card dashboard-release-checkpoint-evidence-vault">
      <div class="card-topline">
        <span class="eyebrow">Checkpoint evidence vault</span>
        <span class="badge ${badgeTone(status)}">${vault.readyToArchive ? "ready" : "needs evidence"}</span>
      </div>
      <h3>${vault.readyToArchive ? "Checkpoint evidence is archived" : "Checkpoint evidence needs attention"}</h3>
      <p>${escapeHtml(vault.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${vault.entryCount}</strong><small>entries</small></span>
        <span><strong>${vault.presentCount}</strong><small>present</small></span>
        <span><strong>${vault.missingCount}</strong><small>missing</small></span>
        <span><strong>${vault.checksumCount}</strong><small>checksums</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Review</strong> ${escapeHtml(vault.reviewId ?? "missing")}</span>
        <span><strong>Active signoff</strong> ${escapeHtml(vault.activeDecision ?? "none")}</span>
        <span><strong>Manifest</strong> ${escapeHtml(vault.manifestSha256.slice(0, 16))}</span>
        <span><strong>Vault</strong> ${escapeHtml(vault.vaultJson)}</span>
      </div>
      <div class="runner-package-files">
        ${vault.entries
          .slice(0, 8)
          .map(
            (entry) => `
              <div class="runner-package-file ${entry.present ? "present" : "missing"}">
                <strong>${escapeHtml(entry.label)}</strong>
                <small>${escapeHtml(entry.kind)} - ${entry.bytes} bytes</small>
                <code>${escapeHtml(entry.path || "missing")}</code>
                <span>${escapeHtml(entry.warning ?? entry.summary)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-checkpoint-evidence-vault-command">
          Copy vault command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-checkpoint-evidence-vault-notes">
          Copy attachment notes
        </button>
      </div>
    </article>
  `;
}

function renderReleaseEvidenceAttachmentReview(
  review: FlowReleaseEvidenceAttachmentReview | null,
) {
  if (!review) {
    return "";
  }

  const status =
    !review.readyForHandoff ||
    review.blockedCount > 0 ||
    review.missingCount > 0 ||
    review.releaseGateBlockingCount > 0
      ? "blocked"
      : review.inlineOnlyCount > 0 || review.checksumMissingCount > 0
        ? "warning"
        : "ready";

  return `
    <article class="feature-card dashboard-release-evidence-attachment-review">
      <div class="card-topline">
        <span class="eyebrow">Attachment review</span>
        <span class="badge ${badgeTone(status)}">${review.readyForHandoff ? "ready" : "needs review"}</span>
      </div>
      <h3>${review.readyForHandoff ? "Evidence attachments are handoff-ready" : "Evidence attachments need review"}</h3>
      <p>${escapeHtml(review.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${review.itemCount}</strong><small>items</small></span>
        <span><strong>${review.attachableCount}</strong><small>attachable</small></span>
        <span><strong>${review.missingCount}</strong><small>missing</small></span>
        <span><strong>${review.inlineOnlyCount}</strong><small>inline</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Blocked</strong> ${review.blockedCount}</span>
        <span><strong>Checksum missing</strong> ${review.checksumMissingCount}</span>
        <span><strong>Manifest</strong> ${escapeHtml(review.manifestSha256.slice(0, 16))}</span>
        <span><strong>Vault</strong> ${escapeHtml(review.vaultJson)}</span>
      </div>
      ${
        review.firstBlocker
          ? `<p class="soft-warning">${escapeHtml(review.firstBlocker)}</p>`
          : ""
      }
      <div class="runner-package-files">
        ${review.items
          .slice(0, 8)
          .map(
            (item) => `
              <div class="runner-package-file ${item.attachable ? "present" : "missing"}">
                <strong>${escapeHtml(item.label)}</strong>
                <small>${escapeHtml(item.kind)} - ${escapeHtml(item.state)}</small>
                <code>${escapeHtml(item.path || "missing")}</code>
                <span>${escapeHtml(item.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-evidence-attachment-review-command">
          Copy review command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-evidence-attachment-review-notes">
          Copy handoff notes
        </button>
      </div>
    </article>
  `;
}

function renderReleaseHandoffPacket(packet: FlowReleaseHandoffPacket | null) {
  if (!packet) {
    return "";
  }

  const status =
    !packet.readyToSend || packet.unresolvedBlockerCount > 0 || packet.missingCount > 0
      ? "blocked"
      : packet.inlineNoteCount > 0
        ? "warning"
        : "ready";

  return `
    <article class="feature-card dashboard-release-handoff-packet">
      <div class="card-topline">
        <span class="eyebrow">Release handoff packet</span>
        <span class="badge ${badgeTone(status)}">${packet.readyToSend ? "ready" : "blocked"}</span>
      </div>
      <h3>${packet.readyToSend ? "Release handoff is ready" : "Release handoff needs work"}</h3>
      <p>${escapeHtml(packet.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${packet.sectionCount}</strong><small>sections</small></span>
        <span><strong>${packet.attachableFileCount}</strong><small>files</small></span>
        <span><strong>${packet.inlineNoteCount}</strong><small>notes</small></span>
        <span><strong>${packet.unresolvedBlockerCount}</strong><small>blockers</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Included</strong> ${packet.includedCount}</span>
        <span><strong>Missing</strong> ${packet.missingCount}</span>
        <span><strong>Manifest</strong> ${escapeHtml(packet.manifestSha256.slice(0, 16))}</span>
        <span><strong>Packet</strong> ${escapeHtml(packet.packetJson)}</span>
      </div>
      ${packet.firstBlocker ? `<p class="soft-warning">${escapeHtml(packet.firstBlocker)}</p>` : ""}
      <div class="runner-package-files">
        ${packet.sections
          .slice(0, 8)
          .map(
            (section) => `
              <div class="runner-package-file ${section.included ? "present" : "missing"}">
                <strong>${escapeHtml(section.title)}</strong>
                <small>${escapeHtml(section.kind)}</small>
                <code>${escapeHtml(section.path || "inline")}</code>
                <span>${escapeHtml(section.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-handoff-packet-command">
          Copy packet command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-handoff-packet-copy">
          Copy handoff packet
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-handoff-file-checklist">
          Copy file checklist
        </button>
      </div>
    </article>
  `;
}

function renderReleaseHandoffAuditTrail(trail: FlowReleaseHandoffAuditTrail | null) {
  if (!trail) {
    return "";
  }

  const status =
    trail.unresolvedBlockerCount > 0 || trail.blockedCount > 0
      ? "blocked"
      : trail.sentCount > 0 || trail.readyCount > 0
        ? "ready"
        : "warning";

  return `
    <article class="feature-card dashboard-release-handoff-audit-trail">
      <div class="card-topline">
        <span class="eyebrow">Release handoff audit trail</span>
        <span class="badge ${badgeTone(status)}">${trail.latestState ?? "draft"}</span>
      </div>
      <h3>${trail.activePacketId ? "Handoff history is recorded" : "No active handoff packet"}</h3>
      <p>${escapeHtml(trail.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${trail.recordCount}</strong><small>records</small></span>
        <span><strong>${trail.readyCount}</strong><small>ready</small></span>
        <span><strong>${trail.sentCount}</strong><small>sent</small></span>
        <span><strong>${trail.blockedCount}</strong><small>blocked</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Active packet</strong> ${escapeHtml(trail.activePacketId ?? "none")}</span>
        <span><strong>Latest packet</strong> ${escapeHtml(trail.latestPacketId ?? "none")}</span>
        <span><strong>Carryover</strong> ${trail.blockerCarryoverCount}</span>
        <span><strong>Trail</strong> ${escapeHtml(trail.trailJson)}</span>
      </div>
      ${
        trail.unresolvedBlockerCount > 0
          ? `<p class="soft-warning">${trail.unresolvedBlockerCount} unresolved blocker(s) remain on the active handoff packet.</p>`
          : ""
      }
      <div class="runner-package-files">
        ${trail.records
          .slice(-8)
          .reverse()
          .map(
            (record) => `
              <div class="runner-package-file ${record.blockerCarryover > 0 ? "missing" : "present"}">
                <strong>${escapeHtml(record.packetId)}</strong>
                <small>${escapeHtml(record.state)} - ${escapeHtml(record.operator)}</small>
                <code>${escapeHtml(record.packetJson)}</code>
                <span>${escapeHtml(record.acknowledgementNote)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-handoff-audit-command">
          Copy audit command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-handoff-audit-summary">
          Copy audit summary
        </button>
      </div>
    </article>
  `;
}

function renderReleaseHandoffGovernanceReview(
  review: FlowReleaseHandoffGovernanceReview | null,
) {
  if (!review) {
    return "";
  }

  const status = review.approvedForExternalHandoff
    ? "ready"
    : review.releaseGateBlockingCount > 0 || review.blockedCarryoverCount > 0
      ? "blocked"
      : "warning";

  return `
    <article class="feature-card dashboard-release-handoff-governance-review">
      <div class="card-topline">
        <span class="eyebrow">Release handoff governance</span>
        <span class="badge ${badgeTone(status)}">${escapeHtml(review.state)}</span>
      </div>
      <h3>${review.approvedForExternalHandoff ? "External handoff is approved" : "External handoff is on hold"}</h3>
      <p>${escapeHtml(review.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${review.scoreOutOf100}</strong><small>score</small></span>
        <span><strong>${review.findingCount}</strong><small>findings</small></span>
        <span><strong>${review.acknowledgementGapCount}</strong><small>ack gaps</small></span>
        <span><strong>${review.blockedCarryoverCount}</strong><small>carryover</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Latest packet</strong> ${escapeHtml(review.latestPacketId ?? "none")}</span>
        <span><strong>Active packet</strong> ${escapeHtml(review.activePacketId ?? "none")}</span>
        <span><strong>Gate blocks</strong> ${review.releaseGateBlockingCount}</span>
        <span><strong>Review</strong> ${escapeHtml(review.reviewJson)}</span>
      </div>
      ${
        review.releaseGateBlockingCount > 0
          ? `<p class="soft-warning">${review.releaseGateBlockingCount} governance blocker(s) must be resolved before external handoff.</p>`
          : ""
      }
      <div class="runner-package-files">
        ${review.findings
          .slice(0, 8)
          .map(
            (finding) => `
              <div class="runner-package-file ${finding.releaseGateBlocking ? "missing" : "present"}">
                <strong>${escapeHtml(finding.title)}</strong>
                <small>${escapeHtml(finding.source)} - ${escapeHtml(finding.state)}</small>
                <code>${escapeHtml(finding.evidencePath || "inline")}</code>
                <span>${escapeHtml(finding.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-handoff-governance-command">
          Copy governance command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-handoff-governance-notes">
          Copy governance notes
        </button>
      </div>
    </article>
  `;
}

function renderReleaseHandoffDispatchChecklist(
  checklist: FlowReleaseHandoffDispatchChecklist | null,
) {
  if (!checklist) {
    return "";
  }

  const status = checklist.readyToDispatch
    ? "ready"
    : checklist.releaseGateBlockingCount > 0
      ? "blocked"
      : "warning";

  return `
    <article class="feature-card dashboard-release-handoff-dispatch-checklist">
      <div class="card-topline">
        <span class="eyebrow">Release handoff dispatch</span>
        <span class="badge ${badgeTone(status)}">${escapeHtml(checklist.state)}</span>
      </div>
      <h3>${checklist.readyToDispatch ? "Dispatch checklist is ready" : "Dispatch checklist is on hold"}</h3>
      <p>${escapeHtml(checklist.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${checklist.readyCount}/${checklist.itemCount}</strong><small>ready</small></span>
        <span><strong>${checklist.recipientCount}</strong><small>recipients</small></span>
        <span><strong>${checklist.attachmentCount}</strong><small>attachments</small></span>
        <span><strong>${checklist.releaseGateBlockingCount}</strong><small>blocks</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Latest packet</strong> ${escapeHtml(checklist.latestPacketId ?? "none")}</span>
        <span><strong>Active packet</strong> ${escapeHtml(checklist.activePacketId ?? "none")}</span>
        <span><strong>Checklist</strong> ${escapeHtml(checklist.checklistJson)}</span>
        <span><strong>Governance</strong> ${escapeHtml(checklist.governanceReviewJson)}</span>
      </div>
      ${
        checklist.releaseGateBlockingCount > 0
          ? `<p class="soft-warning">${checklist.releaseGateBlockingCount} dispatch blocker(s) remain before any external send.</p>`
          : ""
      }
      <div class="runner-package-files">
        ${checklist.items
          .slice(0, 8)
          .map(
            (item) => `
              <div class="runner-package-file ${item.releaseGateBlocking ? "missing" : "present"}">
                <strong>${escapeHtml(item.title)}</strong>
                <small>${escapeHtml(item.source)} - ${escapeHtml(item.state)}</small>
                <code>${escapeHtml(item.evidencePath || "inline")}</code>
                <span>${escapeHtml(item.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-handoff-dispatch-command">
          Copy checklist command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-handoff-dispatch-checklist">
          Copy dispatch checklist
        </button>
      </div>
    </article>
  `;
}

function renderReleaseHandoffDispatchAuditTrail(
  trail: FlowReleaseHandoffDispatchAuditTrail | null,
) {
  if (!trail) {
    return "";
  }

  const status =
    trail.unresolvedBlockerCount > 0 || trail.blockedCount > 0
      ? "blocked"
      : trail.approvedCount > 0 || trail.sentManuallyCount > 0
        ? "ready"
        : "warning";

  return `
    <article class="feature-card dashboard-release-handoff-dispatch-audit">
      <div class="card-topline">
        <span class="eyebrow">Release handoff dispatch audit</span>
        <span class="badge ${badgeTone(status)}">${escapeHtml(trail.latestState ?? "draft")}</span>
      </div>
      <h3>${trail.unresolvedBlockerCount > 0 ? "Dispatch decision has carryover" : "Dispatch decisions are recorded locally"}</h3>
      <p>${escapeHtml(trail.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${trail.recordCount}</strong><small>records</small></span>
        <span><strong>${trail.approvedCount}</strong><small>approved</small></span>
        <span><strong>${trail.sentManuallyCount}</strong><small>manual sent</small></span>
        <span><strong>${trail.unresolvedBlockerCount}</strong><small>blocks</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Latest checklist</strong> ${escapeHtml(trail.latestChecklistId ?? "none")}</span>
        <span><strong>Active checklist</strong> ${escapeHtml(trail.activeChecklistId ?? "none")}</span>
        <span><strong>Final decisions</strong> ${trail.finalDecisionCount}</span>
        <span><strong>Trail</strong> ${escapeHtml(trail.trailJson)}</span>
      </div>
      ${
        trail.unresolvedBlockerCount > 0
          ? `<p class="soft-warning">${trail.unresolvedBlockerCount} dispatch blocker(s) remain on the active checklist.</p>`
          : ""
      }
      <div class="runner-package-files">
        ${trail.records
          .slice(-8)
          .reverse()
          .map(
            (record) => `
              <div class="runner-package-file ${record.blockerCarryover > 0 ? "missing" : "present"}">
                <strong>${escapeHtml(record.checklistId)}</strong>
                <small>${escapeHtml(record.state)} - ${escapeHtml(record.operator)}</small>
                <code>${escapeHtml(record.checklistJson)}</code>
                <span>${escapeHtml(record.finalDecisionNote)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-handoff-dispatch-audit-command">
          Copy audit command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-handoff-dispatch-audit-summary">
          Copy audit summary
        </button>
      </div>
    </article>
  `;
}

function renderReleaseHandoffDispatchGovernanceReview(
  review: FlowReleaseHandoffDispatchGovernanceReview | null,
) {
  if (!review) {
    return "";
  }

  const status = review.approvedForExternalHandoff
    ? "ready"
    : review.releaseGateBlockingCount > 0 || review.blockedCarryoverCount > 0
      ? "blocked"
      : "warning";

  return `
    <article class="feature-card dashboard-release-handoff-dispatch-governance">
      <div class="card-topline">
        <span class="eyebrow">Release handoff dispatch governance</span>
        <span class="badge ${badgeTone(status)}">${escapeHtml(review.state)}</span>
      </div>
      <h3>${review.approvedForExternalHandoff ? "Dispatch handoff is governed" : "Dispatch handoff remains gated"}</h3>
      <p>${escapeHtml(review.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${review.scoreOutOf100}</strong><small>score</small></span>
        <span><strong>${review.findingCount}</strong><small>findings</small></span>
        <span><strong>${review.finalDecisionGapCount}</strong><small>decision gaps</small></span>
        <span><strong>${review.blockedCarryoverCount}</strong><small>carryover</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Latest checklist</strong> ${escapeHtml(review.latestChecklistId ?? "none")}</span>
        <span><strong>Active checklist</strong> ${escapeHtml(review.activeChecklistId ?? "none")}</span>
        <span><strong>Gate blocks</strong> ${review.releaseGateBlockingCount}</span>
        <span><strong>Review</strong> ${escapeHtml(review.reviewJson)}</span>
      </div>
      ${
        review.releaseGateBlockingCount > 0
          ? `<p class="soft-warning">${review.releaseGateBlockingCount} dispatch governance blocker(s) must be resolved before handoff completion.</p>`
          : ""
      }
      <div class="runner-package-files">
        ${review.findings
          .slice(0, 8)
          .map(
            (finding) => `
              <div class="runner-package-file ${finding.releaseGateBlocking ? "missing" : "present"}">
                <strong>${escapeHtml(finding.title)}</strong>
                <small>${escapeHtml(finding.source)} - ${escapeHtml(finding.state)}</small>
                <code>${escapeHtml(finding.evidencePath || "inline")}</code>
                <span>${escapeHtml(finding.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-handoff-dispatch-governance-command">
          Copy governance command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-handoff-dispatch-governance-notes">
          Copy governance notes
        </button>
      </div>
    </article>
  `;
}

function renderReleaseHandoffCompletionLedger(
  ledger: FlowReleaseHandoffCompletionLedger | null,
) {
  if (!ledger) {
    return "";
  }
  return `
    <article class="feature-card dashboard-release-handoff-completion-ledger">
      <div class="card-topline">
        <span class="eyebrow">Release handoff completion</span>
        <span class="badge ${badgeTone(ledger.blockedOutcomeCount > 0 ? "blocked" : "ready")}">
          ${ledger.latestState ?? "draft"}
        </span>
      </div>
      <h3>Governed local completion ledger</h3>
      <p>${escapeHtml(ledger.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${ledger.recordCount}</strong><small>records</small></span>
        <span><strong>${ledger.approvedOutcomeCount}</strong><small>approved</small></span>
        <span><strong>${ledger.blockedOutcomeCount}</strong><small>blocked</small></span>
        <span><strong>${ledger.releaseGateBlockingCount}</strong><small>gate blocks</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Active completion</strong> ${escapeHtml(ledger.activeCompletionId ?? "none")}</span>
        <span><strong>Latest governance</strong> ${escapeHtml(ledger.latestGovernanceReviewId ?? "none")}</span>
        <span><strong>Governance state</strong> ${escapeHtml(ledger.latestGovernanceState ?? "unknown")}</span>
        <span><strong>Ledger</strong> ${escapeHtml(ledger.ledgerJson)}</span>
      </div>
      ${
        ledger.releaseGateBlockingCount > 0 || ledger.unresolvedBlockerCount > 0
          ? `<p class="soft-warning">This ledger is carrying ${ledger.releaseGateBlockingCount} gate blocker(s) and ${ledger.unresolvedBlockerCount} unresolved blocker(s).</p>`
          : ""
      }
      <div class="runner-package-files">
        ${ledger.records
          .slice()
          .reverse()
          .slice(0, 8)
          .map(
            (record) => `
              <div class="runner-package-file ${
                record.releaseGateBlockingCount > 0 || record.unresolvedBlockerCount > 0
                  ? "missing"
                  : "present"
              }">
                <strong>${escapeHtml(record.operator)} - ${escapeHtml(record.state)}</strong>
                <small>${escapeHtml(record.governanceState)} governance, ${record.governanceScoreOutOf100}/100</small>
                <code>${escapeHtml(record.governanceReviewJson || "inline")}</code>
                <span>${escapeHtml(record.outcomeNote)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-handoff-completion-command">
          Copy completion command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-handoff-completion-summary">
          Copy completion summary
        </button>
      </div>
    </article>
  `;
}

function renderReleasePublicationControl(control: FlowReleasePublicationControl | null) {
  if (!control) {
    return "";
  }
  return `
    <article class="feature-card dashboard-release-publication-control">
      <div class="card-topline">
        <span class="eyebrow">Release publication control</span>
        <span class="badge ${badgeTone(control.status)}">${escapeHtml(control.state)}</span>
      </div>
      <h3>Local-only publication copy</h3>
      <p>${escapeHtml(control.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${control.scoreOutOf100}</strong><small>score</small></span>
        <span><strong>${control.publicationBlockerCount}</strong><small>blockers</small></span>
        <span><strong>${control.approvedOutcomeCount}</strong><small>approved</small></span>
        <span><strong>${control.blockedOutcomeCount}</strong><small>blocked</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Ready</strong> ${control.readyToPublish ? "yes" : "no"}</span>
        <span><strong>Active completion</strong> ${escapeHtml(control.activeCompletionId ?? "none")}</span>
        <span><strong>Latest governance</strong> ${escapeHtml(control.latestGovernanceReviewId ?? "none")}</span>
        <span><strong>Control</strong> ${escapeHtml(control.controlJson)}</span>
      </div>
      ${
        control.releaseGateBlockingCount > 0
          ? `<p class="soft-warning">${control.releaseGateBlockingCount} publication blocker(s) prevent release notes or external-send instructions from leaving local review.</p>`
          : ""
      }
      <div class="runner-package-files">
        ${control.blockers
          .slice(0, 8)
          .map(
            (blocker) => `
              <div class="runner-package-file ${blocker.releaseGateBlocking ? "missing" : "present"}">
                <strong>${escapeHtml(blocker.summary)}</strong>
                <small>${escapeHtml(blocker.kind)}</small>
                <code>${escapeHtml(blocker.evidencePath || "inline")}</code>
                <span>${escapeHtml(blocker.nextAction)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-publication-command">
          Copy publication command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-publication-notes">
          Copy release notes
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-publication-send">
          Copy send instructions
        </button>
      </div>
    </article>
  `;
}

function renderReleaseOutboundReviewLedger(ledger: FlowReleaseOutboundReviewLedger | null) {
  if (!ledger) {
    return "";
  }
  return `
    <article class="feature-card dashboard-release-outbound-review">
      <div class="card-topline">
        <span class="eyebrow">Release outbound review</span>
        <span class="badge ${badgeTone(ledger.blockedReviewCount > 0 ? "blocked" : "ready")}">
          ${escapeHtml(ledger.latestState ?? "draft")}
        </span>
      </div>
      <h3>Operator-reviewed outbound copy</h3>
      <p>${escapeHtml(ledger.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${ledger.recordCount}</strong><small>records</small></span>
        <span><strong>${ledger.reviewedCount}</strong><small>reviewed</small></span>
        <span><strong>${ledger.manualPublicationCount}</strong><small>manual</small></span>
        <span><strong>${ledger.copySafeCount}</strong><small>copy-safe</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Active review</strong> ${escapeHtml(ledger.activeReviewId ?? "none")}</span>
        <span><strong>Latest control</strong> ${escapeHtml(ledger.latestPublicationControlId ?? "none")}</span>
        <span><strong>Publication</strong> ${escapeHtml(ledger.latestPublicationState ?? "none")}</span>
        <span><strong>Ledger</strong> ${escapeHtml(ledger.ledgerJson)}</span>
      </div>
      ${
        ledger.releaseGateBlockingCount > 0 || ledger.unresolvedBlockerCount > 0
          ? `<p class="soft-warning">This outbound review keeps ${ledger.releaseGateBlockingCount} gate blocker(s) and ${ledger.unresolvedBlockerCount} unresolved blocker(s) local.</p>`
          : ""
      }
      <div class="runner-package-files">
        ${ledger.records
          .slice()
          .reverse()
          .slice(0, 8)
          .map(
            (record) => `
              <div class="runner-package-file ${
                record.copySafe && !record.externallyMutatedByFriday ? "present" : "missing"
              }">
                <strong>${escapeHtml(record.reviewer)} - ${escapeHtml(record.state)}</strong>
                <small>${record.copySafe ? "copy-safe" : "local hold"} / ${escapeHtml(record.publicationState)}</small>
                <code>${escapeHtml(record.publicationControlJson || "inline")}</code>
                <span>${escapeHtml(record.reviewNote)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-outbound-review-command">
          Copy review command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-outbound-review-notes">
          Copy review notes
        </button>
      </div>
    </article>
  `;
}

function renderReleaseExternalReceiptArchive(
  archive: FlowReleaseExternalReceiptArchive | null,
) {
  if (!archive) {
    return "";
  }
  return `
    <article class="feature-card dashboard-release-external-receipt">
      <div class="card-topline">
        <span class="eyebrow">Release external receipts</span>
        <span class="badge ${badgeTone(archive.blockedReceiptCount > 0 ? "blocked" : "ready")}">
          ${escapeHtml(archive.latestState ?? "draft")}
        </span>
      </div>
      <h3>Operator-owned receipt archive</h3>
      <p>${escapeHtml(archive.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${archive.recordCount}</strong><small>records</small></span>
        <span><strong>${archive.attachedReceiptCount}</strong><small>attached</small></span>
        <span><strong>${archive.verifiedReceiptCount}</strong><small>verified</small></span>
        <span><strong>${archive.staleOrMissingCount}</strong><small>stale/missing</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Active receipt</strong> ${escapeHtml(archive.activeReceiptId ?? "none")}</span>
        <span><strong>Latest review</strong> ${escapeHtml(archive.latestOutboundReviewId ?? "none")}</span>
        <span><strong>Review state</strong> ${escapeHtml(archive.latestOutboundReviewState ?? "none")}</span>
        <span><strong>Archive</strong> ${escapeHtml(archive.archiveJson)}</span>
      </div>
      ${
        archive.releaseGateBlockingCount > 0 || archive.unresolvedBlockerCount > 0
          ? `<p class="soft-warning">Receipt evidence stays local while ${archive.releaseGateBlockingCount} gate blocker(s) and ${archive.unresolvedBlockerCount} unresolved blocker(s) remain.</p>`
          : ""
      }
      <div class="runner-package-files">
        ${archive.records
          .slice()
          .reverse()
          .slice(0, 8)
          .map(
            (record) => `
              <div class="runner-package-file ${
                record.receiptVerified && !record.externallyMutatedByFriday
                  ? "present"
                  : "missing"
              }">
                <strong>${escapeHtml(record.operator)} - ${escapeHtml(record.receiptKind)}:${escapeHtml(record.state)}</strong>
                <small>${record.receiptAttached ? "attached" : "not attached"} / ${record.receiptVerified ? "verified" : "not verified"}</small>
                <code>${escapeHtml(record.evidencePath ?? record.externalReference ?? "no evidence recorded")}</code>
                <span>${escapeHtml(record.receiptNote)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-external-receipt-command">
          Copy receipt command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-external-receipt-notes">
          Copy audit notes
        </button>
      </div>
    </article>
  `;
}

function renderReleaseReceiptReviewBoard(
  report: FlowReleaseReceiptReviewBoardReport | null,
) {
  if (!report) {
    return "";
  }
  return `
    <article class="feature-card dashboard-release-receipt-review">
      <div class="card-topline">
        <span class="eyebrow">Release receipt review</span>
        <span class="badge ${badgeTone(report.status)}">
          ${escapeHtml(report.decision)}
        </span>
      </div>
      <h3>External completion review board</h3>
      <p>${escapeHtml(report.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${report.scoreOutOf100}</strong><small>score</small></span>
        <span><strong>${report.recordCount}</strong><small>receipts</small></span>
        <span><strong>${report.verifiedReceiptCount}</strong><small>verified</small></span>
        <span><strong>${report.releaseGateBlockingCount}</strong><small>blockers</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Ready</strong> ${report.readyForExternalCompletion ? "yes" : "no"}</span>
        <span><strong>Active receipt</strong> ${escapeHtml(report.activeReceiptId ?? "none")}</span>
        <span><strong>Latest receipt</strong> ${escapeHtml(report.latestReceiptState ?? "none")}</span>
        <span><strong>Latest review</strong> ${escapeHtml(report.latestOutboundReviewId ?? "none")}</span>
        <span><strong>Archive</strong> ${escapeHtml(report.archiveJson)}</span>
      </div>
      ${
        report.releaseGateBlockingCount > 0 || report.unresolvedBlockerCount > 0
          ? `<p class="soft-warning">External completion stays held while ${report.releaseGateBlockingCount} gate blocker(s) or ${report.unresolvedBlockerCount} unresolved blocker(s) remain.</p>`
          : ""
      }
      <div class="runner-package-files">
        ${report.findings
          .slice(0, 8)
          .map(
            (finding) => `
              <div class="runner-package-file ${
                !finding.releaseGateBlocking && finding.decision === "verified"
                  ? "present"
                  : "missing"
              }">
                <strong>${escapeHtml(finding.decision)} - ${escapeHtml(finding.source)}</strong>
                <small>${finding.releaseGateBlocking ? "release gate hold" : "operator review"} / ${escapeHtml(finding.receiptId || "none")}</small>
                <code>${escapeHtml(finding.evidencePath || "no evidence recorded")}</code>
                <span>${escapeHtml(finding.summary)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-receipt-review-command">
          Copy review command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-receipt-review-notes">
          Copy review notes
        </button>
      </div>
    </article>
  `;
}

function renderReleaseClosureLedger(ledger: FlowReleaseClosureLedger | null) {
  if (!ledger) {
    return "";
  }
  return `
    <article class="feature-card dashboard-release-closure-ledger">
      <div class="card-topline">
        <span class="eyebrow">Release closure</span>
        <span class="badge ${badgeTone(ledger.blockedOutcomeCount > 0 ? "blocked" : "ready")}">
          ${escapeHtml(ledger.latestState ?? "draft")}
        </span>
      </div>
      <h3>Local closure ledger</h3>
      <p>${escapeHtml(ledger.summary)}</p>
      <div class="dashboard-history-metrics">
        <span><strong>${ledger.recordCount}</strong><small>records</small></span>
        <span><strong>${ledger.closedCount}</strong><small>closed</small></span>
        <span><strong>${ledger.carryoverCount}</strong><small>carryover</small></span>
        <span><strong>${ledger.blockedCount}</strong><small>blocked</small></span>
      </div>
      <div class="meta-list">
        <span><strong>Active closure</strong> ${escapeHtml(ledger.activeClosureId ?? "none")}</span>
        <span><strong>Latest review</strong> ${escapeHtml(ledger.latestReceiptReviewId ?? "none")}</span>
        <span><strong>Review decision</strong> ${escapeHtml(ledger.latestReviewDecision ?? "none")}</span>
        <span><strong>Ledger</strong> ${escapeHtml(ledger.ledgerJson)}</span>
      </div>
      ${
        ledger.releaseGateBlockingCount > 0 || ledger.unresolvedBlockerCount > 0
          ? `<p class="soft-warning">Release closure stays local while ${ledger.releaseGateBlockingCount} gate blocker(s) and ${ledger.unresolvedBlockerCount} unresolved blocker(s) remain.</p>`
          : ""
      }
      <div class="runner-package-files">
        ${ledger.records
          .slice()
          .reverse()
          .slice(0, 8)
          .map(
            (record) => `
              <div class="runner-package-file ${
                record.state === "closed" && !record.externallyMutatedByFriday
                  ? "present"
                  : "missing"
              }">
                <strong>${escapeHtml(record.operator)} - ${escapeHtml(record.state)}</strong>
                <small>${record.readyForExternalCompletion ? "ready" : "not ready"} / ${escapeHtml(record.reviewDecision)}</small>
                <code>${escapeHtml(record.receiptReviewJson || "inline")}</code>
                <span>${escapeHtml(record.closureNote)}</span>
              </div>
            `,
          )
          .join("")}
      </div>
      <div class="actions">
        <button type="button" class="secondary" data-action="dashboard-release-closure-command">
          Copy closure command
        </button>
        <button type="button" class="secondary" data-action="dashboard-release-closure-summary">
          Copy closure summary
        </button>
      </div>
    </article>
  `;
}

function renderDashboardCard(
  card: FlowDashboardProductUiCardBinding,
  actionStates: UiState["dashboardActionStates"],
) {
  return `
    <article class="feature-card dashboard-card ${card.status}">
      <div class="card-topline">
        <span class="eyebrow">${escapeHtml(card.cardId)}</span>
        <span class="badge ${badgeTone(card.status)}">${escapeHtml(card.status)}</span>
      </div>
      <h3>${escapeHtml(card.title)}</h3>
      <p>${escapeHtml(card.primaryMetric)}</p>
      <div class="meta-list">
        <span><strong>Score</strong> ${card.scoreOutOf100}/100</span>
        <span><strong>Actions</strong> ${card.actionCount}</span>
        <span><strong>Source</strong> ${escapeHtml(card.sourceJson)}</span>
      </div>
      <div class="actions dashboard-actions">
        ${card.actions
          .map((action) => {
            const state = actionStates[action.actionId] ?? "idle";
            return `
              <button
                type="button"
                class="secondary dashboard-action ${state}"
                data-action="dashboard-action"
                data-action-id="${escapeHtml(action.actionId)}"
                aria-label="${escapeHtml(action.buttonState.ariaLabel)}"
                ${action.buttonState.disabled || state === "loading" ? "disabled" : ""}
                title="${escapeHtml(action.command)}"
              >
                ${escapeHtml(dashboardActionLabel(action, state))}
              </button>
            `;
          })
          .join("")}
      </div>
    </article>
  `;
}

function signedDelta(value: number) {
  return value > 0 ? `+${value}` : String(value);
}

function renderDashboardRail(state: UiState) {
  const binding = state.dashboardBinding;
  const history = binding.history;

  return `
    <div class="dashboard-rail">
      <article class="feature-card dashboard-history-card">
        <div class="card-topline">
          <span class="eyebrow">Export history</span>
          <span class="badge ${badgeTone(history.trendLabel === "regressed" ? "blocked" : "ready")}">
            ${escapeHtml(history.trendLabel)}
          </span>
        </div>
        <div class="dashboard-history-metrics">
          <span><strong>${history.recordCount}</strong><small>records</small></span>
          <span><strong>${signedDelta(history.scoreDeltaFromPrevious)}</strong><small>score</small></span>
          <span><strong>${signedDelta(history.readinessDeltaFromPrevious)}</strong><small>readiness</small></span>
        </div>
        <p>
          Latest ${history.latestScoreOutOf100 ?? "n/a"} / 100,
          previous ${history.previousScoreOutOf100 ?? "n/a"} / 100.
        </p>
      </article>

      <article class="feature-card dashboard-links-card">
        <div class="card-topline">
          <span class="eyebrow">Release links</span>
          <span class="badge ${badgeTone("ready")}">${binding.releaseLinks.length} files</span>
        </div>
        <div class="dashboard-link-list">
          ${binding.releaseLinks
            .map(
              (link) => `
                <button
                  type="button"
                  class="secondary dashboard-link"
                  data-action="dashboard-release-link"
                  data-release-link-id="${escapeHtml(link.id)}"
                  title="${escapeHtml(link.path)}"
                  aria-label="${escapeHtml(link.buttonState.ariaLabel)}"
                >
                  <span>${escapeHtml(link.label)}</span>
                  <small>${escapeHtml(link.section)}</small>
                </button>
              `,
            )
            .join("")}
        </div>
      </article>

      <article class="feature-card dashboard-prompts-card">
        <div class="card-topline">
          <span class="eyebrow">Screenshot prompts</span>
          <span class="badge ${badgeTone(binding.screenshotPrompts.length ? "pending" : "ready")}">
            ${binding.screenshotPrompts.length}
          </span>
        </div>
        <div class="dashboard-prompt-list">
          ${binding.screenshotPrompts
            .map(
              (prompt) => `
                <button
                  type="button"
                  class="secondary dashboard-prompt"
                  data-action="dashboard-screenshot-prompt"
                  data-prompt-route="${escapeHtml(prompt.route)}"
                  title="${escapeHtml(prompt.captureCommand)}"
                >
                  <span>${escapeHtml(prompt.title)}</span>
                  <small>${escapeHtml(prompt.route)} - ${escapeHtml(prompt.viewportId)}</small>
                </button>
              `,
            )
            .join("")}
        </div>
      </article>
    </div>
  `;
}

function renderDashboard(state: UiState) {
  const binding = state.dashboardBinding;

  return `
    <section class="section-stack">
      <div class="section-header">
        <div>
          <h2>${escapeHtml(binding.title)}</h2>
          <p>${escapeHtml(binding.summary)}</p>
        </div>
        <div class="actions">
          <input id="dashboard-import-json" type="file" accept="application/json,.json" hidden />
          <input
            id="dashboard-import-host-bridge-json"
            type="file"
            accept="application/json,.json"
            hidden
          />
          <input
            id="dashboard-import-runner-json"
            type="file"
            accept="application/json,.json"
            hidden
          />
          <button type="button" class="secondary" data-action="dashboard-import-click">
            Import JSON
          </button>
          <button type="button" class="secondary" data-action="dashboard-host-bridge-import-click">
            Import host bridge
          </button>
          <button type="button" class="secondary" data-action="dashboard-runner-import-click">
            Import runner
          </button>
          <span class="badge ${badgeTone(binding.status)}">${binding.scoreOutOf100} / 100</span>
        </div>
      </div>

      <article class="hero-card dashboard-hero">
        <div class="card-topline">
          <span class="eyebrow">${escapeHtml(binding.productName)} route contract</span>
          <span class="badge ${badgeTone(binding.status)}">${escapeHtml(binding.status)}</span>
        </div>
        <h3>${escapeHtml(binding.route)} is reading typed dashboard data</h3>
        <p>
          ${binding.boundCardCount}/${binding.cardCount} cards are bound from the dashboard
          product UI contract, with ${binding.actionCount} local action handoff(s) ready for the
          next wiring pass.
        </p>
        <div class="hero-facts">
          <span class="pill">${escapeHtml(binding.sourceLabel)}</span>
          <span class="pill">${binding.localOnly ? "Local only" : "Remote allowed"}</span>
          <span class="pill">${binding.fallback ? "Offline fallback" : "Imported JSON"}</span>
          <span class="pill">${escapeHtml(binding.sourceFile)}</span>
          <span class="pill">${escapeHtml(binding.panelJsonCommand)}</span>
          <span class="pill">${escapeHtml(binding.exportCommand)}</span>
        </div>
      </article>

      <div class="card-grid dashboard-grid">
        ${binding.cards
          .map((card) => renderDashboardCard(card, state.dashboardActionStates))
          .join("")}
      </div>

      ${renderDashboardRail(state)}

      ${renderDashboardActionResults(state.dashboardActionResults, state.dashboardRunnerUx)}
      ${renderRunnerApprovalModal(
        state.dashboardRunnerApprovalUi,
        state.dashboardRunnerApprovalReason,
      )}
      ${renderLiveRunnerState(
        state.dashboardLiveRunnerState,
        state.dashboardRunnerCancellationUx,
        state.dashboardRunnerCancellationDrafts,
      )}
      ${renderRunnerOperatorReview(state.dashboardRunnerOperatorReview)}
      ${renderRunnerReleasePackage(state.dashboardRunnerReleasePackage)}
      ${renderRunnerReleaseTimeline(state.dashboardRunnerReleaseTimeline)}
      ${renderReleaseChecklist(
        state.dashboardReleaseChecklist,
        state.dashboardReleaseChecklistReason,
      )}
      ${renderReleaseQa(state.dashboardReleaseQa)}
      ${renderReleaseExportKit(state.dashboardReleaseExportKit)}
      ${renderReleaseDeploymentGate(state.dashboardReleaseDeploymentGate)}
      ${renderReleaseCandidateArchive(state.dashboardReleaseCandidateArchive)}
      ${renderReleasePromotionLedger(state.dashboardReleasePromotionLedger)}
      ${renderReleasePostPromotionMonitor(state.dashboardReleasePostPromotionMonitor)}
      ${renderReleaseRollbackDrill(state.dashboardReleaseRollbackDrill)}
      ${renderReleaseStabilityBoard(state.dashboardReleaseStabilityBoard)}
      ${renderReleaseRecoveryRunbook(state.dashboardReleaseRecoveryRunbook)}
      ${renderReleaseIncidentArchive(state.dashboardReleaseIncidentArchive)}
      ${renderReleasePreventionPlan(state.dashboardReleasePreventionPlan)}
      ${renderReleaseOwnerFollowUpBoard(state.dashboardReleaseOwnerFollowUpBoard)}
      ${renderReleaseEvidenceSlaMonitor(state.dashboardReleaseEvidenceSlaMonitor)}
      ${renderReleaseEscalationLedger(state.dashboardReleaseEscalationLedger)}
      ${renderReleaseCheckpointReview(state.dashboardReleaseCheckpointReview)}
      ${renderReleaseCheckpointSignoffLedger(state.dashboardReleaseCheckpointSignoffLedger)}
      ${renderReleaseCheckpointEvidenceVault(state.dashboardReleaseCheckpointEvidenceVault)}
      ${renderReleaseEvidenceAttachmentReview(state.dashboardReleaseEvidenceAttachmentReview)}
      ${renderReleaseHandoffPacket(state.dashboardReleaseHandoffPacket)}
      ${renderReleaseHandoffAuditTrail(state.dashboardReleaseHandoffAuditTrail)}
      ${renderReleaseHandoffGovernanceReview(state.dashboardReleaseHandoffGovernanceReview)}
      ${renderReleaseHandoffDispatchChecklist(state.dashboardReleaseHandoffDispatchChecklist)}
      ${renderReleaseHandoffDispatchAuditTrail(state.dashboardReleaseHandoffDispatchAuditTrail)}
      ${renderReleaseHandoffDispatchGovernanceReview(state.dashboardReleaseHandoffDispatchGovernanceReview)}
      ${renderReleaseHandoffCompletionLedger(state.dashboardReleaseHandoffCompletionLedger)}
      ${renderReleasePublicationControl(state.dashboardReleasePublicationControl)}
      ${renderReleaseOutboundReviewLedger(state.dashboardReleaseOutboundReviewLedger)}
      ${renderReleaseExternalReceiptArchive(state.dashboardReleaseExternalReceiptArchive)}
      ${renderReleaseReceiptReviewBoard(state.dashboardReleaseReceiptReviewBoard)}
      ${renderReleaseClosureLedger(state.dashboardReleaseClosureLedger)}

      <article class="feature-card">
        <div class="card-topline">
          <span class="eyebrow">Next wiring targets</span>
          <span class="badge ${badgeTone("pending")}">local only</span>
        </div>
        <div class="note-list">
          ${binding.nextActions.map((action) => `<span>${escapeHtml(action)}</span>`).join("")}
        </div>
      </article>
    </section>
  `;
}

function renderShell(surface: FlowSurface, state: UiState, engine: FlowBrowserEngine) {
  const copy = SURFACE_COPY[surface];
  const sections = copy.sections;
  const activeSection = sections.includes(state.activeSection)
    ? state.activeSection
    : sections[0];

  let body = "";
  switch (activeSection) {
    case "dashboard":
      body = renderDashboard(state);
      break;
    case "workspace":
      body = renderWorkspace(state, engine);
      break;
    case "packs":
      body = renderPacks(state, engine);
      break;
    case "settings":
      body = renderSettings(state, engine);
      break;
    case "delivery":
      body = renderDelivery(surface, state);
      break;
    default:
      body = renderOverview(surface, state, engine);
      break;
  }

  return `
    <div class="app-shell ${surface}">
      <header class="app-header">
        <div>
          <span class="eyebrow">${escapeHtml(copy.eyebrow)}</span>
          <div class="title-row">
            <h1>${escapeHtml(copy.title)}</h1>
            <span class="pill">${escapeHtml(surface)}</span>
          </div>
        </div>
        <div class="header-actions">
          <button type="button" class="secondary" data-action="go-settings">Settings</button>
          <button type="button" class="secondary" data-action="go-packs">Model packs</button>
        </div>
      </header>

      <nav class="tab-bar" aria-label="Flow workspace sections">
        ${sections
          .map(
            (section) => `
              <button
                type="button"
                class="tab ${section === activeSection ? "active" : ""}"
                data-action="change-section"
                data-section="${section}"
              >
                ${escapeHtml(SECTION_LABELS[section])}
              </button>
            `,
          )
          .join("")}
      </nav>

      ${body}
    </div>
  `;
}

export async function mountFlowApp(surfaceInput: string) {
  const root = document.getElementById("flow-app");
  if (!root) {
    return;
  }
  const mountRoot = root;

  const surface = normalizeSurface(surfaceInput);
  const engine = new FlowBrowserEngine();
  const [settings, draft, readiness] = await Promise.all([
    engine.settings(),
    engine.workbenchDraft(),
    engine.runtimeReadiness(),
  ]);

  const initialTask = draft.task || settings.defaultTask;
  const dashboardBinding = engine.dashboardBinding();
  const state: UiState = {
    settings,
    draft: {
      task: initialTask,
      prompt: draft.prompt || engine.defaultPrompt(initialTask),
      imageSources: draft.imageSources ?? "",
    },
    readiness,
    quickContext: null,
    dashboardBinding,
    activeSection: defaultSection(surface),
    status: "Ready. Flow will stay local-first unless you deliberately wire remote providers later.",
    output: "",
    running: false,
    lastPlan: null,
    lastPack: null,
    dashboardActionStates: {},
    dashboardActionResults: readDashboardCommandResults(dashboardBinding.route),
    dashboardRunnerUx: null,
    dashboardRunnerApprovalUi: null,
    dashboardRunnerApprovalReason: "",
    dashboardLiveRunnerState: null,
    dashboardRunnerCancellationUx: null,
    dashboardRunnerCancellationDrafts: readRunnerCancellationDrafts(),
    dashboardRunnerOperatorReview: null,
    dashboardRunnerReleasePackage: null,
    dashboardRunnerReleaseTimeline: null,
    dashboardReleaseChecklist: null,
    dashboardReleaseChecklistReason: "",
    dashboardReleaseQa: null,
    dashboardReleaseExportKit: null,
    dashboardReleaseDeploymentGate: null,
    dashboardReleaseCandidateArchive: null,
    dashboardReleasePromotionLedger: null,
    dashboardReleasePostPromotionMonitor: null,
    dashboardReleaseRollbackDrill: null,
    dashboardReleaseStabilityBoard: null,
    dashboardReleaseRecoveryRunbook: null,
    dashboardReleaseIncidentArchive: null,
    dashboardReleasePreventionPlan: null,
    dashboardReleaseOwnerFollowUpBoard: null,
    dashboardReleaseEvidenceSlaMonitor: null,
    dashboardReleaseEscalationLedger: null,
    dashboardReleaseCheckpointReview: null,
    dashboardReleaseCheckpointSignoffLedger: null,
    dashboardReleaseCheckpointEvidenceVault: null,
    dashboardReleaseEvidenceAttachmentReview: null,
    dashboardReleaseHandoffPacket: null,
    dashboardReleaseHandoffAuditTrail: null,
    dashboardReleaseHandoffGovernanceReview: null,
    dashboardReleaseHandoffDispatchChecklist: null,
    dashboardReleaseHandoffDispatchAuditTrail: null,
    dashboardReleaseHandoffDispatchGovernanceReview: null,
    dashboardReleaseHandoffCompletionLedger: null,
    dashboardReleasePublicationControl: null,
    dashboardReleaseOutboundReviewLedger: null,
    dashboardReleaseExternalReceiptArchive: null,
    dashboardReleaseReceiptReviewBoard: null,
    dashboardReleaseClosureLedger: null,
  };

  function render() {
    mountRoot.innerHTML = renderShell(surface, state, engine);
    bind();
  }

  async function refreshRuntimeStatus(statusMessage?: string) {
    if (statusMessage) {
      state.status = statusMessage;
    }
    state.readiness = await engine.runtimeReadiness(state.settings.localOnly);
  }

  async function refreshQuickContext(silent = false) {
    if (!silent) {
      state.status = "Refreshing active tab context...";
      render();
    }

    state.quickContext = await requestQuickContext();
    state.status = state.quickContext
      ? `Captured context from ${state.quickContext.title || "the active tab"}.`
      : "No active tab context was available.";
  }

  async function updateSettings(patch: Partial<FlowUiSettings>) {
    state.settings = await engine.saveSettings(patch);
    await refreshRuntimeStatus("Saved browser settings.");
    render();
  }

  async function updateDraft(patch: Partial<FlowWorkbenchDraft>, rerender = false) {
    state.draft = await engine.saveDraft(patch);
    if (rerender) {
      render();
    }
  }

  async function installPack(modelKey: string) {
    state.status = "Preparing model pack...";
    render();

    try {
      await engine.ensurePack(modelKey, (message) => {
        const statusEl = mountRoot.querySelector<HTMLPreElement>("#flow-status");
        if (statusEl) {
          statusEl.textContent = message;
        }
      });
      await refreshRuntimeStatus("Model pack is ready for offline local use.");
    } catch (error) {
      state.status = `Pack error: ${String(error)}`;
    }

    render();
  }

  async function removePack(modelKey: string) {
    state.status = "Removing cached pack files...";
    render();

    await engine.removePack(modelKey);
    await refreshRuntimeStatus("Cached pack files were removed.");
    render();
  }

  async function copyOutput() {
    if (!state.output.trim()) {
      return;
    }

    if (!navigator.clipboard?.writeText) {
      state.status = "Clipboard access is unavailable in this browser surface.";
      render();
      return;
    }

    await navigator.clipboard.writeText(state.output);
    state.status = "Copied local output to the clipboard.";
    render();
  }

  async function applyOutput() {
    if (!state.output.trim()) {
      return;
    }

    const applied = await replaceSelection(state.output);
    state.status = applied
      ? "Applied the local output back into the active page."
      : "Could not apply the local output to the page selection.";
    render();
  }

  async function runFlow(taskOverride?: FlowTask) {
    const task = taskOverride ?? state.draft.task;
    const previousTask = state.draft.task;
    const wasDefaultPrompt =
      !state.draft.prompt.trim() ||
      state.draft.prompt.trim() === engine.defaultPrompt(previousTask);

    if (task !== previousTask) {
      const nextPrompt = wasDefaultPrompt
        ? engine.defaultPrompt(task)
        : state.draft.prompt;
      state.draft = await engine.saveDraft({
        task,
        prompt: nextPrompt,
        imageSources: state.draft.imageSources,
      });
      state.activeSection = "workspace";
      render();
    }

    if (state.settings.captureActiveTabContext) {
      await refreshQuickContext(true);
    }

    const modality = engine.modalityForTask(task);
    const preferredModel = engine.preferredModelForTask(task, state.settings);
    const prompt = state.draft.prompt.trim() || engine.defaultPrompt(task);
    const imageSources = state.draft.imageSources
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean);

    if ((task === "ocr-image" || task === "multimodal-ask") && imageSources.length === 0) {
      state.status = "Add at least one image URL before running OCR or multimodal tasks.";
      state.activeSection = "workspace";
      render();
      return;
    }

    const request: FlowInferenceRequest = {
      task,
      modality,
      prompt,
      selectionText: state.quickContext?.selectionText,
      pageText: state.quickContext?.pageText,
      imageSources,
      localOnly: state.settings.localOnly,
      preferredModel,
    };

    state.running = true;
    state.output = "";
    state.status = "Planning local execution...";
    state.activeSection = "workspace";
    render();

    try {
      const outputEl = () => mountRoot.querySelector<HTMLPreElement>("#flow-output");
      const result = await engine.run(request, (chunk) => {
        state.output += chunk;
        const el = outputEl();
        if (el) {
          el.textContent = state.output;
        }
      });

      if (!state.output.trim()) {
        state.output = result.output;
      }

      state.lastPlan = result.plan;
      state.lastPack = result.pack;
      state.status = [
        `Model ${result.plan.selectedModel ?? "unknown"}`,
        `Pack ${result.pack.displayName}`,
        `Target ${result.plan.deviceTarget}`,
      ].join(" | ");

      if (
        task === "rewrite-selection" &&
        state.settings.autoApplyRewrite &&
        state.quickContext?.selectionText &&
        state.output.trim()
      ) {
        const applied = await replaceSelection(state.output);
        state.status = applied
          ? `${state.status} | rewrite applied to page`
          : `${state.status} | could not apply rewrite automatically`;
      }

      await refreshRuntimeStatus(state.status);
    } catch (error) {
      state.status = `Error: ${String(error)}`;
    } finally {
      state.running = false;
      render();
    }
  }

  async function runDashboardAction(actionId: string) {
    const action = state.dashboardBinding.cards
      .flatMap((card) => card.actions)
      .find((item) => item.actionId === actionId);

    if (!action || action.buttonState.disabled) {
      return;
    }

    state.dashboardActionStates = { ...state.dashboardActionStates, [actionId]: "loading" };
    state.status = `Checking local dashboard permission: ${action.command}`;
    render();

    const needsConfirmation =
      action.buttonState.requiresConfirmation || action.buttonState.destructive;
    const confirmed =
      !needsConfirmation ||
      globalThis.confirm?.(`Prepare this local dashboard command?\n\n${action.command}`) === true;
    const result = dispatchDashboardCommand(action, { confirmed });

    if (result.status === "prepared") {
      try {
        await navigator.clipboard?.writeText(result.command);
        state.status = `${result.message} Command copied to clipboard.`;
      } catch {
        state.status = `${result.message} Clipboard copy is unavailable here.`;
      }
    } else {
      state.status = result.message;
    }

    state.dashboardActionStates = {
      ...state.dashboardActionStates,
      [actionId]:
        result.status === "prepared" ? "success" : result.status === "failed" ? "error" : "blocked",
    };
    state.dashboardActionResults = persistDashboardCommandResult(
      state.dashboardBinding.route,
      result,
      state.dashboardActionResults,
    );
    render();
  }

  function showDashboardReleaseLink(linkId: string) {
    const link = state.dashboardBinding.releaseLinks.find((item) => item.id === linkId);
    if (!link) {
      return;
    }

    state.status = `Local artifact ready: ${link.path}`;
    render();
  }

  function showDashboardScreenshotPrompt(route: string) {
    const prompt = state.dashboardBinding.screenshotPrompts.find((item) => item.route === route);
    if (!prompt) {
      return;
    }

    state.status = `Capture prompt: ${prompt.captureCommand}`;
    render();
  }

  async function importDashboardJson(file: File) {
    try {
      const text = await file.text();
      state.dashboardBinding = normalizeFridayDashboardBinding(JSON.parse(text), file.name);
      state.dashboardActionStates = {};
      state.dashboardActionResults = readDashboardCommandResults(state.dashboardBinding.route);
      state.status = `Imported local dashboard JSON from ${file.name}.`;
    } catch (error) {
      state.status = `Dashboard import failed: ${String(error)}`;
    }

    render();
  }

  async function importDashboardHostBridgeJson(file: File) {
    try {
      const text = await file.text();
      const results = normalizeDashboardHostCommandResults(JSON.parse(text));
      state.dashboardActionResults = results;
      state.status = `Imported ${results.length} host bridge command result(s) from ${file.name}.`;
    } catch (error) {
      state.status = `Host bridge import failed: ${String(error)}`;
    }

    render();
  }

  async function importDashboardRunnerJson(file: File) {
    try {
      const text = await file.text();
      const parsed = JSON.parse(text) as unknown;
      const results = normalizeTrustedHostRunnerResults(parsed);
      const runnerUx = normalizeTrustedHostRunnerUx(parsed);
      const approvalUi = normalizeTrustedHostRunnerApprovalUi(parsed);
      const liveState = normalizeTrustedHostLiveRunnerState(parsed);
      const operatorReview = normalizeTrustedHostRunnerOperatorReview(parsed);
      const releasePackage = normalizeTrustedRunnerReleasePackage(parsed);
      const releaseTimeline = normalizeTrustedRunnerReleaseTimeline(parsed);
      const releaseChecklist = normalizeReleaseOperatorChecklist(parsed);
      const releaseQa = normalizeReleaseQaCommandCenter(parsed);
      const releaseExportKit = normalizeReleaseEvidenceExportKit(parsed);
      const releaseDeploymentGate = normalizeReleaseDeploymentGate(parsed);
      const releaseCandidateArchive = normalizeReleaseCandidateArchive(parsed);
      const releasePromotionLedger = normalizeReleasePromotionLedger(parsed);
      const releasePostPromotionMonitor = normalizeReleasePostPromotionMonitor(parsed);
      const releaseRollbackDrill = normalizeReleaseRollbackDrill(parsed);
      const releaseStabilityBoard = normalizeReleaseStabilityBoard(parsed);
      const releaseRecoveryRunbook = normalizeReleaseRecoveryRunbook(parsed);
      const releaseIncidentArchive = normalizeReleaseIncidentArchive(parsed);
      const releasePreventionPlan = normalizeReleasePreventionPlan(parsed);
      const releaseOwnerFollowUpBoard = normalizeReleaseOwnerFollowUpBoard(parsed);
      const releaseEvidenceSlaMonitor = normalizeReleaseEvidenceSlaMonitor(parsed);
      const releaseEscalationLedger = normalizeReleaseEscalationLedger(parsed);
      const releaseCheckpointReview = normalizeReleaseCheckpointReview(parsed);
      const releaseCheckpointSignoffLedger =
        normalizeReleaseCheckpointSignoffLedger(parsed);
      const releaseCheckpointEvidenceVault =
        normalizeReleaseCheckpointEvidenceVault(parsed);
      const releaseEvidenceAttachmentReview =
        normalizeReleaseEvidenceAttachmentReview(parsed);
      const releaseHandoffPacket = normalizeReleaseHandoffPacket(parsed);
      const releaseHandoffAuditTrail = normalizeReleaseHandoffAuditTrail(parsed);
      const releaseHandoffGovernanceReview =
        normalizeReleaseHandoffGovernanceReview(parsed);
      const releaseHandoffDispatchChecklist =
        normalizeReleaseHandoffDispatchChecklist(parsed);
      const releaseHandoffDispatchAuditTrail =
        normalizeReleaseHandoffDispatchAuditTrail(parsed);
      const releaseHandoffDispatchGovernanceReview =
        normalizeReleaseHandoffDispatchGovernanceReview(parsed);
      const releaseHandoffCompletionLedger =
        normalizeReleaseHandoffCompletionLedger(parsed);
      const releasePublicationControl =
        normalizeReleasePublicationControl(parsed);
      const releaseOutboundReviewLedger =
        normalizeReleaseOutboundReviewLedger(parsed);
      const releaseExternalReceiptArchive =
        normalizeReleaseExternalReceiptArchive(parsed);
      const releaseReceiptReviewBoard =
        normalizeReleaseReceiptReviewBoard(parsed);
      const releaseClosureLedger =
        normalizeReleaseClosureLedger(parsed);
      const cancellationUx =
        normalizeTrustedHostRunnerCancellationUx(parsed) ??
        (liveState ? buildTrustedHostRunnerCancellationUx(liveState) : null);
      state.dashboardRunnerUx = runnerUx ?? state.dashboardRunnerUx;
      state.dashboardRunnerApprovalUi = approvalUi ?? state.dashboardRunnerApprovalUi;
      state.dashboardLiveRunnerState = liveState ?? state.dashboardLiveRunnerState;
      state.dashboardRunnerCancellationUx =
        cancellationUx ?? state.dashboardRunnerCancellationUx;
      state.dashboardRunnerOperatorReview =
        operatorReview ?? state.dashboardRunnerOperatorReview;
      state.dashboardRunnerReleasePackage =
        releasePackage ?? state.dashboardRunnerReleasePackage;
      state.dashboardRunnerReleaseTimeline =
        releaseTimeline ?? state.dashboardRunnerReleaseTimeline;
      state.dashboardReleaseChecklist =
        releaseChecklist ?? state.dashboardReleaseChecklist;
      state.dashboardReleaseQa = releaseQa ?? state.dashboardReleaseQa;
      state.dashboardReleaseExportKit =
        releaseExportKit ?? state.dashboardReleaseExportKit;
      state.dashboardReleaseDeploymentGate =
        releaseDeploymentGate ?? state.dashboardReleaseDeploymentGate;
      state.dashboardReleaseCandidateArchive =
        releaseCandidateArchive ?? state.dashboardReleaseCandidateArchive;
      state.dashboardReleasePromotionLedger =
        releasePromotionLedger ?? state.dashboardReleasePromotionLedger;
      state.dashboardReleasePostPromotionMonitor =
        releasePostPromotionMonitor ?? state.dashboardReleasePostPromotionMonitor;
      state.dashboardReleaseRollbackDrill =
        releaseRollbackDrill ?? state.dashboardReleaseRollbackDrill;
      state.dashboardReleaseStabilityBoard =
        releaseStabilityBoard ?? state.dashboardReleaseStabilityBoard;
      state.dashboardReleaseRecoveryRunbook =
        releaseRecoveryRunbook ?? state.dashboardReleaseRecoveryRunbook;
      state.dashboardReleaseIncidentArchive =
        releaseIncidentArchive ?? state.dashboardReleaseIncidentArchive;
      state.dashboardReleasePreventionPlan =
        releasePreventionPlan ?? state.dashboardReleasePreventionPlan;
      state.dashboardReleaseOwnerFollowUpBoard =
        releaseOwnerFollowUpBoard ?? state.dashboardReleaseOwnerFollowUpBoard;
      state.dashboardReleaseEvidenceSlaMonitor =
        releaseEvidenceSlaMonitor ?? state.dashboardReleaseEvidenceSlaMonitor;
      state.dashboardReleaseEscalationLedger =
        releaseEscalationLedger ?? state.dashboardReleaseEscalationLedger;
      state.dashboardReleaseCheckpointReview =
        releaseCheckpointReview ?? state.dashboardReleaseCheckpointReview;
      state.dashboardReleaseCheckpointSignoffLedger =
        releaseCheckpointSignoffLedger ?? state.dashboardReleaseCheckpointSignoffLedger;
      state.dashboardReleaseCheckpointEvidenceVault =
        releaseCheckpointEvidenceVault ?? state.dashboardReleaseCheckpointEvidenceVault;
      state.dashboardReleaseEvidenceAttachmentReview =
        releaseEvidenceAttachmentReview ?? state.dashboardReleaseEvidenceAttachmentReview;
      state.dashboardReleaseHandoffPacket =
        releaseHandoffPacket ?? state.dashboardReleaseHandoffPacket;
      state.dashboardReleaseHandoffAuditTrail =
        releaseHandoffAuditTrail ?? state.dashboardReleaseHandoffAuditTrail;
      state.dashboardReleaseHandoffGovernanceReview =
        releaseHandoffGovernanceReview ?? state.dashboardReleaseHandoffGovernanceReview;
      state.dashboardReleaseHandoffDispatchChecklist =
        releaseHandoffDispatchChecklist ?? state.dashboardReleaseHandoffDispatchChecklist;
      state.dashboardReleaseHandoffDispatchAuditTrail =
        releaseHandoffDispatchAuditTrail ?? state.dashboardReleaseHandoffDispatchAuditTrail;
      state.dashboardReleaseHandoffDispatchGovernanceReview =
        releaseHandoffDispatchGovernanceReview ??
        state.dashboardReleaseHandoffDispatchGovernanceReview;
      state.dashboardReleaseHandoffCompletionLedger =
        releaseHandoffCompletionLedger ?? state.dashboardReleaseHandoffCompletionLedger;
      state.dashboardReleasePublicationControl =
        releasePublicationControl ?? state.dashboardReleasePublicationControl;
      state.dashboardReleaseOutboundReviewLedger =
        releaseOutboundReviewLedger ?? state.dashboardReleaseOutboundReviewLedger;
      state.dashboardReleaseExternalReceiptArchive =
        releaseExternalReceiptArchive ?? state.dashboardReleaseExternalReceiptArchive;
      state.dashboardReleaseReceiptReviewBoard =
        releaseReceiptReviewBoard ?? state.dashboardReleaseReceiptReviewBoard;
      state.dashboardReleaseClosureLedger =
        releaseClosureLedger ?? state.dashboardReleaseClosureLedger;
      state.dashboardActionResults = [...results, ...state.dashboardActionResults].slice(0, 8);
      state.status = releaseClosureLedger
        ? `Imported release closure ledger with ${releaseClosureLedger.recordCount} record(s) from ${file.name}.`
        : releaseReceiptReviewBoard
        ? `Imported release receipt review board ${releaseReceiptReviewBoard.decision} at ${releaseReceiptReviewBoard.scoreOutOf100}/100 from ${file.name}.`
        : releaseExternalReceiptArchive
        ? `Imported release external receipt archive with ${releaseExternalReceiptArchive.recordCount} record(s) from ${file.name}.`
        : releaseOutboundReviewLedger
        ? `Imported release outbound review ledger with ${releaseOutboundReviewLedger.recordCount} record(s) from ${file.name}.`
        : releasePublicationControl
        ? `Imported release publication control ${releasePublicationControl.state} at ${releasePublicationControl.scoreOutOf100}/100 from ${file.name}.`
        : releaseHandoffCompletionLedger
        ? `Imported release handoff completion ledger with ${releaseHandoffCompletionLedger.recordCount} record(s) from ${file.name}.`
        : releaseHandoffDispatchGovernanceReview
        ? `Imported release handoff dispatch governance at ${releaseHandoffDispatchGovernanceReview.scoreOutOf100}/100 from ${file.name}.`
        : releaseHandoffDispatchAuditTrail
        ? `Imported release handoff dispatch audit with ${releaseHandoffDispatchAuditTrail.recordCount} record(s) from ${file.name}.`
        : releaseHandoffDispatchChecklist
        ? `Imported release handoff dispatch checklist with ${releaseHandoffDispatchChecklist.itemCount} item(s) from ${file.name}.`
        : releaseHandoffGovernanceReview
        ? `Imported release handoff governance review at ${releaseHandoffGovernanceReview.scoreOutOf100}/100 from ${file.name}.`
        : releaseHandoffAuditTrail
        ? `Imported release handoff audit trail with ${releaseHandoffAuditTrail.recordCount} record(s) from ${file.name}.`
        : releaseHandoffPacket
        ? `Imported release handoff packet with ${releaseHandoffPacket.sectionCount} section(s) from ${file.name}.`
        : releaseEvidenceAttachmentReview
        ? `Imported evidence attachment review with ${releaseEvidenceAttachmentReview.itemCount} item(s) from ${file.name}.`
        : releaseCheckpointEvidenceVault
        ? `Imported checkpoint evidence vault with ${releaseCheckpointEvidenceVault.entryCount} entries from ${file.name}.`
        : releaseCheckpointSignoffLedger
        ? `Imported checkpoint signoff ledger with ${releaseCheckpointSignoffLedger.recordCount} record(s) from ${file.name}.`
        : releaseCheckpointReview
        ? `Imported release checkpoint review ${releaseCheckpointReview.decision} at ${releaseCheckpointReview.scoreOutOf100}/100 from ${file.name}.`
        : releaseEscalationLedger
        ? `Imported release escalation ledger with ${releaseEscalationLedger.entryCount} record(s) from ${file.name}.`
        : releaseEvidenceSlaMonitor
        ? `Imported release evidence SLA monitor at ${releaseEvidenceSlaMonitor.scoreOutOf100}/100 from ${file.name}.`
        : releaseOwnerFollowUpBoard
        ? `Imported owner follow-up board at ${releaseOwnerFollowUpBoard.scoreOutOf100}/100 from ${file.name}.`
        : releasePreventionPlan
        ? `Imported prevention plan at ${releasePreventionPlan.scoreOutOf100}/100 from ${file.name}.`
        : releaseIncidentArchive
        ? `Imported incident archive with ${releaseIncidentArchive.incidentCount} incident(s) from ${file.name}.`
        : releaseRecoveryRunbook
        ? `Imported recovery runbook at ${releaseRecoveryRunbook.scoreOutOf100}/100 from ${file.name}.`
        : releaseStabilityBoard
        ? `Imported stability board at ${releaseStabilityBoard.scoreOutOf100}/100 from ${file.name}.`
        : releaseRollbackDrill
        ? `Imported rollback drill at ${releaseRollbackDrill.scoreOutOf100}/100 from ${file.name}.`
        : releasePostPromotionMonitor
        ? `Imported post-promotion monitor at ${releasePostPromotionMonitor.scoreOutOf100}/100 from ${file.name}.`
        : releasePromotionLedger
        ? `Imported release promotion ledger with ${releasePromotionLedger.recordCount} record(s) from ${file.name}.`
        : releaseCandidateArchive
        ? `Imported release candidate archive with ${releaseCandidateArchive.candidateCount} candidate(s) from ${file.name}.`
        : releaseDeploymentGate
        ? `Imported deployment gate ${releaseDeploymentGate.decision} at ${releaseDeploymentGate.scoreOutOf100}/100 from ${file.name}.`
        : releaseExportKit
        ? `Imported release evidence export kit with ${releaseExportKit.manifest.fileCount} file(s) from ${file.name}.`
        : releaseQa
        ? `Imported release QA command center at ${releaseQa.scoreOutOf100}/100 from ${file.name}.`
        : releaseChecklist
        ? `Imported release checklist with ${releaseChecklist.blockingCount} blocking issue(s) from ${file.name}.`
        : releaseTimeline
        ? `Imported trusted runner release timeline with ${releaseTimeline.packageCount} package(s) from ${file.name}.`
        : releasePackage
        ? `Imported trusted runner release package with ${releasePackage.manifest.evidenceCount} evidence item(s) from ${file.name}.`
        : operatorReview
        ? `Imported trusted runner operator review with ${operatorReview.matchedCount} matched record(s) from ${file.name}.`
        : cancellationUx
        ? `Imported trusted runner cancellation UX with ${cancellationUx.controls.length} control(s) from ${file.name}.`
        : liveState
        ? `Imported live trusted runner state with ${liveState.recordCount} tracked record(s) from ${file.name}.`
        : approvalUi
        ? `Imported trusted runner approval UI with ${approvalUi.controls.length} control(s) from ${file.name}.`
        : runnerUx
        ? `Imported trusted runner UX with ${runnerUx.resultCount} history result(s) from ${file.name}.`
        : `Imported ${results.length} trusted runner result(s) from ${file.name}.`;
    } catch (error) {
      state.status = `Trusted runner import failed: ${String(error)}`;
    }

    render();
  }

  function updateRunnerCancellationDraft(controlId: string, reason: string) {
    state.dashboardRunnerCancellationDrafts = {
      ...state.dashboardRunnerCancellationDrafts,
      [controlId]: reason,
    };
    writeRunnerCancellationDrafts(state.dashboardRunnerCancellationDrafts);
  }

  function runnerCommandWithReason(control: FlowDashboardRunnerApprovalControl) {
    const reason = state.dashboardRunnerApprovalReason.trim();
    if (!control.command.trim()) {
      return "";
    }
    return control.command.replace(
      /"<(?:audit|denial|cancel) reason>"/g,
      `"${reason.replaceAll('"', "'")}"`,
    );
  }

  async function runRunnerApprovalControl(controlId: string) {
    const control = state.dashboardRunnerApprovalUi?.controls.find((item) => item.id === controlId);
    if (!control) {
      return;
    }
    if (control.disabled) {
      state.status = control.disabledReason ?? "This approval control is unavailable.";
      render();
      return;
    }
    if (control.kind === "snooze") {
      state.status = "Trusted runner approval draft snoozed in the dashboard.";
      render();
      return;
    }
    if (control.kind === "undo") {
      state.dashboardRunnerApprovalReason = "";
      state.status = "Trusted runner approval draft cleared.";
      render();
      return;
    }
    if (control.requiresReason && !state.dashboardRunnerApprovalReason.trim()) {
      state.status = "Add a short audit reason before preparing this trusted runner action.";
      render();
      return;
    }

    const command = runnerCommandWithReason(control);
    if (!command.trim()) {
      state.status = control.detail;
      render();
      return;
    }

    try {
      await navigator.clipboard?.writeText(command);
      state.status = `${control.label}: command copied. ${
        control.requiresApproval ? "Approval is preserved in the command." : ""
      }`;
    } catch {
      state.status = `${control.label}: ${command}`;
    }
    render();
  }

  async function copyRunnerAffordance(affordanceId: string) {
    const affordance = state.dashboardRunnerUx?.affordances.find(
      (item) => item.id === affordanceId,
    );
    if (!affordance) {
      return;
    }
    if (affordance.disabled) {
      state.status = affordance.disabledReason ?? "This trusted runner action is unavailable.";
      render();
      return;
    }

    try {
      await navigator.clipboard?.writeText(affordance.command);
      state.status = `${affordance.label}: command copied. ${
        affordance.requiresApproval ? "Approval is still required before execution." : ""
      }`;
    } catch {
      state.status = `${affordance.label}: ${affordance.command}`;
    }
    render();
  }

  function runnerCancellationCommandWithReason(control: FlowDashboardRunnerCancellationControl) {
    if (!control.command.trim()) {
      return "";
    }
    const fallback =
      state.dashboardRunnerCancellationUx?.draft.defaultReason ??
      "Operator reviewed live runner state";
    const reason = (state.dashboardRunnerCancellationDrafts[control.id] ?? fallback).trim();
    return control.command.replace(
      /"<[^"]*reason>"/g,
      `"${reason.replaceAll('"', "'")}"`,
    );
  }

  async function copyRunnerCancellationControl(controlId: string) {
    const control = state.dashboardRunnerCancellationUx?.controls.find(
      (item) => item.id === controlId,
    );
    if (!control) {
      return;
    }
    if (control.disabled) {
      state.status = control.disabledReason ?? "This runner recovery control is unavailable.";
      render();
      return;
    }

    const reason = (
      state.dashboardRunnerCancellationDrafts[control.id] ??
      state.dashboardRunnerCancellationUx?.draft.defaultReason ??
      ""
    ).trim();
    if (control.requiresReason && !reason) {
      state.status = "Add a short operator reason before preparing this runner recovery command.";
      render();
      return;
    }

    const command = runnerCancellationCommandWithReason(control);
    if (!command.trim()) {
      state.status = control.detail;
      render();
      return;
    }

    try {
      await navigator.clipboard?.writeText(command);
      state.status = `${control.label}: local recovery command copied.`;
    } catch {
      state.status = `${control.label}: ${command}`;
    }
    render();
  }

  async function copyRunnerIncidentNote(noteId: string) {
    const note = state.dashboardRunnerOperatorReview?.incidentNotes.find(
      (item) => item.id === noteId,
    );
    if (!note) {
      return;
    }
    try {
      await navigator.clipboard?.writeText(note.exportMarkdown);
      state.status = `${note.title}: incident note copied.`;
    } catch {
      state.status = `${note.title}: ${note.exportMarkdown}`;
    }
    render();
  }

  function releaseChecklistSignoffCommand() {
    const checklist = state.dashboardReleaseChecklist;
    if (!checklist) {
      return "";
    }
    const command = checklist.commands.find((item) => item.includes("--friday-release-signoff"));
    if (!command) {
      return "";
    }
    const reason =
      state.dashboardReleaseChecklistReason.trim() ||
      "Reviewed package, timeline, release review, TODO, and changelog.";
    return command
      .replace('"<signoff reason>"', `"${reason.replaceAll('"', "'")}"`)
      .replace('"<operator>"', '"operator"');
  }

  async function copyReleaseChecklistSignoff() {
    const command = releaseChecklistSignoffCommand();
    if (!command) {
      state.status = "No release checklist signoff command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release checklist signoff command copied.";
    } catch {
      state.status = `Release checklist signoff: ${command}`;
    }
    render();
  }

  async function copyReleaseQaCommand(checkId: string) {
    const check = state.dashboardReleaseQa?.checks.find((item) => item.id === checkId);
    if (!check) {
      return;
    }
    try {
      await navigator.clipboard?.writeText(check.command);
      state.status = `${check.label}: command copied.`;
    } catch {
      state.status = `${check.label}: ${check.command}`;
    }
    render();
  }

  async function copyReleaseExportKitNote() {
    const kit = state.dashboardReleaseExportKit;
    if (!kit) {
      return;
    }
    try {
      await navigator.clipboard?.writeText(kit.operatorCopy);
      state.status = "Release evidence export note copied.";
    } catch {
      state.status = kit.operatorCopy;
    }
    render();
  }

  async function copyReleaseExportKitCommand() {
    const commands = state.dashboardReleaseExportKit?.manifest.commands ?? [];
    const command = commands[commands.length - 1];
    if (!command) {
      state.status = "No release evidence export command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release evidence export command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseDeploymentGateNote() {
    const gate = state.dashboardReleaseDeploymentGate;
    if (!gate) {
      return;
    }
    try {
      await navigator.clipboard?.writeText(gate.operatorCopy);
      state.status = "Release deployment gate note copied.";
    } catch {
      state.status = gate.operatorCopy;
    }
    render();
  }

  async function copyReleaseDeploymentGateCommand() {
    const command = state.dashboardReleaseDeploymentGate?.commands[0] ?? "";
    if (!command) {
      state.status = "No release deployment gate command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release deployment gate command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseCandidateArchiveCommand() {
    const command = state.dashboardReleaseCandidateArchive?.commands[0] ?? "";
    if (!command) {
      state.status = "No release candidate archive command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release candidate archive command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleasePromotionLedgerCommand() {
    const command = state.dashboardReleasePromotionLedger?.commands[0] ?? "";
    if (!command) {
      state.status = "No release promotion ledger command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release promotion ledger command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleasePostPromotionMonitorCommand() {
    const command = state.dashboardReleasePostPromotionMonitor?.commands[0] ?? "";
    if (!command) {
      state.status = "No post-promotion monitor command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Post-promotion monitor command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseRollbackDrillCommand() {
    const command = state.dashboardReleaseRollbackDrill?.commands[0] ?? "";
    if (!command) {
      state.status = "No rollback drill command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Rollback drill command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseRollbackDryRunCommand() {
    const command = state.dashboardReleaseRollbackDrill?.dryRunCommand ?? "";
    if (!command) {
      state.status = "No rollback dry-run command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Rollback dry-run command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseStabilityBoardCommand() {
    const command = state.dashboardReleaseStabilityBoard?.commands[0] ?? "";
    if (!command) {
      state.status = "No stability board command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Stability board command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseRecoveryRunbookCommand() {
    const command = state.dashboardReleaseRecoveryRunbook?.commands[0] ?? "";
    if (!command) {
      state.status = "No recovery runbook command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Recovery runbook command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseRecoveryRunbookFirstPhase() {
    const phase = state.dashboardReleaseRecoveryRunbook?.phases
      .slice()
      .sort((left, right) => left.order - right.order)[0];
    const command = phase?.command ?? "";
    if (!command) {
      state.status = "No recovery phase command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = `${phase?.label ?? "Recovery phase"} command copied.`;
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseIncidentArchiveCommand() {
    const command = state.dashboardReleaseIncidentArchive?.commands[0] ?? "";
    if (!command) {
      state.status = "No incident archive command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Incident archive command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseIncidentArchiveFollowUp() {
    const latest = state.dashboardReleaseIncidentArchive?.entries.slice(-1)[0];
    const followUp = latest?.followUpActions[0] ?? "";
    if (!followUp) {
      state.status = "No incident follow-up action is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(followUp);
      state.status = "Latest incident follow-up copied.";
    } catch {
      state.status = followUp;
    }
    render();
  }

  async function copyReleasePreventionPlanCommand() {
    const command = state.dashboardReleasePreventionPlan?.commands[0] ?? "";
    if (!command) {
      state.status = "No prevention plan command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Prevention plan command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleasePreventionPlanOwnerCopy() {
    const copy = state.dashboardReleasePreventionPlan?.ownerReadyCopy ?? "";
    if (!copy) {
      state.status = "No owner-ready prevention copy is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Owner-ready prevention actions copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseOwnerFollowUpBoardCommand() {
    const command = state.dashboardReleaseOwnerFollowUpBoard?.commands[0] ?? "";
    if (!command) {
      state.status = "No owner follow-up board command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Owner follow-up board command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseOwnerFollowUpBoardAssignments() {
    const copy = state.dashboardReleaseOwnerFollowUpBoard?.assignmentCopy ?? "";
    if (!copy) {
      state.status = "No owner follow-up assignments are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Owner follow-up assignments copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseEvidenceSlaMonitorCommand() {
    const command = state.dashboardReleaseEvidenceSlaMonitor?.commands[0] ?? "";
    if (!command) {
      state.status = "No release evidence SLA monitor command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release evidence SLA monitor command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseEvidenceSlaMonitorEscalations() {
    const copy = state.dashboardReleaseEvidenceSlaMonitor?.escalationCopy ?? "";
    if (!copy) {
      state.status = "No release evidence SLA escalation copy is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release evidence SLA escalations copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseEscalationLedgerCommand() {
    const command = state.dashboardReleaseEscalationLedger?.commands[0] ?? "";
    if (!command) {
      state.status = "No release escalation ledger command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release escalation ledger command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseEscalationLedgerOwnerResponses() {
    const copy = state.dashboardReleaseEscalationLedger?.ownerResponseCopy ?? "";
    if (!copy) {
      state.status = "No release escalation owner response copy is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release escalation owner responses copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseCheckpointReviewCommand() {
    const command = state.dashboardReleaseCheckpointReview?.commands[0] ?? "";
    if (!command) {
      state.status = "No release checkpoint review command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release checkpoint review command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseCheckpointReviewNotes() {
    const copy = state.dashboardReleaseCheckpointReview?.reviewNotesCopy ?? "";
    if (!copy) {
      state.status = "No release checkpoint review notes are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release checkpoint review notes copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseCheckpointSignoffCommand() {
    const command = state.dashboardReleaseCheckpointSignoffLedger?.commands[0] ?? "";
    if (!command) {
      state.status = "No release checkpoint signoff command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release checkpoint signoff command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseCheckpointSignoffNotes() {
    const copy = state.dashboardReleaseCheckpointSignoffLedger?.releaseNotesCopy ?? "";
    if (!copy) {
      state.status = "No release checkpoint signoff notes are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release checkpoint signoff notes copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseCheckpointEvidenceVaultCommand() {
    const command = state.dashboardReleaseCheckpointEvidenceVault?.commands[0] ?? "";
    if (!command) {
      state.status = "No release checkpoint evidence vault command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release checkpoint evidence vault command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseCheckpointEvidenceVaultNotes() {
    const copy = state.dashboardReleaseCheckpointEvidenceVault?.attachmentNotesCopy ?? "";
    if (!copy) {
      state.status = "No release checkpoint evidence vault notes are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release checkpoint evidence vault notes copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseEvidenceAttachmentReviewCommand() {
    const command = state.dashboardReleaseEvidenceAttachmentReview?.commands[0] ?? "";
    if (!command) {
      state.status = "No release evidence attachment review command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release evidence attachment review command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseEvidenceAttachmentReviewNotes() {
    const copy = state.dashboardReleaseEvidenceAttachmentReview?.handoffNotesCopy ?? "";
    if (!copy) {
      state.status = "No release evidence attachment review notes are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release evidence attachment handoff notes copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseHandoffPacketCommand() {
    const command = state.dashboardReleaseHandoffPacket?.commands[0] ?? "";
    if (!command) {
      state.status = "No release handoff packet command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release handoff packet command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseHandoffPacket() {
    const copy = state.dashboardReleaseHandoffPacket?.handoffPacketCopy ?? "";
    if (!copy) {
      state.status = "No release handoff packet copy is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release handoff packet copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseHandoffFileChecklist() {
    const copy = state.dashboardReleaseHandoffPacket?.fileChecklistCopy ?? "";
    if (!copy) {
      state.status = "No release handoff file checklist is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release handoff file checklist copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseHandoffAuditCommand() {
    const command = state.dashboardReleaseHandoffAuditTrail?.commands[0] ?? "";
    if (!command) {
      state.status = "No release handoff audit command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release handoff audit command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseHandoffAuditSummary() {
    const copy = state.dashboardReleaseHandoffAuditTrail?.auditSummaryCopy ?? "";
    if (!copy) {
      state.status = "No release handoff audit summary is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release handoff audit summary copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseHandoffGovernanceCommand() {
    const command = state.dashboardReleaseHandoffGovernanceReview?.commands[0] ?? "";
    if (!command) {
      state.status = "No release handoff governance command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release handoff governance command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseHandoffGovernanceNotes() {
    const copy = state.dashboardReleaseHandoffGovernanceReview?.governanceNotesCopy ?? "";
    if (!copy) {
      state.status = "No release handoff governance notes are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release handoff governance notes copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseHandoffDispatchCommand() {
    const command = state.dashboardReleaseHandoffDispatchChecklist?.commands[0] ?? "";
    if (!command) {
      state.status = "No release handoff dispatch checklist command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release handoff dispatch checklist command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseHandoffDispatchChecklist() {
    const copy =
      state.dashboardReleaseHandoffDispatchChecklist?.dispatchChecklistCopy ?? "";
    if (!copy) {
      state.status = "No release handoff dispatch checklist is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release handoff dispatch checklist copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseHandoffDispatchAuditCommand() {
    const command = state.dashboardReleaseHandoffDispatchAuditTrail?.commands[0] ?? "";
    if (!command) {
      state.status = "No release handoff dispatch audit command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release handoff dispatch audit command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseHandoffDispatchAuditSummary() {
    const copy = state.dashboardReleaseHandoffDispatchAuditTrail?.auditSummaryCopy ?? "";
    if (!copy) {
      state.status = "No release handoff dispatch audit summary is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release handoff dispatch audit summary copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseHandoffDispatchGovernanceCommand() {
    const command = state.dashboardReleaseHandoffDispatchGovernanceReview?.commands[0] ?? "";
    if (!command) {
      state.status = "No release handoff dispatch governance command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release handoff dispatch governance command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseHandoffDispatchGovernanceNotes() {
    const copy =
      state.dashboardReleaseHandoffDispatchGovernanceReview?.governanceNotesCopy ?? "";
    if (!copy) {
      state.status = "No release handoff dispatch governance notes are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release handoff dispatch governance notes copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseHandoffCompletionCommand() {
    const command = state.dashboardReleaseHandoffCompletionLedger?.commands[0] ?? "";
    if (!command) {
      state.status = "No release handoff completion command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release handoff completion command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseHandoffCompletionSummary() {
    const copy =
      state.dashboardReleaseHandoffCompletionLedger?.completionSummaryCopy ?? "";
    if (!copy) {
      state.status = "No release handoff completion summary is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release handoff completion summary copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleasePublicationCommand() {
    const command = state.dashboardReleasePublicationControl?.commands[0] ?? "";
    if (!command) {
      state.status = "No release publication command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release publication command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleasePublicationNotes() {
    const copy = state.dashboardReleasePublicationControl?.releaseNotesCopy ?? "";
    if (!copy) {
      state.status = "No release publication notes are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release publication notes copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleasePublicationSendInstructions() {
    const copy =
      state.dashboardReleasePublicationControl?.externalSendInstructionsCopy ?? "";
    if (!copy) {
      state.status = "No release publication send instructions are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release publication send instructions copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseOutboundReviewCommand() {
    const command = state.dashboardReleaseOutboundReviewLedger?.commands[0] ?? "";
    if (!command) {
      state.status = "No release outbound review command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release outbound review command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseOutboundReviewNotes() {
    const copy =
      state.dashboardReleaseOutboundReviewLedger?.outboundSummaryCopy ?? "";
    if (!copy) {
      state.status = "No release outbound review notes are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release outbound review notes copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseExternalReceiptCommand() {
    const command = state.dashboardReleaseExternalReceiptArchive?.commands[0] ?? "";
    if (!command) {
      state.status = "No release external receipt command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release external receipt command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseExternalReceiptNotes() {
    const copy =
      state.dashboardReleaseExternalReceiptArchive?.auditNotesCopy ?? "";
    if (!copy) {
      state.status = "No release external receipt audit notes are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release external receipt audit notes copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseReceiptReviewCommand() {
    const command = state.dashboardReleaseReceiptReviewBoard?.commands[0] ?? "";
    if (!command) {
      state.status = "No release receipt review command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release receipt review command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseReceiptReviewNotes() {
    const copy =
      state.dashboardReleaseReceiptReviewBoard?.reviewNotesCopy ?? "";
    if (!copy) {
      state.status = "No release receipt review notes are available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release receipt review notes copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  async function copyReleaseClosureCommand() {
    const command = state.dashboardReleaseClosureLedger?.commands[0] ?? "";
    if (!command) {
      state.status = "No release closure command is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(command);
      state.status = "Release closure command copied.";
    } catch {
      state.status = command;
    }
    render();
  }

  async function copyReleaseClosureSummary() {
    const copy =
      state.dashboardReleaseClosureLedger?.closureSummaryCopy ?? "";
    if (!copy) {
      state.status = "No release closure summary is available.";
      render();
      return;
    }
    try {
      await navigator.clipboard?.writeText(copy);
      state.status = "Release closure summary copied.";
    } catch {
      state.status = copy;
    }
    render();
  }

  function bind() {
    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-import-click']")
      ?.addEventListener("click", () => {
        mountRoot.querySelector<HTMLInputElement>("#dashboard-import-json")?.click();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-host-bridge-import-click']")
      ?.addEventListener("click", () => {
        mountRoot.querySelector<HTMLInputElement>("#dashboard-import-host-bridge-json")?.click();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-runner-import-click']")
      ?.addEventListener("click", () => {
        mountRoot.querySelector<HTMLInputElement>("#dashboard-import-runner-json")?.click();
      });

    mountRoot
      .querySelector<HTMLInputElement>("#dashboard-import-json")
      ?.addEventListener("change", (event) => {
        const input = event.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        if (file) {
          void importDashboardJson(file);
        }
      });

    mountRoot
      .querySelector<HTMLInputElement>("#dashboard-import-host-bridge-json")
      ?.addEventListener("change", (event) => {
        const input = event.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        if (file) {
          void importDashboardHostBridgeJson(file);
        }
      });

    mountRoot
      .querySelector<HTMLInputElement>("#dashboard-import-runner-json")
      ?.addEventListener("change", (event) => {
        const input = event.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        if (file) {
          void importDashboardRunnerJson(file);
        }
      });

    mountRoot
      .querySelector<HTMLTextAreaElement>("#dashboard-runner-approval-reason")
      ?.addEventListener("input", (event) => {
        state.dashboardRunnerApprovalReason = (event.currentTarget as HTMLTextAreaElement).value;
      });

    mountRoot
      .querySelector(".dashboard-runner-approval")
      ?.addEventListener("keydown", (event) => {
        const keyboardEvent = event as KeyboardEvent;
        if (keyboardEvent.ctrlKey && keyboardEvent.key === "Enter") {
          keyboardEvent.preventDefault();
          void runRunnerApprovalControl("approve");
        } else if (keyboardEvent.key === "Escape") {
          keyboardEvent.preventDefault();
          void runRunnerApprovalControl("deny");
        }
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='dashboard-runner-approval-control']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const controlId = button.dataset.runnerApprovalControlId;
          if (controlId) {
            void runRunnerApprovalControl(controlId);
          }
        });
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='dashboard-runner-affordance']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const affordanceId = button.dataset.runnerAffordanceId;
          if (affordanceId) {
            void copyRunnerAffordance(affordanceId);
          }
        });
      });

    mountRoot
      .querySelectorAll<HTMLTextAreaElement>("[data-runner-cancellation-reason-id]")
      .forEach((textarea) => {
        textarea.addEventListener("input", () => {
          const controlId = textarea.dataset.runnerCancellationReasonId;
          if (controlId) {
            updateRunnerCancellationDraft(controlId, textarea.value);
          }
        });
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='dashboard-runner-cancellation-control']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const controlId = button.dataset.runnerCancellationControlId;
          if (controlId) {
            void copyRunnerCancellationControl(controlId);
          }
        });
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='dashboard-runner-incident-copy']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const noteId = button.dataset.runnerIncidentId;
          if (noteId) {
            void copyRunnerIncidentNote(noteId);
          }
        });
      });

    mountRoot
      .querySelector<HTMLTextAreaElement>("#dashboard-release-checklist-reason")
      ?.addEventListener("input", (event) => {
        state.dashboardReleaseChecklistReason = (event.currentTarget as HTMLTextAreaElement).value;
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-checklist-signoff']")
      ?.addEventListener("click", () => {
        void copyReleaseChecklistSignoff();
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='dashboard-release-qa-command']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const checkId = button.dataset.releaseQaCommandId;
          if (checkId) {
            void copyReleaseQaCommand(checkId);
          }
        });
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-export-kit-copy']")
      ?.addEventListener("click", () => {
        void copyReleaseExportKitNote();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-export-kit-command']")
      ?.addEventListener("click", () => {
        void copyReleaseExportKitCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-deployment-gate-copy']")
      ?.addEventListener("click", () => {
        void copyReleaseDeploymentGateNote();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-deployment-gate-command']")
      ?.addEventListener("click", () => {
        void copyReleaseDeploymentGateCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-candidate-archive-command']")
      ?.addEventListener("click", () => {
        void copyReleaseCandidateArchiveCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-promotion-ledger-command']")
      ?.addEventListener("click", () => {
        void copyReleasePromotionLedgerCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-post-promotion-monitor-command']")
      ?.addEventListener("click", () => {
        void copyReleasePostPromotionMonitorCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-rollback-drill-command']")
      ?.addEventListener("click", () => {
        void copyReleaseRollbackDrillCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-rollback-drill-dry-run']")
      ?.addEventListener("click", () => {
        void copyReleaseRollbackDryRunCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-stability-board-command']")
      ?.addEventListener("click", () => {
        void copyReleaseStabilityBoardCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-recovery-runbook-command']")
      ?.addEventListener("click", () => {
        void copyReleaseRecoveryRunbookCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-recovery-runbook-phase']")
      ?.addEventListener("click", () => {
        void copyReleaseRecoveryRunbookFirstPhase();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-incident-archive-command']")
      ?.addEventListener("click", () => {
        void copyReleaseIncidentArchiveCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-incident-archive-follow-up']")
      ?.addEventListener("click", () => {
        void copyReleaseIncidentArchiveFollowUp();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-prevention-plan-command']")
      ?.addEventListener("click", () => {
        void copyReleasePreventionPlanCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-prevention-plan-owner-copy']")
      ?.addEventListener("click", () => {
        void copyReleasePreventionPlanOwnerCopy();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-owner-followup-board-command']")
      ?.addEventListener("click", () => {
        void copyReleaseOwnerFollowUpBoardCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-owner-followup-board-assignment']")
      ?.addEventListener("click", () => {
        void copyReleaseOwnerFollowUpBoardAssignments();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-evidence-sla-monitor-command']")
      ?.addEventListener("click", () => {
        void copyReleaseEvidenceSlaMonitorCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-evidence-sla-monitor-escalation']")
      ?.addEventListener("click", () => {
        void copyReleaseEvidenceSlaMonitorEscalations();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-escalation-ledger-command']")
      ?.addEventListener("click", () => {
        void copyReleaseEscalationLedgerCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-escalation-ledger-owner-response']")
      ?.addEventListener("click", () => {
        void copyReleaseEscalationLedgerOwnerResponses();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-checkpoint-review-command']")
      ?.addEventListener("click", () => {
        void copyReleaseCheckpointReviewCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-checkpoint-review-notes']")
      ?.addEventListener("click", () => {
        void copyReleaseCheckpointReviewNotes();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-checkpoint-signoff-command']")
      ?.addEventListener("click", () => {
        void copyReleaseCheckpointSignoffCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-checkpoint-signoff-notes']")
      ?.addEventListener("click", () => {
        void copyReleaseCheckpointSignoffNotes();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-checkpoint-evidence-vault-command']")
      ?.addEventListener("click", () => {
        void copyReleaseCheckpointEvidenceVaultCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-checkpoint-evidence-vault-notes']")
      ?.addEventListener("click", () => {
        void copyReleaseCheckpointEvidenceVaultNotes();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-evidence-attachment-review-command']")
      ?.addEventListener("click", () => {
        void copyReleaseEvidenceAttachmentReviewCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-evidence-attachment-review-notes']")
      ?.addEventListener("click", () => {
        void copyReleaseEvidenceAttachmentReviewNotes();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-packet-command']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffPacketCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-packet-copy']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffPacket();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-file-checklist']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffFileChecklist();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-audit-command']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffAuditCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-audit-summary']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffAuditSummary();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-governance-command']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffGovernanceCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-governance-notes']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffGovernanceNotes();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-dispatch-command']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffDispatchCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-dispatch-checklist']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffDispatchChecklist();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-dispatch-audit-command']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffDispatchAuditCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-dispatch-audit-summary']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffDispatchAuditSummary();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-dispatch-governance-command']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffDispatchGovernanceCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-dispatch-governance-notes']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffDispatchGovernanceNotes();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-completion-command']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffCompletionCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-handoff-completion-summary']")
      ?.addEventListener("click", () => {
        void copyReleaseHandoffCompletionSummary();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-publication-command']")
      ?.addEventListener("click", () => {
        void copyReleasePublicationCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-publication-notes']")
      ?.addEventListener("click", () => {
        void copyReleasePublicationNotes();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-publication-send']")
      ?.addEventListener("click", () => {
        void copyReleasePublicationSendInstructions();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-outbound-review-command']")
      ?.addEventListener("click", () => {
        void copyReleaseOutboundReviewCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-outbound-review-notes']")
      ?.addEventListener("click", () => {
        void copyReleaseOutboundReviewNotes();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-external-receipt-command']")
      ?.addEventListener("click", () => {
        void copyReleaseExternalReceiptCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-external-receipt-notes']")
      ?.addEventListener("click", () => {
        void copyReleaseExternalReceiptNotes();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-receipt-review-command']")
      ?.addEventListener("click", () => {
        void copyReleaseReceiptReviewCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-receipt-review-notes']")
      ?.addEventListener("click", () => {
        void copyReleaseReceiptReviewNotes();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-closure-command']")
      ?.addEventListener("click", () => {
        void copyReleaseClosureCommand();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='dashboard-release-closure-summary']")
      ?.addEventListener("click", () => {
        void copyReleaseClosureSummary();
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='dashboard-action']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const actionId = button.dataset.actionId;
          if (actionId) {
            void runDashboardAction(actionId);
          }
        });
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='dashboard-release-link']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const linkId = button.dataset.releaseLinkId;
          if (linkId) {
            showDashboardReleaseLink(linkId);
          }
        });
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='dashboard-screenshot-prompt']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const route = button.dataset.promptRoute;
          if (route) {
            showDashboardScreenshotPrompt(route);
          }
        });
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='change-section']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const next = button.dataset.section as FlowWorkspaceSection | undefined;
          if (!next) {
            return;
          }
          state.activeSection = next;
          render();
        });
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='go-settings']")
      ?.addEventListener("click", () => {
        state.activeSection = "settings";
        render();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='go-packs']")
      ?.addEventListener("click", () => {
        state.activeSection = "packs";
        render();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='open-workspace']")
      ?.addEventListener("click", () => {
        state.activeSection = "workspace";
        render();
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='quick-run']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const task = button.dataset.task as FlowTask | undefined;
          if (task) {
            void runFlow(task);
          }
        });
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='download-pack']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const modelKey = button.dataset.modelKey;
          if (modelKey) {
            void installPack(modelKey);
          }
        });
      });

    mountRoot
      .querySelectorAll<HTMLButtonElement>("[data-action='remove-pack']")
      .forEach((button) => {
        button.addEventListener("click", () => {
          const modelKey = button.dataset.modelKey;
          if (modelKey) {
            void removePack(modelKey);
          }
        });
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='refresh-runtime']")
      ?.addEventListener("click", () => {
        void refreshRuntimeStatus("Refreshed browser runtime state.").then(() => render());
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='refresh-context']")
      ?.addEventListener("click", () => {
        void refreshQuickContext().then(() => render());
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='clear-context']")
      ?.addEventListener("click", () => {
        state.quickContext = null;
        state.status = "Cleared the cached tab context.";
        render();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='toggle-overlay']")
      ?.addEventListener("click", () => {
        void toggleOverlay();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='run-flow']")
      ?.addEventListener("click", () => {
        void runFlow();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='copy-output']")
      ?.addEventListener("click", () => {
        void copyOutput();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='apply-output']")
      ?.addEventListener("click", () => {
        void applyOutput();
      });

    mountRoot
      .querySelector<HTMLButtonElement>("[data-action='clear-output']")
      ?.addEventListener("click", () => {
        state.output = "";
        state.status = "Cleared the local output buffer.";
        render();
      });

    mountRoot
      .querySelector<HTMLSelectElement>("#flow-task")
      ?.addEventListener("change", (event) => {
        const select = event.currentTarget as HTMLSelectElement;
        const nextTask = select.value as FlowTask;
        const wasDefaultPrompt =
          !state.draft.prompt.trim() ||
          state.draft.prompt.trim() === engine.defaultPrompt(state.draft.task);
        const nextPrompt = wasDefaultPrompt ? engine.defaultPrompt(nextTask) : state.draft.prompt;
        void updateDraft(
          {
            task: nextTask,
            prompt: nextPrompt,
            imageSources: state.draft.imageSources,
          },
          true,
        );
      });

    mountRoot
      .querySelector<HTMLSelectElement>("#flow-model")
      ?.addEventListener("change", (event) => {
        const select = event.currentTarget as HTMLSelectElement;
        const patch =
          engine.modalityForTask(state.draft.task) === "ocr"
            ? { preferredOcrModel: select.value }
            : engine.modalityForTask(state.draft.task) === "vision-language"
              ? { preferredVisionLanguageModel: select.value }
              : { preferredChatModel: select.value };
        void updateSettings(patch);
      });

    mountRoot
      .querySelector<HTMLTextAreaElement>("#flow-prompt")
      ?.addEventListener("input", (event) => {
        const textarea = event.currentTarget as HTMLTextAreaElement;
        state.draft.prompt = textarea.value;
        void updateDraft({ prompt: textarea.value });
      });

    mountRoot
      .querySelector<HTMLInputElement>("#flow-images")
      ?.addEventListener("input", (event) => {
        const input = event.currentTarget as HTMLInputElement;
        state.draft.imageSources = input.value;
        void updateDraft({ imageSources: input.value });
      });

    mountRoot
      .querySelector<HTMLInputElement>("#setting-local-only")
      ?.addEventListener("change", (event) => {
        const input = event.currentTarget as HTMLInputElement;
        void updateSettings({ localOnly: input.checked });
      });

    mountRoot
      .querySelector<HTMLInputElement>("#setting-auto-apply")
      ?.addEventListener("change", (event) => {
        const input = event.currentTarget as HTMLInputElement;
        void updateSettings({ autoApplyRewrite: input.checked });
      });

    mountRoot
      .querySelector<HTMLInputElement>("#setting-capture-context")
      ?.addEventListener("change", (event) => {
        const input = event.currentTarget as HTMLInputElement;
        void updateSettings({ captureActiveTabContext: input.checked });
      });

    mountRoot
      .querySelector<HTMLSelectElement>("#setting-default-task")
      ?.addEventListener("change", (event) => {
        const select = event.currentTarget as HTMLSelectElement;
        const nextTask = select.value as FlowTask;
        void updateSettings({ defaultTask: nextTask });
      });

    mountRoot
      .querySelector<HTMLSelectElement>("#setting-chat-model")
      ?.addEventListener("change", (event) => {
        const select = event.currentTarget as HTMLSelectElement;
        void updateSettings({ preferredChatModel: select.value });
      });

    mountRoot
      .querySelector<HTMLSelectElement>("#setting-ocr-model")
      ?.addEventListener("change", (event) => {
        const select = event.currentTarget as HTMLSelectElement;
        void updateSettings({ preferredOcrModel: select.value });
      });

    mountRoot
      .querySelector<HTMLSelectElement>("#setting-vlm-model")
      ?.addEventListener("change", (event) => {
        const select = event.currentTarget as HTMLSelectElement;
        void updateSettings({ preferredVisionLanguageModel: select.value });
      });
  }

  if (state.settings.captureActiveTabContext) {
    state.quickContext = await requestQuickContext();
  }

  render();
}
