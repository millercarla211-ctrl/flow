import { Archive, ListChecks, Sparkles, TextQuote } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { resolveFridayModel } from "@/features/ai";
import { tryRunTauriLocalChat } from "@/features/ai/tauri-local-chat";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { EmptyState, INPUT_CLASS, RecordShell, TEXTAREA_CLASS } from "./primitives";
import { STORAGE_KEYS, type CanvasArtifact } from "./types";

type CanvasTransform = "polish" | "summarize" | "actions";

const TRANSFORM_ACTIONS: Array<{
  key: CanvasTransform;
  label: string;
  icon: typeof Sparkles;
}> = [
  { key: "polish", label: "Polish", icon: Sparkles },
  { key: "summarize", label: "Summarize", icon: TextQuote },
  { key: "actions", label: "Actions", icon: ListChecks },
];

function transformPrompt(kind: CanvasTransform, text: string) {
  const instructions = {
    polish: "Rewrite this artifact so it is clearer and more professional. Keep the meaning.",
    summarize: "Summarize this artifact into a compact answer-first note.",
    actions: "Extract concrete next actions from this artifact as concise bullets.",
  } satisfies Record<CanvasTransform, string>;

  return `${instructions[kind]}\n\nArtifact:\n${text}`;
}

function fallbackTransform(kind: CanvasTransform, text: string) {
  const clean = text.trim().replace(/\s+/g, " ");
  if (kind === "summarize") {
    return clean.split(/(?<=[.!?])\s+/).slice(0, 3).join(" ");
  }
  if (kind === "actions") {
    return [
      "- Review the artifact for missing context.",
      "- Decide what should be saved, shared, or turned into a task.",
      "- Run the relevant lightweight verification before finalizing.",
    ].join("\n");
  }
  return clean;
}

function ArtifactPreview({ artifact }: { artifact: CanvasArtifact }) {
  const content = artifact.content.trim();
  if (!content) {
    return (
      <p className="text-xs leading-5 text-[var(--muted-foreground)]">
        No preview content yet.
      </p>
    );
  }

  if (artifact.kind === "Code" || artifact.kind === "UI") {
    return (
      <pre className="max-h-80 overflow-auto rounded-md border border-[var(--border)] bg-[var(--background)] p-3 whitespace-pre-wrap text-xs leading-5 text-[var(--muted-foreground)]">
        {content}
      </pre>
    );
  }

  return (
    <div className="max-h-80 space-y-2 overflow-auto rounded-md border border-[var(--border)] bg-[var(--background)] p-3 text-sm leading-6 text-[var(--foreground)]">
      {content.split(/\n+/).map((line, index) => {
        const clean = line.trim();
        if (!clean) return null;
        if (clean.startsWith("# ")) {
          return (
            <h3 key={`${clean}-${index}`} className="text-base font-semibold">
              {clean.slice(2)}
            </h3>
          );
        }
        if (clean.startsWith("## ")) {
          return (
            <h4 key={`${clean}-${index}`} className="text-sm font-semibold">
              {clean.slice(3)}
            </h4>
          );
        }
        if (clean.startsWith("- ")) {
          return (
            <p key={`${clean}-${index}`} className="pl-3 text-[var(--muted-foreground)]">
              {clean}
            </p>
          );
        }
        return <p key={`${clean}-${index}`}>{clean}</p>;
      })}
    </div>
  );
}

