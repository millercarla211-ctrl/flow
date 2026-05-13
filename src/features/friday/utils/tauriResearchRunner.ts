import { invoke } from "@tauri-apps/api/core";

import { isTauriRuntime } from "@/platform/tauriRuntime";

import type {
  FridayMemory,
  FridayProject,
  ProjectContextItem,
  ResearchCitation,
} from "../components/local-workspaces/types";

type FridayResearchContext = {
  projectName?: string;
  projectInstructions?: string;
  contextItems: Array<{
    label: string;
    kind: string;
    content: string;
  }>;
};

export type FridayResearchDraftResult = {
  plan: string[];
  report: string;
  model: string;
  generatedTokens: number;
  totalTimeMs: number;
  tokensPerSecond: number;
};

export function createResearchContext({
  project,
  contextItems,
  memories,
}: {
  project: FridayProject | null;
  contextItems: ProjectContextItem[];
  memories: FridayMemory[];
}): FridayResearchContext | undefined {
  const context = [
    ...contextItems.slice(0, 4).map((item) => ({
      label: item.label,
      kind: item.kind,
      content: item.content,
    })),
    ...memories.slice(0, 2).map((memory) => ({
      label: memory.title,
      kind: "memory",
      content: memory.body,
    })),
  ];

  if (!project && context.length === 0) return undefined;

  return {
    projectName: project?.name,
    projectInstructions: project?.instructions,
    contextItems: context,
  };
}

export async function tryDraftTauriResearch({
  topic,
  citations,
  context,
}: {
  topic: string;
  citations: ResearchCitation[];
  context?: FridayResearchContext;
}): Promise<FridayResearchDraftResult | null> {
  if (!isTauriRuntime() || !topic.trim()) return null;

  try {
    return await invoke<FridayResearchDraftResult>("friday_local_research", {
      topic,
      citations,
      context,
    });
  } catch (error) {
    console.warn("Friday local research fell back to deterministic local draft:", error);
    return null;
  }
}
