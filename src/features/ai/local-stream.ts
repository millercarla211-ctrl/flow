import type { UIMessage } from "ai";

import type { FridayModelOption } from "./model-routing";

const STREAM_WORD_DELAY_MS = 18;

export type FridayChatContext = {
  projectName?: string;
  projectInstructions?: string;
  contextItems?: Array<{
    label: string;
    kind: string;
    content: string;
  }>;
};

export function getTextFromUiMessage(message: UIMessage | undefined): string {
  if (!message) return "";

  return message.parts
    .map((part) => {
      if (part.type === "text") return part.text;
      return "";
    })
    .join(" ")
    .replace(/\s+/g, " ")
    .trim();
}

function formatContext(context?: FridayChatContext): string[] {
  if (!context?.projectName && !context?.contextItems?.length && !context?.projectInstructions) {
    return [];
  }

  const lines = ["", "Active context:"];

  if (context.projectName) {
    lines.push(`- Project: ${context.projectName}`);
  }

  if (context.projectInstructions) {
    lines.push(`- Instructions: ${context.projectInstructions}`);
  }

  for (const item of context.contextItems?.slice(0, 4) ?? []) {
    const content = item.content.length > 140 ? `${item.content.slice(0, 137)}...` : item.content;
    lines.push(`- ${item.kind}: ${item.label} - ${content}`);
  }

  return lines;
}

export function createLocalAssistantDraft(
  prompt: string,
  model: FridayModelOption,
  context?: FridayChatContext,
): string {
  const trimmedPrompt = prompt.trim();
  const promptLine = trimmedPrompt ? `You asked: "${trimmedPrompt}"` : "Ask me what to work on.";

  return [
    `Friday is running in local-first mode with ${model.label}.`,
    promptLine,
    ...formatContext(context),
    "",
    "I can help with the next step from the local workspace shell now. Cloud providers, web agents, and remote connectors stay off until you explicitly enable them.",
    "",
    "Suggested path:",
    "- Use Ask for quick drafting and coding help.",
    "- Use Research when you need cited source work and an editable plan.",
    "- Use Voice when you want the current WhisperFlow Beater dictation stack.",
  ].join("\n");
}

export async function* streamLocalText(
  text: string,
  signal?: AbortSignal,
): AsyncGenerator<string, void, void> {
  const chunks = text.match(/\S+\s*/g) ?? [text];

  for (const chunk of chunks) {
    if (signal?.aborted) return;
    await new Promise((resolve) => setTimeout(resolve, STREAM_WORD_DELAY_MS));
    yield chunk;
  }
}

export async function collectLocalTextStream(text: string): Promise<string> {
  let result = "";
  for await (const chunk of streamLocalText(text)) {
    result += chunk;
  }
  return result;
}
