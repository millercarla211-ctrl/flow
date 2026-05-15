import type {
  FlowDashboardActionKind,
  FlowDashboardPanelStatus,
  FlowDashboardProductUiActionBinding,
  FlowDashboardProductUiBinding,
  FlowDashboardProductUiButtonState,
  FlowDashboardProductUiCardBinding,
  FlowDashboardProductUiHistoryBinding,
  FlowDashboardProductUiReleaseLink,
  FlowDashboardProductUiScreenshotPrompt,
  FlowDashboardScreenshotStatus,
} from "./protocol";

type DashboardCard = FlowDashboardProductUiBinding["cards"][number];
type UnknownRecord = Record<string, unknown>;

function isRecord(value: unknown): value is UnknownRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function prop(record: UnknownRecord, camel: string, snake = camel) {
  return record[camel] ?? record[snake];
}

function stringProp(record: UnknownRecord, camel: string, snake = camel, fallback = "") {
  const value = prop(record, camel, snake);
  return typeof value === "string" ? value : fallback;
}

function numberProp(record: UnknownRecord, camel: string, snake = camel, fallback = 0) {
  const value = prop(record, camel, snake);
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function boolProp(record: UnknownRecord, camel: string, snake = camel, fallback = false) {
  const value = prop(record, camel, snake);
  return typeof value === "boolean" ? value : fallback;
}

function arrayProp<T>(
  record: UnknownRecord,
  camel: string,
  snake: string,
  mapper: (value: unknown) => T,
): T[] {
  const value = prop(record, camel, snake);
  return Array.isArray(value) ? value.map(mapper) : [];
}

function action(
  actionId: string,
  label: string,
  kind: FlowDashboardActionKind,
  command: string,
): FlowDashboardProductUiActionBinding {
  const verb =
    kind === "run-check"
      ? "Run"
      : kind === "recover"
        ? "Recover"
        : kind === "capture"
          ? "Capture"
          : "Open";
  const loadingLabel =
    kind === "run-check"
      ? "Running..."
      : kind === "recover"
        ? "Recovering..."
        : kind === "capture"
          ? "Capturing..."
          : "Opening...";

  return {
    actionId,
    label,
    kind,
    command,
    localOnly: true,
    enabled: command.trim().length > 0,
    buttonState: {
      disabled: command.trim().length === 0,
      disabledReason: command.trim().length === 0 ? "Action command is empty." : null,
      idleLabel: label,
      loadingLabel,
      successLabel: `${verb} complete`,
      errorLabel: `${verb} failed`,
      ariaLabel: `${verb} dashboard action: ${label}`,
      destructive: false,
      requiresConfirmation: false,
    },
  };
}

function normalizeButtonState(
  value: unknown,
  fallback: FlowDashboardProductUiButtonState,
): FlowDashboardProductUiButtonState {
  if (!isRecord(value)) {
    return fallback;
  }

  return {
    disabled: boolProp(value, "disabled", "disabled", fallback.disabled),
    disabledReason:
      typeof prop(value, "disabledReason", "disabled_reason") === "string"
        ? stringProp(value, "disabledReason", "disabled_reason")
        : fallback.disabledReason,
    idleLabel: stringProp(value, "idleLabel", "idle_label", fallback.idleLabel),
    loadingLabel: stringProp(value, "loadingLabel", "loading_label", fallback.loadingLabel),
    successLabel: stringProp(value, "successLabel", "success_label", fallback.successLabel),
    errorLabel: stringProp(value, "errorLabel", "error_label", fallback.errorLabel),
    ariaLabel: stringProp(value, "ariaLabel", "aria_label", fallback.ariaLabel),
    destructive: boolProp(value, "destructive", "destructive", fallback.destructive),
    requiresConfirmation: boolProp(
      value,
      "requiresConfirmation",
      "requires_confirmation",
      fallback.requiresConfirmation,
    ),
  };
}

function normalizeAction(value: unknown, cardId: string): FlowDashboardProductUiActionBinding {
  if (!isRecord(value)) {
    return action(`${cardId}-action`, "Open", "open", "");
  }

  const kind = stringProp(value, "kind", "kind", "open") as FlowDashboardActionKind;
  const fallback = action(
    stringProp(value, "actionId", "action_id", `${cardId}-action`),
    stringProp(value, "label", "label", "Open"),
    kind,
    stringProp(value, "command", "command"),
  );

  return {
    ...fallback,
    localOnly: boolProp(value, "localOnly", "local_only", fallback.localOnly),
    enabled: boolProp(value, "enabled", "enabled", fallback.enabled),
    buttonState: normalizeButtonState(prop(value, "buttonState", "button_state"), fallback.buttonState),
  };
}

function normalizeCard(value: unknown): FlowDashboardProductUiCardBinding {
  if (!isRecord(value)) {
    return {
      cardId: "unknown",
      title: "Unknown",
      status: "warning",
      scoreOutOf100: 0,
      primaryMetric: "No card data was available.",
      sourceJson: "",
      actionCount: 0,
      actions: [],
    };
  }

  const cardId = stringProp(value, "cardId", "card_id", "unknown");
  const actions = arrayProp(value, "actions", "actions", (item) => normalizeAction(item, cardId));

  return {
    cardId,
    title: stringProp(value, "title", "title", cardId),
    status: stringProp(value, "status", "status", "warning") as FlowDashboardPanelStatus,
    scoreOutOf100: numberProp(value, "scoreOutOf100", "score_out_of_100"),
    primaryMetric: stringProp(value, "primaryMetric", "primary_metric"),
    sourceJson: stringProp(value, "sourceJson", "source_json"),
    actionCount: numberProp(value, "actionCount", "action_count", actions.length),
    actions,
  };
}

function normalizeHistory(value: unknown): FlowDashboardProductUiHistoryBinding {
  if (!isRecord(value)) {
    return defaultFridayDashboardBinding().history;
  }

  const latest = prop(value, "latestScoreOutOf100", "latest_score_out_of_100");
  const previous = prop(value, "previousScoreOutOf100", "previous_score_out_of_100");

  return {
    recordCount: numberProp(value, "recordCount", "record_count"),
    scoreDeltaFromPrevious: numberProp(value, "scoreDeltaFromPrevious", "score_delta_from_previous"),
    readinessDeltaFromPrevious: numberProp(
      value,
      "readinessDeltaFromPrevious",
      "readiness_delta_from_previous",
    ),
    latestScoreOutOf100: typeof latest === "number" ? latest : null,
    previousScoreOutOf100: typeof previous === "number" ? previous : null,
    trendLabel: stringProp(value, "trendLabel", "trend_label", "not-enough-history"),
  };
}

function normalizeReleaseLink(value: unknown): FlowDashboardProductUiReleaseLink {
  if (!isRecord(value)) {
    const fallback = action("release-link-unknown", "Open artifact", "open", "");
    return {
      id: "unknown",
      label: "Unknown",
      kind: "artifact",
      path: "",
      section: "other",
      localOnly: true,
      buttonState: fallback.buttonState,
    };
  }

  const id = stringProp(value, "id", "id", "unknown");
  const label = stringProp(value, "label", "label", id);
  const path = stringProp(value, "path", "path");
  const fallback = action(`release-link-${id}`, `Open ${label}`, "open", path);

  return {
    id,
    label,
    kind: stringProp(value, "kind", "kind", "artifact"),
    path,
    section: stringProp(value, "section", "section", "other"),
    localOnly: boolProp(value, "localOnly", "local_only", true),
    buttonState: normalizeButtonState(prop(value, "buttonState", "button_state"), fallback.buttonState),
  };
}

function normalizeScreenshotPrompt(value: unknown): FlowDashboardProductUiScreenshotPrompt {
  if (!isRecord(value)) {
    return {
      route: "/",
      title: "Unknown",
      viewportId: "desktop",
      status: "missing",
      prompt: "Capture this route.",
      captureCommand: "",
    };
  }

  return {
    route: stringProp(value, "route", "route", "/"),
    title: stringProp(value, "title", "title", "Route"),
    viewportId: stringProp(value, "viewportId", "viewport_id", "desktop"),
    status: stringProp(value, "status", "status", "missing") as FlowDashboardScreenshotStatus,
    prompt: stringProp(value, "prompt", "prompt"),
    captureCommand: stringProp(value, "captureCommand", "capture_command"),
  };
}

const DASHBOARD_CARDS: FlowDashboardProductUiBinding["cards"] = [
  {
    cardId: "completion-loop",
    title: "Completion Loop",
    status: "warning",
    scoreOutOf100: 100,
    primaryMetric: "Friday Dashboard Visible UI Execution is complete at 100/100.",
    sourceJson: "tmp/friday-dashboard/completion.json",
    actionCount: 1,
    actions: [action("completion-loop-open", "Open completion", "open", "flow --completion")],
  },
  {
    cardId: "operator-readiness",
    title: "Operator Readiness",
    status: "warning",
    scoreOutOf100: 86,
    primaryMetric: "Local runtime is usable with one readiness warning.",
    sourceJson: "tmp/friday-dashboard/readiness.json",
    actionCount: 2,
    actions: [
      action("readiness-run", "Run readiness", "run-check", "flow --friday-readiness"),
      action("readiness-recover", "Recover media", "recover", "flow --friday-media-affordances"),
    ],
  },
  {
    cardId: "route-bindings",
    title: "Route Bindings",
    status: "ready",
    scoreOutOf100: 100,
    primaryMetric: "Friday routes are bound to tracked desktop and web UI files.",
    sourceJson: "tmp/friday-dashboard/route-bindings.json",
    actionCount: 1,
    actions: [
      action("route-bindings-open", "Open routes", "open", "flow --friday-live-ui-routes"),
    ],
  },
  {
    cardId: "route-visuals",
    title: "Route Visuals",
    status: "warning",
    scoreOutOf100: 60,
    primaryMetric: "Screenshot capture prompts are ready for missing route views.",
    sourceJson: "tmp/friday-dashboard/route-visuals.json",
    actionCount: 1,
    actions: [
      action("route-visuals-capture", "Capture routes", "capture", "flow --friday-route-visuals"),
    ],
  },
  {
    cardId: "execution-handoffs",
    title: "Execution Handoffs",
    status: "ready",
    scoreOutOf100: 100,
    primaryMetric: "Local UI actions map to explicit command handoffs.",
    sourceJson: "tmp/friday-dashboard/execution-handoffs.json",
    actionCount: 1,
    actions: [
      action(
        "execution-handoffs-open",
        "Open handoffs",
        "open",
        "flow --friday-execution-handoffs",
      ),
    ],
  },
  {
    cardId: "screenshot-history",
    title: "Screenshot History",
    status: "warning",
    scoreOutOf100: 50,
    primaryMetric: "Missing captures are tracked with repeatable screenshot prompts.",
    sourceJson: "tmp/friday-dashboard/route-visuals.json",
    actionCount: 1,
    actions: [
      action("screenshot-history-capture", "Capture missing", "capture", "flow --friday-route-visuals"),
    ],
  },
  {
    cardId: "export-history",
    title: "Export History",
    status: "ready",
    scoreOutOf100: 100,
    primaryMetric: "Recent dashboard exports can be compared between checkpoints.",
    sourceJson: "tmp/friday-dashboard/dashboard-history.json",
    actionCount: 1,
    actions: [
      action(
        "export-history-refresh",
        "Refresh export",
        "run-check",
        "flow --friday-dashboard-export tmp/friday-dashboard",
      ),
    ],
  },
  {
    cardId: "release-review",
    title: "Release Review",
    status: "warning",
    scoreOutOf100: 72,
    primaryMetric: "Release links are available while the visible UI loop continues.",
    sourceJson: "tmp/friday-dashboard/release-review.json",
    actionCount: 1,
    actions: [
      action(
        "release-review-open",
        "Open release review",
        "open",
        "flow --friday-dashboard-panel tmp/friday-dashboard",
      ),
    ],
  },
];

const RELEASE_LINKS: FlowDashboardProductUiBinding["releaseLinks"] = [
  {
    id: "todo",
    label: "TODO",
    kind: "markdown",
    path: "TODO.md",
    section: "release-notes",
    localOnly: true,
    buttonState: action("release-link-todo", "Open TODO", "open", "TODO.md").buttonState,
  },
  {
    id: "changelog",
    label: "Changelog",
    kind: "markdown",
    path: "CHANGELOG.md",
    section: "release-notes",
    localOnly: true,
    buttonState: action("release-link-changelog", "Open changelog", "open", "CHANGELOG.md")
      .buttonState,
  },
  {
    id: "route-visuals",
    label: "Route visuals",
    kind: "json",
    path: "tmp/friday-dashboard/route-visuals.json",
    section: "visual-review",
    localOnly: true,
    buttonState: action("release-link-route-visuals", "Open visuals", "open", "tmp/friday-dashboard/route-visuals.json")
      .buttonState,
  },
  {
    id: "dashboard-history",
    label: "Dashboard history",
    kind: "json",
    path: "tmp/friday-dashboard/dashboard-history.json",
    section: "export-artifacts",
    localOnly: true,
    buttonState: action("release-link-dashboard-history", "Open history", "open", "tmp/friday-dashboard/dashboard-history.json")
      .buttonState,
  },
  {
    id: "manifest",
    label: "Manifest",
    kind: "json",
    path: "tmp/friday-dashboard/manifest.json",
    section: "export-artifacts",
    localOnly: true,
    buttonState: action("release-link-manifest", "Open manifest", "open", "tmp/friday-dashboard/manifest.json")
      .buttonState,
  },
];

const SCREENSHOT_PROMPTS: FlowDashboardProductUiBinding["screenshotPrompts"] = [
  {
    route: "/ask",
    title: "Ask desktop",
    viewportId: "desktop",
    status: "missing",
    prompt: "Capture the Friday Ask route in the desktop viewport.",
    captureCommand: "agent-browser screenshot --route /ask --viewport desktop",
  },
  {
    route: "/research",
    title: "Research desktop",
    viewportId: "desktop",
    status: "missing",
    prompt: "Capture the Friday Research route in the desktop viewport.",
    captureCommand: "agent-browser screenshot --route /research --viewport desktop",
  },
  {
    route: "/voice",
    title: "Voice mobile",
    viewportId: "mobile",
    status: "missing",
    prompt: "Capture the Friday Voice route in the mobile viewport.",
    captureCommand: "agent-browser screenshot --route /voice --viewport mobile",
  },
];

export function defaultFridayDashboardBinding(): FlowDashboardProductUiBinding {
  return {
    productName: "Friday",
    route: "/dashboard",
    title: "Friday Dashboard",
    sourceFile: "extensions/flow-webext/src/ui/app.ts",
    exportDir: "tmp/friday-dashboard",
    status: "warning",
    scoreOutOf100: 100,
    summary:
      "Render imported local dashboard data when available, with a bundled local-only snapshot as the offline fallback.",
    panelJsonCommand: "flow --friday-dashboard-panel-json tmp/friday-dashboard",
    exportCommand: "flow --friday-dashboard-export tmp/friday-dashboard",
    sourceKind: "embedded-snapshot",
    sourceLabel: "Bundled local dashboard snapshot",
    localOnly: true,
    fallback: true,
    cardCount: DASHBOARD_CARDS.length,
    boundCardCount: DASHBOARD_CARDS.length,
    actionCount: DASHBOARD_CARDS.reduce((total, card) => total + card.actionCount, 0),
    warningCount: DASHBOARD_CARDS.filter((card) => card.status === "warning").length,
    blockingCount: DASHBOARD_CARDS.filter((card) => card.status === "blocked").length,
    cards: DASHBOARD_CARDS,
    history: {
      recordCount: 8,
      scoreDeltaFromPrevious: 20,
      readinessDeltaFromPrevious: 0,
      latestScoreOutOf100: 100,
      previousScoreOutOf100: 80,
      trendLabel: "improving",
    },
    releaseLinks: RELEASE_LINKS,
    screenshotPrompts: SCREENSHOT_PROMPTS,
    nextActions: [
      "Import fresh JSON from `flow --friday-dashboard-product-ui-json tmp/friday-dashboard` when reviewing a new local checkpoint.",
      "Keep the embedded snapshot available only as an offline fallback.",
    ],
  };
}

export function normalizeFridayDashboardBinding(
  value: unknown,
  sourceLabel = "Imported dashboard JSON",
): FlowDashboardProductUiBinding {
  const fallback = defaultFridayDashboardBinding();
  if (!isRecord(value)) {
    return fallback;
  }

  const groupedActions = new Map<string, FlowDashboardProductUiActionBinding[]>();
  for (const actionValue of arrayProp(value, "actionBindings", "action_bindings", (item) => item)) {
    if (!isRecord(actionValue)) {
      continue;
    }

    const cardId = stringProp(actionValue, "cardId", "card_id", "unknown");
    const actions = groupedActions.get(cardId) ?? [];
    actions.push(normalizeAction(actionValue, cardId));
    groupedActions.set(cardId, actions);
  }

  const cards = arrayProp(value, "cards", "cards", normalizeCard).map((card) => {
    if (card.actions.length > 0) {
      return card;
    }

    const actions = groupedActions.get(card.cardId) ?? [];
    return {
      ...card,
      actions,
      actionCount: actions.length || card.actionCount,
    };
  });
  const actionCount = cards.reduce((total, card) => total + card.actions.length, 0);

  return {
    productName: stringProp(value, "productName", "product_name", fallback.productName),
    route: stringProp(value, "route", "route", fallback.route),
    title: stringProp(value, "title", "title", fallback.title),
    sourceFile: stringProp(value, "sourceFile", "source_file", fallback.sourceFile),
    exportDir: stringProp(value, "exportDir", "export_dir", fallback.exportDir),
    status: stringProp(value, "status", "status", fallback.status) as FlowDashboardPanelStatus,
    scoreOutOf100: numberProp(value, "scoreOutOf100", "score_out_of_100", fallback.scoreOutOf100),
    summary: stringProp(value, "summary", "summary", fallback.summary),
    panelJsonCommand: stringProp(
      value,
      "panelJsonCommand",
      "panel_json_command",
      fallback.panelJsonCommand,
    ),
    exportCommand: stringProp(value, "exportCommand", "export_command", fallback.exportCommand),
    sourceKind: "imported-json",
    sourceLabel,
    localOnly: true,
    fallback: false,
    cardCount: numberProp(value, "cardCount", "card_count", cards.length),
    boundCardCount: numberProp(value, "boundCardCount", "bound_card_count", cards.length),
    actionCount: numberProp(value, "actionCount", "action_count", actionCount),
    warningCount: numberProp(value, "warningCount", "warning_count", fallback.warningCount),
    blockingCount: numberProp(value, "blockingCount", "blocking_count", fallback.blockingCount),
    cards,
    history: normalizeHistory(prop(value, "history", "history")),
    releaseLinks: arrayProp(value, "releaseLinks", "release_links", normalizeReleaseLink),
    screenshotPrompts: arrayProp(
      value,
      "screenshotPrompts",
      "screenshot_prompts",
      normalizeScreenshotPrompt,
    ),
    nextActions: arrayProp(value, "nextActions", "next_actions", (item) =>
      typeof item === "string" ? item : "",
    ).filter(Boolean),
  };
}
