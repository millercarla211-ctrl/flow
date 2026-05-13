import { useMemo, useState } from "react";
import { useChat } from "@ai-sdk/react";
import { DefaultChatTransport } from "ai";
import { Send, Square, Trash2 } from "lucide-react";

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

export function FridayAskView() {
  const [input, setInput] = useState("");
  const [modelKey, setModelKey] = useState("qwen35-4b-revised-q4km");
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

  const submitPrompt = () => {
    const text = input.trim();
    if (!text || isBusy) return;
    sendMessage({ text });
    setInput("");
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
            Local-first streaming chat with model routing. Remote gateway models are visible but disabled
            until cloud mode is explicitly enabled.
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
                {messages.map((message) => (
                  <div
                    key={message.id}
                    className={`flex ${message.role === "user" ? "justify-end" : "justify-start"}`}
                  >
                    <div
                      className={`max-w-[78%] rounded-lg border px-4 py-3 ${
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
                  </div>
                ))}
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
