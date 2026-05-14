import { DEFAULT_FRIDAY_MODEL_KEY } from "@/features/ai";

import type { ResearchBrief } from "../components/local-workspaces/types";
import { parseFridayStreamPayload } from "./providerHealth";

export type ProviderResearchResult =
  | {
      ok: true;
      checkedAt: string;
      latencyMs: number;
      modelKey: string;
      report: string;
    }
  | {
      ok: false;
      checkedAt: string;
      latencyMs: number;
      message: string;
      modelKey: string;
    };

function nowMs() {
  return typeof performance === "undefined" ? Date.now() : performance.now();
}

type ProviderResearchOptions = {
  brief: ResearchBrief;
  fetcher?: typeof fetch;
  modelKey?: string;
  route?: string;
  timeoutMs?: number;
};

export function buildProviderResearchPrompt(brief: ResearchBrief) {
  const citations =
    brief.citations
      ?.map((citation, index) => {
        return `[${index + 1}] ${citation.label} (${citation.kind}): ${citation.excerpt}`;
      })
      .join("\n") || "No approved citations were provided.";
  const plan = brief.plan.map((step, index) => `${index + 1}. ${step}`).join("\n");

  return [
    `Research topic: ${brief.topic}`,
    "",
    "Use only the approved local evidence below. Do not invent citations or claim web browsing.",
    "",
    "Current plan:",
    plan,
    "",
    "Approved evidence:",
    citations,
    "",
    "Write a concise markdown research brief with:",
    "- Answer",
    "- Evidence",
    "- Gaps / next checks",
    "- Citations using [1], [2] style only when supported by the approved evidence",
  ].join("\n");
}

export async function synthesizeResearchWithProvider({
  brief,
  fetcher = fetch,
  modelKey = DEFAULT_FRIDAY_MODEL_KEY,
  route = "/api/friday/chat",
  timeoutMs = 45_000,
}: ProviderResearchOptions): Promise<ProviderResearchResult> {
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
          enabledTools: ["research-synthesis"],
          localOnly: false,
          surface: "research",
        },
        messages: [
          {
            id: `research-synthesis-${brief.id}`,
            role: "user",
            parts: [{ type: "text", text: buildProviderResearchPrompt(brief) }],
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
        ok: false,
        checkedAt: new Date().toISOString(),
        latencyMs,
        message: parsed.errorText || `Server returned ${response.status}.`,
        modelKey,
      };
    }

    if (parsed.errorText) {
      return {
        ok: false,
        checkedAt: new Date().toISOString(),
        latencyMs,
        message: parsed.errorText,
        modelKey,
      };
    }

    if (!parsed.text) {
      return {
        ok: false,
        checkedAt: new Date().toISOString(),
        latencyMs,
        message: "Provider synthesis returned no text.",
        modelKey,
      };
    }

    return {
      ok: true,
      checkedAt: new Date().toISOString(),
      latencyMs,
      modelKey,
      report: parsed.text,
    };
  } catch (error) {
    const latencyMs = Math.round(nowMs() - startedAt);
    const errorMessage = error instanceof Error ? error.message.trim() : "";
    const message =
      error instanceof DOMException && error.name === "AbortError"
        ? "Provider synthesis timed out."
        : error instanceof Error
          ? errorMessage || "Provider synthesis failed."
          : "Provider synthesis failed.";

    return {
      ok: false,
      checkedAt: new Date().toISOString(),
      latencyMs,
      message,
      modelKey,
    };
  } finally {
    clearTimeout(timeout);
  }
}
