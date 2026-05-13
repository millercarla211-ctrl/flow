import { convertToModelMessages, streamText } from "ai";

import {
  createFridayGatewaySystemPrompt,
  resolveFridayGatewayChatRequest,
} from "@/features/ai/gateway-routing";

export const maxDuration = 60;
export const runtime = "nodejs";

export async function POST(request: Request) {
  const payload = await request.json().catch(() => null);
  const resolved = resolveFridayGatewayChatRequest(payload);

  if (!resolved.ok) {
    return Response.json({ error: resolved.error }, { status: resolved.status });
  }

  const result = streamText({
    model: resolved.gatewayModel,
    system: createFridayGatewaySystemPrompt(resolved.context),
    messages: await convertToModelMessages(resolved.messages),
  });

  return result.toUIMessageStreamResponse();
}
