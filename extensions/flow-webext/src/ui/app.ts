import { requestQuickContext, replaceSelection, toggleOverlay } from "../runtime/browser-api";
import {
  buildTrustedHostRunnerCancellationUx,
  dispatchDashboardCommand,
  normalizeReleaseOperatorChecklist,
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
  type FlowReleaseOperatorChecklistReport,
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
      state.dashboardActionResults = [...results, ...state.dashboardActionResults].slice(0, 8);
      state.status = releaseChecklist
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
