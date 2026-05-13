import type { VercelRequest, VercelResponse } from "@vercel/node";
import {
  convertToModelMessages,
  createUIMessageStream,
  pipeUIMessageStreamToResponse,
  streamText,
  type UIMessage,
} from "ai";

import {
  createLocalAssistantDraft,
  type FridayChatContext,
  getTextFromUiMessage,
  resolveFridayModel,
  streamLocalText,
} from "../../src/features/ai";

type FridayChatRequestBody = {
  messages?: UIMessage[];
  model?: string;
  context?: FridayChatContext;
};

function isCloudGatewayEnabled(): boolean {
  return process.env.FRIDAY_ENABLE_CLOUD_AI === "true";
}

export default async function handler(req: VercelRequest, res: VercelResponse) {
  if (req.method !== "POST") {
    res.status(405).json({ error: "Method not allowed" });
    return;
  }

  const body = req.body as FridayChatRequestBody;
  const messages = body.messages ?? [];
  const selectedModel = resolveFridayModel(body.model);

  if (
    selectedModel.provider === "gateway" &&
    selectedModel.gatewayModel &&
    isCloudGatewayEnabled()
  ) {
    const contextLines = [
      body.context?.projectName ? `Active project: ${body.context.projectName}` : "",
      body.context?.projectInstructions
        ? `Project instructions: ${body.context.projectInstructions}`
        : "",
      ...(body.context?.contextItems ?? []).map(
        (item) => `${item.kind}: ${item.label} - ${item.content}`,
      ),
    ].filter(Boolean);

    const result = streamText({
      model: selectedModel.gatewayModel,
      messages: await convertToModelMessages(messages),
      system: [
        "You are Friday, a local-first AI workspace assistant. Be concise, practical, and explicit about remote provider boundaries.",
        ...contextLines,
      ].join("\n"),
    });

    result.pipeUIMessageStreamToResponse(res, { originalMessages: messages });
    return;
  }

  const prompt = getTextFromUiMessage(messages[messages.length - 1]);
  const text = createLocalAssistantDraft(prompt, selectedModel, body.context);
  const textId = `friday-api-local-${Date.now().toString(36)}`;
  const stream = createUIMessageStream<UIMessage>({
    originalMessages: messages,
    execute: async ({ writer }) => {
      writer.write({ type: "start" });
      writer.write({ type: "start-step" });
      writer.write({ type: "text-start", id: textId });
      for await (const delta of streamLocalText(text)) {
        writer.write({ type: "text-delta", id: textId, delta });
      }
      writer.write({ type: "text-end", id: textId });
      writer.write({ type: "finish-step" });
      writer.write({ type: "finish", finishReason: "stop" });
    },
  });

  pipeUIMessageStreamToResponse({ response: res, stream });
}
