import { Folder } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DEFAULT_FRIDAY_MODEL_KEY, FRIDAY_LOCAL_MODELS } from "@/features/ai";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
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

  const selectedModel = useMemo(
    () => FRIDAY_LOCAL_MODELS.find((model) => model.key === modelKey),
    [modelKey],
  );
  const selectedProject = useMemo(
    () => items.find((project) => project.id === selectedProjectId) ?? null,
    [items, selectedProjectId],
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
  };

  const saveSelectedProject = () => {
    if (!selectedProject) return;
    updateItem(selectedProject.id, {
      name: editName.trim() || "Untitled project",
      instructions: editInstructions.trim(),
      modelKey: editModelKey,
    });
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
