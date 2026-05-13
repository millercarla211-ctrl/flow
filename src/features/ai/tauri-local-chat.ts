import { invoke } from "@tauri-apps/api/core";

import { isTauriRuntime } from "@/platform/tauriRuntime";

import type { FridayChatContext } from "./local-stream";
import type { FridayModelOption } from "./model-routing";

export type FridayLocalChatResult = {
  text: string;
  model: string;
  generatedTokens: number;
  totalTimeMs: number;
  tokensPerSecond: number;
};

export async function tryRunTauriLocalChat({
  prompt,
  model,
  context,
}: {
  prompt: string;
  model: FridayModelOption;
  context?: FridayChatContext;
}): Promise<FridayLocalChatResult | null> {
  if (!isTauriRuntime() || model.provider !== "local" || !prompt.trim()) {
    return null;
  }

  try {
    const result = await invoke<FridayLocalChatResult>("friday_local_chat", {
      prompt,
      modelKey: model.key,
      context,
    });
    return result.text.trim() ? result : null;
  } catch (error) {
    console.warn("Friday local chat fell back to preview transport:", error);
    return null;
  }
}
