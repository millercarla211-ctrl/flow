import type { VercelRequest, VercelResponse } from "@vercel/node";
import { groq } from "@ai-sdk/groq";
import {
  convertToModelMessages,
  createUIMessageStream,
  pipeUIMessageStreamToResponse,
  streamText,
  type UIMessage,
} from "ai";

import {
  createLocalAssistantDraft,
  createFridayGatewaySystemPrompt,
  type FridayChatContext,
  getTextFromUiMessage,
  isCloudModel,
  resolveFridayGatewayChatRequest,
  resolveFridayModel,
  streamLocalText,
} from "../../src/features/ai";

type FridayChatRequestBody = {
  messages?: UIMessage[];
  model?: string;
  context?: FridayChatContext;
  allowCloud?: boolean;
};

export default async function handler(req: VercelRequest, res: VercelResponse) {
  if (req.method !== "POST") {
    res.status(405).json({ error: "Method not allowed" });
    return;
  }

  const body = req.body as FridayChatRequestBody;
  const messages = body.messages ?? [];
  const selectedModel = resolveFridayModel(body.model);

  if (isCloudModel(selectedModel) && body.allowCloud === true) {
    const resolved = resolveFridayGatewayChatRequest(body, {
      cloudEnabled: process.env.FRIDAY_ENABLE_CLOUD_AI === "true",
      groqEnabled: Boolean(process.env.GROQ_API_KEY) || process.env.FRIDAY_ENABLE_GROQ_AI === "true",
    });

    if (!resolved.ok) {
      res.status(resolved.status).json({ error: resolved.error });
      return;
    }

    const result = streamText({
      model: resolved.provider === "groq" ? groq(resolved.modelId) : resolved.modelId,
      messages: await convertToModelMessages(resolved.messages),
      system: createFridayGatewaySystemPrompt(resolved.context),
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
