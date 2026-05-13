import { useEffect, useMemo, useRef, useState } from "react";
import { useChat } from "@ai-sdk/react";
import { DefaultChatTransport } from "ai";
import {
  Archive,
  CalendarClock,
  FileText,
  type LucideIcon,
  Pin,
  Send,
  Square,
  Trash2,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  LocalFridayChatTransport,
  FRIDAY_MODEL_OPTIONS,
  isGatewayModelAvailable,
  resolveFridayModel,
} from "@/features/ai";
import { isTauriRuntime } from "@/platform/tauriRuntime";

import { AiMarkdown } from "./AiMarkdown";
import { AskThreadRail } from "./AskThreadRail";
import { ProjectContextPanel } from "./ProjectContextPanel";
import { useSettings } from "../../settings/queries";
import { makeLocalRecord, useLocalList, useLocalSettings } from "../hooks/useLocalPersistence";
import { rankAskContext } from "../utils/localRetrieval";
import { textFromMessage, titleFromText } from "../utils/text";
import {
  STORAGE_KEYS,
  type CanvasArtifact,
  type ConnectorSettings,
  type FridayAutomation,
  type FridayAskThread,
  type FridayMemory,
  type FridayProject,
  type ProjectContextItem,
  type ResearchBrief,
} from "./local-workspaces/types";

type SaveTarget = "artifact" | "memory" | "research" | "automation";

const MESSAGE_SAVE_ACTIONS: Array<{
  target: SaveTarget;
  label: string;
  icon: LucideIcon;
}> = [
  { target: "artifact", label: "Artifact", icon: Archive },
  { target: "memory", label: "Memory", icon: Pin },
  { target: "research", label: "Research", icon: FileText },
  { target: "automation", label: "Follow-up", icon: CalendarClock },
];

