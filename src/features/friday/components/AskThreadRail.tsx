import { MessageSquare, Plus, Trash2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

import type { FridayAskThread } from "./local-workspaces/types";

type AskThreadRailProps = {
  activeThreadId: string;
  threads: FridayAskThread[];
  onCreateThread: () => void;
  onSelectThread: (thread: FridayAskThread) => void;
  onDeleteThread: (threadId: string) => void;
};

function formatThreadTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(date);
}

export function AskThreadRail({
  activeThreadId,
  threads,
  onCreateThread,
  onSelectThread,
  onDeleteThread,
}: AskThreadRailProps) {
  return (
    <div className="flex shrink-0 flex-col gap-2 rounded-lg border border-[var(--border)] bg-[var(--secondary)] p-3">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="ui-text-section-label ui-color-muted">Threads</div>
          <p className="mt-1 text-xs text-[var(--muted-foreground)]">
            Saved locally for this workspace.
          </p>
        </div>
        <Button type="button" size="sm" onClick={onCreateThread}>
          <Plus size={14} />
          New
        </Button>
      </div>

      <div className="flex gap-2 overflow-x-auto pb-1">
        {threads.length === 0 ? (
          <div className="flex min-h-12 min-w-64 items-center rounded-md border border-dashed border-[var(--border)] px-3 text-xs text-[var(--muted-foreground)]">
            Start a prompt to create the first Ask thread.
          </div>
        ) : (
          threads.map((thread) => {
            const selected = thread.id === activeThreadId;
            return (
              <div
                key={thread.id}
                className={`group flex min-w-56 max-w-64 items-center gap-3 rounded-md border px-3 py-2 text-left transition-colors ${
                  selected
                    ? "border-[var(--foreground)] bg-[var(--background)] text-[var(--foreground)]"
                    : "border-[var(--border)] bg-[var(--card)] text-[var(--muted-foreground)] hover:border-[var(--border-hover)] hover:text-[var(--foreground)]"
                }`}
              >
                <button
                  type="button"
                  onClick={() => onSelectThread(thread)}
                  className="flex min-w-0 flex-1 items-center gap-3 text-left"
                >
                  <MessageSquare className="shrink-0" size={15} />
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-xs font-semibold">{thread.title}</span>
                    <span className="mt-1 flex items-center gap-2 text-[10px] text-[var(--muted-foreground)]">
                      <span>{thread.messageCount} messages</span>
                      <span>{formatThreadTime(thread.updatedAt)}</span>
                    </span>
                  </span>
                </button>
                {thread.projectName && (
                  <Badge
                    variant="outline"
                    className="max-w-24 shrink-0 truncate border-[var(--border)] bg-[var(--secondary)] text-[10px]"
                  >
                    {thread.projectName}
                  </Badge>
                )}
                <button
                  type="button"
                  aria-label={`Delete ${thread.title}`}
                  onClick={() => onDeleteThread(thread.id)}
                  className="rounded p-1 text-[var(--muted-foreground)] opacity-0 transition-opacity hover:bg-[var(--secondary)] hover:text-[var(--foreground)] group-hover:opacity-100"
                >
                  <Trash2 size={13} />
                </button>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
