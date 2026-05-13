import { Bot } from "lucide-react";
import { useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { EmptyState, INPUT_CLASS, RecordShell } from "./primitives";
import { STORAGE_KEYS, type AgentTask } from "./types";

export function AgentsWorkspace() {
  const { items, addItem, updateItem, removeItem } = useLocalList<AgentTask>(STORAGE_KEYS.agents);
  const [title, setTitle] = useState("");
  const [target, setTarget] = useState<AgentTask["target"]>("browser");

  const createTask = () => {
    const cleanTitle = title.trim();
    if (!cleanTitle) return;
    addItem(makeLocalRecord("agent", { title: cleanTitle, target, status: "Needs approval" }));
    setTitle("");
  };

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-[1fr_150px_auto]">
        <input
          className={INPUT_CLASS}
          value={title}
          onChange={(event) => setTitle(event.target.value)}
          placeholder="Agent task"
        />
        <select
          className={INPUT_CLASS}
          value={target}
          onChange={(event) => setTarget(event.target.value as AgentTask["target"])}
        >
          <option value="browser">Browser</option>
          <option value="code">Code</option>
          <option value="files">Files</option>
        </select>
        <Button type="button" onClick={createTask}>
          Queue
        </Button>
      </div>
      {items.length === 0 ? (
        <EmptyState
          title="No agent tasks queued"
          body="Queue a task first. Friday keeps every task in approval mode before any tool execution."
        />
      ) : (
        <div className="space-y-2">
          {items.map((task) => (
            <RecordShell
              key={task.id}
              icon={<Bot size={15} />}
              title={task.title}
              subtitle={`${task.target} task`}
            >
              <div className="mt-3 flex flex-wrap gap-2">
                <Badge variant="outline" className="border-[var(--border)]">
                  {task.status}
                </Badge>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => updateItem(task.id, { status: "Queued" })}
                >
                  Approve queue
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => removeItem(task.id)}
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
