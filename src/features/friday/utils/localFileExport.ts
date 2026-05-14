import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

import type {
  AgentTask,
  CanvasArtifact,
  FridayAskThread,
  FridayAutomation,
  FridayMemory,
  FridayProject,
  ProjectContextItem,
  ResearchBrief,
} from "../components/local-workspaces/types";

type FridayProjectExportBundle = {
  agents: AgentTask[];
  artifacts: CanvasArtifact[];
  automations: FridayAutomation[];
  contextItems: ProjectContextItem[];
  memories: FridayMemory[];
  project: FridayProject;
  research: ResearchBrief[];
  threads: FridayAskThread[];
};

type FridayExportResult =
  | { status: "saved" }
  | { status: "cancelled" }
  | { status: "error"; message: string };

type FridayExportRequest = {
  content: string;
  defaultPath: string;
  filters: Array<{ name: string; extensions: string[] }>;
  title: string;
};

function safeLocalFilename(title: string, extension: string) {
  const clean = title
    .trim()
    .replace(/[<>:"/\\|?*\x00-\x1F]/g, "-")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "")
    .toLowerCase();
  return `${clean || "friday-export"}.${extension}`;
}

export function formatFridayLocalFileExportError(error: unknown) {
  if (error instanceof Error) {
    return error.message.trim() || "Friday file export failed.";
  }

  const message = String(error).trim();
  return message || "Friday file export failed.";
}

function artifactExtension(kind: CanvasArtifact["kind"]) {
  if (kind === "Code") return "txt";
  if (kind === "UI") return "tsx";
  return "md";
}

function artifactExportContent(artifact: Pick<CanvasArtifact, "title" | "kind" | "content">) {
  if (artifact.kind === "Code" || artifact.kind === "UI") {
    return artifact.content;
  }

  const title = artifact.title.trim();
  const content = artifact.content.trim();
  return [title ? `# ${title}` : "", content].filter(Boolean).join("\n\n");
}

function researchBriefExportContent(brief: ResearchBrief) {
  const plan = brief.plan.map((step, index) => `${index + 1}. ${step}`).join("\n");
  const citations =
    brief.citations
      ?.map((citation, index) => {
        return `- [${index + 1}] ${citation.label} (${citation.kind}): ${citation.excerpt}`;
      })
      .join("\n") ?? "";

  return [
    `# Research: ${brief.topic}`,
    "",
    brief.projectName ? `Project: ${brief.projectName}` : "",
    `Sources: ${brief.sources.join(", ") || "Local only"}`,
    `Status: ${brief.status ?? "Planned"}`,
    "",
    "## Plan",
    plan,
    "",
    "## Report",
    brief.report?.trim() || "No report drafted yet.",
    "",
    citations ? "## Citations" : "",
    citations,
  ]
    .filter(Boolean)
    .join("\n");
}

function agentTaskExportContent(task: AgentTask) {
  const plan = task.plan?.map((step, index) => `${index + 1}. ${step}`).join("\n") ?? "";
  const log = task.log?.map((line) => `- ${line}`).join("\n") ?? "";

  return [
    `# Agent run: ${task.title}`,
    "",
    task.projectName ? `Project: ${task.projectName}` : "",
    `Target: ${task.target}`,
    `Status: ${task.status}`,
    task.lastModel ? `Model: ${task.lastModel}` : "",
    task.lastTokensPerSecond ? `Speed: ${task.lastTokensPerSecond.toFixed(1)} tok/s` : "",
    "",
    task.brief ? "## Brief" : "",
    task.brief,
    "",
    plan ? "## Plan" : "",
    plan,
    "",
    log ? "## Run Log" : "",
    log,
    "",
    "## Result",
    task.result?.trim() || "No result captured yet.",
  ]
    .filter(Boolean)
    .join("\n");
}

function messageText(message: FridayAskThread["messages"][number]) {
  return message.parts
    .map((part) => (part.type === "text" ? part.text : ""))
    .join(" ")
    .replace(/\s+/g, " ")
    .trim();
}

function markdownSection(title: string, lines: string[]) {
  const cleanLines = lines.filter((line) => line.trim().length > 0);
  if (cleanLines.length === 0) return "";
  return [`## ${title}`, "", ...cleanLines].join("\n");
}

function projectBundleExportContent(bundle: FridayProjectExportBundle) {
  const { project } = bundle;
  const contextLines = bundle.contextItems.map((item, index) =>
    [`### ${index + 1}. ${item.label}`, `Kind: ${item.kind}`, "", item.content].join("\n"),
  );
  const memoryLines = bundle.memories.map((memory, index) =>
    [
      `### ${index + 1}. ${memory.title}`,
      `Scope: ${memory.scope}${memory.pinned ? " / pinned" : ""}`,
      "",
      memory.body,
    ].join("\n"),
  );
  const threadLines = bundle.threads.map((thread, index) => {
    const messages = thread.messages
      .map((message) => {
        const text = messageText(message);
        return text ? `- ${message.role}: ${text}` : "";
      })
      .filter(Boolean)
      .join("\n");
    return [
      `### ${index + 1}. ${thread.title}`,
      `Model: ${thread.modelKey}`,
      `Messages: ${thread.messageCount}`,
      messages,
    ]
      .filter(Boolean)
      .join("\n");
  });
  const artifactLines = bundle.artifacts.map((artifact, index) =>
    [`### ${index + 1}. ${artifact.title}`, `Kind: ${artifact.kind}`, "", artifact.content].join(
      "\n",
    ),
  );
  const researchLines = bundle.research.map((brief, index) =>
    [
      `### ${index + 1}. ${brief.topic}`,
      `Status: ${brief.status ?? "Planned"}`,
      `Sources: ${brief.sources.join(", ") || "Local only"}`,
      "",
      brief.report || "No report drafted yet.",
    ].join("\n"),
  );
  const agentLines = bundle.agents.map((task, index) =>
    [
      `### ${index + 1}. ${task.title}`,
      `Target: ${task.target}`,
      `Status: ${task.status}`,
      task.result ?? task.brief ?? "",
    ]
      .filter(Boolean)
      .join("\n"),
  );
  const automationLines = bundle.automations.map((automation, index) =>
    [
      `### ${index + 1}. ${automation.title}`,
      `Cadence: ${automation.cadence}`,
      `Enabled: ${automation.enabled ? "yes" : "no"}`,
      automation.instruction ?? "",
      automation.lastResult ? `Last result:\n${automation.lastResult}` : "",
    ]
      .filter(Boolean)
      .join("\n"),
  );

  return [
    `# Friday Project: ${project.name}`,
    "",
    `Model: ${project.modelKey}`,
    `Updated: ${project.updatedAt}`,
    "",
    "## Instructions",
    "",
    project.instructions || "No custom instructions.",
    "",
    markdownSection("Context", contextLines),
    markdownSection("Memories", memoryLines),
    markdownSection("Ask Threads", threadLines),
    markdownSection("Artifacts", artifactLines),
    markdownSection("Research", researchLines),
    markdownSection("Agent Runs", agentLines),
    markdownSection("Automations", automationLines),
  ]
    .filter(Boolean)
    .join("\n\n");
}

async function exportFridayFile(request: FridayExportRequest): Promise<FridayExportResult> {
  const outputPath = await save({
    title: request.title,
    defaultPath: request.defaultPath,
    filters: request.filters,
  });
  if (!outputPath) return { status: "cancelled" };

  try {
    await invoke("export_friday_artifact_to_path", {
      content: request.content,
      outputPath,
    });
    return { status: "saved" };
  } catch (error) {
    return {
      status: "error",
      message: formatFridayLocalFileExportError(error),
    };
  }
}

export function exportFridayArtifact(
  artifact: Pick<CanvasArtifact, "title" | "kind" | "content">,
) {
  const extension = artifactExtension(artifact.kind);
  return exportFridayFile({
    title: "Export Friday artifact",
    defaultPath: safeLocalFilename(artifact.title || "friday-artifact", extension),
    content: artifactExportContent(artifact),
    filters: [
      { name: `${artifact.kind} artifact`, extensions: [extension] },
      { name: "Text", extensions: ["txt", "md"] },
    ],
  });
}

export function exportFridayResearchBrief(brief: ResearchBrief) {
  return exportFridayFile({
    title: "Export Friday research brief",
    defaultPath: safeLocalFilename(`research-${brief.topic}`, "md"),
    content: researchBriefExportContent(brief),
    filters: [
      { name: "Markdown", extensions: ["md"] },
      { name: "Text", extensions: ["txt"] },
    ],
  });
}

export function exportFridayAgentTask(task: AgentTask) {
  return exportFridayFile({
    title: "Export Friday agent run",
    defaultPath: safeLocalFilename(`agent-${task.title}`, "md"),
    content: agentTaskExportContent(task),
    filters: [
      { name: "Markdown", extensions: ["md"] },
      { name: "Text", extensions: ["txt"] },
    ],
  });
}

export function exportFridayProjectBundle(bundle: FridayProjectExportBundle) {
  return exportFridayFile({
    title: "Export Friday project",
    defaultPath: safeLocalFilename(`project-${bundle.project.name}`, "md"),
    content: projectBundleExportContent(bundle),
    filters: [
      { name: "Markdown", extensions: ["md"] },
      { name: "Text", extensions: ["txt"] },
    ],
  });
}
