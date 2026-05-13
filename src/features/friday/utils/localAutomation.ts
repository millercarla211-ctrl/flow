import type { FridayAutomation } from "../components/local-workspaces/types";

export type FridayAutomationCadence = "Hourly" | "Daily" | "Weekly" | "Manual";

const CADENCE_MS: Record<Exclude<FridayAutomationCadence, "Manual">, number> = {
  Hourly: 60 * 60 * 1000,
  Daily: 24 * 60 * 60 * 1000,
  Weekly: 7 * 24 * 60 * 60 * 1000,
};

export function isFridayAutomationCadence(value: string): value is FridayAutomationCadence {
  return value === "Hourly" || value === "Daily" || value === "Weekly" || value === "Manual";
}

export function nextScheduledAutomationRun(cadence: string, from = new Date()) {
  if (!isFridayAutomationCadence(cadence) || cadence === "Manual") return undefined;
  return new Date(from.getTime() + CADENCE_MS[cadence]).toISOString();
}

export function isAutomationDue(value?: string, now = Date.now()) {
  if (!value) return false;
  const date = new Date(value);
  return !Number.isNaN(date.getTime()) && date.getTime() <= now;
}

export function selectNextDueAutomation<
  T extends Pick<FridayAutomation, "cadence" | "enabled" | "nextRunAt">,
>(
  automations: T[],
  now = Date.now(),
) {
  return automations
    .filter(
      (automation) =>
        automation.enabled &&
        automation.cadence !== "Manual" &&
        isAutomationDue(automation.nextRunAt, now),
    )
    .sort((left, right) => {
      const leftTime = left.nextRunAt
        ? new Date(left.nextRunAt).getTime()
        : Number.MAX_SAFE_INTEGER;
      const rightTime = right.nextRunAt
        ? new Date(right.nextRunAt).getTime()
        : Number.MAX_SAFE_INTEGER;
      return leftTime - rightTime;
    })[0];
}

export function createAutomationPrompt(automation: Pick<FridayAutomation, "cadence" | "instruction" | "title">) {
  const instruction = automation.instruction?.trim();
  return [
    "Run this local Friday automation. Produce a concise result note with outcome, next action, and any blocker.",
    `Automation: ${automation.title}`,
    `Cadence: ${automation.cadence}`,
    instruction ? `Instruction: ${instruction}` : "Instruction: create the most useful local follow-up note.",
  ].join("\n");
}

export function createAutomationFallbackResult(automation: Pick<FridayAutomation, "title">) {
  return `Local run completed for "${automation.title}". Add an instruction to make this automation more specific.`;
}