export function CanvasWorkspace() {
  const { items, addItem, updateItem, removeItem } = useLocalList<CanvasArtifact>(
    STORAGE_KEYS.artifacts,
  );
  const [title, setTitle] = useState("");
  const [kind, setKind] = useState<CanvasArtifact["kind"]>("Doc");
  const [content, setContent] = useState("");
  const [selectedArtifactId, setSelectedArtifactId] = useState("");
  const [editTitle, setEditTitle] = useState("");
  const [editKind, setEditKind] = useState<CanvasArtifact["kind"]>("Doc");
  const [editContent, setEditContent] = useState("");
  const [transforming, setTransforming] = useState<CanvasTransform | null>(null);
  const [lastTransform, setLastTransform] = useState<string | null>(null);
  const selectedArtifact = useMemo(
    () => items.find((artifact) => artifact.id === selectedArtifactId) ?? null,
    [items, selectedArtifactId],
  );

  useEffect(() => {
    if (selectedArtifactId && selectedArtifact) return;
    setSelectedArtifactId(items[0]?.id ?? "");
  }, [items, selectedArtifact, selectedArtifactId]);

  useEffect(() => {
    if (!selectedArtifact) {
      setEditTitle("");
      setEditKind("Doc");
      setEditContent("");
      return;
    }
    setEditTitle(selectedArtifact.title);
    setEditKind(selectedArtifact.kind);
    setEditContent(selectedArtifact.content);
  }, [selectedArtifact]);

  const createArtifact = () => {
    const cleanTitle = title.trim();
    if (!cleanTitle) return;
    const artifact = makeLocalRecord("artifact", {
      title: cleanTitle,
      kind,
      content: content.trim(),
    });
    addItem(artifact);
    setSelectedArtifactId(artifact.id);
    setTitle("");
    setContent("");
  };

  const updateSelectedArtifact = () => {
    if (!selectedArtifact) return;
    updateItem(selectedArtifact.id, {
      title: editTitle.trim() || "Untitled artifact",
      kind: editKind,
      content: editContent,
    });
  };

  const runTransform = async (transform: CanvasTransform) => {
    const source = editContent.trim();
    if (!source || transforming) return;
    setTransforming(transform);
    const localRun = await tryRunTauriLocalChat({
      prompt: transformPrompt(transform, source),
      model: resolveFridayModel("qwen3-0.6b"),
    });
    const nextContent = localRun?.text.trim() || fallbackTransform(transform, source);
    setEditContent(nextContent);
    setLastTransform(
      localRun
        ? `${localRun.model} / ${localRun.tokensPerSecond.toFixed(1)} tok/s`
        : "Preview fallback",
    );
    setTransforming(null);
  };

  return (
    <div className="grid gap-4 lg:grid-cols-[1fr_1fr]">
      <div className="space-y-3">
        <input
          className={INPUT_CLASS}
          value={title}
          onChange={(event) => setTitle(event.target.value)}
          placeholder="Artifact title"
        />
        <select
          className={INPUT_CLASS}
          value={kind}
          onChange={(event) => setKind(event.target.value as CanvasArtifact["kind"])}
        >
          <option>Doc</option>
          <option>Code</option>
          <option>Markdown</option>
          <option>UI</option>
        </select>
        <textarea
          className={TEXTAREA_CLASS}
          value={content}
          onChange={(event) => setContent(event.target.value)}
          placeholder="Draft content"
        />
        <Button type="button" onClick={createArtifact}>
          Save artifact
        </Button>
        {selectedArtifact && (
          <div className="rounded-lg border border-[var(--border)] bg-[var(--secondary)] p-3">
            <div className="mb-3 text-xs font-semibold uppercase tracking-[0.14em] text-[var(--muted-foreground)]">
              Edit selected artifact
            </div>
            <input
              className={INPUT_CLASS}
              value={editTitle}
              onChange={(event) => setEditTitle(event.target.value)}
              placeholder="Artifact title"
            />
            <select
              className={`${INPUT_CLASS} mt-3`}
              value={editKind}
              onChange={(event) => setEditKind(event.target.value as CanvasArtifact["kind"])}
            >
              <option>Doc</option>
              <option>Code</option>
              <option>Markdown</option>
              <option>UI</option>
            </select>
            <textarea
              className={`${TEXTAREA_CLASS} mt-3 min-h-56`}
              value={editContent}
              onChange={(event) => setEditContent(event.target.value)}
              placeholder="Artifact content"
            />
            <div className="mt-3 flex flex-wrap items-center gap-2">
              <Button type="button" onClick={updateSelectedArtifact}>
                Save edits
              </Button>
              {TRANSFORM_ACTIONS.map((action) => {
                const Icon = action.icon;
                return (
                  <Button
                    key={action.key}
                    type="button"
                    variant="outline"
                    onClick={() => void runTransform(action.key)}
                    disabled={!editContent.trim() || Boolean(transforming)}
                  >
                    <Icon size={13} />
                    {transforming === action.key ? "Working" : action.label}
                  </Button>
                );
              })}
              {lastTransform && (
                <Badge variant="outline" className="border-[var(--border)]">
                  {lastTransform}
                </Badge>
              )}
            </div>
          </div>
        )}
        {selectedArtifact && (
          <div className="rounded-lg border border-[var(--border)] bg-[var(--secondary)] p-3">
            <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
              <div className="text-xs font-semibold uppercase tracking-[0.14em] text-[var(--muted-foreground)]">
                Preview selected artifact
              </div>
              <Badge variant="outline" className="border-[var(--border)]">
                {selectedArtifact.kind}
              </Badge>
            </div>
            <ArtifactPreview
              artifact={{
                ...selectedArtifact,
                title: editTitle || selectedArtifact.title,
                kind: editKind,
                content: editContent,
              }}
            />
          </div>
        )}
      </div>
      <div className="space-y-2">
        {items.length === 0 ? (
          <EmptyState
            title="No artifacts saved"
            body="Create docs, code drafts, markdown, or UI notes locally."
          />
        ) : (
          items.map((artifact) => (
            <div key={artifact.id} className="block w-full text-left">
              <RecordShell
                icon={<Archive size={15} />}
                title={artifact.title}
                subtitle={artifact.kind}
              >
                <p className="mt-2 line-clamp-3 text-xs leading-5 text-[var(--muted-foreground)]">
                  {artifact.content || "No content yet"}
                </p>
                <div className="mt-3 flex flex-wrap gap-2">
                  <Button
                    type="button"
                    size="sm"
                    variant={selectedArtifactId === artifact.id ? "default" : "outline"}
                    onClick={(event) => {
                      event.stopPropagation();
                      setSelectedArtifactId(artifact.id);
                    }}
                  >
                    Edit
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={(event) => {
                      event.stopPropagation();
                      removeItem(artifact.id);
                    }}
                  >
                    Remove
                  </Button>
                </div>
              </RecordShell>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
