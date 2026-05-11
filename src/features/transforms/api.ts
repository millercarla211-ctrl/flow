import { invoke } from "@tauri-apps/api/core";
import type { TransformPreset, TransformResult } from "../../types";

export async function getTransformPresets(): Promise<TransformPreset[]> {
  return invoke<TransformPreset[]>("get_transform_presets");
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
