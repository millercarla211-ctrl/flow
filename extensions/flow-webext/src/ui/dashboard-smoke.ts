import {
  defaultFridayDashboardBinding,
  normalizeFridayDashboardBinding,
} from "../runtime/dashboard-binding";
import {
  dispatchDashboardCommand,
  normalizeDashboardHostCommandResults,
  normalizeTrustedHostRunnerResults,
  normalizeTrustedHostRunnerUx,
} from "../runtime/dashboard-actions";
import type { FlowDashboardProductUiBinding } from "../runtime/protocol";

type DashboardSmokeCheck = {
  id: string;
  passed: boolean;
  evidence: string;
};

type DashboardSmokeReport = {
  scoreOutOf100: number;
  checks: DashboardSmokeCheck[];
};

function check(id: string, passed: boolean, evidence: string): DashboardSmokeCheck {
  return { id, passed, evidence };
}

function score(checks: DashboardSmokeCheck[]) {
  if (checks.length === 0) {
    return 0;
  }

  const passed = checks.filter((item) => item.passed).length;
  return Math.round((passed / checks.length) * 100);
}

export function dashboardSectionSmokeReport(
  binding: FlowDashboardProductUiBinding = defaultFridayDashboardBinding(),
): DashboardSmokeReport {
  const imported = normalizeFridayDashboardBinding(
    {
      product_name: "Friday",
      route: "/dashboard",
      title: "Imported dashboard",
      source_file: "extensions/flow-webext/src/ui/app.ts",
      export_dir: "tmp/friday-dashboard",
      status: "ready",
      score_out_of_100: 100,
      summary: "Imported local JSON.",
      panel_json_command: "flow --friday-dashboard-panel-json tmp/friday-dashboard",
      export_command: "flow --friday-dashboard-export tmp/friday-dashboard",
      card_count: 1,
      bound_card_count: 1,
      action_count: 1,
      warning_count: 0,
      blocking_count: 0,
      cards: [
        {
          card_id: "imported-card",
          title: "Imported card",
          status: "ready",
          score_out_of_100: 100,
          primary_metric: "Imported from local JSON.",
          source_json: "tmp/friday-dashboard/imported.json",
          action_count: 1,
        },
      ],
      action_bindings: [
        {
          card_id: "imported-card",
          action_id: "imported-action",
          label: "Open imported",
          kind: "open",
          command: "flow --completion",
          local_only: true,
          enabled: true,
        },
      ],
      history: {
        record_count: 1,
        score_delta_from_previous: 0,
        readiness_delta_from_previous: 0,
        latest_score_out_of_100: 100,
        previous_score_out_of_100: 100,
        trend_label: "steady",
      },
      release_links: [
        {
          id: "summary",
          label: "Summary",
          kind: "markdown",
          path: "tmp/friday-dashboard/summary.md",
          section: "release-notes",
          local_only: true,
        },
      ],
      screenshot_prompts: [
        {
          route: "/dashboard",
          title: "Dashboard desktop",
          viewport_id: "desktop",
          status: "missing",
          prompt: "Capture dashboard.",
          capture_command: "agent-browser screenshot --route /dashboard",
        },
      ],
      next_actions: ["Review imported dashboard."],
    },
    "smoke-import.json",
  );
  const actionCount = binding.cards.reduce((total, card) => total + card.actions.length, 0);
  const executableAction = imported.cards[0]?.actions[0];
  const preparedResult = executableAction
    ? dispatchDashboardCommand(executableAction, {
        confirmed: true,
        now: "2026-05-15T00:00:00.000Z",
      })
    : null;
  const confirmationResult = executableAction
    ? dispatchDashboardCommand(
        {
          ...executableAction,
          buttonState: {
            ...executableAction.buttonState,
            requiresConfirmation: true,
          },
        },
        { now: "2026-05-15T00:00:01.000Z" },
      )
    : null;
  const blockedResult = executableAction
    ? dispatchDashboardCommand(
        {
          ...executableAction,
          localOnly: false,
        },
        { confirmed: true, now: "2026-05-15T00:00:02.000Z" },
      )
    : null;
  const failedResult = executableAction
    ? dispatchDashboardCommand(
        {
          ...executableAction,
          command: "",
        },
        { confirmed: true, now: "2026-05-15T00:00:03.000Z" },
      )
    : null;
  const hostBridgeResults = normalizeDashboardHostCommandResults({
    records: [
      {
        action_id: "host-open",
        label: "Open host report",
        kind: "open",
        command: "flow --completion",
        status: "awaiting-approval",
        approval_state: "required",
        audit: {
          stdout_summary: "not executed; waiting for operator approval",
          stderr_summary: "",
          recorded_at_unix_ms: 1,
        },
      },
    ],
  });
  const trustedRunnerResults = normalizeTrustedHostRunnerResults({
    action_id: "host-open",
    label: "Open host report",
    command: "flow --completion",
    status: "succeeded",
    approved: true,
    stdout_summary: "Flow Completion Loop",
    stderr_summary: "",
    recorded_at_unix_ms: 2,
  });
  const trustedRunnerUx = normalizeTrustedHostRunnerUx({
    history_json: "tmp/friday-dashboard/trusted-host-runner-history.json",
    result_count: 3,
    latest_status: "timed-out",
    status_summaries: [
      {
        status: "succeeded",
        count: 1,
        title: "Succeeded",
        description: "Approved host commands completed successfully.",
        tone: "ready",
      },
      {
        status: "timed-out",
        count: 1,
        title: "Timed out",
        description: "The runner stopped the command after its timeout.",
        tone: "warn",
      },
      {
        status: "cancelled",
        count: 1,
        title: "Cancelled",
        description: "The operator cancelled this run.",
        tone: "muted",
      },
    ],
    affordances: [
      {
        id: "copy-command-host-open",
        kind: "copy-command",
        action_id: "host-open",
        status: "timed-out",
        label: "Copy command",
        command: "flow --completion",
        detail: "Copy the original local command without executing it.",
        requires_approval: false,
        disabled: false,
      },
      {
        id: "retry-with-approval-host-open",
        kind: "retry",
        action_id: "host-open",
        status: "timed-out",
        label: "Retry with approval",
        command:
          "flow --friday-trusted-host-runner tmp/friday-dashboard --action-id host-open --approve --execute",
        detail: "Prepare the same runner action again.",
        requires_approval: true,
        disabled: false,
      },
    ],
    operator_notes: [
      {
        id: "release-review-link",
        label: "Release review",
        detail: "Attach history to release review.",
        release_review_path: "tmp/friday-dashboard/release-review.json",
      },
    ],
  });
  const checks = [
    check(
      "local-fallback-labelled",
      binding.localOnly && binding.fallback && binding.sourceKind === "embedded-snapshot",
      `${binding.sourceKind}: ${binding.sourceLabel}`,
    ),
    check(
      "local-json-importable",
      imported.localOnly &&
        !imported.fallback &&
        imported.sourceKind === "imported-json" &&
        imported.cards[0]?.actions.length === 1,
      `${imported.sourceKind}: ${imported.sourceLabel}`,
    ),
    check(
      "cards-renderable",
      binding.cards.length === binding.cardCount && binding.boundCardCount === binding.cardCount,
      `${binding.boundCardCount}/${binding.cardCount} cards bound`,
    ),
    check(
      "action-buttons-renderable",
      actionCount === binding.actionCount &&
        binding.cards.every((card) =>
          card.actions.every(
            (action) =>
              action.localOnly &&
              action.command.trim().length > 0 &&
              action.buttonState.idleLabel.trim().length > 0 &&
              action.buttonState.loadingLabel.trim().length > 0 &&
              action.buttonState.successLabel.trim().length > 0 &&
              action.buttonState.errorLabel.trim().length > 0 &&
              action.buttonState.ariaLabel.trim().length > 0,
          ),
        ),
      `${actionCount}/${binding.actionCount} actions renderable`,
    ),
    check(
      "command-dispatch-success",
      preparedResult?.status === "prepared" && preparedResult.permission === "allowed",
      preparedResult?.message ?? "missing dispatch result",
    ),
    check(
      "command-dispatch-confirmation",
      confirmationResult?.status === "blocked" &&
        confirmationResult.permission === "confirmation-required",
      confirmationResult?.message ?? "missing confirmation result",
    ),
    check(
      "command-dispatch-blocked",
      blockedResult?.status === "blocked" && blockedResult.permission === "blocked",
      blockedResult?.message ?? "missing blocked result",
    ),
    check(
      "command-dispatch-failure",
      failedResult?.status === "failed" && failedResult.permission === "blocked",
      failedResult?.message ?? "missing failed result",
    ),
    check(
      "host-bridge-importable",
      hostBridgeResults.length === 1 &&
        hostBridgeResults[0]?.permission === "confirmation-required" &&
        hostBridgeResults[0]?.status === "prepared",
      `${hostBridgeResults.length} host bridge record(s) normalized`,
    ),
    check(
      "trusted-runner-importable",
      trustedRunnerResults.length === 1 &&
        trustedRunnerResults[0]?.permission === "allowed" &&
        trustedRunnerResults[0]?.status === "succeeded",
      `${trustedRunnerResults.length} trusted runner result(s) normalized`,
    ),
    check(
      "trusted-runner-ux-grouped",
      trustedRunnerUx?.statusSummaries.some(
        (summary) => summary.status === "timed-out" && summary.count === 1,
      ) === true &&
        trustedRunnerUx?.statusSummaries.some(
          (summary) => summary.status === "cancelled" && summary.count === 1,
        ) === true,
      `${trustedRunnerUx?.statusSummaries.length ?? 0} trusted runner status groups`,
    ),
    check(
      "trusted-runner-ux-affordances",
      trustedRunnerUx?.affordances.some(
        (affordance) => affordance.kind === "retry" && affordance.requiresApproval,
      ) === true &&
        trustedRunnerUx?.affordances.some(
          (affordance) => affordance.kind === "copy-command" && !affordance.disabled,
        ) === true,
      `${trustedRunnerUx?.affordances.length ?? 0} trusted runner affordance(s)`,
    ),
    check(
      "history-rail-renderable",
      binding.history.recordCount > 0 &&
        binding.history.latestScoreOutOf100 != null &&
        binding.history.previousScoreOutOf100 != null &&
        binding.history.trendLabel.trim().length > 0,
      `${binding.history.recordCount} history records`,
    ),
    check(
      "release-links-renderable",
      binding.releaseLinks.length > 0 &&
        binding.releaseLinks.every(
          (link) =>
            link.localOnly &&
            link.path.trim().length > 0 &&
            link.buttonState.ariaLabel.trim().length > 0,
        ),
      `${binding.releaseLinks.length} release links`,
    ),
    check(
      "screenshot-prompts-renderable",
      binding.screenshotPrompts.length > 0 &&
        binding.screenshotPrompts.every(
          (prompt) =>
            prompt.route.trim().length > 0 &&
            prompt.viewportId.trim().length > 0 &&
            prompt.captureCommand.trim().length > 0,
        ),
      `${binding.screenshotPrompts.length} screenshot prompts`,
    ),
  ];

  return {
    scoreOutOf100: score(checks),
    checks,
  };
}
