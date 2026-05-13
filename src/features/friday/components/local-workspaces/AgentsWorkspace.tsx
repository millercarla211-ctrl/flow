import { Archive, Bot, CalendarClock, Play } from "lucide-react";
import { useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { createLocalAgentRun } from "../../utils/localAgentRunner";
import { tryRunTauriAgentTask } from "../../utils/tauriAgentRunner";
import { EmptyState, INPUT_CLASS, RecordShell, TEXTAREA_CLASS } from "./primitives";
import {
  STORAGE_KEYS,
  type AgentTask,
  type CanvasArtifact,
  type FridayAutomation,
} from "./types";

export function AgentsWorkspace() {
  const { items, addItem, updateItem, removeItem } = useLocalList<AgentTask>(STORAGE_KEYS.agents);
  const artifacts = useLocalList<CanvasArtifact>(STORAGE_KEYS.artifacts);
  const automations = useLocalList<FridayAutomation>(STORAGE_KEYS.automations);
  const [title, setTitle] = useState("");
  const [brief, setBrief] = useState("");
  const [target, setTarget] = useState<AgentTask["target"]>("browser");

  const createTask = () => {
    const cleanTitle = title.trim();
    if (!cleanTitle) return;
    addItem(
      makeLocalRecord("agent", {
        title: cleanTitle,
        brief: brief.trim(),
        target,
        status: "Needs approval",
      }),
    );
    setTitle("");
    setBrief("");
  };

  const runTask = async (task: AgentTask) => {
    updateItem(task.id, { status: "Running" });
    const localRun = await tryRunTauriAgentTask(task);
    updateItem(
      task.id,
      localRun
        ? {
            status: "Completed",
            plan: localRun.plan,
            log: localRun.log,
            result: localRun.result,
            inspectedWorkspace: localRun.inspectedWorkspace,
            lastModel: localRun.model,
            lastTokensPerSecond: localRun.tokensPerSecond,
            lastTotalTimeMs: localRun.totalTimeMs,
          }
        : createLocalAgentRun(task),
    );
  };

  const saveTaskArtifact = (task: AgentTask) => {
    if (!task.result) return;
    const content = [
      `# Agent run: ${task.title}`,
      task.brief ? `\n## Brief\n${task.brief}` : "",
      `\n## Result\n${task.result}`,
    ]
      .filter(Boolean)
      .join("\n");
    artifacts.addItem(
      makeLocalRecord("artifact", {
        title: `Agent run: ${task.title}`,
        kind: "Markdown",
        content,
      }),
    );
  };

  const scheduleTaskFollowUp = (task: AgentTask) => {
    automations.addItem(
      makeLocalRecord("automation", {
        title: `Follow up agent task: ${task.title}`,
        instruction: [
          task.brief ? `Original brief: ${task.brief}` : "",
          task.result ? `Previous result: ${task.result}` : "",
        ]
          .filter(Boolean)
          .join("\n\n"),
        cadence: "Manual",
        enabled: true,
      }),
    );
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
      <textarea
        className={TEXTAREA_CLASS}
        value={brief}
        onChange={(event) => setBrief(event.target.value)}
        placeholder="Add URL, folder, file names, constraints, acceptance criteria, or notes for this agent run."
      />
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
              {task.brief && (
                <p className="mt-2 rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3 text-xs leading-5 text-[var(--muted-foreground)]">
                  {task.brief}
                </p>
              )}
              {task.plan && task.plan.length > 0 && (
                <ol className="mt-3 space-y-1 text-xs leading-5 text-[var(--muted-foreground)]">
                  {task.plan.map((step) => (
                    <li key={step}>{step}</li>
                  ))}
                </ol>
              )}
              {task.log && task.log.length > 0 && (
                <div className="mt-3 rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--muted-foreground)]">
                    Run log
                  </div>
                  <div className="mt-2 space-y-1 text-xs leading-5 text-[var(--muted-foreground)]">
                    {task.log.map((line) => (
                      <p key={line}>{line}</p>
                    ))}
                  </div>
                </div>
              )}
              {task.result && (
                <pre className="mt-3 max-h-44 overflow-auto rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3 whitespace-pre-wrap text-xs leading-5 text-[var(--muted-foreground)]">
                  {task.result}
                </pre>
              )}
              <div className="mt-3 flex flex-wrap gap-2">
                <Badge variant="outline" className="border-[var(--border)]">
                  {task.status}
                </Badge>
                {task.lastTokensPerSecond && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    {task.lastModel} / {task.lastTokensPerSecond.toFixed(1)} tok/s
                  </Badge>
                )}
                {task.inspectedWorkspace && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    Files inspected
                  </Badge>
                )}
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => updateItem(task.id, { status: "Queued" })}
                  disabled={task.status !== "Needs approval"}
                >
                  Approve queue
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => runTask(task)}
                  disabled={task.status === "Needs approval" || task.status === "Running"}
                >
                  <Play size={13} />
                  Run local plan
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => saveTaskArtifact(task)}
                  disabled={!task.result}
                >
                  <Archive size={13} />
                  Save artifact
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => scheduleTaskFollowUp(task)}
                >
                  <CalendarClock size={13} />
                  Follow up
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
