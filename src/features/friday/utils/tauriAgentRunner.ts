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
};

export async function tryRunTauriAgentTask(task: AgentTask): Promise<FridayAgentRunResult | null> {
  if (!isTauriRuntime() || !task.title.trim()) return null;

  try {
    return await invoke<FridayAgentRunResult>("friday_local_agent_run", {
      title: task.title,
      target: task.target,
    });
  } catch (error) {
    console.warn("Friday local agent runner fell back to static runbook:", error);
    return null;
  }
}
