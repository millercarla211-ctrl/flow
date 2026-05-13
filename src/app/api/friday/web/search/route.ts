import { parseDuckDuckGoLiteResults } from "@/features/friday/utils/webSearch";

export const runtime = "nodejs";
export const maxDuration = 20;

function normalizeQuery(value: unknown) {
  return typeof value === "string" ? value.replace(/\s+/g, " ").trim().slice(0, 240) : "";
}

export async function POST(request: Request) {
  const payload = (await request.json().catch(() => null)) as { query?: string } | null;
  const query = normalizeQuery(payload?.query);

  if (query.length < 3) {
    return Response.json(
      { ok: false, message: "Search needs at least three characters.", query },
      { status: 400 },
    );
  }

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 10_000);

  try {
    const searchUrl = new URL("https://lite.duckduckgo.com/lite/");
    searchUrl.searchParams.set("q", query);

    const response = await fetch(searchUrl, {
      headers: {
        accept: "text/html",
        "user-agent": "FridayResearchBot/0.1 (+https://flow-roan-theta.vercel.app)",
      },
      signal: controller.signal,
    });

    if (!response.ok) {
      return Response.json(
        { ok: false, message: `Search returned ${response.status}.`, query },
        { status: 502 },
      );
    }

    const html = await response.text();
    const results = parseDuckDuckGoLiteResults(html);

    if (results.length === 0) {
      return Response.json(
        { ok: false, message: "No readable search results were found.", query },
        { status: 502 },
      );
    }

    return Response.json({
      ok: true,
      query,
      results,
      searchedAt: new Date().toISOString(),
    });
  } catch (error) {
    const message =
      error instanceof DOMException && error.name === "AbortError"
        ? "Web search timed out."
        : error instanceof Error
          ? error.message
          : "Web search failed.";

    return Response.json({ ok: false, message, query }, { status: 500 });
  } finally {
    clearTimeout(timeout);
  }
}
