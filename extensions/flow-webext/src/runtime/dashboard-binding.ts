import type { FlowDashboardProductUiBinding } from "./protocol";

type DashboardCard = FlowDashboardProductUiBinding["cards"][number];

function action(
  actionId: string,
  label: string,
  kind: DashboardCard["actions"][number]["kind"],
  command: string,
): DashboardCard["actions"][number] {
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

const DASHBOARD_CARDS: FlowDashboardProductUiBinding["cards"] = [
  {
    cardId: "completion-loop",
    title: "Completion Loop",
    status: "warning",
    scoreOutOf100: 20,
    primaryMetric: "Friday Dashboard Visible UI Execution is active at 20/100.",
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

export function defaultFridayDashboardBinding(): FlowDashboardProductUiBinding {
  return {
    productName: "Friday",
    route: "/dashboard",
    title: "Friday Dashboard",
    sourceFile: "extensions/flow-webext/src/ui/app.ts",
    exportDir: "tmp/friday-dashboard",
    status: "warning",
    scoreOutOf100: 40,
    summary:
      "Render the live dashboard contract and local action button states in the visible browser surface while history, release links, and smoke paths are wired.",
    panelJsonCommand: "flow --friday-dashboard-panel-json tmp/friday-dashboard",
    exportCommand: "flow --friday-dashboard-export tmp/friday-dashboard",
    cardCount: DASHBOARD_CARDS.length,
    boundCardCount: DASHBOARD_CARDS.length,
    actionCount: DASHBOARD_CARDS.reduce((total, card) => total + card.actionCount, 0),
    warningCount: DASHBOARD_CARDS.filter((card) => card.status === "warning").length,
    blockingCount: DASHBOARD_CARDS.filter((card) => card.status === "blocked").length,
    cards: DASHBOARD_CARDS,
    nextActions: [
      "Wire dashboard action buttons to explicit local command handoffs.",
      "Render history deltas, release-review links, and screenshot prompts in the visible dashboard.",
      "Add a TypeScript smoke path that proves the dashboard section renders from typed data.",
    ],
  };
}
