import { Pin } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { EmptyState, INPUT_CLASS, RecordShell, TEXTAREA_CLASS } from "./primitives";
import { STORAGE_KEYS, type FridayMemory, type FridayProject } from "./types";

export function MemoryWorkspace() {
  const { items, addItem, updateItem, removeItem } = useLocalList<FridayMemory>(
    STORAGE_KEYS.memory,
  );
  const projects = useLocalList<FridayProject>(STORAGE_KEYS.projects);
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [scope, setScope] = useState<FridayMemory["scope"]>("Global");
  const [projectId, setProjectId] = useState("none");
  const [filter, setFilter] = useState<FridayMemory["scope"] | "All">("All");
  const [selectedMemoryId, setSelectedMemoryId] = useState("");
  const [editTitle, setEditTitle] = useState("");
  const [editBody, setEditBody] = useState("");
  const [editScope, setEditScope] = useState<FridayMemory["scope"]>("Global");
  const [editProjectId, setEditProjectId] = useState("none");

  const selectedProject = projects.items.find((project) => project.id === projectId) ?? null;
  const visibleItems = useMemo(
    () => items.filter((memory) => filter === "All" || memory.scope === filter),
    [filter, items],
  );
  const selectedMemory = useMemo(
    () => items.find((memory) => memory.id === selectedMemoryId) ?? null,
    [items, selectedMemoryId],
  );
  const editProject = projects.items.find((project) => project.id === editProjectId) ?? null;
  const canAddMemory =
    Boolean(title.trim()) &&
    Boolean(body.trim()) &&
    (scope !== "Project" || Boolean(selectedProject));
  const canSaveMemory =
    Boolean(selectedMemory) &&
    Boolean(editTitle.trim()) &&
    Boolean(editBody.trim()) &&
    (editScope !== "Project" || Boolean(editProject));

  useEffect(() => {
    if (selectedMemoryId && selectedMemory) return;
    setSelectedMemoryId(visibleItems[0]?.id ?? "");
  }, [selectedMemory, selectedMemoryId, visibleItems]);

  useEffect(() => {
    if (projectId === "none") return;
    if (projects.items.some((project) => project.id === projectId)) return;
    setProjectId("none");
  }, [projectId, projects.items]);

  useEffect(() => {
    if (editProjectId === "none") return;
    if (projects.items.some((project) => project.id === editProjectId)) return;
    setEditProjectId("none");
  }, [editProjectId, projects.items]);

  useEffect(() => {
    if (!selectedMemory) {
      setEditTitle("");
      setEditBody("");
      setEditScope("Global");
      setEditProjectId("none");
      return;
    }
    setEditTitle(selectedMemory.title);
    setEditBody(selectedMemory.body);
    setEditScope(selectedMemory.scope);
    setEditProjectId(selectedMemory.projectId ?? "none");
  }, [selectedMemory]);

  const addMemory = () => {
    if (!canAddMemory) return;
    const cleanTitle = title.trim();
    const cleanBody = body.trim();
    addItem(
      makeLocalRecord("memory", {
        title: cleanTitle,
        body: cleanBody,
        scope,
        pinned: true,
        projectId: scope === "Project" ? selectedProject?.id : undefined,
        projectName: scope === "Project" ? selectedProject?.name : undefined,
      }),
    );
    setTitle("");
    setBody("");
  };

  const saveSelectedMemory = () => {
    if (!selectedMemory || !canSaveMemory) return;
    const cleanTitle = editTitle.trim();
    const cleanBody = editBody.trim();
    updateItem(selectedMemory.id, {
      title: cleanTitle,
      body: cleanBody,
      scope: editScope,
      projectId: editScope === "Project" ? editProject?.id : undefined,
      projectName: editScope === "Project" ? editProject?.name : undefined,
    });
  };

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-[1fr_160px_180px_auto]">
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
        <select
          className={INPUT_CLASS}
          value={projectId}
          onChange={(event) => setProjectId(event.target.value)}
          disabled={scope !== "Project"}
        >
          <option value="none">No project</option>
          {projects.items.map((project) => (
            <option key={project.id} value={project.id}>
              {project.name}
            </option>
          ))}
        </select>
        <Button type="button" onClick={addMemory} disabled={!canAddMemory}>
          Add memory
        </Button>
      </div>
      {scope === "Project" && !selectedProject && (
        <p className="text-xs text-[var(--muted-foreground)]">
          Choose a project before saving a project memory.
        </p>
      )}
      <textarea
        className={TEXTAREA_CLASS}
        value={body}
        onChange={(event) => setBody(event.target.value)}
        placeholder="What should Friday remember?"
      />
      {selectedMemory && (
        <div className="rounded-lg border border-[var(--border)] bg-[var(--secondary)] p-3">
          <div className="mb-3 text-xs font-semibold uppercase tracking-[0.14em] text-[var(--muted-foreground)]">
            Edit selected memory
          </div>
          <div className="grid gap-3 md:grid-cols-[1fr_160px_180px_auto]">
            <input
              className={INPUT_CLASS}
              value={editTitle}
              onChange={(event) => setEditTitle(event.target.value)}
              placeholder="Memory title"
            />
            <select
              className={INPUT_CLASS}
              value={editScope}
              onChange={(event) => setEditScope(event.target.value as FridayMemory["scope"])}
            >
              <option>Global</option>
              <option>Project</option>
              <option>Voice</option>
            </select>
            <select
              className={INPUT_CLASS}
              value={editProjectId}
              onChange={(event) => setEditProjectId(event.target.value)}
              disabled={editScope !== "Project"}
            >
              <option value="none">No project</option>
              {projects.items.map((project) => (
                <option key={project.id} value={project.id}>
                  {project.name}
                </option>
              ))}
            </select>
            <Button type="button" onClick={saveSelectedMemory} disabled={!canSaveMemory}>
              Save memory
            </Button>
          </div>
          {editScope === "Project" && !editProject && (
            <p className="mt-2 text-xs text-[var(--muted-foreground)]">
              Choose a project before saving this as a project memory.
            </p>
          )}
          <textarea
            className={`${TEXTAREA_CLASS} mt-3`}
            value={editBody}
            onChange={(event) => setEditBody(event.target.value)}
            placeholder="Memory body"
          />
        </div>
      )}
      <div className="flex flex-wrap gap-2">
        {(["All", "Global", "Project", "Voice"] as const).map((item) => (
          <button
            key={item}
            type="button"
            onClick={() => setFilter(item)}
            className={`rounded-md border px-3 py-1.5 text-xs transition-colors ${
              filter === item
                ? "border-[var(--border-hover)] bg-[var(--secondary)] text-[var(--foreground)]"
                : "border-[var(--border)] text-[var(--muted-foreground)]"
            }`}
          >
            {item}
          </button>
        ))}
      </div>
      {visibleItems.length === 0 ? (
        <EmptyState
          title="No editable memories yet"
          body="Add explicit memories here. Friday will treat these as user-controlled local facts."
        />
      ) : (
        <div className="space-y-2">
          {visibleItems.map((memory) => (
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
                {memory.projectName && (
                  <Badge variant="outline" className="border-[var(--border)]">
                    {memory.projectName}
                  </Badge>
                )}
                <Button
                  type="button"
                  size="sm"
                  variant={selectedMemoryId === memory.id ? "default" : "outline"}
                  onClick={() => setSelectedMemoryId(memory.id)}
                >
                  Edit
                </Button>
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