export function FridayAskView() {
  const [input, setInput] = useState("");
  const [modelKey, setModelKey] = useState("qwen35-4b-revised-q4km");
  const [saveNotice, setSaveNotice] = useState<string | null>(null);
  const [tauriRuntime, setTauriRuntime] = useState(false);
  const artifacts = useLocalList<CanvasArtifact>(STORAGE_KEYS.artifacts);
  const memories = useLocalList<FridayMemory>(STORAGE_KEYS.memory);
  const researchBriefs = useLocalList<ResearchBrief>(STORAGE_KEYS.research);
  const automations = useLocalList<FridayAutomation>(STORAGE_KEYS.automations);
  const askThreads = useLocalList<FridayAskThread>(STORAGE_KEYS.askThreads);
  const projects = useLocalList<FridayProject>(STORAGE_KEYS.projects);
  const projectContextItems = useLocalList<ProjectContextItem>(STORAGE_KEYS.projectContext);
  const connectors = useLocalSettings<ConnectorSettings>(STORAGE_KEYS.connectors, {
    localFiles: true,
    webSearch: false,
    aiGateway: false,
    mcpConnectors: false,
  });
  const [activeProjectId, setActiveProjectId] = useState("none");
  const [activeThreadId, setActiveThreadId] = useState("new");
  const lastSavedThreadSignature = useRef("");
  const selectedModel = useMemo(() => resolveFridayModel(modelKey), [modelKey]);
  const selectedProject = useMemo(
    () => projects.items.find((project) => project.id === activeProjectId) ?? null,
    [activeProjectId, projects.items],
  );
  const activeContextItems = useMemo(
    () =>
      selectedProject
        ? projectContextItems.items.filter((item) => item.projectId === selectedProject.id)
        : [],
    [projectContextItems.items, selectedProject],
  );
  const retrievedAskContext = useMemo(
    () =>
      rankAskContext({
        query: input,
        contextItems: activeContextItems,
        memories: memories.items,
        projectId: selectedProject?.id,
      }),
    [activeContextItems, input, memories.items, selectedProject?.id],
  );
  const chatContext = useMemo(
    () =>
      selectedProject || retrievedAskContext.length > 0
        ? {
            projectName: selectedProject?.name,
            projectInstructions: selectedProject?.instructions,
            contextItems: retrievedAskContext.map((item) => ({
              label: item.label,
              kind: item.kind,
              content: item.content,
            })),
          }
        : undefined,
    [retrievedAskContext, selectedProject],
  );
  const transport = useMemo(() => {
    if (
      selectedModel.provider === "gateway" &&
      connectors.settings.aiGateway &&
      isGatewayModelAvailable(selectedModel)
    ) {
      return new DefaultChatTransport({
        api: "/api/friday/chat",
        body: { model: modelKey, context: chatContext, allowCloud: true },
      });
    }
    return new LocalFridayChatTransport(
      () => modelKey,
      () => chatContext,
    );
  }, [chatContext, connectors.settings.aiGateway, modelKey, selectedModel]);
  const { messages, sendMessage, status, stop, setMessages, error } = useChat({ transport });
  const isBusy = status === "submitted" || status === "streaming";
  const visibleThreads = useMemo(
    () =>
      selectedProject
        ? askThreads.items.filter((thread) => thread.projectId === selectedProject.id)
        : askThreads.items,
    [askThreads.items, selectedProject],
  );
  const latestUserText = useMemo(() => {
    const latestUserMessage = [...messages].reverse().find((message) => message.role === "user");
    return latestUserMessage ? textFromMessage(latestUserMessage) : "";
  }, [messages]);
  const settingsQuery = useSettings(undefined, tauriRuntime);
  const localExecutionLabel = useMemo(() => {
    if (!tauriRuntime) return "Preview fallback";
    const settings = settingsQuery.data;
    if (!settings) return "Checking desktop LLM";
    if (!settings.llm_enabled || settings.llm_provider === "none") return "Desktop LLM off";
    if (settings.llm_provider === "local") {
      return settings.llm_model ? `Desktop local: ${settings.llm_model}` : "Desktop local ready";
    }
    return `Desktop provider: ${settings.llm_provider}`;
  }, [settingsQuery.data, tauriRuntime]);

  useEffect(() => {
    setTauriRuntime(isTauriRuntime());
  }, []);

  useEffect(() => {
    if (activeProjectId === "none") return;
    if (projects.items.some((project) => project.id === activeProjectId)) return;
    setActiveProjectId(projects.items[0]?.id ?? "none");
  }, [activeProjectId, projects.items]);

  useEffect(() => {
    if (activeThreadId === "new") return;
    if (askThreads.items.some((thread) => thread.id === activeThreadId)) return;
    setActiveThreadId("new");
    setMessages([]);
  }, [activeThreadId, askThreads.items, setMessages]);

  useEffect(() => {
    if (activeThreadId === "new" || messages.length === 0 || isBusy) return;

    const thread = askThreads.items.find((item) => item.id === activeThreadId);
    if (!thread) return;

    const firstUserText = textFromMessage(
      messages.find((message) => message.role === "user") ?? { parts: [] },
    );
    const nextTitle =
      thread.messageCount === 0 || thread.title === "New chat"
        ? titleFromText(firstUserText, "New chat")
        : thread.title;
    const signature = JSON.stringify({
      id: activeThreadId,
      messages,
      modelKey,
      projectId: selectedProject?.id,
      title: nextTitle,
    });
    const storedSignature = JSON.stringify({
      id: thread.id,
      messages: thread.messages,
      modelKey: thread.modelKey,
      projectId: thread.projectId,
      title: thread.title,
    });

    if (signature === lastSavedThreadSignature.current || signature === storedSignature) {
      lastSavedThreadSignature.current = signature;
      return;
    }

    askThreads.updateItem(activeThreadId, {
      title: nextTitle,
      modelKey,
      messageCount: messages.length,
      messages,
      projectId: selectedProject?.id,
      projectName: selectedProject?.name,
    });
    lastSavedThreadSignature.current = signature;
  }, [
    activeThreadId,
    askThreads,
    isBusy,
    messages,
    modelKey,
    selectedProject?.id,
    selectedProject?.name,
  ]);

  const createThread = () => {
    const thread = makeLocalRecord("thread", {
      title: "New chat",
      modelKey,
      messageCount: 0,
      messages: [],
      projectId: selectedProject?.id,
      projectName: selectedProject?.name,
    });
    askThreads.addItem(thread);
    setActiveThreadId(thread.id);
    setMessages([]);
    showSavedNotice("New Ask thread ready");
  };

  const openThread = (thread: FridayAskThread) => {
    setActiveThreadId(thread.id);
    setModelKey(thread.modelKey);
    setActiveProjectId(thread.projectId ?? "none");
    setMessages(thread.messages);
  };

  const deleteThread = (threadId: string) => {
    askThreads.removeItem(threadId);
    if (activeThreadId === threadId) {
      setActiveThreadId("new");
      setMessages([]);
    }
    showSavedNotice("Thread deleted");
  };

  const ensureActiveThread = (firstPrompt: string) => {
    if (
      activeThreadId !== "new" &&
      askThreads.items.some((thread) => thread.id === activeThreadId)
    ) {
      return;
    }

    const thread = makeLocalRecord("thread", {
      title: titleFromText(firstPrompt, "New chat"),
      modelKey,
      messageCount: 0,
      messages: [],
      projectId: selectedProject?.id,
      projectName: selectedProject?.name,
    });
    askThreads.addItem(thread);
    setActiveThreadId(thread.id);
  };

  const submitPrompt = () => {
    const text = input.trim();
    if (!text || isBusy) return;
    ensureActiveThread(text);
    sendMessage({ text });
    setInput("");
  };

  const showSavedNotice = (label: string) => {
    setSaveNotice(label);
    window.setTimeout(() => setSaveNotice(null), 1800);
  };

  const saveAssistantMessage = (target: SaveTarget, text: string) => {
    if (!text) return;
    const title = titleFromText(text, "Friday response");

    if (target === "artifact") {
      artifacts.addItem(
        makeLocalRecord("artifact", {
          title,
          kind: "Markdown",
          content: text,
          projectId: selectedProject?.id,
          projectName: selectedProject?.name,
        }),
      );
      showSavedNotice("Saved to Artifacts");
      return;
    }

    if (target === "memory") {
      memories.addItem(
        makeLocalRecord("memory", {
          title,
          body: text,
          scope: selectedProject ? "Project" : "Global",
          pinned: true,
          projectId: selectedProject?.id,
          projectName: selectedProject?.name,
        }),
      );
      showSavedNotice("Pinned to Memory");
      return;
    }

    if (target === "research") {
      researchBriefs.addItem(
        makeLocalRecord("research", {
          topic: latestUserText || title,
          sources: ["Ask Friday"],
          projectId: selectedProject?.id,
          projectName: selectedProject?.name,
          plan: [
            "Review the saved assistant answer.",
            "Add source notes or local files.",
            "Convert the answer into a cited report.",
          ],
        }),
      );
      showSavedNotice("Saved as Research brief");
      return;
    }

    automations.addItem(
      makeLocalRecord("automation", {
        title: `Follow up: ${title}`,
        cadence: "Manual",
        enabled: true,
        projectId: selectedProject?.id,
        projectName: selectedProject?.name,
      }),
    );
    showSavedNotice("Saved as Automation");
  };

  return (
    <div className="flex h-full min-h-0 flex-col gap-4">
      <div className="flex shrink-0 flex-wrap items-center justify-between gap-3">
        <div>
          <div className="ui-text-section-label ui-color-muted">Assistant</div>
          <h1 className="mt-2 text-2xl font-semibold tracking-tight text-[var(--foreground)]">
            Ask Friday
          </h1>
          <p className="mt-2 max-w-2xl text-sm leading-6 text-[var(--muted-foreground)]">
            Local-first streaming chat with model routing. Remote gateway models are visible but
            disabled until cloud mode is explicitly enabled.
          </p>
        </div>
        <Badge
          variant="outline"
          className="border-[var(--border)] bg-[var(--secondary)] text-[var(--foreground)]"
        >
          Local mode
        </Badge>
        <Badge
          variant="outline"
          className="border-[var(--border)] bg-[var(--secondary)] text-[var(--foreground)]"
        >
          {localExecutionLabel}
        </Badge>
      </div>

      <div className="flex shrink-0 flex-wrap gap-2">
        {FRIDAY_MODEL_OPTIONS.map((model) => {
          const available =
            model.provider === "local" ||
            (connectors.settings.aiGateway && isGatewayModelAvailable(model));
          const selected = model.key === modelKey;
          return (
            <button
              key={model.key}
              type="button"
              disabled={!available}
              onClick={() => setModelKey(model.key)}
              className={`rounded-md border px-3 py-2 text-left transition-colors ${
                selected
                  ? "border-[var(--foreground)] bg-[var(--foreground)] text-[var(--background)]"
                  : "border-[var(--border)] bg-[var(--card)] text-[var(--muted-foreground)] hover:border-[var(--border-hover)] hover:text-[var(--foreground)]"
              } disabled:cursor-not-allowed disabled:opacity-45`}
              title={
                model.provider === "gateway" && !connectors.settings.aiGateway
                  ? "Enable AI Gateway in Connectors before selecting cloud models."
                  : model.disabledReason
              }
            >
              <div className="text-xs font-semibold">{model.label}</div>
              <div className="mt-0.5 text-[10px] opacity-75">
                {model.speedLabel} / {model.privacyLabel}
              </div>
            </button>
          );
        })}
      </div>

      <AskThreadRail
        activeThreadId={activeThreadId}
        threads={visibleThreads}
        onCreateThread={createThread}
        onSelectThread={openThread}
        onDeleteThread={deleteThread}
      />

      <ProjectContextPanel
        projects={projects.items}
        activeProjectId={activeProjectId}
        selectedProject={selectedProject}
        activeContextItems={activeContextItems}
        onActiveProjectChange={setActiveProjectId}
        onAddContextItem={projectContextItems.addItem}
        onRemoveContextItem={projectContextItems.removeItem}
        onNotice={showSavedNotice}
      />

      <div className="flex shrink-0 flex-wrap items-center gap-2 rounded-lg border border-[var(--border)] bg-[var(--secondary)] px-3 py-2">
        <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--muted-foreground)]">
          Local context
        </span>
        {retrievedAskContext.length === 0 ? (
          <span className="text-xs text-[var(--muted-foreground)]">
            Save memories or attach project notes to make Ask more aware.
          </span>
        ) : (
          retrievedAskContext.slice(0, 4).map((item) => (
            <Badge
              key={`${item.source}-${item.id}`}
              variant="outline"
              className="border-[var(--border)] bg-[var(--background)] text-[var(--foreground)]"
              title={item.content}
            >
              {item.kind}: {item.label}
            </Badge>
          ))
        )}
      </div>

      <Card className="min-h-0 flex-1 overflow-hidden py-0">
        <CardContent className="flex h-full min-h-0 flex-col p-0">
          {saveNotice && (
            <div className="border-b border-[var(--border)] bg-[var(--secondary)] px-4 py-2 text-xs font-medium text-[var(--foreground)]">
              {saveNotice}
            </div>
          )}
          <div className="min-h-0 flex-1 overflow-y-auto px-4 py-4">
            {messages.length === 0 ? (
              <div className="flex h-full min-h-[280px] items-center justify-center text-center">
                <div className="max-w-md">
                  <div className="text-sm font-semibold text-[var(--foreground)]">
                    Start with a real task
                  </div>
                  <p className="mt-2 text-sm leading-6 text-[var(--muted-foreground)]">
                    Ask for a UI change, a research plan, a code review, or a rewrite. The first
                    shell streams through Friday's local preview transport.
                  </p>
                </div>
              </div>
            ) : (
              <div className="space-y-4">
                {messages.map((message) => {
                  const messageText = textFromMessage(message);
                  const isAssistant = message.role === "assistant";
                  return (
                    <div
                      key={message.id}
                      className={`flex ${message.role === "user" ? "justify-end" : "justify-start"}`}
                    >
                      <div className="max-w-[78%]">
                        <div
                          className={`rounded-lg border px-4 py-3 ${
                            message.role === "user"
                              ? "border-[var(--foreground)] bg-[var(--foreground)] text-[var(--background)]"
                              : "border-[var(--border)] bg-[var(--secondary)] text-[var(--foreground)]"
                          }`}
                        >
                          {message.parts.map((part, index) =>
                            part.type === "text" ? (
                              message.role === "user" ? (
                                <p key={index} className="text-sm leading-6">
                                  {part.text}
                                </p>
                              ) : (
                                <AiMarkdown key={index}>{part.text}</AiMarkdown>
                              )
                            ) : null,
                          )}
                        </div>
                        {isAssistant && messageText && (
                          <div className="mt-2 flex flex-wrap gap-1.5">
                            {MESSAGE_SAVE_ACTIONS.map(({ target, label, icon: Icon }) => (
                              <button
                                key={target}
                                type="button"
                                onClick={() => saveAssistantMessage(target, messageText)}
                                className="inline-flex h-7 items-center gap-1.5 rounded-md border border-[var(--border)] bg-[var(--background)] px-2 text-[11px] font-medium text-[var(--muted-foreground)] transition-colors hover:bg-[var(--secondary)] hover:text-[var(--foreground)]"
                              >
                                <Icon size={12} />
                                {label}
                              </button>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>

          {error && (
            <div className="border-t border-[var(--border)] px-4 py-2 text-xs text-[var(--destructive)]">
              Friday could not complete that response. Try again with the local model.
            </div>
          )}

          <div className="border-t border-[var(--border)] p-3">
            <div className="flex items-end gap-2 rounded-lg border border-[var(--border)] bg-[var(--background)] p-2">
              <textarea
                value={input}
                onChange={(event) => setInput(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" && !event.shiftKey) {
                    event.preventDefault();
                    submitPrompt();
                  }
                }}
                className="min-h-16 flex-1 resize-none bg-transparent px-2 py-2 text-sm text-[var(--foreground)] outline-none placeholder:text-[var(--muted-foreground)]"
                placeholder="Ask Friday to research, write, code, or plan..."
                disabled={isBusy}
              />
              <div className="flex gap-1">
                {messages.length > 0 && (
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    onClick={() => {
                      if (activeThreadId !== "new") {
                        askThreads.updateItem(activeThreadId, {
                          title: "New chat",
                          messageCount: 0,
                          messages: [],
                        });
                      }
                      setMessages([]);
                    }}
                    aria-label="Clear chat"
                  >
                    <Trash2 size={16} />
                  </Button>
                )}
                {isBusy ? (
                  <Button type="button" size="icon" onClick={stop} aria-label="Stop response">
                    <Square size={16} />
                  </Button>
                ) : (
                  <Button
                    type="button"
                    size="icon"
                    onClick={submitPrompt}
                    disabled={!input.trim()}
                    aria-label="Send message"
                  >
                    <Send size={16} />
                  </Button>
                )}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
