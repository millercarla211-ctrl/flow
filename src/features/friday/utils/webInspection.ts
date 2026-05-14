export type WebInspectionResult =
  | {
      ok: true;
      excerpt: string;
      fetchedAt: string;
      title: string;
      url: string;
    }
  | {
      ok: false;
      message: string;
      url?: string;
    };

const PRIVATE_HOSTS = new Set(["localhost", "0.0.0.0", "127.0.0.1", "::1"]);

type WebInspectionOptions = {
  fetcher?: typeof fetch;
  route?: string;
  timeoutMs?: number;
};

function createWebInspectionFailure(error: unknown, url?: string): WebInspectionResult {
  const message =
    error instanceof DOMException && error.name === "AbortError"
      ? "Web inspection timed out."
      : error instanceof Error
        ? error.message
        : "Web inspection failed.";

  return { ok: false, message, url };
}

export function normalizeWebInspectionUrl(input: string): WebInspectionResult {
  try {
    const url = new URL(input.trim());
    if (url.protocol !== "http:" && url.protocol !== "https:") {
      return { ok: false, message: "Only http and https URLs are supported." };
    }

    if (PRIVATE_HOSTS.has(url.hostname.toLowerCase())) {
      return { ok: false, message: "Local network URLs are not available for web inspection." };
    }

    return {
      ok: true,
      excerpt: "",
      fetchedAt: "",
      title: "",
      url: url.toString(),
    };
  } catch {
    return { ok: false, message: "Enter a valid URL to inspect." };
  }
}

export function extractHtmlTitle(html: string) {
  const title = html.match(/<title[^>]*>([\s\S]*?)<\/title>/i)?.[1]?.trim() ?? "";
  return decodeHtmlEntities(title.replace(/\s+/g, " ")).slice(0, 140);
}

export function extractReadableHtmlText(html: string) {
  const text = html
    .replace(/<script[\s\S]*?<\/script>/gi, " ")
    .replace(/<style[\s\S]*?<\/style>/gi, " ")
    .replace(/<noscript[\s\S]*?<\/noscript>/gi, " ")
    .replace(/<svg[\s\S]*?<\/svg>/gi, " ")
    .replace(/<[^>]+>/g, " ")
    .replace(/\s+/g, " ")
    .trim();

  return decodeHtmlEntities(text).slice(0, 2_400);
}

export function decodeHtmlEntities(value: string) {
  return value
    .replace(/&nbsp;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'");
}

export async function inspectWebSource(
  url: string,
  {
    fetcher = fetch,
    route = "/api/friday/web/inspect",
    timeoutMs = 15_000,
  }: WebInspectionOptions = {},
): Promise<WebInspectionResult> {
  const normalized = normalizeWebInspectionUrl(url);
  if (!normalized.ok) return normalized;

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetcher(route, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal: controller.signal,
      body: JSON.stringify({ url: normalized.url }),
    });
    const payload = (await response.json().catch(() => null)) as WebInspectionResult | null;
    if (!payload) return { ok: false, message: "Web inspection returned an unreadable response." };
    return payload;
  } catch (error) {
    return createWebInspectionFailure(error, normalized.url);
  } finally {
    clearTimeout(timeout);
  }
}
