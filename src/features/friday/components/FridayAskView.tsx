import { useEffect, useMemo, useState } from "react";
import { useChat } from "@ai-sdk/react";
import { DefaultChatTransport } from "ai";
import {
  Archive,
  CalendarClock,
  FileText,
  Folder,
  type LucideIcon,
  Paperclip,
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

import { AiMarkdown } from "./AiMarkdown";
import { makeLocalRecord, useLocalList } from "../hooks/useLocalPersistence";
import {
  STORAGE_KEYS,
  type CanvasArtifact,
  type FridayAutomation,
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

function textFromMessage(message: { parts: Array<{ type: string; text?: string }> }) {
  return message.parts
    .filter((part) => part.type === "text" && typeof part.text === "string")
    .map((part) => part.text)
    .join("\n")
    .trim();
}

function titleFromText(text: string, fallback: string) {
  const firstLine = text
    .split("\n")
    .map((line) => line.replace(/^#+\s*/, "").trim())
    .find(Boolean);

  if (!firstLine) return fallback;
  return firstLine.length > 64 ? `${firstLine.slice(0, 61)}...` : firstLine;
}

export function FridayAskView() {
  const [input, setInput] = useState("");
  const [modelKey, setModelKey] = useState("qwen35-4b-revised-q4km");
  const [saveNotice, setSaveNotice] = useState<string | null>(null);
  const artifacts = useLocalList<CanvasArtifact>(STORAGE_KEYS.artifacts);
  const memories = useLocalList<FridayMemory>(STORAGE_KEYS.memory);
  const researchBriefs = useLocalList<ResearchBrief>(STORAGE_KEYS.research);
  const automations = useLocalList<FridayAutomation>(STORAGE_KEYS.automations);
  const projects = useLocalList<FridayProject>(STORAGE_KEYS.projects);
  const projectContextItems = useLocalList<ProjectContextItem>(STORAGE_KEYS.projectContext);
  const [activeProjectId, setActiveProjectId] = useState("none");
  const [contextDraft, setContextDraft] = useState("");
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
  const chatContext = useMemo(
    () =>
      selectedProject
        ? {
            projectName: selectedProject.name,
            projectInstructions: selectedProject.instructions,
            contextItems: activeContextItems.map((item) => ({
              label: item.label,
              kind: item.kind,
              content: item.content,
            })),
          }
        : undefined,
    [activeContextItems, selectedProject],
  );
  const transport = useMemo(() => {
    if (selectedModel.provider === "gateway" && isGatewayModelAvailable(selectedModel)) {
      return new DefaultChatTransport({
        api: "/api/friday/chat",
        body: { model: modelKey, context: chatContext },
      });
    }
    return new LocalFridayChatTransport(
      () => modelKey,
      () => chatContext,
    );
  }, [chatContext, modelKey, selectedModel]);
  const { messages, sendMessage, status, stop, setMessages, error } = useChat({ transport });
  const isBusy = status === "submitted" || status === "streaming";
  const latestUserText = useMemo(() => {
    const latestUserMessage = [...messages].reverse().find((message) => message.role === "user");
    return latestUserMessage ? textFromMessage(latestUserMessage) : "";
  }, [messages]);

  useEffect(() => {
    if (activeProjectId === "none") return;
    if (projects.items.some((project) => project.id === activeProjectId)) return;
    setActiveProjectId(projects.items[0]?.id ?? "none");
  }, [activeProjectId, projects.items]);

  const submitPrompt = () => {
    const text = input.trim();
    if (!text || isBusy) return;
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

  const addContextNote = () => {
    const cleanContext = contextDraft.trim();
    if (!cleanContext || !selectedProject) return;
    projectContextItems.addItem(
      makeLocalRecord("context", {
        projectId: selectedProject.id,
        projectName: selectedProject.name,
        label: titleFromText(cleanContext, "Project note"),
        kind: "note",
        content: cleanContext,
      }),
    );
    setContextDraft("");
    showSavedNotice("Project context added");
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
      </div>

      <div className="flex shrink-0 flex-wrap gap-2">
        {FRIDAY_MODEL_OPTIONS.map((model) => {
          const available = isGatewayModelAvailable(model);
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
              title={model.disabledReason}
            >
              <div className="text-xs font-semibold">{model.label}</div>
              <div className="mt-0.5 text-[10px] opacity-75">
                {model.speedLabel} / {model.privacyLabel}
              </div>
            </button>
          );
        })}
      </div>

      <Card className="shrink-0 py-0">
        <CardContent className="p-3">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex min-w-[220px] flex-1 items-center gap-2">
              <Folder size={15} className="text-[var(--muted-foreground)]" />
              <select
                className="h-8 min-w-0 flex-1 rounded-md border border-[var(--border)] bg-[var(--background)] px-2 text-xs text-[var(--foreground)] outline-none"
                value={activeProjectId}
                onChange={(event) => setActiveProjectId(event.target.value)}
              >
                <option value="none">No active project</option>
                {projects.items.map((project) => (
                  <option key={project.id} value={project.id}>
                    {project.name}
                  </option>
                ))}
              </select>
            </div>
            <Badge variant="outline" className="border-[var(--border)]">
              {selectedProject ? `${activeContextItems.length} context items` : "Local only"}
            </Badge>
          </div>
          {selectedProject ? (
            <div className="mt-3 grid gap-2 md:grid-cols-[1fr_auto]">
              <input
                className="h-8 rounded-md border border-[var(--border)] bg-[var(--background)] px-3 text-xs text-[var(--foreground)] outline-none placeholder:text-[var(--muted-foreground)]"
                value={contextDraft}
                onChange={(event) => setContextDraft(event.target.value)}
                placeholder="Add a project note, file summary, or instruction for this chat..."
              />
              <Button type="button" size="sm" variant="outline" onClick={addContextNote}>
                <Paperclip size={14} />
                Add context
              </Button>
              {activeContextItems.length > 0 && (
                <div className="md:col-span-2 flex flex-wrap gap-1.5">
                  {activeContextItems.slice(0, 5).map((item) => (
                    <span
                      key={item.id}
                      className="rounded-md border border-[var(--border)] bg-[var(--background)] px-2 py-1 text-[11px] text-[var(--muted-foreground)]"
                      title={item.content}
                    >
                      {item.label}
                    </span>
                  ))}
                </div>
              )}
            </div>
          ) : (
            <p className="mt-2 text-xs leading-5 text-[var(--muted-foreground)]">
              Select or create a Project to give Ask reusable instructions and context.
            </p>
          )}
        </CardContent>
      </Card>

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
                    onClick={() => setMessages([])}
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
