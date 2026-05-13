import { Folder } from "lucide-react";
import { useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { DEFAULT_FRIDAY_MODEL_KEY, FRIDAY_LOCAL_MODELS } from "@/features/ai";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { EmptyState, INPUT_CLASS, ModelSelect, RecordShell, TEXTAREA_CLASS } from "./primitives";
import { STORAGE_KEYS, type FridayProject } from "./types";

export function ProjectsWorkspace() {
  const { items, addItem, removeItem } = useLocalList<FridayProject>(STORAGE_KEYS.projects);
  const [name, setName] = useState("");
  const [instructions, setInstructions] = useState("");
  const [modelKey, setModelKey] = useState(DEFAULT_FRIDAY_MODEL_KEY);

  const selectedModel = useMemo(
    () => FRIDAY_LOCAL_MODELS.find((model) => model.key === modelKey),
    [modelKey],
  );

  const createProject = () => {
    const cleanName = name.trim();
    if (!cleanName) return;
    addItem(
      makeLocalRecord("project", { name: cleanName, instructions: instructions.trim(), modelKey }),
    );
    setName("");
    setInstructions("");
  };

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-[1fr_220px_auto]">
        <input
          className={INPUT_CLASS}
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="Project name"
        />
        <ModelSelect value={modelKey} onChange={setModelKey} />
        <Button type="button" onClick={createProject}>
          New project
        </Button>
      </div>
      <textarea
        className={TEXTAREA_CLASS}
        value={instructions}
        onChange={(event) => setInstructions(event.target.value)}
        placeholder={`Default instructions for ${selectedModel?.label ?? "this model"}`}
      />
      {items.length === 0 ? (
        <EmptyState
          title="No projects yet"
          body="Create a project to keep files, instructions, memories, and model choices together."
        />
      ) : (
        <div className="space-y-2">
          {items.map((project) => (
            <RecordShell
              key={project.id}
              icon={<Folder size={15} />}
              title={project.name}
              subtitle={
                FRIDAY_LOCAL_MODELS.find((model) => model.key === project.modelKey)?.label ??
                project.modelKey
              }
            >
              <p className="mt-2 text-xs leading-5 text-[var(--muted-foreground)]">
                {project.instructions || "No custom instructions yet"}
              </p>
              <Button
                className="mt-3"
                type="button"
                size="sm"
                variant="outline"
                onClick={() => removeItem(project.id)}
              >
                Remove
              </Button>
            </RecordShell>
          ))}
        </div>
      )}
    </div>
  );
}
