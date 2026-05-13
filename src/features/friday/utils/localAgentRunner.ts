import type { AgentTask } from "../components/local-workspaces/types";

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

export function createLocalAgentRun(task: AgentTask) {
  const cleanTitle = formatTitle(task.title);
  const brief = task.brief?.trim();
  const plan = TARGET_PLANS[task.target];
  const log = [
    `Approved local ${task.target} task.`,
    `Prepared ${plan.length} guarded execution steps.`,
    "Desktop workspace inspection is available in the Tauri app; preview mode keeps this as a static runbook.",
  ];
  const result = [
    `Local runbook ready for: ${cleanTitle}`,
    brief ? `Task brief: ${brief}` : "",
    "",
    ...plan.map((step, index) => `${index + 1}. ${step}`),
    "",
    "No remote provider or destructive tool was used.",
  ].join("\n");

  return {
    status: "Completed" as const,
    plan,
    log,
    result,
  };
}
