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

  const createAutomation = () => {
    const cleanTitle = title.trim();
    if (!cleanTitle) return;
    addItem(makeLocalRecord("automation", { title: cleanTitle, cadence, enabled: true }));
    setTitle("");
  };

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-[1fr_150px_auto]">
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
              <div className="mt-3 flex flex-wrap gap-2">
                <Badge variant="outline" className="border-[var(--border)]">
                  {automation.enabled ? "Active" : "Paused"}
                </Badge>
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
