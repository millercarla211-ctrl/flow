import { lookup } from "node:dns/promises";
import { isIP } from "node:net";

import {
  extractHtmlTitle,
  extractReadableHtmlText,
  normalizeWebInspectionUrl,
} from "@/features/friday/utils/webInspection";

export const runtime = "nodejs";
export const maxDuration = 20;

const MAX_BYTES = 1_000_000;
const PRIVATE_HOST_SUFFIXES = [".localhost", ".local", ".internal", ".lan"];

function isPrivateIp(address: string) {
  if (address.startsWith("10.")) return true;
  if (address.startsWith("127.")) return true;
  if (address.startsWith("169.254.")) return true;
  if (address.startsWith("192.168.")) return true;

  const second = Number(address.split(".")[1]);
  if (address.startsWith("172.") && second >= 16 && second <= 31) return true;

  const lower = address.toLowerCase();
  if (lower === "::1") return true;
  if (lower.startsWith("fc") || lower.startsWith("fd") || lower.startsWith("fe80:")) return true;

  return false;
}

async function assertPublicHostname(hostname: string) {
  const lowerHost = hostname.toLowerCase();
  if (PRIVATE_HOST_SUFFIXES.some((suffix) => lowerHost.endsWith(suffix))) {
    return "Private hostnames are not available for web inspection.";
  }

  if (isIP(hostname)) {
    return isPrivateIp(hostname) ? "Private network URLs are blocked." : null;
  }

  const addresses = await lookup(hostname, { all: true }).catch(() => []);
  if (addresses.length === 0) return "Unable to resolve this URL.";
  if (addresses.some((address) => isPrivateIp(address.address))) {
    return "Private network URLs are blocked.";
  }

  return null;
}

async function readLimitedText(response: Response) {
  const reader = response.body?.getReader();
  if (!reader) return "";

  const chunks: Uint8Array[] = [];
  let total = 0;

  while (total < MAX_BYTES) {
    const { done, value } = await reader.read();
    if (done || !value) break;
    chunks.push(value);
    total += value.byteLength;
  }

  return new TextDecoder("utf-8", { fatal: false }).decode(
    new Uint8Array(chunks.flatMap((chunk) => [...chunk])),
  );
}

export async function POST(request: Request) {
  const payload = (await request.json().catch(() => null)) as { url?: string } | null;
  const normalized = normalizeWebInspectionUrl(payload?.url ?? "");

  if (!normalized.ok) {
    return Response.json(normalized, { status: 400 });
  }

  const url = new URL(normalized.url);
  const hostError = await assertPublicHostname(url.hostname);
  if (hostError) {
    return Response.json({ ok: false, message: hostError, url: normalized.url }, { status: 400 });
  }

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 10_000);

  try {
    const response = await fetch(normalized.url, {
      headers: {
        accept: "text/html, text/plain;q=0.9, application/xhtml+xml;q=0.8",
        "user-agent": "FridayResearchBot/0.1 (+https://flow-roan-theta.vercel.app)",
      },
      redirect: "follow",
      signal: controller.signal,
    });

    if (!response.ok) {
      return Response.json(
        {
          ok: false,
          message: `Source returned ${response.status}.`,
          url: response.url || normalized.url,
        },
        { status: 502 },
      );
    }

    const rawText = await readLimitedText(response);
    const contentType = response.headers.get("content-type") ?? "";
    const isHtml = contentType.includes("html") || rawText.includes("<html");
    const title = isHtml ? extractHtmlTitle(rawText) : new URL(response.url).hostname;
    const excerpt = isHtml
      ? extractReadableHtmlText(rawText)
      : rawText.replace(/\s+/g, " ").trim().slice(0, 2_400);

    if (!excerpt) {
      return Response.json(
        { ok: false, message: "No readable text was found in this source.", url: response.url },
        { status: 422 },
      );
    }

    return Response.json({
      ok: true,
      excerpt,
      fetchedAt: new Date().toISOString(),
      title: title || new URL(response.url).hostname,
      url: response.url || normalized.url,
    });
  } catch (error) {
    const message =
      error instanceof DOMException && error.name === "AbortError"
        ? "Web inspection timed out."
        : error instanceof Error
          ? error.message
          : "Web inspection failed.";

    return Response.json({ ok: false, message, url: normalized.url }, { status: 500 });
  } finally {
    clearTimeout(timeout);
  }
}
