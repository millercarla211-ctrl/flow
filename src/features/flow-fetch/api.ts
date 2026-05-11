import { invoke } from "@tauri-apps/api/core";
import type { FlowFetchLink } from "../../types";

export async function listFlowFetchLinks(limit: number = 30): Promise<FlowFetchLink[]> {
  return invoke<FlowFetchLink[]>("list_flow_fetch_links", { limit });
}

export async function deleteFlowFetchLink(id: string): Promise<boolean> {
  return invoke<boolean>("delete_flow_fetch_link", { id });
}

export async function copyFlowFetchLink(url: string): Promise<void> {
  await invoke("copy_flow_fetch_link", { url });
}
