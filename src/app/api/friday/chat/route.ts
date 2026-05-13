import { convertToModelMessages, streamText } from "ai";
import { groq } from "@ai-sdk/groq";

import {
  createFridayGatewaySystemPrompt,
  resolveFridayGatewayChatRequest,
} from "@/features/ai/gateway-routing";

export const maxDuration = 60;
export const runtime = "nodejs";

export async function POST(request: Request) {
  const payload = await request.json().catch(() => null);
  const resolved = resolveFridayGatewayChatRequest(payload, {
    cloudEnabled: process.env.FRIDAY_ENABLE_CLOUD_AI === "true",
    groqEnabled: Boolean(process.env.GROQ_API_KEY) || process.env.FRIDAY_ENABLE_GROQ_AI === "true",
  });

  if (!resolved.ok) {
    return Response.json({ error: resolved.error }, { status: resolved.status });
  }

  const result = streamText({
    model: resolved.provider === "groq" ? groq(resolved.modelId) : resolved.modelId,
    system: createFridayGatewaySystemPrompt(resolved.context),
    messages: await convertToModelMessages(resolved.messages),
  });

  return result.toUIMessageStreamResponse();
}
