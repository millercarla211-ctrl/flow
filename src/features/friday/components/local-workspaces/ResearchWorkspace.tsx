import { Archive, CalendarClock, FileText, Pin, Plus, Quote } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList, useLocalSettings } from "../../hooks/useLocalPersistence";
import { createLocalResearchDraft } from "../../utils/localResearch";
import { createResearchContext, tryDraftTauriResearch } from "../../utils/tauriResearchRunner";
import { firstExplicitUrl } from "../../utils/externalTargets";
import { EmptyState, INPUT_CLASS, RecordShell } from "./primitives";
import {
  DEFAULT_CONNECTORS,
  STORAGE_KEYS,
  type CanvasArtifact,
  type ConnectorSettings,
  type FridayAutomation,
  type FridayMemory,
  type FridayProject,
  type ProjectContextItem,
  type ResearchBrief,
} from "./types";

const RESEARCH_SOURCES = [
  {
    label: "Local files",
    isAvailable: (connectors: ConnectorSettings) => connectors.localFiles,
    unavailableLabel: "Local files off",
  },
  {
    label: "Web",
    isAvailable: (connectors: ConnectorSettings) => connectors.webSearch,
    unavailableLabel: "Enable web connector",
  },
  {
    label: "Academic",
    isAvailable: (connectors: ConnectorSettings) => connectors.webSearch,
    unavailableLabel: "Enable web connector",
  },
  {
    label: "Premium data",
    isAvailable: (connectors: ConnectorSettings) => connectors.mcpConnectors,
    unavailableLabel: "Enable MCP connector",
  },
];

