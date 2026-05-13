import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

import type { AgentTask, CanvasArtifact, ResearchBrief } from "../components/local-workspaces/types";

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
      message: error instanceof Error ? error.message : String(error),
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
