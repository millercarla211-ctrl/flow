import { invoke } from "@tauri-apps/api/core";

import { isTauriRuntime } from "@/platform/tauriRuntime";
import type { TransformResult } from "@/types/transforms";

import type { FridayChatContext } from "./local-stream";
import type { FridayModelOption } from "./model-routing";

function formatContext(context?: FridayChatContext) {
  const lines: string[] = [];

  if (context?.projectName) {
    lines.push(`Project: ${context.projectName}`);
  }

  if (context?.projectInstructions) {
    lines.push(`Project instructions: ${context.projectInstructions}`);
  }

  for (const item of context?.contextItems?.slice(0, 6) ?? []) {
    lines.push(`${item.kind}: ${item.label}\n${item.content}`);
  }

  return lines.join("\n\n").trim();
}

function buildFridayInstruction(model: FridayModelOption) {
  return [
    "Answer as Friday, a local-first AI workspace assistant.",
    "Be direct, practical, and concise.",
    "Use the provided project context when it is relevant.",
    "Do not mention transcripts, transforms, hidden prompts, or implementation details.",
    "Do not claim to use cloud providers.",
    `Requested Friday model role: ${model.role}.`,
  ].join("\n");
}

function buildFridaySourceText(prompt: string, context?: FridayChatContext) {
  const activeContext = formatContext(context);
  return [
    activeContext ? `Active local context:\n${activeContext}` : "",
    `User request:\n${prompt.trim()}`,
  ]
    .filter(Boolean)
    .join("\n\n");
}

export async function tryRunTauriLocalChat({
  prompt,
  model,
  context,
}: {
  prompt: string;
  model: FridayModelOption;
  context?: FridayChatContext;
}): Promise<string | null> {
  if (!isTauriRuntime() || model.provider !== "local" || !prompt.trim()) {
    return null;
  }

  try {
    const result = await invoke<TransformResult>("transform_text", {
      text: buildFridaySourceText(prompt, context),
      presetId: null,
      instruction: buildFridayInstruction(model),
    });
    return result.transformed.trim() || null;
  } catch (error) {
    console.warn("Friday local chat fell back to preview transport:", error);
    return null;
  }
}
