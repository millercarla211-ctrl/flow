import type { AgentTask } from "../components/local-workspaces/types";
import type { FridayAgentContext } from "./tauriAgentRunner";

const TARGET_PLANS: Record<AgentTask["target"], string[]> = {
  browser: [
    "Inspect the explicit URL when desktop read-only fetching is available.",
    "Summarize status, title, content excerpt, and obvious follow-up checks.",
    "Prepare the smallest safe browser verification plan for manual or approved execution.",
  ],
  code: [
    "Locate the smallest owning module for the requested behavior.",
    "Read the current implementation and nearby tests before editing.",
    "Patch the narrow code path, then run the lightest relevant verification.",
  ],
  files: [
    "Identify the exact files or folders in scope.",
    "Inspect metadata and content before making any filesystem change.",
    "Prepare a reversible update plan and preserve unrelated user work.",
  ],
};

function formatTitle(title: string) {
  return title.trim().replace(/\s+/g, " ");
}

function extractPathHints(...values: Array<string | undefined>) {
  const text = values.filter(Boolean).join(" ");
  const matches =
    text.match(
      /(?:[A-Za-z]:\\[^\s"'<>]+|(?:\.{1,2}\/)?(?:src|app|api|components|lib|scripts|public|assets|docs|tests|src-tauri)\/[^\s"'<>]+|[\w.-]+\.(?:ts|tsx|js|mjs|json|md|rs|css|scss|html|yml|yaml))/g,
    ) ?? [];

  return Array.from(new Set(matches)).slice(0, 8);
}

function extractAcceptanceHints(brief?: string) {
  if (!brief) return [];

  return brief
    .split(/\r?\n|[.;]/)
    .map((line) => line.trim())
    .filter((line) => /must|should|verify|accept|done|pass|fix|keep|preserve/i.test(line))
    .slice(0, 6);
}

function contextPlan(context?: FridayAgentContext) {
  if (!context) return [];

  const plan: string[] = [];
  if (context.projectName) {
    plan.push(`Apply project instructions for ${context.projectName}.`);
  }
  if (context.contextItems.length > 0) {
    plan.push(`Use ${context.contextItems.length} approved project context item${context.contextItems.length === 1 ? "" : "s"} before touching the task.`);
  }
  return plan;
}

function targetPlan(task: AgentTask, pathHints: string[], acceptanceHints: string[]) {
  if (task.target === "code") {
    return [
      pathHints.length > 0
        ? `Inspect the referenced code path${pathHints.length === 1 ? "" : "s"}: ${pathHints.join(", ")}.`
        : "Find the smallest owning code module before editing.",
      "Read nearby types, state hooks, and tests before changing behavior.",
      acceptanceHints.length > 0
        ? `Verify the stated acceptance criteria: ${acceptanceHints.join(" | ")}.`
        : "Run the lightest relevant typecheck or smoke test after the patch.",
    ];
  }

  if (task.target === "files") {
    return [
      pathHints.length > 0
        ? `Inspect referenced file path${pathHints.length === 1 ? "" : "s"} read-only first: ${pathHints.join(", ")}.`
        : "Identify the exact files or folders in scope before changing anything.",
      "Separate user-owned content from generated or rebuildable artifacts.",
      acceptanceHints.length > 0
        ? `Preserve these constraints: ${acceptanceHints.join(" | ")}.`
        : "Prepare a reversible update plan before writing or deleting files.",
    ];
  }

  return TARGET_PLANS.browser;
}

function contextSummary(context?: FridayAgentContext) {
  if (!context) return [];

  return [
    context.projectName ? `Project: ${context.projectName}` : "",
    context.projectInstructions ? `Instructions: ${context.projectInstructions}` : "",
    ...context.contextItems.slice(0, 5).map((item) => {
      const content = item.content.length > 180 ? `${item.content.slice(0, 177)}...` : item.content;
      return `${item.kind} ${item.label}: ${content}`;
    }),
  ].filter(Boolean);
}

export function createLocalAgentRun(task: AgentTask, context?: FridayAgentContext) {
  const cleanTitle = formatTitle(task.title);
  const brief = task.brief?.trim();
  const pathHints = extractPathHints(task.title, brief);
  const acceptanceHints = extractAcceptanceHints(brief);
  const plan = [...contextPlan(context), ...targetPlan(task, pathHints, acceptanceHints)];
  const contextLines = contextSummary(context);
  const log = [
    `Approved local ${task.target} task.`,
    pathHints.length > 0
      ? `Detected ${pathHints.length} scoped path hint${pathHints.length === 1 ? "" : "s"}.`
      : "No explicit path hint was detected.",
    contextLines.length > 0
      ? `Loaded ${contextLines.length} local context line${contextLines.length === 1 ? "" : "s"}.`
      : "No project context was attached.",
    `Prepared ${plan.length} guarded execution step${plan.length === 1 ? "" : "s"}.`,
    "Desktop workspace inspection is available in the Tauri app; preview mode keeps this as a static runbook.",
  ];
  const result = [
    `Local runbook ready for: ${cleanTitle}`,
    brief ? `Task brief: ${brief}` : "",
    pathHints.length > 0 ? `Detected scope: ${pathHints.join(", ")}` : "",
    contextLines.length > 0 ? `\nContext used:\n${contextLines.map((line) => `- ${line}`).join("\n")}` : "",
    "",
    "Execution plan:",
    ...plan.map((step, index) => `${index + 1}. ${step}`),
    acceptanceHints.length > 0
      ? `\nAcceptance checklist:\n${acceptanceHints.map((hint) => `- ${hint}`).join("\n")}`
      : "",
    "",
    "No remote provider or destructive tool was used.",
  ].join("\n");

  return {
    inspectedWorkspace: contextLines.length > 0 || pathHints.length > 0,
    status: "Completed" as const,
    plan,
    log,
    result,
  };
}
