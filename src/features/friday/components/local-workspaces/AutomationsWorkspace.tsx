import { CalendarClock } from "lucide-react";
import { useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { useFridayAutomationRunner } from "../../hooks/useFridayAutomationRunner";
import { isAutomationDue, nextScheduledAutomationRun } from "../../utils/localAutomation";
import { EmptyState, INPUT_CLASS, RecordShell, TEXTAREA_CLASS } from "./primitives";
import { STORAGE_KEYS, type FridayProject } from "./types";

export function AutomationsWorkspace() {
  const {
    addItem,
    dueAutomation,
    dueCount,
    items,
    removeItem,
    runAutomation,
    runningAutomationId,
    updateItem,
  } = useFridayAutomationRunner();
  const [title, setTitle] = useState("");
  const [instruction, setInstruction] = useState("");
  const [cadence, setCadence] = useState("Daily");
  const [nextRunAt, setNextRunAt] = useState("");
  const [projectId, setProjectId] = useState("none");
  const projects = useLocalList<FridayProject>(STORAGE_KEYS.projects);
  const selectedProject = useMemo(
    () => projects.items.find((project) => project.id === projectId) ?? null,
    [projectId, projects.items],
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
        projectId: selectedProject?.id,
        projectName: selectedProject?.name,
        runCount: 0,
      }),
    );
    setTitle("");
    setInstruction("");
    setNextRunAt("");
  };

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-[1fr_150px_220px_200px_auto]">
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
        <select
          className={INPUT_CLASS}
          value={projectId}
          onChange={(event) => setProjectId(event.target.value)}
        >
          <option value="none">No project</option>
          {projects.items.map((project) => (
            <option key={project.id} value={project.id}>
              {project.name}
            </option>
          ))}
        </select>
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
          Background runner active
        </Badge>
        <Badge variant="outline" className="border-[var(--border)]">
          {dueCount} due
        </Badge>
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={!dueAutomation || Boolean(runningAutomationId)}
          onClick={() => {
            if (dueAutomation) void runAutomation(dueAutomation, "Scheduled");
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
                {automation.projectName && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    {automation.projectName}
                  </Badge>
                )}
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
