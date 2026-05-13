import type { UIMessage } from "ai";

import type { FridayChatContext } from "./local-stream";
import { resolveFridayModel, type FridayModelOption } from "./model-routing";

type FridayGatewayChatPayload = {
  allowCloud?: boolean;
  context?: FridayChatContext;
  messages?: UIMessage[];
  model?: string;
};

type FridayGatewayRuntime = {
  cloudEnabled?: boolean;
};

export type FridayGatewayChatRequest =
  | {
      ok: true;
      context?: FridayChatContext;
      gatewayModel: string;
      messages: UIMessage[];
      model: FridayModelOption;
    }
  | {
      ok: false;
      error: string;
      status: 400 | 403;
    };

function isUiMessageList(value: unknown): value is UIMessage[] {
  return (
    Array.isArray(value) &&
    value.every((message) => {
      if (!message || typeof message !== "object") return false;
      const candidate = message as Partial<UIMessage>;
      return (
        typeof candidate.id === "string" &&
        (candidate.role === "user" ||
          candidate.role === "assistant" ||
          candidate.role === "system") &&
        Array.isArray(candidate.parts)
      );
    })
  );
}

function isCloudEnabled(runtime?: FridayGatewayRuntime) {
  return runtime?.cloudEnabled ?? process.env.NEXT_PUBLIC_FRIDAY_ENABLE_CLOUD_AI === "true";
}

export function resolveFridayGatewayChatRequest(
  payload: FridayGatewayChatPayload | null | undefined,
  runtime?: FridayGatewayRuntime,
): FridayGatewayChatRequest {
  if (!payload?.allowCloud) {
    return {
      ok: false,
      status: 403,
      error: "Cloud AI is disabled for this request.",
    };
  }

  if (!isCloudEnabled(runtime)) {
    return {
      ok: false,
      status: 403,
      error: "Cloud AI is disabled in this Friday build.",
    };
  }

  if (!isUiMessageList(payload.messages)) {
    return {
      ok: false,
      status: 400,
      error: "Friday chat requires UI messages.",
    };
  }

  const model = resolveFridayModel(payload.model);
  if (model.provider !== "gateway" || !model.gatewayModel) {
    return {
      ok: false,
      status: 400,
      error: "Choose an enabled gateway model for cloud chat.",
    };
  }

  return {
    ok: true,
    context: payload.context,
    gatewayModel: model.gatewayModel,
    messages: payload.messages,
    model,
  };
}

export function createFridayGatewaySystemPrompt(context?: FridayChatContext) {
  const lines = [
    "You are Friday, a local-first AI workspace assistant.",
    "Be concise, practical, and explicit about uncertainty.",
    "Use cloud capability only for this approved chat response. Do not claim access to local files, apps, or private data unless the user provided that context in the message.",
  ];

  if (context?.projectName) {
    lines.push(`Active project: ${context.projectName}`);
  }

  if (context?.projectInstructions) {
    lines.push(`Project instructions: ${context.projectInstructions}`);
  }

  for (const item of context?.contextItems?.slice(0, 8) ?? []) {
    const content = item.content.length > 500 ? `${item.content.slice(0, 497)}...` : item.content;
    lines.push(`Context ${item.kind} - ${item.label}: ${content}`);
  }

  return lines.join("\n");
}
