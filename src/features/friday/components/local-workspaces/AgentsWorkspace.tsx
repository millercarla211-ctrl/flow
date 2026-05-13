import { openUrl } from "@tauri-apps/plugin-opener";
import { Archive, Bot, CalendarClock, ExternalLink, FileDown, Globe2, Play } from "lucide-react";
import { useEffect, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { isTauriRuntime } from "@/platform/tauriRuntime";
import { makeLocalRecord, useLocalList, useLocalSettings } from "../../hooks/useLocalPersistence";
import { firstExplicitUrl } from "../../utils/externalTargets";
import { exportFridayAgentTask } from "../../utils/localFileExport";
import { createLocalAgentRun } from "../../utils/localAgentRunner";
import { tryRunTauriAgentTask } from "../../utils/tauriAgentRunner";
import { inspectWebSource } from "../../utils/webInspection";
import { EmptyState, INPUT_CLASS, RecordShell, TEXTAREA_CLASS } from "./primitives";
import {
  DEFAULT_CONNECTORS,
  STORAGE_KEYS,
  type AgentTask,
  type CanvasArtifact,
  type ConnectorSettings,
  type FridayAutomation,
  type FridayMemory,
  type FridayProject,
  type ProjectContextItem,
} from "./types";

export function AgentsWorkspace() {
  const { items, addItem, updateItem, removeItem } = useLocalList<AgentTask>(STORAGE_KEYS.agents);
  const artifacts = useLocalList<CanvasArtifact>(STORAGE_KEYS.artifacts);
  const automations = useLocalList<FridayAutomation>(STORAGE_KEYS.automations);
  const projects = useLocalList<FridayProject>(STORAGE_KEYS.projects);
  const projectContext = useLocalList<ProjectContextItem>(STORAGE_KEYS.projectContext);
  const memories = useLocalList<FridayMemory>(STORAGE_KEYS.memory);
  const connectors = useLocalSettings<ConnectorSettings>(STORAGE_KEYS.connectors, DEFAULT_CONNECTORS);
  const [title, setTitle] = useState("");
  const [brief, setBrief] = useState("");
  const [target, setTarget] = useState<AgentTask["target"]>("browser");
  const [projectId, setProjectId] = useState("none");
  const [tauriRuntime, setTauriRuntime] = useState(false);
  const [exportStateById, setExportStateById] = useState<
    Record<string, { status: "ok" | "error"; message: string }>
  >({});
  const [inspectStateById, setInspectStateById] = useState<
    Record<string, { status: "running" | "ok" | "error"; message: string }>
  >({});
  const browserInspectionEnabled = connectors.settings.webSearch && !tauriRuntime;

  useEffect(() => {
    setTauriRuntime(isTauriRuntime());
  }, []);

  const projectForTask = (task: AgentTask) =>
    task.projectId ? projects.items.find((project) => project.id === task.projectId) ?? null : null;

  const contextForTask = (task: AgentTask) => {
    const project = projectForTask(task);
    if (!project) return undefined;
    return {
      projectName: project.name,
      projectInstructions: project.instructions,
      contextItems: [
        ...projectContext.items
          .filter((item) => item.projectId === project.id)
          .slice(0, 5)
          .map((item) => ({
            label: item.label,
            kind: item.kind,
            content: item.content,
          })),
        ...memories.items
          .filter(
            (memory) =>
              memory.pinned &&
              (!memory.projectId || memory.projectId === project.id) &&
              (memory.scope === "Global" || memory.scope === "Project"),
          )
          .slice(0, 3)
          .map((memory) => ({
            label: memory.title,
            kind: "memory",
            content: memory.body,
          })),
      ],
    };
  };

  const createTask = () => {
    const cleanTitle = title.trim();
    if (!cleanTitle) return;
    const selectedProject = projects.items.find((project) => project.id === projectId) ?? null;
    addItem(
      makeLocalRecord("agent", {
        title: cleanTitle,
        brief: brief.trim(),
        target,
        status: "Needs approval",
        projectId: selectedProject?.id,
        projectName: selectedProject?.name,
      }),
    );
    setTitle("");
    setBrief("");
  };

  const runTask = async (task: AgentTask) => {
    updateItem(task.id, { status: "Running" });
    const localRun = await tryRunTauriAgentTask(task, contextForTask(task));
    updateItem(
      task.id,
      localRun
        ? {
            status: "Completed",
            plan: localRun.plan,
            log: localRun.log,
            result: localRun.result,
            inspectedWorkspace: localRun.inspectedWorkspace,
            inspectedUrl: localRun.inspectedUrl,
            lastModel: localRun.model,
            lastTokensPerSecond: localRun.tokensPerSecond,
            lastTotalTimeMs: localRun.totalTimeMs,
          }
        : createLocalAgentRun(task),
    );
  };

  const inspectTaskUrl = async (task: AgentTask, url: string) => {
    if (!browserInspectionEnabled) return;
    setInspectStateById((current) => ({
      ...current,
      [task.id]: { status: "running", message: "Inspecting URL" },
    }));

    const result = await inspectWebSource(url);
    if (!result.ok) {
      setInspectStateById((current) => ({
        ...current,
        [task.id]: { status: "error", message: result.message },
      }));
      updateItem(task.id, {
        log: [
          ...(task.log ?? []),
          `URL inspection failed for ${result.url ?? url}: ${result.message}`,
        ],
        status: "Blocked",
      });
      return;
    }

    const nextLog = [
      ...(task.log ?? []),
      `Inspected explicit URL: ${result.url}`,
      `Captured source title: ${result.title}`,
      `Fetched at ${new Date(result.fetchedAt).toLocaleString()}.`,
    ];
    const resultText = [
      `Browser inspection: ${result.title}`,
      `URL: ${result.url}`,
      "",
      result.excerpt,
    ].join("\n");

    updateItem(task.id, {
      inspectedUrl: true,
      log: nextLog,
      result: resultText,
      status: "Completed",
    });
    setInspectStateById((current) => ({
      ...current,
      [task.id]: { status: "ok", message: "URL inspected" },
    }));
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
        projectId: task.projectId,
        projectName: task.projectName,
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
        projectId: task.projectId,
        projectName: task.projectName,
      }),
    );
  };

  const exportTask = async (task: AgentTask) => {
    const result = await exportFridayAgentTask(task);
    if (result.status === "cancelled") return;
    setExportStateById((current) => ({
      ...current,
      [task.id]:
        result.status === "saved"
          ? { status: "ok", message: "Run exported" }
          : { status: "error", message: result.message },
    }));
  };

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-[1fr_150px_200px_auto]">
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
          {items.map((task) => {
            const taskUrl = firstExplicitUrl(task.title, task.brief);
            const exportState = exportStateById[task.id];
            const inspectState = inspectStateById[task.id];
            return (
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
                {task.projectName && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    {task.projectName}
                  </Badge>
                )}
                {task.inspectedWorkspace && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    Files inspected
                  </Badge>
                )}
                {task.inspectedUrl && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    URL inspected
                  </Badge>
                )}
                {exportState && (
                  <Badge
                    variant="outline"
                    className={
                      exportState.status === "ok"
                        ? "border-[var(--border)] text-[var(--foreground)]"
                        : "border-red-500/40 text-red-300"
                    }
                  >
                    {exportState.message}
                  </Badge>
                )}
                {inspectState && (
                  <Badge
                    variant="outline"
                    className={
                      inspectState.status === "ok"
                        ? "border-emerald-500/40 text-emerald-300"
                        : inspectState.status === "error"
                          ? "border-red-500/40 text-red-300"
                          : "border-[var(--border)]"
                    }
                  >
                    {inspectState.message}
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
                {taskUrl && (
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => openUrl(taskUrl)}
                  >
                    <ExternalLink size={13} />
                    Open URL
                  </Button>
                )}
                {taskUrl && task.target === "browser" && (
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    title={
                      tauriRuntime
                        ? "URL inspection is handled by the desktop agent runner."
                        : browserInspectionEnabled
                          ? "Run a read-only URL inspection for this approved browser task."
                          : "Enable the Web connector before inspecting browser task URLs."
                    }
                    onClick={() => void inspectTaskUrl(task, taskUrl)}
                    disabled={!browserInspectionEnabled || inspectState?.status === "running"}
                  >
                    <Globe2 size={13} />
                    {inspectState?.status === "running" ? "Inspecting" : "Inspect URL"}
                  </Button>
                )}
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
                  onClick={() => void exportTask(task)}
                  disabled={!task.result && !task.plan?.length && !task.brief}
                >
                  <FileDown size={13} />
                  Export
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
            );
          })}
        </div>
      )}
    </div>
  );
}
