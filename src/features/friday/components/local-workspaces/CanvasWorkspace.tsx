import { Archive } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { EmptyState, INPUT_CLASS, RecordShell, TEXTAREA_CLASS } from "./primitives";
import { STORAGE_KEYS, type CanvasArtifact } from "./types";

export function CanvasWorkspace() {
  const { items, addItem, removeItem } = useLocalList<CanvasArtifact>(STORAGE_KEYS.artifacts);
  const [title, setTitle] = useState("");
  const [kind, setKind] = useState<CanvasArtifact["kind"]>("Doc");
  const [content, setContent] = useState("");

  const createArtifact = () => {
    const cleanTitle = title.trim();
    if (!cleanTitle) return;
    addItem(makeLocalRecord("artifact", { title: cleanTitle, kind, content: content.trim() }));
    setTitle("");
    setContent("");
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
      </div>
      <div className="space-y-2">
        {items.length === 0 ? (
          <EmptyState
            title="No artifacts saved"
            body="Create docs, code drafts, markdown, or UI notes locally."
          />
        ) : (
          items.map((artifact) => (
            <RecordShell
              key={artifact.id}
              icon={<Archive size={15} />}
              title={artifact.title}
              subtitle={artifact.kind}
            >
              <p className="mt-2 line-clamp-3 text-xs leading-5 text-[var(--muted-foreground)]">
                {artifact.content || "No content yet"}
              </p>
              <Button
                className="mt-3"
                type="button"
                size="sm"
                variant="outline"
                onClick={() => removeItem(artifact.id)}
              >
                Remove
              </Button>
            </RecordShell>
          ))
        )}
      </div>
    </div>
  );
}
