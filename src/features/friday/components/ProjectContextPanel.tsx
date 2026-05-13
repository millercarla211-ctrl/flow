import { FileText, Folder, Paperclip, X } from "lucide-react";
import { useRef, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { makeLocalRecord } from "../hooks/useLocalPersistence";
import type { FridayProject, ProjectContextItem } from "./local-workspaces/types";
import {
  CONTEXT_FILE_ACCEPT,
  isSupportedContextFile,
  readContextFile,
} from "../utils/contextFiles";
import { titleFromText } from "../utils/text";

export function ProjectContextPanel({
  projects,
  activeProjectId,
  selectedProject,
  activeContextItems,
  onActiveProjectChange,
  onAddContextItem,
  onRemoveContextItem,
  onNotice,
}: {
  projects: FridayProject[];
  activeProjectId: string;
  selectedProject: FridayProject | null;
  activeContextItems: ProjectContextItem[];
  onActiveProjectChange: (projectId: string) => void;
  onAddContextItem: (item: ProjectContextItem) => void;
  onRemoveContextItem: (itemId: string) => void;
  onNotice: (label: string) => void;
}) {
  const [contextDraft, setContextDraft] = useState("");
  const [fileError, setFileError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const addContextNote = () => {
    const cleanContext = contextDraft.trim();
    if (!cleanContext || !selectedProject) return;
    onAddContextItem(
      makeLocalRecord("context", {
        projectId: selectedProject.id,
        projectName: selectedProject.name,
        label: titleFromText(cleanContext, "Project note"),
        kind: "note",
        content: cleanContext,
      }),
    );
    setContextDraft("");
    setFileError(null);
    onNotice("Project context added");
  };

  const importFiles = async (files: FileList | null) => {
    if (!files?.length || !selectedProject) return;

    let importedCount = 0;
    const rejected: string[] = [];

    for (const file of Array.from(files).slice(0, 6)) {
      if (!isSupportedContextFile(file)) {
        rejected.push(file.name);
        continue;
      }

      const content = await readContextFile(file);
      onAddContextItem(
        makeLocalRecord("context", {
          projectId: selectedProject.id,
          projectName: selectedProject.name,
          label: file.name,
          kind: "file",
          content,
        }),
      );
      importedCount += 1;
    }

    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }

    setFileError(rejected.length > 0 ? `Skipped unsupported files: ${rejected.join(", ")}` : null);
    if (importedCount > 0) {
      onNotice(
        `Imported ${importedCount} file${importedCount === 1 ? "" : "s"} into project context`,
      );
    }
  };

  return (
    <Card className="shrink-0 py-0">
      <CardContent className="p-3">
        <div className="flex flex-wrap items-center gap-3">
          <div className="flex min-w-[220px] flex-1 items-center gap-2">
            <Folder size={15} className="text-[var(--muted-foreground)]" />
            <select
              className="h-8 min-w-0 flex-1 rounded-md border border-[var(--border)] bg-[var(--background)] px-2 text-xs text-[var(--foreground)] outline-none"
              value={activeProjectId}
              onChange={(event) => onActiveProjectChange(event.target.value)}
            >
              <option value="none">No active project</option>
              {projects.map((project) => (
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
          <div className="mt-3 grid gap-2 md:grid-cols-[1fr_auto_auto]">
            <input
              className="h-8 rounded-md border border-[var(--border)] bg-[var(--background)] px-3 text-xs text-[var(--foreground)] outline-none placeholder:text-[var(--muted-foreground)]"
              value={contextDraft}
              onChange={(event) => setContextDraft(event.target.value)}
              placeholder="Add a project note, file summary, or instruction for this chat..."
            />
            <Button type="button" size="sm" variant="outline" onClick={addContextNote}>
              <Paperclip size={14} />
              Add note
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              onClick={() => fileInputRef.current?.click()}
            >
              <FileText size={14} />
              Import file
            </Button>
            <input
              ref={fileInputRef}
              className="hidden"
              type="file"
              multiple
              accept={CONTEXT_FILE_ACCEPT}
              onChange={(event) => void importFiles(event.target.files)}
            />
            {fileError && (
              <p className="md:col-span-3 text-xs text-[var(--destructive)]">{fileError}</p>
            )}
            {activeContextItems.length > 0 && (
              <div className="md:col-span-3 flex flex-wrap gap-1.5">
                {activeContextItems.slice(0, 8).map((item) => (
                  <span
                    key={item.id}
                    className="inline-flex items-center gap-1 rounded-md border border-[var(--border)] bg-[var(--background)] px-2 py-1 text-[11px] text-[var(--muted-foreground)]"
                    title={item.content}
                  >
                    {item.kind === "file" && <FileText size={11} />}
                    {item.label}
                    <button
                      type="button"
                      aria-label={`Remove ${item.label}`}
                      className="rounded-sm p-0.5 text-[var(--muted-foreground)] hover:bg-[var(--secondary)] hover:text-[var(--foreground)]"
                      onClick={() => onRemoveContextItem(item.id)}
                    >
                      <X size={10} />
                    </button>
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
  );
}
