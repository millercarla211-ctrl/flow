import { FileText, Plus } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { makeLocalRecord, useLocalList } from "../../hooks/useLocalPersistence";
import { EmptyState, INPUT_CLASS, RecordShell } from "./primitives";
import { STORAGE_KEYS, type ResearchBrief } from "./types";

const RESEARCH_SOURCES = ["Local files", "Web", "Academic", "Premium data"];

export function ResearchWorkspace() {
  const { items, addItem, removeItem } = useLocalList<ResearchBrief>(STORAGE_KEYS.research);
  const [topic, setTopic] = useState("");
  const [sources, setSources] = useState(["Local files", "Web"]);

  const createBrief = () => {
    const cleanTopic = topic.trim();
    if (!cleanTopic) return;
    addItem(
      makeLocalRecord("research", {
        topic: cleanTopic,
        sources,
        plan: [
          `Define the decision or answer needed for ${cleanTopic}.`,
          "Collect source notes inside the selected scopes.",
          "Draft a cited answer with assumptions and open questions.",
        ],
      }),
    );
    setTopic("");
  };

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-[1fr_auto]">
        <input
          className={INPUT_CLASS}
          value={topic}
          onChange={(event) => setTopic(event.target.value)}
          placeholder="Research topic or question"
        />
        <Button type="button" onClick={createBrief}>
          <Plus size={16} />
          Create plan
        </Button>
      </div>
      <div className="flex flex-wrap gap-2">
        {RESEARCH_SOURCES.map((source) => {
          const enabled = sources.includes(source);
          return (
            <button
              key={source}
              type="button"
              onClick={() =>
                setSources((current) =>
                  enabled ? current.filter((item) => item !== source) : [...current, source],
                )
              }
              className={`rounded-md border px-3 py-1.5 text-xs transition-colors ${
                enabled
                  ? "border-[var(--border-hover)] bg-[var(--secondary)] text-[var(--foreground)]"
                  : "border-[var(--border)] text-[var(--muted-foreground)]"
              }`}
            >
              {source}
            </button>
          );
        })}
      </div>
      {items.length === 0 ? (
        <EmptyState
          title="No research briefs yet"
          body="Create a topic and Friday will keep the plan locally until source connectors are enabled."
        />
      ) : (
        <div className="space-y-2">
          {items.map((brief) => (
            <RecordShell
              key={brief.id}
              icon={<FileText size={15} />}
              title={brief.topic}
              subtitle={`Sources: ${brief.sources.join(", ") || "Local only"}`}
            >
              <ol className="mt-3 space-y-1 text-xs leading-5 text-[var(--muted-foreground)]">
                {brief.plan.map((step) => (
                  <li key={step}>{step}</li>
                ))}
              </ol>
              <Button
                className="mt-3"
                type="button"
                size="sm"
                variant="outline"
                onClick={() => removeItem(brief.id)}
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
