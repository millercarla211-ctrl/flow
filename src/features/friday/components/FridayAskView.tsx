import { useMemo, useState } from "react";
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

import { AiMarkdown } from "./AiMarkdown";
import { makeLocalRecord, useLocalList } from "../hooks/useLocalPersistence";
import {
  STORAGE_KEYS,
  type CanvasArtifact,
  type FridayAutomation,
  type FridayMemory,
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
  const selectedModel = useMemo(() => resolveFridayModel(modelKey), [modelKey]);
  const transport = useMemo(() => {
    if (selectedModel.provider === "gateway" && isGatewayModelAvailable(selectedModel)) {
      return new DefaultChatTransport({
        api: "/api/friday/chat",
        body: { model: modelKey },
      });
    }
    return new LocalFridayChatTransport(() => modelKey);
  }, [modelKey, selectedModel]);
  const { messages, sendMessage, status, stop, setMessages, error } = useChat({ transport });
  const isBusy = status === "submitted" || status === "streaming";
  const latestUserText = useMemo(() => {
    const latestUserMessage = [...messages].reverse().find((message) => message.role === "user");
    return latestUserMessage ? textFromMessage(latestUserMessage) : "";
  }, [messages]);

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
      artifacts.addItem(makeLocalRecord("artifact", { title, kind: "Markdown", content: text }));
      showSavedNotice("Saved to Artifacts");
      return;
    }

    if (target === "memory") {
      memories.addItem(
        makeLocalRecord("memory", {
          title,
          body: text,
          scope: "Global",
          pinned: true,
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
