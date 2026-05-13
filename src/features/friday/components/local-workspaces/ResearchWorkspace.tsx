import { FileText, Plus, Quote } from "lucide-react";
import { useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { createLocalResearchDraft } from "../../utils/localResearch";
import { EmptyState, INPUT_CLASS, RecordShell } from "./primitives";
import {
  STORAGE_KEYS,
  type FridayMemory,
  type FridayProject,
  type ProjectContextItem,
  type ResearchBrief,
} from "./types";

const RESEARCH_SOURCES = ["Local files", "Web", "Academic", "Premium data"];

export function ResearchWorkspace() {
  const { items, addItem, removeItem } = useLocalList<ResearchBrief>(STORAGE_KEYS.research);
  const projects = useLocalList<FridayProject>(STORAGE_KEYS.projects);
  const projectContext = useLocalList<ProjectContextItem>(STORAGE_KEYS.projectContext);
  const memories = useLocalList<FridayMemory>(STORAGE_KEYS.memory);
  const [topic, setTopic] = useState("");
  const [sources, setSources] = useState(["Local files", "Web"]);
  const [projectId, setProjectId] = useState("none");

  const selectedProject = useMemo(
    () => projects.items.find((project) => project.id === projectId) ?? null,
    [projectId, projects.items],
  );

  const selectedProjectContext = useMemo(
    () =>
      selectedProject
        ? projectContext.items.filter((item) => item.projectId === selectedProject.id)
        : projectContext.items,
    [projectContext.items, selectedProject],
  );

  const availableMemories = useMemo(
    () =>
      memories.items.filter(
        (memory) =>
          memory.pinned &&
          (!selectedProject || !memory.projectId || memory.projectId === selectedProject.id),
      ),
    [memories.items, selectedProject],
  );

  const createBrief = () => {
    const cleanTopic = topic.trim();
    if (!cleanTopic) return;
    const localDraft = createLocalResearchDraft({
      topic: cleanTopic,
      project: selectedProject,
      contextItems: sources.includes("Local files") ? selectedProjectContext : [],
      memories: availableMemories,
    });

    addItem(
      makeLocalRecord("research", {
        topic: cleanTopic,
        sources,
        plan: localDraft.plan,
        citations: localDraft.citations,
        report: localDraft.report,
        status: "Drafted",
        projectId: selectedProject?.id,
        projectName: selectedProject?.name,
      }),
    );
    setTopic("");
  };

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-[1fr_auto]">
        <input
          className={INPUT_CLASS}
          value={topic}
          onChange={(event) => setTopic(event.target.value)}
          placeholder="Research topic or question"
        />
        <Button type="button" onClick={createBrief}>
          <Plus size={16} />
          Draft brief
        </Button>
      </div>
      <div className="grid gap-3 md:grid-cols-[1fr_auto]">
        <select
          className={INPUT_CLASS}
          value={projectId}
          onChange={(event) => setProjectId(event.target.value)}
        >
          <option value="none">All local context</option>
          {projects.items.map((project) => (
            <option key={project.id} value={project.id}>
              {project.name}
            </option>
          ))}
        </select>
        <Badge variant="outline" className="h-9 justify-center border-[var(--border)] px-3">
          {selectedProjectContext.length} context / {availableMemories.length} memories
        </Badge>
      </div>
      <div className="flex flex-wrap gap-2">
        {RESEARCH_SOURCES.map((source) => {
          const enabled = sources.includes(source);
          return (
            <button
              key={source}
              type="button"
              onClick={() =>
                setSources((current) =>
                  enabled ? current.filter((item) => item !== source) : [...current, source],
                )
              }
              className={`rounded-md border px-3 py-1.5 text-xs transition-colors ${
                enabled
                  ? "border-[var(--border-hover)] bg-[var(--secondary)] text-[var(--foreground)]"
                  : "border-[var(--border)] text-[var(--muted-foreground)]"
              }`}
            >
              {source}
            </button>
          );
        })}
      </div>
      {items.length === 0 ? (
        <EmptyState
          title="No research briefs yet"
          body="Create a topic and Friday will keep the plan locally until source connectors are enabled."
        />
      ) : (
        <div className="space-y-2">
          {items.map((brief) => (
            <RecordShell
              key={brief.id}
              icon={<FileText size={15} />}
              title={brief.topic}
              subtitle={`Sources: ${brief.sources.join(", ") || "Local only"}`}
            >
              <ol className="mt-3 space-y-1 text-xs leading-5 text-[var(--muted-foreground)]">
                {brief.plan.map((step) => (
                  <li key={step}>{step}</li>
                ))}
              </ol>
              {brief.citations && brief.citations.length > 0 && (
                <div className="mt-3 grid gap-2">
                  {brief.citations.map((citation, index) => (
                    <div
                      key={`${citation.id}-${index}`}
                      className="rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3"
                    >
                      <div className="flex items-center gap-2 text-xs font-semibold text-[var(--foreground)]">
                        <Quote size={13} />[{index + 1}] {citation.label}
                        <Badge
                          variant="outline"
                          className="ml-auto border-[var(--border)] text-[10px]"
                        >
                          {citation.kind}
                        </Badge>
                      </div>
                      <p className="mt-2 text-xs leading-5 text-[var(--muted-foreground)]">
                        {citation.excerpt}
                      </p>
                    </div>
                  ))}
                </div>
              )}
              {brief.report && (
                <pre className="mt-3 max-h-48 overflow-auto rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3 whitespace-pre-wrap text-xs leading-5 text-[var(--muted-foreground)]">
                  {brief.report}
                </pre>
              )}
              <div className="mt-3 flex flex-wrap gap-2">
                {brief.projectName && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    {brief.projectName}
                  </Badge>
                )}
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => removeItem(brief.id)}
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
