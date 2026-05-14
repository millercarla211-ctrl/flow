import {
  Archive,
  Bot,
  CalendarClock,
  FileDown,
  FileText,
  Globe2,
  Pin,
  Plus,
  Quote,
  Search,
  Sparkles,
} from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList, useLocalSettings } from "../../hooks/useLocalPersistence";
import { createLocalResearchDraft, createResearchAgentTaskDraft } from "../../utils/localResearch";
import { exportFridayResearchBrief } from "../../utils/localFileExport";
import { synthesizeResearchWithProvider } from "../../utils/providerResearch";
import { inspectWebSource } from "../../utils/webInspection";
import { searchWebSources, type WebSearchResultItem } from "../../utils/webSearch";
import { createResearchContext, tryDraftTauriResearch } from "../../utils/tauriResearchRunner";
import { firstExplicitUrl } from "../../utils/externalTargets";
import { isTauriRuntime } from "@/platform/tauriRuntime";
import { EmptyState, INPUT_CLASS, RecordShell } from "./primitives";
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
  const agents = useLocalList<AgentTask>(STORAGE_KEYS.agents);
  const automations = useLocalList<FridayAutomation>(STORAGE_KEYS.automations);
  const connectors = useLocalSettings<ConnectorSettings>(STORAGE_KEYS.connectors, DEFAULT_CONNECTORS);
  const [topic, setTopic] = useState("");
  const [sources, setSources] = useState(["Local files"]);
  const [projectId, setProjectId] = useState("none");
  const [isDrafting, setIsDrafting] = useState(false);
  const [tauriRuntime, setTauriRuntime] = useState(false);
  const [exportStateById, setExportStateById] = useState<
    Record<string, { status: "ok" | "error"; message: string }>
  >({});
  const [providerStateById, setProviderStateById] = useState<
    Record<string, { status: "running" | "ok" | "error"; message: string }>
  >({});
  const [webStateById, setWebStateById] = useState<
    Record<string, { status: "running" | "ok" | "error"; message: string }>
  >({});
  const [webResultsById, setWebResultsById] = useState<Record<string, WebSearchResultItem[]>>({});
  const cloudEnvEnabled =
    process.env.NEXT_PUBLIC_FRIDAY_ENABLE_CLOUD_AI === "true" ||
    process.env.NEXT_PUBLIC_FRIDAY_ENABLE_GROQ_AI === "true";
  const providerSynthesisEnabled =
    connectors.settings.aiGateway && cloudEnvEnabled && !tauriRuntime;
  const webInspectionEnabled = connectors.settings.webSearch && !tauriRuntime;

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

  useEffect(() => {
    setTauriRuntime(isTauriRuntime());
  }, []);

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
        instruction: [
          `Research topic: ${brief.topic}`,
          brief.sources?.length ? `Sources: ${brief.sources.join(", ")}` : "",
          brief.report ? `Current report:\n${brief.report}` : "",
          "Check whether the brief needs fresher sources, clearer citations, or a concrete next action.",
        ]
          .filter(Boolean)
          .join("\n\n"),
        cadence: "Manual",
        enabled: true,
        projectId: brief.projectId,
        projectName: brief.projectName,
      }),
    );
  };

  const queueBriefAgent = (brief: ResearchBrief) => {
    agents.addItem(
      makeLocalRecord("agent", {
        ...createResearchAgentTaskDraft(brief),
        projectId: brief.projectId,
        projectName: brief.projectName,
      }),
    );
  };

  const exportBrief = async (brief: ResearchBrief) => {
    const result = await exportFridayResearchBrief(brief);
    if (result.status === "cancelled") return;
    setExportStateById((current) => ({
      ...current,
      [brief.id]:
        result.status === "saved"
          ? { status: "ok", message: "Brief exported" }
          : { status: "error", message: result.message },
    }));
  };

  const synthesizeBrief = async (brief: ResearchBrief) => {
    if (!providerSynthesisEnabled) return;
    setProviderStateById((current) => ({
      ...current,
      [brief.id]: { status: "running", message: "Provider synthesis running" },
    }));
    const result = await synthesizeResearchWithProvider({ brief });
    if (!result.ok) {
      setProviderStateById((current) => ({
        ...current,
        [brief.id]: { status: "error", message: result.message },
      }));
      return;
    }

    updateItem(brief.id, {
      lastModel: result.modelKey,
      lastTotalTimeMs: result.latencyMs,
      report: result.report,
      status: "Drafted",
    });
    setProviderStateById((current) => ({
      ...current,
      [brief.id]: { status: "ok", message: `Synthesized in ${(result.latencyMs / 1000).toFixed(1)}s` },
    }));
  };

  const inspectBriefUrl = async (brief: ResearchBrief, url: string) => {
    if (!webInspectionEnabled) return;
    setWebStateById((current) => ({
      ...current,
      [brief.id]: { status: "running", message: "Inspecting source" },
    }));
    const result = await inspectWebSource(url);
    if (!result.ok) {
      setWebStateById((current) => ({
        ...current,
        [brief.id]: { status: "error", message: result.message },
      }));
      return;
    }

    const nextCitation = {
      id: `web:${result.url}`,
      label: result.title || result.url,
      kind: "web" as const,
      excerpt: result.excerpt,
    };
    const citations = brief.citations ?? [];
    const nextCitations = citations.some((citation) => citation.id === nextCitation.id)
      ? citations.map((citation) => (citation.id === nextCitation.id ? nextCitation : citation))
      : [...citations, nextCitation];
    const nextReport = [
      brief.report,
      "",
      `### Inspected Web Source`,
      `[${nextCitations.length}] ${nextCitation.label}: ${nextCitation.excerpt}`,
    ]
      .filter(Boolean)
      .join("\n");

    updateItem(brief.id, {
      citations: nextCitations,
      inspectedUrl: true,
      report: nextReport,
      sources: Array.from(new Set([...brief.sources, "Web"])),
      status: "Drafted",
    });
    setWebStateById((current) => ({
      ...current,
      [brief.id]: { status: "ok", message: "Source inspected" },
    }));
  };

  const findBriefSources = async (brief: ResearchBrief) => {
    if (!webInspectionEnabled) return;
    setWebStateById((current) => ({
      ...current,
      [brief.id]: { status: "running", message: "Searching sources" },
    }));
    const result = await searchWebSources(brief.topic);
    if (!result.ok) {
      setWebStateById((current) => ({
        ...current,
        [brief.id]: { status: "error", message: result.message },
      }));
      return;
    }

    setWebResultsById((current) => ({ ...current, [brief.id]: result.results }));
    setWebStateById((current) => ({
      ...current,
      [brief.id]: { status: "ok", message: `${result.results.length} sources found` },
    }));
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
            const exportState = exportStateById[brief.id];
            const providerState = providerStateById[brief.id];
            const webState = webStateById[brief.id];
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
              {webResultsById[brief.id]?.length > 0 && (
                <div className="mt-3 grid gap-2">
                  {webResultsById[brief.id].map((result) => (
                    <div
                      key={result.url}
                      className="rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3"
                    >
                      <div className="flex flex-wrap items-start justify-between gap-2">
                        <div className="min-w-0">
                          <div className="truncate text-xs font-semibold text-[var(--foreground)]">
                            {result.title}
                          </div>
                          <div className="mt-1 text-[11px] text-[var(--muted-foreground)]">
                            {result.source}
                          </div>
                        </div>
                        <div className="flex shrink-0 gap-2">
                          <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            onClick={() => openUrl(result.url)}
                          >
                            Open
                          </Button>
                          <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            onClick={() => void inspectBriefUrl(brief, result.url)}
                            disabled={webState?.status === "running"}
                          >
                            Inspect
                          </Button>
                        </div>
                      </div>
                      <p className="mt-2 text-xs leading-5 text-[var(--muted-foreground)]">
                        {result.snippet}
                      </p>
                    </div>
                  ))}
                </div>
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
                {providerState && (
                  <Badge
                    variant="outline"
                    className={
                      providerState.status === "ok"
                        ? "border-emerald-500/40 text-emerald-300"
                        : providerState.status === "error"
                          ? "border-red-500/40 text-red-300"
                          : "border-[var(--border)]"
                    }
                  >
                    {providerState.message}
                  </Badge>
                )}
                {webState && (
                  <Badge
                    variant="outline"
                    className={
                      webState.status === "ok"
                        ? "border-emerald-500/40 text-emerald-300"
                        : webState.status === "error"
                          ? "border-red-500/40 text-red-300"
                          : "border-[var(--border)]"
                    }
                  >
                    {webState.message}
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
                  title={
                    tauriRuntime
                      ? "Web source search is available in the hosted Friday workspace."
                      : webInspectionEnabled
                        ? "Search for candidate sources for this research topic."
                        : "Enable the Web connector before searching sources."
                  }
                  onClick={() => void findBriefSources(brief)}
                  disabled={!webInspectionEnabled || webState?.status === "running"}
                >
                  <Search size={13} />
                  {webState?.status === "running" ? "Searching" : "Find sources"}
                </Button>
                {briefUrl && (
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    title={
                      tauriRuntime
                        ? "Web source inspection is available in the hosted Friday workspace."
                        : webInspectionEnabled
                          ? "Fetch this URL as an approved research citation."
                          : "Enable the Web connector before inspecting URL sources."
                    }
                    onClick={() => void inspectBriefUrl(brief, briefUrl)}
                    disabled={!webInspectionEnabled || webState?.status === "running"}
                  >
                    <Globe2 size={13} />
                    {webState?.status === "running" ? "Inspecting" : "Inspect URL"}
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
                  onClick={() => queueBriefAgent(brief)}
                >
                  <Bot size={13} />
                  Queue agent
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  title={
                    tauriRuntime
                      ? "Provider synthesis is available in the hosted Friday workspace."
                      : providerSynthesisEnabled
                        ? "Use the configured provider to synthesize this local brief."
                        : "Enable Cloud AI in Connectors and configure provider env first."
                  }
                  onClick={() => void synthesizeBrief(brief)}
                  disabled={!providerSynthesisEnabled || providerState?.status === "running"}
                >
                  <Sparkles size={13} />
                  {providerState?.status === "running" ? "Synthesizing" : "Provider synthesize"}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => void exportBrief(brief)}
                  disabled={!brief.report && !brief.citations?.length}
                >
                  <FileDown size={13} />
                  Export
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
