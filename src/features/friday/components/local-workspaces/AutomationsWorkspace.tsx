import { CalendarClock } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { resolveFridayModel } from "@/features/ai";
import { tryRunTauriLocalChat } from "@/features/ai/tauri-local-chat";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import {
  createAutomationFallbackResult,
  createAutomationPrompt,
  isAutomationDue,
  nextScheduledAutomationRun,
} from "../../utils/localAutomation";
import { EmptyState, INPUT_CLASS, RecordShell, TEXTAREA_CLASS } from "./primitives";
import { STORAGE_KEYS, type FridayAutomation } from "./types";

export function AutomationsWorkspace() {
  const { items, addItem, updateItem, removeItem } = useLocalList<FridayAutomation>(
    STORAGE_KEYS.automations,
  );
  const [title, setTitle] = useState("");
  const [instruction, setInstruction] = useState("");
  const [cadence, setCadence] = useState("Daily");
  const [nextRunAt, setNextRunAt] = useState("");
  const [autoRunEnabled, setAutoRunEnabled] = useState(true);
  const [runningAutomationId, setRunningAutomationId] = useState<string | null>(null);
  const dueAutomations = useMemo(
    () =>
      items.filter(
        (automation) =>
          automation.enabled &&
          automation.cadence !== "Manual" &&
          isAutomationDue(automation.nextRunAt),
      ),
    [items],
  );

  const formatRunTime = (value?: string) => {
    if (!value) return "Not scheduled";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return "Invalid date";
    return new Intl.DateTimeFormat(undefined, {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    }).format(date);
  };

  const createAutomation = () => {
    const cleanTitle = title.trim();
    if (!cleanTitle) return;
    addItem(
      makeLocalRecord("automation", {
        title: cleanTitle,
        instruction: instruction.trim(),
        cadence,
        enabled: true,
        nextRunAt:
          nextRunAt && cadence !== "Manual"
            ? new Date(nextRunAt).toISOString()
            : nextScheduledAutomationRun(cadence),
        runCount: 0,
      }),
    );
    setTitle("");
    setInstruction("");
    setNextRunAt("");
  };

  const runAutomation = useCallback(
    async (automation: FridayAutomation, mode: "Manual" | "Scheduled" = "Manual") => {
      if (runningAutomationId) return;
      setRunningAutomationId(automation.id);
      const now = new Date().toISOString();
      try {
        const localRun = await tryRunTauriLocalChat({
          prompt: createAutomationPrompt(automation),
          model: resolveFridayModel("qwen3-0.6b"),
        });
        updateItem(automation.id, {
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
      } catch (error) {
        const message = error instanceof Error ? error.message : "Unknown automation error";
        updateItem(automation.id, {
          lastRunAt: now,
          lastResult: `Automation failed: ${message}`,
          lastRunStatus: "Failed",
          lastError: message,
          runCount: automation.runCount ?? 0,
          lastRunMode: mode,
          nextRunAt: nextScheduledAutomationRun(automation.cadence),
        });
      } finally {
        setRunningAutomationId(null);
      }
    },
    [runningAutomationId, updateItem],
  );

  useEffect(() => {
    if (!autoRunEnabled || runningAutomationId || dueAutomations.length === 0) return;
    const timer = window.setTimeout(() => {
      void runAutomation(dueAutomations[0], "Scheduled");
    }, 800);
    return () => window.clearTimeout(timer);
  }, [autoRunEnabled, dueAutomations, runAutomation, runningAutomationId]);

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-[1fr_150px_220px_auto]">
        <input
          className={INPUT_CLASS}
          value={title}
          onChange={(event) => setTitle(event.target.value)}
          placeholder="Automation name"
        />
        <select
          className={INPUT_CLASS}
          value={cadence}
          onChange={(event) => setCadence(event.target.value)}
        >
          <option>Hourly</option>
          <option>Daily</option>
          <option>Weekly</option>
          <option>Manual</option>
        </select>
        <input
          className={INPUT_CLASS}
          type="datetime-local"
          value={nextRunAt}
          onChange={(event) => setNextRunAt(event.target.value)}
        />
        <Button type="button" onClick={createAutomation}>
          Schedule
        </Button>
      </div>
      <textarea
        className={TEXTAREA_CLASS}
        value={instruction}
        onChange={(event) => setInstruction(event.target.value)}
        placeholder="What should Friday do when this runs?"
      />
      <div className="flex flex-wrap items-center gap-2">
        <Badge variant="outline" className="border-[var(--border)]">
          {autoRunEnabled ? "Auto-run active" : "Auto-run paused"}
        </Badge>
        <Badge variant="outline" className="border-[var(--border)]">
          {dueAutomations.length} due
        </Badge>
        <Button
          type="button"
          size="sm"
          variant="outline"
          onClick={() => setAutoRunEnabled((enabled) => !enabled)}
        >
          {autoRunEnabled ? "Pause due runner" : "Resume due runner"}
        </Button>
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={dueAutomations.length === 0 || Boolean(runningAutomationId)}
          onClick={() => {
            const nextDueAutomation = dueAutomations[0];
            if (nextDueAutomation) void runAutomation(nextDueAutomation, "Scheduled");
          }}
        >
          Run next due
        </Button>
      </div>
      {items.length === 0 ? (
        <EmptyState
          title="No automations scheduled"
          body="Create a local reminder or recurring task. Background execution stays explicit."
        />
      ) : (
        <div className="space-y-2">
          {items.map((automation) => (
            <RecordShell
              key={automation.id}
              icon={<CalendarClock size={15} />}
              title={automation.title}
              subtitle={automation.cadence}
            >
              <div className="mt-3 grid gap-2 text-xs leading-5 text-[var(--muted-foreground)] md:grid-cols-3">
                <div>Next: {formatRunTime(automation.nextRunAt)}</div>
                <div>Last: {formatRunTime(automation.lastRunAt)}</div>
                <div>Runs: {automation.runCount ?? 0}</div>
              </div>
              {automation.lastRunMode && (
                <div className="mt-3 flex flex-wrap gap-2">
                  <Badge variant="outline" className="border-[var(--border)]">
                    Last run: {automation.lastRunMode}
                  </Badge>
                  {automation.lastRunStatus && (
                    <Badge
                      variant="outline"
                      className={
                        automation.lastRunStatus === "Completed"
                          ? "border-emerald-500/40 text-emerald-300"
                          : "border-red-500/40 text-red-300"
                      }
                    >
                      {automation.lastRunStatus}
                    </Badge>
                  )}
                </div>
              )}
              {automation.lastResult && (
                <p className="mt-3 rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3 text-xs leading-5 text-[var(--muted-foreground)]">
                  {automation.lastResult}
                </p>
              )}
              <div className="mt-3 flex flex-wrap gap-2">
                <Badge variant="outline" className="border-[var(--border)]">
                  {automation.enabled ? "Active" : "Paused"}
                </Badge>
                {isAutomationDue(automation.nextRunAt) && automation.enabled && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    Due now
                  </Badge>
                )}
                {automation.lastTokensPerSecond && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    {automation.lastModel} / {automation.lastTokensPerSecond.toFixed(1)} tok/s
                  </Badge>
                )}
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => updateItem(automation.id, { enabled: !automation.enabled })}
                >
                  {automation.enabled ? "Pause" : "Resume"}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => void runAutomation(automation, "Manual")}
                  disabled={!automation.enabled || runningAutomationId === automation.id}
                >
                  {runningAutomationId === automation.id ? "Running" : "Run now"}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => removeItem(automation.id)}
                >
                  Remove
                </Button>
              </div>
            </RecordShell>
          ))}
        </div>
      )}
    </div>
  );
}
