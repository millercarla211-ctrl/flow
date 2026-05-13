import { Pin } from "lucide-react";
import { useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { EmptyState, INPUT_CLASS, RecordShell, TEXTAREA_CLASS } from "./primitives";
import { STORAGE_KEYS, type FridayMemory } from "./types";

export function MemoryWorkspace() {
  const { items, addItem, updateItem, removeItem } = useLocalList<FridayMemory>(
    STORAGE_KEYS.memory,
  );
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [scope, setScope] = useState<FridayMemory["scope"]>("Global");

  const addMemory = () => {
    const cleanTitle = title.trim();
    const cleanBody = body.trim();
    if (!cleanTitle || !cleanBody) return;
    addItem(makeLocalRecord("memory", { title: cleanTitle, body: cleanBody, scope, pinned: true }));
    setTitle("");
    setBody("");
  };

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-[1fr_160px_auto]">
        <input
          className={INPUT_CLASS}
          value={title}
          onChange={(event) => setTitle(event.target.value)}
          placeholder="Memory title"
        />
        <select
          className={INPUT_CLASS}
          value={scope}
          onChange={(event) => setScope(event.target.value as FridayMemory["scope"])}
        >
          <option>Global</option>
          <option>Project</option>
          <option>Voice</option>
        </select>
        <Button type="button" onClick={addMemory}>
          Add memory
        </Button>
      </div>
      <textarea
        className={TEXTAREA_CLASS}
        value={body}
        onChange={(event) => setBody(event.target.value)}
        placeholder="What should Friday remember?"
      />
      {items.length === 0 ? (
        <EmptyState
          title="No editable memories yet"
          body="Add explicit memories here. Friday will treat these as user-controlled local facts."
        />
      ) : (
        <div className="space-y-2">
          {items.map((memory) => (
            <RecordShell
              key={memory.id}
              icon={<Pin size={15} />}
              title={memory.title}
              subtitle={memory.scope}
            >
              <p className="mt-2 text-xs leading-5 text-[var(--muted-foreground)]">{memory.body}</p>
              <div className="mt-3 flex flex-wrap gap-2">
                <Badge variant="outline" className="border-[var(--border)]">
                  {memory.pinned ? "Pinned" : "Unpinned"}
                </Badge>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => updateItem(memory.id, { pinned: !memory.pinned })}
                >
                  Toggle pin
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => removeItem(memory.id)}
                >
                  Remove
                </Button>
              </div>
            </RecordShell>
          ))}
        </div>
      )}
    </div>
  );
}
