import { invoke } from "@tauri-apps/api/core";
import type { ScratchpadEntry, ScratchpadVersion } from "../../types";

export async function listScratchpadEntries(
  searchQuery: string | null,
): Promise<ScratchpadEntry[]> {
  return invoke<ScratchpadEntry[]>("list_scratchpad_entries", { searchQuery });
}

export async function listScratchpadVersions(entryId: string): Promise<ScratchpadVersion[]> {
  return invoke<ScratchpadVersion[]>("list_scratchpad_versions", { entryId });
}

export async function createScratchpadEntry(
  body: string,
  source: string = "manual",
): Promise<ScratchpadEntry> {
  return invoke<ScratchpadEntry>("create_scratchpad_entry", { body, source });
}

export async function updateScratchpadEntry(input: {
  id: string;
  title?: string | null;
  body: string;
}): Promise<ScratchpadEntry | null> {
  return invoke<ScratchpadEntry | null>("update_scratchpad_entry", input);
}

export async function deleteScratchpadEntry(id: string): Promise<boolean> {
  return invoke<boolean>("delete_scratchpad_entry", { id });
}
