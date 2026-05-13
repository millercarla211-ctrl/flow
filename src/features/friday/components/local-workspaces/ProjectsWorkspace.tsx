import { FileDown, FileText, Folder, Paperclip, X } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DEFAULT_FRIDAY_MODEL_KEY, FRIDAY_LOCAL_MODELS } from "@/features/ai";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import {
  CONTEXT_FILE_ACCEPT,
  isSupportedContextFile,
  readContextFile,
} from "../../utils/contextFiles";
import { exportFridayProjectBundle } from "../../utils/localFileExport";
import { titleFromText } from "../../utils/text";
import { EmptyState, INPUT_CLASS, ModelSelect, RecordShell, TEXTAREA_CLASS } from "./primitives";
import {
  STORAGE_KEYS,
  type AgentTask,
  type CanvasArtifact,
  type FridayAskThread,
  type FridayAutomation,
  type FridayMemory,
  type FridayProject,
  type ProjectContextItem,
  type ResearchBrief,
} from "./types";

export function ProjectsWorkspace() {
  const { items, addItem, updateItem, removeItem } = useLocalList<FridayProject>(
    STORAGE_KEYS.projects,
  );
  const projectContext = useLocalList<ProjectContextItem>(STORAGE_KEYS.projectContext);
  const memories = useLocalList<FridayMemory>(STORAGE_KEYS.memory);
  const askThreads = useLocalList<FridayAskThread>(STORAGE_KEYS.askThreads);
  const artifacts = useLocalList<CanvasArtifact>(STORAGE_KEYS.artifacts);
  const research = useLocalList<ResearchBrief>(STORAGE_KEYS.research);
  const agents = useLocalList<AgentTask>(STORAGE_KEYS.agents);
  const automations = useLocalList<FridayAutomation>(STORAGE_KEYS.automations);
  const [name, setName] = useState("");
  const [instructions, setInstructions] = useState("");
  const [modelKey, setModelKey] = useState(DEFAULT_FRIDAY_MODEL_KEY);
  const [selectedProjectId, setSelectedProjectId] = useState("");
  const [editName, setEditName] = useState("");
  const [editInstructions, setEditInstructions] = useState("");
  const [editModelKey, setEditModelKey] = useState(DEFAULT_FRIDAY_MODEL_KEY);
  const [contextDraft, setContextDraft] = useState("");
  const [contextFileError, setContextFileError] = useState<string | null>(null);
  const [exportStateById, setExportStateById] = useState<
    Record<string, { status: "ok" | "error"; message: string }>
  >({});
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const selectedModel = useMemo(
    () => FRIDAY_LOCAL_MODELS.find((model) => model.key === modelKey),
    [modelKey],
  );
  const selectedProject = useMemo(
    () => items.find((project) => project.id === selectedProjectId) ?? null,
    [items, selectedProjectId],
  );
  const selectedProjectContext = useMemo(
    () =>
      selectedProject
        ? projectContext.items.filter((item) => item.projectId === selectedProject.id)
        : [],
    [projectContext.items, selectedProject],
  );

  useEffect(() => {
    if (selectedProjectId && selectedProject) return;
    setSelectedProjectId(items[0]?.id ?? "");
  }, [items, selectedProject, selectedProjectId]);

  useEffect(() => {
    if (!selectedProject) {
      setEditName("");
      setEditInstructions("");
      setEditModelKey(DEFAULT_FRIDAY_MODEL_KEY);
      return;
    }
    setEditName(selectedProject.name);
    setEditInstructions(selectedProject.instructions);
    setEditModelKey(selectedProject.modelKey);
    setContextDraft("");
    setContextFileError(null);
  }, [selectedProject]);

  const createProject = () => {
    const cleanName = name.trim();
    if (!cleanName) return;
    addItem(
      makeLocalRecord("project", { name: cleanName, instructions: instructions.trim(), modelKey }),
    );
    setName("");
    setInstructions("");
  };

  const removeProject = (projectId: string) => {
    removeItem(projectId);
    projectContext.removeWhere((contextItem) => contextItem.projectId === projectId);
    memories.updateWhere(
      (item) => item.projectId === projectId,
      { projectId: undefined, projectName: undefined, scope: "Global" },
    );
    askThreads.updateWhere(
      (item) => item.projectId === projectId,
      { projectId: undefined, projectName: undefined },
    );
    artifacts.updateWhere(
      (item) => item.projectId === projectId,
      { projectId: undefined, projectName: undefined },
    );
    research.updateWhere(
      (item) => item.projectId === projectId,
      { projectId: undefined, projectName: undefined },
    );
    agents.updateWhere(
      (item) => item.projectId === projectId,
      { projectId: undefined, projectName: undefined },
    );
    automations.updateWhere(
      (item) => item.projectId === projectId,
      { projectId: undefined, projectName: undefined },
    );
  };

  const saveSelectedProject = () => {
    if (!selectedProject) return;
    const nextName = editName.trim() || "Untitled project";
    updateItem(selectedProject.id, {
      name: nextName,
      instructions: editInstructions.trim(),
      modelKey: editModelKey,
    });
    projectContext.updateWhere(
      (item) => item.projectId === selectedProject.id && item.projectName !== nextName,
      { projectName: nextName },
    );
    memories.updateWhere(
      (item) => item.projectId === selectedProject.id && item.projectName !== nextName,
      { projectName: nextName },
    );
    askThreads.updateWhere(
      (item) => item.projectId === selectedProject.id && item.projectName !== nextName,
      { projectName: nextName },
    );
    artifacts.updateWhere(
      (item) => item.projectId === selectedProject.id && item.projectName !== nextName,
      { projectName: nextName },
    );
    research.updateWhere(
      (item) => item.projectId === selectedProject.id && item.projectName !== nextName,
      { projectName: nextName },
    );
    agents.updateWhere(
      (item) => item.projectId === selectedProject.id && item.projectName !== nextName,
      { projectName: nextName },
    );
    automations.updateWhere(
      (item) => item.projectId === selectedProject.id && item.projectName !== nextName,
      { projectName: nextName },
    );
  };

  const addContextNote = () => {
    if (!selectedProject) return;
    const cleanContext = contextDraft.trim();
    if (!cleanContext) return;
    projectContext.addItem(
      makeLocalRecord("context", {
        projectId: selectedProject.id,
        projectName: selectedProject.name,
        label: titleFromText(cleanContext, "Project note"),
        kind: "note",
        content: cleanContext,
      }),
    );
    setContextDraft("");
    setContextFileError(null);
  };

  const importContextFiles = async (files: FileList | null) => {
    if (!files?.length || !selectedProject) return;
    const importedItems: ProjectContextItem[] = [];
    const rejected: string[] = [];

    for (const file of Array.from(files).slice(0, 8)) {
      if (!isSupportedContextFile(file)) {
        rejected.push(file.name);
        continue;
      }

      importedItems.push(
        makeLocalRecord("context", {
          projectId: selectedProject.id,
          projectName: selectedProject.name,
          label: file.name,
          kind: "file",
          content: await readContextFile(file),
        }),
      );
    }

    projectContext.addItems(importedItems);

    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
    setContextFileError(
      rejected.length > 0
        ? `Skipped unsupported files: ${rejected.join(", ")}`
        : importedItems.length > 0
          ? null
          : "No supported text files selected.",
    );
  };

  const countProjectRecords = (projectId: string) => ({
    context: projectContext.items.filter((item) => item.projectId === projectId).length,
    memories: memories.items.filter((item) => item.projectId === projectId).length,
    threads: askThreads.items.filter((item) => item.projectId === projectId).length,
    artifacts: artifacts.items.filter((item) => item.projectId === projectId).length,
    research: research.items.filter((item) => item.projectId === projectId).length,
    agents: agents.items.filter((item) => item.projectId === projectId).length,
    automations: automations.items.filter((item) => item.projectId === projectId).length,
  });

  const exportProject = async (project: FridayProject) => {
    const result = await exportFridayProjectBundle({
      project,
      contextItems: projectContext.items.filter((item) => item.projectId === project.id),
      memories: memories.items.filter((item) => item.projectId === project.id),
      threads: askThreads.items.filter((item) => item.projectId === project.id),
      artifacts: artifacts.items.filter((item) => item.projectId === project.id),
      research: research.items.filter((item) => item.projectId === project.id),
      agents: agents.items.filter((item) => item.projectId === project.id),
      automations: automations.items.filter((item) => item.projectId === project.id),
    });

    if (result.status === "cancelled") return;
    setExportStateById((current) => ({
      ...current,
      [project.id]:
        result.status === "saved"
          ? { status: "ok", message: "Project exported" }
          : { status: "error", message: result.message },
    }));
  };

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-[1fr_220px_auto]">
        <input
          className={INPUT_CLASS}
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="Project name"
        />
        <ModelSelect value={modelKey} onChange={setModelKey} />
        <Button type="button" onClick={createProject}>
          New project
        </Button>
      </div>
      <textarea
        className={TEXTAREA_CLASS}
        value={instructions}
        onChange={(event) => setInstructions(event.target.value)}
        placeholder={`Default instructions for ${selectedModel?.label ?? "this model"}`}
      />
      {selectedProject && (
        <div className="rounded-lg border border-[var(--border)] bg-[var(--secondary)] p-3">
          <div className="mb-3 text-xs font-semibold uppercase tracking-[0.14em] text-[var(--muted-foreground)]">
            Edit selected project
          </div>
          <div className="grid gap-3 md:grid-cols-[1fr_220px_auto]">
            <input
              className={INPUT_CLASS}
              value={editName}
              onChange={(event) => setEditName(event.target.value)}
              placeholder="Project name"
            />
            <ModelSelect value={editModelKey} onChange={setEditModelKey} />
            <Button type="button" onClick={saveSelectedProject}>
              Save project
            </Button>
          </div>
          <textarea
            className={`${TEXTAREA_CLASS} mt-3`}
            value={editInstructions}
            onChange={(event) => setEditInstructions(event.target.value)}
            placeholder="Project instructions"
          />
          <div className="mt-4 rounded-lg border border-[var(--border)] bg-[var(--background)] p-3">
            <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
              <div className="text-xs font-semibold uppercase tracking-[0.14em] text-[var(--muted-foreground)]">
                Project context
              </div>
              <Badge variant="outline" className="border-[var(--border)]">
                {selectedProjectContext.length} items
              </Badge>
            </div>
            <div className="grid gap-2 md:grid-cols-[1fr_auto_auto]">
              <input
                className={INPUT_CLASS}
                value={contextDraft}
                onChange={(event) => setContextDraft(event.target.value)}
                placeholder="Add a note, requirement, file summary, or instruction"
              />
              <Button type="button" variant="outline" onClick={addContextNote}>
                <Paperclip size={14} />
                Add note
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={() => fileInputRef.current?.click()}
              >
                <FileText size={14} />
                Import files
              </Button>
              <input
                ref={fileInputRef}
                className="hidden"
                type="file"
                multiple
                accept={CONTEXT_FILE_ACCEPT}
                onChange={(event) => void importContextFiles(event.target.files)}
              />
            </div>
            {contextFileError && (
              <p className="mt-2 text-xs text-[var(--destructive)]">{contextFileError}</p>
            )}
            {selectedProjectContext.length > 0 && (
              <div className="mt-3 flex flex-wrap gap-1.5">
                {selectedProjectContext.map((item) => (
                  <span
                    key={item.id}
                    className="inline-flex max-w-full items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--secondary)] px-2 py-1 text-[11px] text-[var(--muted-foreground)]"
                    title={item.content}
                  >
                    {item.kind === "file" && <FileText size={11} />}
                    <span className="max-w-48 truncate">{item.label}</span>
                    <button
                      type="button"
                      aria-label={`Remove ${item.label}`}
                      className="rounded-sm p-0.5 text-[var(--muted-foreground)] hover:bg-[var(--background)] hover:text-[var(--foreground)]"
                      onClick={() => projectContext.removeItem(item.id)}
                    >
                      <X size={10} />
                    </button>
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
      {items.length === 0 ? (
        <EmptyState
          title="No projects yet"
          body="Create a project to keep files, instructions, memories, and model choices together."
        />
      ) : (
        <div className="space-y-2">
          {items.map((project) => {
            const counts = countProjectRecords(project.id);
            const exportState = exportStateById[project.id];
            return (
              <RecordShell
                key={project.id}
                icon={<Folder size={15} />}
                title={project.name}
                subtitle={
                  FRIDAY_LOCAL_MODELS.find((model) => model.key === project.modelKey)?.label ??
                  project.modelKey
                }
              >
                <p className="mt-2 text-xs leading-5 text-[var(--muted-foreground)]">
                  {project.instructions || "No custom instructions yet"}
                </p>
                <div className="mt-3 flex flex-wrap gap-2">
                  {Object.entries(counts).map(([label, value]) => (
                    <Badge key={label} variant="outline" className="border-[var(--border)]">
                      {value} {label}
                    </Badge>
                  ))}
                  <Badge variant="outline" className="border-[var(--border)]">
                    Local
                  </Badge>
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
                </div>
                <div className="mt-3 flex flex-wrap gap-2">
                  <Button
                    type="button"
                    size="sm"
                    variant={selectedProjectId === project.id ? "default" : "outline"}
                    onClick={() => setSelectedProjectId(project.id)}
                  >
                    Edit
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => void exportProject(project)}
                  >
                    <FileDown size={13} />
                    Export
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => removeProject(project.id)}
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
