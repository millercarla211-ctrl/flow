import { Archive } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { EmptyState, INPUT_CLASS, RecordShell, TEXTAREA_CLASS } from "./primitives";
import { STORAGE_KEYS, type CanvasArtifact } from "./types";

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
            <Button className="mt-3" type="button" onClick={updateSelectedArtifact}>
              Save edits
            </Button>
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
