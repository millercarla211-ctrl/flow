import type { AgentTask } from "../components/local-workspaces/types";

const TARGET_PLANS: Record<AgentTask["target"], string[]> = {
  browser: [
    "Open the target surface in a controlled browser session.",
    "Capture the current state, visible errors, console warnings, and network failures.",
    "Verify the core user path, then report the smallest safe fix.",
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
  const plan = TARGET_PLANS[task.target];
  const log = [
    `Approved local ${task.target} task.`,
    `Prepared ${plan.length} guarded execution steps.`,
    "Tool execution is waiting for an explicit workspace-backed runner.",
  ];
  const result = [
    `Local runbook ready for: ${cleanTitle}`,
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
