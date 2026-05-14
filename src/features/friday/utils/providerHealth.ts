import { DEFAULT_FRIDAY_MODEL_KEY, FRIDAY_GROQ_MODELS } from "@/features/ai";

export type ProviderHealthStatus = "idle" | "checking" | "ready" | "blocked" | "error";

export type ProviderHealthResult = {
  checkedAt: string;
  latencyMs: number;
  message: string;
  modelKey: string;
  preview?: string;
  route: string;
  status: Exclude<ProviderHealthStatus, "idle" | "checking">;
};

type ProviderHealthOptions = {
  fetcher?: typeof fetch;
  modelKey?: string;
  route?: string;
  timeoutMs?: number;
};

function nowMs() {
  return typeof performance === "undefined" ? Date.now() : performance.now();
}

export function parseFridayStreamPayload(streamText: string) {
  let text = "";
  let errorText = "";

  for (const line of streamText.split(/\r?\n/)) {
    if (!line.startsWith("data: ")) continue;
    const payload = line.slice("data: ".length).trim();
    if (!payload || payload === "[DONE]") continue;

    try {
      const event = JSON.parse(payload) as {
        delta?: string;
        errorText?: string;
        type?: string;
      };
      if (event.type === "text-delta" && typeof event.delta === "string") {
        text += event.delta;
      }
      if (event.type === "error" && typeof event.errorText === "string") {
        errorText = event.errorText;
      }
    } catch {
      continue;
    }
  }

  return {
    errorText,
    text: text.trim(),
  };
}

export async function checkFridayProviderHealth({
  fetcher = fetch,
  modelKey = DEFAULT_FRIDAY_MODEL_KEY || FRIDAY_GROQ_MODELS[0]?.key,
  route = "/api/friday/chat",
  timeoutMs = 20_000,
}: ProviderHealthOptions = {}): Promise<ProviderHealthResult> {
  const startedAt = nowMs();
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetcher(route, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal: controller.signal,
      body: JSON.stringify({
        allowCloud: true,
        context: {
          enabledTools: [],
          localOnly: false,
          surface: "connectors",
        },
        messages: [
          {
            id: `provider-health-${Date.now().toString(36)}`,
            role: "user",
            parts: [{ type: "text", text: "Reply with exactly: Friday provider ready." }],
          },
        ],
        model: modelKey,
      }),
    });
    const body = await response.text();
    const latencyMs = Math.round(nowMs() - startedAt);
    const parsed = parseFridayStreamPayload(body);

    if (!response.ok) {
      return {
        checkedAt: new Date().toISOString(),
        latencyMs,
        message: `Server returned ${response.status}.`,
        modelKey,
        preview: parsed.errorText || body.slice(0, 160),
        route,
        status: response.status === 403 ? "blocked" : "error",
      };
    }

    if (parsed.errorText) {
      return {
        checkedAt: new Date().toISOString(),
        latencyMs,
        message: parsed.errorText,
        modelKey,
        route,
        status: "error",
      };
    }

    if (!parsed.text) {
      return {
        checkedAt: new Date().toISOString(),
        latencyMs,
        message: "The server route responded, but no model text arrived.",
        modelKey,
        route,
        status: "blocked",
      };
    }

    return {
      checkedAt: new Date().toISOString(),
      latencyMs,
      message: "Provider stream is ready.",
      modelKey,
      preview: parsed.text,
      route,
      status: "ready",
    };
  } catch (error) {
    const latencyMs = Math.round(nowMs() - startedAt);
    const errorMessage = error instanceof Error ? error.message.trim() : "";
    const message =
      error instanceof DOMException && error.name === "AbortError"
        ? "Provider check timed out."
        : error instanceof Error
          ? errorMessage || "Provider check failed."
          : "Provider check failed.";

    return {
      checkedAt: new Date().toISOString(),
      latencyMs,
      message,
      modelKey,
      route,
      status: "error",
    };
  } finally {
    clearTimeout(timeout);
  }
}