export function ResearchWorkspace() {
  const { items, addItem, updateItem, removeItem } = useLocalList<ResearchBrief>(
    STORAGE_KEYS.research,
  );
  const projects = useLocalList<FridayProject>(STORAGE_KEYS.projects);
  const projectContext = useLocalList<ProjectContextItem>(STORAGE_KEYS.projectContext);
  const memories = useLocalList<FridayMemory>(STORAGE_KEYS.memory);
  const artifacts = useLocalList<CanvasArtifact>(STORAGE_KEYS.artifacts);
  const automations = useLocalList<FridayAutomation>(STORAGE_KEYS.automations);
  const connectors = useLocalSettings<ConnectorSettings>(STORAGE_KEYS.connectors, DEFAULT_CONNECTORS);
  const [topic, setTopic] = useState("");
  const [sources, setSources] = useState(["Local files"]);
  const [projectId, setProjectId] = useState("none");
  const [isDrafting, setIsDrafting] = useState(false);

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

  const createBrief = async () => {
    const cleanTopic = topic.trim();
    if (!cleanTopic || isDrafting) return;
    setIsDrafting(true);
    const availableSources = sources.filter((source) =>
      RESEARCH_SOURCES.find((item) => item.label === source)?.isAvailable(connectors.settings),
    );
    const localDraft = createLocalResearchDraft({
      topic: cleanTopic,
      project: selectedProject,
      contextItems: availableSources.includes("Local files") ? selectedProjectContext : [],
      memories: availableMemories,
    });
    const localModelDraft = await tryDraftTauriResearch({
      topic: cleanTopic,
      citations: localDraft.citations,
      allowUrl: availableSources.includes("Web") || availableSources.includes("Academic"),
      context: createResearchContext({
        project: selectedProject,
        contextItems: availableSources.includes("Local files") ? selectedProjectContext : [],
        memories: availableMemories,
      }),
    });

    addItem(
      makeLocalRecord("research", {
        topic: cleanTopic,
        sources: availableSources,
        plan: localModelDraft?.plan ?? localDraft.plan,
        citations: localDraft.citations,
        report: localModelDraft?.report ?? localDraft.report,
        status: "Drafted",
        inspectedUrl: localModelDraft?.inspectedUrl,
        lastModel: localModelDraft?.model,
        lastTokensPerSecond: localModelDraft?.tokensPerSecond,
        lastTotalTimeMs: localModelDraft?.totalTimeMs,
        projectId: selectedProject?.id,
        projectName: selectedProject?.name,
      }),
    );
    setTopic("");
    setIsDrafting(false);
  };

  const saveBriefArtifact = (brief: ResearchBrief) => {
    if (!brief.report) return;
    artifacts.addItem(
      makeLocalRecord("artifact", {
        title: `Research: ${brief.topic}`,
        kind: "Markdown",
        content: brief.report,
        projectId: brief.projectId,
        projectName: brief.projectName,
      }),
    );
  };

  const pinBriefMemory = (brief: ResearchBrief) => {
    const citationSummary =
      brief.citations
        ?.map((citation, index) => `[${index + 1}] ${citation.label}: ${citation.excerpt}`)
        .join("\n") ?? "";
    const body = [brief.report, citationSummary].filter(Boolean).join("\n\nSources:\n");
    if (!body) return;

    memories.addItem(
      makeLocalRecord("memory", {
        title: `Research memory: ${brief.topic}`,
        body,
        scope: brief.projectId ? "Project" : "Global",
        pinned: true,
        projectId: brief.projectId,
        projectName: brief.projectName,
      }),
    );
  };

  const scheduleBriefFollowUp = (brief: ResearchBrief) => {
    automations.addItem(
      makeLocalRecord("automation", {
        title: `Follow up research: ${brief.topic}`,
        cadence: "Manual",
        enabled: true,
        projectId: brief.projectId,
        projectName: brief.projectName,
      }),
    );
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
        <Button type="button" onClick={createBrief} disabled={isDrafting || !topic.trim()}>
          <Plus size={16} />
          {isDrafting ? "Drafting" : "Draft brief"}
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
          const enabled = sources.includes(source.label);
          const available = source.isAvailable(connectors.settings);
          return (
            <button
              key={source.label}
              type="button"
              disabled={!available}
              title={available ? source.label : source.unavailableLabel}
              onClick={() =>
                setSources((current) =>
                  enabled
                    ? current.filter((item) => item !== source.label)
                    : [...current, source.label],
                )
              }
              className={`rounded-md border px-3 py-1.5 text-xs transition-colors ${
                enabled
                  ? "border-[var(--border-hover)] bg-[var(--secondary)] text-[var(--foreground)]"
                  : "border-[var(--border)] text-[var(--muted-foreground)]"
              } disabled:cursor-not-allowed disabled:opacity-45`}
            >
              {source.label}
              {!available && <span className="ml-1 text-[10px]">Off</span>}
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
          {items.map((brief) => {
            const briefUrl = firstExplicitUrl(brief.topic);
            return (
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
                <Badge variant="outline" className="border-[var(--border)]">
                  {brief.status ?? "Planned"}
                </Badge>
                {brief.lastTokensPerSecond && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    {brief.lastModel} / {brief.lastTokensPerSecond.toFixed(1)} tok/s
                  </Badge>
                )}
                {brief.inspectedUrl && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    URL inspected
                  </Badge>
                )}
                {briefUrl && (
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => openUrl(briefUrl)}
                  >
                    Open URL
                  </Button>
                )}
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() =>
                    updateItem(brief.id, {
                      status: brief.status === "Drafted" ? "Planned" : "Drafted",
                    })
                  }
                >
                  Toggle status
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => saveBriefArtifact(brief)}
                  disabled={!brief.report}
                >
                  <Archive size={13} />
                  Save artifact
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => pinBriefMemory(brief)}
                  disabled={!brief.report && !brief.citations?.length}
                >
                  <Pin size={13} />
                  Pin memory
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => scheduleBriefFollowUp(brief)}
                >
                  <CalendarClock size={13} />
                  Follow up
                </Button>
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
            );
          })}
        </div>
      )}
    </div>
  );
}
