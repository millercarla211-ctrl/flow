import { defaultFridayDashboardBinding } from "../runtime/dashboard-binding";
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
  const actionCount = binding.cards.reduce((total, card) => total + card.actions.length, 0);
  const checks = [
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
