import type { FridayMemory, ProjectContextItem } from "../components/local-workspaces/types";

export type AskRetrievedContext = {
  id: string;
  label: string;
  kind: "note" | "file" | "instruction" | "memory";
  content: string;
  score: number;
  source: "project" | "memory";
  projectName?: string;
};

type RankedAskCandidate = AskRetrievedContext & {
  updatedAt: string;
};

type RankAskContextInput = {
  query: string;
  contextItems: ProjectContextItem[];
  memories: FridayMemory[];
  projectId?: string;
  limit?: number;
};

const STOP_WORDS = new Set([
  "about",
  "after",
  "again",
  "also",
  "and",
  "are",
  "can",
  "for",
  "from",
  "how",
  "into",
  "make",
  "our",
  "please",
  "that",
  "the",
  "this",
  "use",
  "what",
  "when",
  "with",
  "you",
]);

function tokenize(value: string): string[] {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, " ")
    .split(/\s+/)
    .map((token) => token.trim())
    .filter((token) => token.length > 2 && !STOP_WORDS.has(token));
}

function scoreCandidate(queryTokens: string[], label: string, content: string) {
  if (queryTokens.length === 0) return 0;

  const haystack = `${label} ${content}`.toLowerCase();
  const tokenSet = new Set(tokenize(haystack));
  let score = 0;

  for (const token of queryTokens) {
    if (tokenSet.has(token)) score += 3;
    if (haystack.includes(token)) score += 1;
  }

  return score;
}

function memoryMatchesProject(memory: FridayMemory, projectId?: string) {
  if (memory.scope === "Global") return true;
  if (!projectId) return !memory.projectId;
  return !memory.projectId || memory.projectId === projectId;
}

function toAskContext(candidate: RankedAskCandidate): AskRetrievedContext {
  return {
    id: candidate.id,
    label: candidate.label,
    kind: candidate.kind,
    content: candidate.content,
    score: candidate.score,
    source: candidate.source,
    projectName: candidate.projectName,
  };
}

function fallbackRank(candidates: RankedAskCandidate[], limit: number): AskRetrievedContext[] {
  return candidates
    .sort((a, b) => {
      if (a.source !== b.source) return a.source === "memory" ? -1 : 1;
      return b.updatedAt.localeCompare(a.updatedAt);
    })
    .slice(0, limit)
    .map(toAskContext);
}

export function rankAskContext({
  query,
  contextItems,
  memories,
  projectId,
  limit = 6,
}: RankAskContextInput): AskRetrievedContext[] {
  const queryTokens = tokenize(query);
  const projectCandidates: RankedAskCandidate[] = contextItems.map((item) => ({
    id: item.id,
    label: item.label,
    kind: item.kind,
    content: item.content,
    score: 0,
    source: "project",
    projectName: item.projectName,
    updatedAt: item.updatedAt,
  }));
  const memoryCandidates: RankedAskCandidate[] = memories
    .filter((memory) => memory.pinned && memoryMatchesProject(memory, projectId))
    .map((memory) => ({
      id: memory.id,
      label: memory.title,
      kind: "memory",
      content: memory.body,
      score: 0,
      source: "memory",
      projectName: memory.projectName,
      updatedAt: memory.updatedAt,
    }));
  const candidates = [...memoryCandidates, ...projectCandidates];

  if (candidates.length === 0) return [];

  const ranked = candidates
    .map((candidate) => ({
      ...candidate,
      score: scoreCandidate(queryTokens, candidate.label, candidate.content),
    }))
    .filter((candidate) => candidate.score > 0)
    .sort((a, b) => b.score - a.score || b.updatedAt.localeCompare(a.updatedAt))
    .slice(0, limit)
    .map(toAskContext);

  return ranked.length > 0 ? ranked : fallbackRank(candidates, limit);
}
