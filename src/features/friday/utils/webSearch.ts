import { decodeHtmlEntities, normalizeWebInspectionUrl } from "./webInspection";

export type WebSearchResultItem = {
  snippet: string;
  source: string;
  title: string;
  url: string;
};

export type WebSearchResult =
  | {
      ok: true;
      query: string;
      results: WebSearchResultItem[];
      searchedAt: string;
    }
  | {
      ok: false;
      message: string;
      query?: string;
    };

function stripTags(value: string) {
  return decodeHtmlEntities(value.replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim());
}

function decodeDuckDuckGoUrl(href: string) {
  const normalized = href.startsWith("//") ? `https:${href}` : href;

  try {
    const parsed = new URL(normalized);
    const redirect = parsed.searchParams.get("uddg");
    return redirect ? decodeURIComponent(redirect) : parsed.toString();
  } catch {
    return normalized;
  }
}

export function parseDuckDuckGoLiteResults(html: string): WebSearchResultItem[] {
  const results: WebSearchResultItem[] = [];
  const linkPattern = /<a[^>]+href="([^"]+)"[^>]*>([\s\S]*?)<\/a>/gi;
  let match: RegExpExecArray | null;

  while ((match = linkPattern.exec(html)) && results.length < 8) {
    const rawHref = match[1];
    const rawTitle = match[2];
    if (!rawHref || rawHref.startsWith("#")) continue;

    const title = stripTags(rawTitle);
    const url = decodeDuckDuckGoUrl(rawHref);
    const normalized = normalizeWebInspectionUrl(url);
    if (!title || !normalized.ok) continue;
    if (results.some((result) => result.url === normalized.url)) continue;

    const afterLink = html.slice(linkPattern.lastIndex, linkPattern.lastIndex + 1_200);
    const snippet =
      stripTags(afterLink.match(/<td[^>]*class="result-snippet"[^>]*>([\s\S]*?)<\/td>/i)?.[1] ?? "")
        .replace(/^&nbsp;/, "")
        .slice(0, 260) || "Search result ready for inspection.";

    results.push({
      snippet,
      source: new URL(normalized.url).hostname.replace(/^www\./, ""),
      title: title.slice(0, 160),
      url: normalized.url,
    });
  }

  return results;
}

export async function searchWebSources(query: string): Promise<WebSearchResult> {
  const cleanQuery = query.trim();
  if (cleanQuery.length < 3) {
    return { ok: false, message: "Search needs at least three characters." };
  }

  const response = await fetch("/api/friday/web/search", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ query: cleanQuery }),
  });
  const payload = (await response.json().catch(() => null)) as WebSearchResult | null;
  if (!payload) return { ok: false, message: "Web search returned an unreadable response." };
  return payload;
}
