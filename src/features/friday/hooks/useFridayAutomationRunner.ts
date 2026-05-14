"use client";

import { useCallback, useMemo, useState } from "react";

import { resolveFridayModel } from "@/features/ai";
import { tryRunTauriLocalChat } from "@/features/ai/tauri-local-chat";
import { useLocalList } from "./useLocalPersistence";
import {
  createAutomationFallbackResult,
  createAutomationFailureResult,
  createAutomationPrompt,
  isAutomationDue,
  nextScheduledAutomationRun,
  selectNextDueAutomation,
} from "../utils/localAutomation";
import { STORAGE_KEYS, type FridayAutomation } from "../components/local-workspaces/types";

let activeAutomationRunId: string | null = null;

export function useFridayAutomationRunner() {
  const automations = useLocalList<FridayAutomation>(STORAGE_KEYS.automations);
  const [runningAutomationId, setRunningAutomationId] = useState<string | null>(null);
  const dueAutomation = useMemo(
    () => selectNextDueAutomation(automations.items),
    [automations.items],
  );
  const dueCount = useMemo(
    () =>
      automations.items.filter(
        (automation) =>
          automation.enabled &&
          automation.cadence !== "Manual" &&
          isAutomationDue(automation.nextRunAt),
      ).length,
    [automations.items],
  );

  const runAutomation = useCallback(
    async (automation: FridayAutomation, mode: "Manual" | "Scheduled" = "Manual") => {
      if (activeAutomationRunId) return false;

      activeAutomationRunId = automation.id;
      setRunningAutomationId(automation.id);
      const now = new Date().toISOString();

      try {
        const localRun = await tryRunTauriLocalChat({
          prompt: createAutomationPrompt(automation),
          model: resolveFridayModel("qwen3-0.6b"),
        });
        automations.updateItem(automation.id, {
          lastRunAt: now,
          lastResult: localRun?.text.trim() || createAutomationFallbackResult(automation),
          lastRunStatus: "Completed",
          lastError: undefined,
          runCount: (automation.runCount ?? 0) + 1,
          lastRunMode: mode,
          nextRunAt: nextScheduledAutomationRun(automation.cadence),
          lastModel: localRun?.model,
          lastTokensPerSecond: localRun?.tokensPerSecond,
          lastTotalTimeMs: localRun?.totalTimeMs,
        });
        return true;
      } catch (error) {
        const failure = createAutomationFailureResult(error);
        automations.updateItem(automation.id, {
          lastRunAt: now,
          lastResult: failure.result,
          lastRunStatus: "Failed",
          lastError: failure.message,
          runCount: automation.runCount ?? 0,
          lastRunMode: mode,
          nextRunAt: nextScheduledAutomationRun(automation.cadence),
        });
        return false;
      } finally {
        activeAutomationRunId = null;
        setRunningAutomationId(null);
      }
    },
    [automations],
  );

  return {
    ...automations,
    dueAutomation,
    dueCount,
    runAutomation,
    runningAutomationId,
  };
}
