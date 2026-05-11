import { invoke } from "@tauri-apps/api/core";
import type { InsightsSummary } from "../../types";

export async function getInsights(days: number = 30): Promise<InsightsSummary> {
  return invoke<InsightsSummary>("get_insights", { days });
}
