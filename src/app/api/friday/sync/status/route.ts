import { checkFridayAuthDatabase } from "@/server/auth/db";

export const runtime = "nodejs";
export const maxDuration = 10;

export async function GET() {
  const checkedAt = new Date().toISOString();
  const result = await checkFridayAuthDatabase();
  const ready =
    result.ok &&
    result.status.authUrlConfigured &&
    result.status.databaseConfigured &&
    result.status.tokenConfigured;

  return Response.json(
    {
      checkedAt,
      latencyMs: result.latencyMs,
      message: result.message,
      ready,
      status: ready ? "ready" : result.ok ? "partial" : "local-only",
      requirements: result.status,
    },
    { status: result.ok || !result.status.databaseConfigured ? 200 : 503 },
  );
}

