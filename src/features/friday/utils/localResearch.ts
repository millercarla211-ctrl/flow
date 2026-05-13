import type {
  FridayMemory,
  FridayProject,
  ProjectContextItem,
  ResearchCitation,
} from "../components/local-workspaces/types";

type ResearchSource = {
  id: string;
  label: string;
  kind: ResearchCitation["kind"];
  content: string;
};

function tokenize(text: string) {
  return new Set(
    text
      .toLowerCase()
      .split(/[^a-z0-9_]+/)
      .filter((token) => token.length > 2),
  );
}

function excerptFor(content: string, topic: string) {
  const sentences = content
    .replace(/\s+/g, " ")
    .split(/(?<=[.!?])\s+/)
    .filter(Boolean);
  const topicTokens = tokenize(topic);
  const bestSentence =
    sentences
      .map((sentence) => ({
        sentence,
        score: [...tokenize(sentence)].filter((token) => topicTokens.has(token)).length,
      }))
      .sort((a, b) => b.score - a.score)[0]?.sentence ?? content;

  return bestSentence.length > 260 ? `${bestSentence.slice(0, 257)}...` : bestSentence;
}

function scoreSource(source: ResearchSource, topic: string) {
  const topicTokens = tokenize(topic);
  if (topicTokens.size === 0) return 0;

  const sourceText = `${source.label} ${source.content}`;
  const sourceTokens = tokenize(sourceText);
  let score = 0;

  for (const token of topicTokens) {
    if (sourceTokens.has(token)) score += 1;
  }

  if (source.kind === "file") score += 0.2;
  if (source.kind === "memory") score += 0.3;
  return score;
}

export function createLocalResearchDraft({
  topic,
  project,
  contextItems,
  memories,
}: {
  topic: string;
  project: FridayProject | null;
  contextItems: ProjectContextItem[];
  memories: FridayMemory[];
}): { citations: ResearchCitation[]; report: string; plan: string[] } {
  const sources: ResearchSource[] = [
    ...contextItems.map((item) => ({
      id: item.id,
      label: item.label,
      kind: item.kind,
      content: item.content,
    })),
    ...memories.map((memory) => ({
      id: memory.id,
      label: memory.title,
      kind: "memory" as const,
      content: memory.body,
    })),
  ];

  const ranked = sources
    .map((source) => ({ source, score: scoreSource(source, topic) }))
    .filter((entry) => entry.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, 5);

  const fallback =
    ranked.length === 0 ? sources.slice(0, 3).map((source) => ({ source, score: 0 })) : ranked;

  const citations = fallback.map(({ source }, index) => ({
    id: source.id,
    label: source.label,
    kind: source.kind,
    excerpt: excerptFor(source.content, topic) || `Local ${source.kind} source ${index + 1}.`,
  }));

  const projectLine = project ? ` inside ${project.name}` : "";
  const sourceLine =
    citations.length > 0
      ? citations.map((citation, index) => `[${index + 1}] ${citation.label}`).join(", ")
      : "No local sources matched yet";

  const report = [
    `## Local Research Draft: ${topic}`,
    "",
    `Friday prepared this local-first brief${projectLine}. Remote search and premium connectors were not used.`,
    "",
    "### Working Answer",
    citations.length > 0
      ? `The strongest local evidence currently comes from ${sourceLine}. Use this as a first pass, then add web or academic sources only if you enable those connectors.`
      : "No local project sources matched this topic yet. Add files, notes, or memories to improve the brief.",
    "",
    "### Source Notes",
    ...citations.map((citation, index) => `- [${index + 1}] ${citation.excerpt}`),
  ].join("\n");

  const plan = [
    `Clarify the decision needed for ${topic}.`,
    citations.length > 0
      ? `Review ${citations.length} local citation${citations.length === 1 ? "" : "s"}.`
      : "Add local files, notes, or memories that mention this topic.",
    "Turn the local draft into a final cited report.",
  ];

  return { citations, report, plan };
}
