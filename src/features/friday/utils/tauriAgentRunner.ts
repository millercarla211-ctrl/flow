import { invoke } from "@tauri-apps/api/core";

import { isTauriRuntime } from "@/platform/tauriRuntime";

import type { AgentTask } from "../components/local-workspaces/types";

export type FridayAgentRunResult = {
  plan: string[];
  log: string[];
  result: string;
  model: string;
  generatedTokens: number;
  totalTimeMs: number;
  tokensPerSecond: number;
  inspectedWorkspace?: boolean;
  inspectedUrl?: boolean;
};

function didInspectWorkspace(log: string[]) {
  return log.some((line) => line.toLowerCase().includes("workspace file snapshot"));
}

function didInspectUrl(log: string[]) {
  return log.some((line) => line.toLowerCase().includes("explicit url"));
}

export async function tryRunTauriAgentTask(task: AgentTask): Promise<FridayAgentRunResult | null> {
  if (!isTauriRuntime() || !task.title.trim()) return null;

  try {
    const result = await invoke<FridayAgentRunResult>("friday_local_agent_run", {
      title: task.title,
      brief: task.brief,
      target: task.target,
    });
    return {
      ...result,
      inspectedWorkspace: didInspectWorkspace(result.log),
      inspectedUrl: didInspectUrl(result.log),
    };
  } catch (error) {
    console.warn("Friday local agent runner fell back to static runbook:", error);
    return null;
  }
}
