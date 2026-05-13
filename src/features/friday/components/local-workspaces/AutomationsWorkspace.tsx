import { CalendarClock } from "lucide-react";
import { useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { EmptyState, INPUT_CLASS, RecordShell } from "./primitives";
import { STORAGE_KEYS, type FridayAutomation } from "./types";

export function AutomationsWorkspace() {
  const { items, addItem, updateItem, removeItem } = useLocalList<FridayAutomation>(
    STORAGE_KEYS.automations,
  );
  const [title, setTitle] = useState("");
  const [cadence, setCadence] = useState("Daily");
  const [nextRunAt, setNextRunAt] = useState("");

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

  const isDue = (value?: string) => {
    if (!value) return false;
    const date = new Date(value);
    return !Number.isNaN(date.getTime()) && date.getTime() <= Date.now();
  };

  const createAutomation = () => {
    const cleanTitle = title.trim();
    if (!cleanTitle) return;
    addItem(
      makeLocalRecord("automation", {
        title: cleanTitle,
        cadence,
        enabled: true,
        nextRunAt: nextRunAt ? new Date(nextRunAt).toISOString() : undefined,
        runCount: 0,
      }),
    );
    setTitle("");
    setNextRunAt("");
  };

  const runAutomation = (automation: FridayAutomation) => {
    const now = new Date().toISOString();
    updateItem(automation.id, {
      lastRunAt: now,
      lastResult: `Manual local run completed for "${automation.title}". No cloud worker was started.`,
      runCount: (automation.runCount ?? 0) + 1,
    });
  };

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
              {automation.lastResult && (
                <p className="mt-3 rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3 text-xs leading-5 text-[var(--muted-foreground)]">
                  {automation.lastResult}
                </p>
              )}
              <div className="mt-3 flex flex-wrap gap-2">
                <Badge variant="outline" className="border-[var(--border)]">
                  {automation.enabled ? "Active" : "Paused"}
                </Badge>
                {isDue(automation.nextRunAt) && automation.enabled && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    Due now
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
                  onClick={() => runAutomation(automation)}
                  disabled={!automation.enabled}
                >
                  Run now
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
