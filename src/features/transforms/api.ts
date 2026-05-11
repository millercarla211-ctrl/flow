import { invoke } from "@tauri-apps/api/core";
import type {
  TransformHistoryEntry,
  TransformPreset,
  TransformResult,
  TransformSource,
} from "../../types";

export async function getTransformPresets(): Promise<TransformPreset[]> {
  return invoke<TransformPreset[]>("get_transform_presets");
}

export async function getTransformSource(): Promise<TransformSource> {
  return invoke<TransformSource>("get_transform_source");
}

export async function listTransformHistory(limit: number = 20): Promise<TransformHistoryEntry[]> {
  return invoke<TransformHistoryEntry[]>("list_transform_history", { limit });
}

export async function deleteTransformHistoryEntry(id: string): Promise<boolean> {
  return invoke<boolean>("delete_transform_history_entry", { id });
}

export async function transformText(input: {
  text: string;
  presetId?: string | null;
  instruction?: string | null;
}): Promise<TransformResult> {
  return invoke<TransformResult>("transform_text", {
    text: input.text,
    presetId: input.presetId,
    instruction: input.instruction,
  });
}
