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
  groqEnabled?: boolean;
};

const MAX_GATEWAY_MESSAGES = 32;
const MAX_GATEWAY_PARTS_PER_MESSAGE = 24;
const MAX_GATEWAY_TEXT_CHARS = 24_000;
const MAX_GATEWAY_PROJECT_NAME_CHARS = 160;
const MAX_GATEWAY_PROJECT_INSTRUCTIONS_CHARS = 2_000;
const MAX_GATEWAY_CONTEXT_ITEMS = 8;
const MAX_GATEWAY_CONTEXT_KIND_CHARS = 48;
const MAX_GATEWAY_CONTEXT_LABEL_CHARS = 160;
const MAX_GATEWAY_CONTEXT_CONTENT_CHARS = 500;

export type FridayGatewayChatRequest =
  | {
      ok: true;
      context?: FridayChatContext;
      modelId: string;
      messages: UIMessage[];
      model: FridayModelOption;
      provider: "gateway" | "groq";
    }
  | {
      ok: false;
      error: string;
      status: 400 | 403;
    };

function getTextPartLength(part: unknown) {
  if (!part || typeof part !== "object") return 0;
  const candidate = part as { text?: unknown; type?: unknown };
  return candidate.type === "text" && typeof candidate.text === "string"
    ? candidate.text.length
    : 0;
}

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
        Array.isArray(candidate.parts) &&
        candidate.parts.length <= MAX_GATEWAY_PARTS_PER_MESSAGE
      );
    })
  );
}

function getUiMessageTextLength(messages: UIMessage[]) {
  return messages.reduce((total, message) => {
    return total + message.parts.reduce((partTotal, part) => partTotal + getTextPartLength(part), 0);
  }, 0);
}

function sanitizeText(value: unknown, maxLength: number) {
  if (typeof value !== "string") return undefined;
  const text = value.replace(/\s+/g, " ").trim();
  if (!text) return undefined;
  return text.length > maxLength ? `${text.slice(0, Math.max(0, maxLength - 3))}...` : text;
}

export function sanitizeFridayGatewayContext(value: unknown): FridayChatContext | undefined {
  if (!value || typeof value !== "object" || Array.isArray(value)) return undefined;

  const payload = value as {
    contextItems?: unknown;
    projectInstructions?: unknown;
    projectName?: unknown;
  };
  const context: FridayChatContext = {};
  const projectName = sanitizeText(payload.projectName, MAX_GATEWAY_PROJECT_NAME_CHARS);
  const projectInstructions = sanitizeText(
    payload.projectInstructions,
    MAX_GATEWAY_PROJECT_INSTRUCTIONS_CHARS,
  );

  if (projectName) {
    context.projectName = projectName;
  }

  if (projectInstructions) {
    context.projectInstructions = projectInstructions;
  }

  if (Array.isArray(payload.contextItems)) {
    const contextItems = payload.contextItems
      .flatMap((item) => {
        if (!item || typeof item !== "object" || Array.isArray(item)) return [];
        const candidate = item as { content?: unknown; kind?: unknown; label?: unknown };
        const content = sanitizeText(candidate.content, MAX_GATEWAY_CONTEXT_CONTENT_CHARS);
        const kind = sanitizeText(candidate.kind, MAX_GATEWAY_CONTEXT_KIND_CHARS);
        const label = sanitizeText(candidate.label, MAX_GATEWAY_CONTEXT_LABEL_CHARS);

        if (!content || !kind || !label) return [];
        return [{ content, kind, label }];
      })
      .slice(0, MAX_GATEWAY_CONTEXT_ITEMS);

    if (contextItems.length > 0) {
      context.contextItems = contextItems;
    }
  }

  return context.projectName || context.projectInstructions || context.contextItems ? context : undefined;
}

function isCloudEnabled(runtime?: FridayGatewayRuntime) {
  return runtime?.cloudEnabled ?? process.env.NEXT_PUBLIC_FRIDAY_ENABLE_CLOUD_AI === "true";
}

function isGroqEnabled(runtime?: FridayGatewayRuntime) {
  return runtime?.groqEnabled ?? process.env.NEXT_PUBLIC_FRIDAY_ENABLE_GROQ_AI === "true";
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

  if (!isUiMessageList(payload.messages)) {
    return {
      ok: false,
      status: 400,
      error: `Friday chat requires up to ${MAX_GATEWAY_MESSAGES} valid UI messages with up to ${MAX_GATEWAY_PARTS_PER_MESSAGE} parts each.`,
    };
  }

  if (payload.messages.length > MAX_GATEWAY_MESSAGES) {
    return {
      ok: false,
      status: 400,
      error: `Friday chat accepts at most ${MAX_GATEWAY_MESSAGES} messages per request.`,
    };
  }

  if (getUiMessageTextLength(payload.messages) > MAX_GATEWAY_TEXT_CHARS) {
    return {
      ok: false,
      status: 400,
      error: `Friday chat accepts at most ${MAX_GATEWAY_TEXT_CHARS.toLocaleString()} text characters per request.`,
    };
  }

  const context = sanitizeFridayGatewayContext(payload.context);
  const model = resolveFridayModel(payload.model);
  if (model.provider === "groq" && model.groqModel) {
    if (!isGroqEnabled(runtime)) {
      return {
        ok: false,
        status: 403,
        error: "Groq is disabled in this Friday build.",
      };
    }

    return {
      ok: true,
      context,
      model,
      modelId: model.groqModel,
      messages: payload.messages,
      provider: "groq",
    };
  }

  if (model.provider === "gateway" && model.gatewayModel) {
    if (!isCloudEnabled(runtime)) {
      return {
        ok: false,
        status: 403,
        error: "AI Gateway is disabled in this Friday build.",
      };
    }

    return {
      ok: true,
      context,
      model,
      modelId: model.gatewayModel,
      messages: payload.messages,
      provider: "gateway",
    };
  }

  if (model.provider === "local") {
    return {
      ok: false,
      status: 400,
      error: "Choose an enabled cloud model for this chat route.",
    };
  }

  return {
    ok: false,
    status: 400,
    error: "Choose an enabled cloud model for this chat route.",
  };
}

export function createFridayGatewaySystemPrompt(context?: FridayChatContext) {
  const safeContext = sanitizeFridayGatewayContext(context);
  const lines = [
    "You are Friday, a local-first AI workspace assistant.",
    "Be concise, practical, and explicit about uncertainty.",
    "Use cloud capability only for this approved chat response. Do not claim access to local files, apps, or private data unless the user provided that context in the message.",
  ];

  if (safeContext?.projectName) {
    lines.push(`Active project: ${safeContext.projectName}`);
  }

  if (safeContext?.projectInstructions) {
    lines.push(`Project instructions: ${safeContext.projectInstructions}`);
  }

  for (const item of safeContext?.contextItems ?? []) {
    lines.push(`Context ${item.kind} - ${item.label}: ${item.content}`);
  }

  return lines.join("\n");
}
