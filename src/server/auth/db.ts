import { createClient } from "@libsql/client";
import { drizzle } from "drizzle-orm/libsql";
import { appSchema } from "./schema";

let tursoClient: ReturnType<typeof createClient> | null = null;
let db: ReturnType<typeof drizzle> | null = null;

export type FridayAuthConfigStatus = {
  authUrlConfigured: boolean;
  databaseConfigured: boolean;
  tokenConfigured: boolean;
};

export function getFridayAuthConfigStatus(): FridayAuthConfigStatus {
  return {
    authUrlConfigured: Boolean(process.env.BETTER_AUTH_URL),
    databaseConfigured: Boolean(process.env.TURSO_DATABASE_URL),
    tokenConfigured: Boolean(process.env.TURSO_AUTH_TOKEN),
  };
}

export function getTursoClient() {
  const databaseUrl = process.env.TURSO_DATABASE_URL;
  if (!databaseUrl) {
    throw new Error("TURSO_DATABASE_URL is required for Friday auth.");
  }

  tursoClient ??= createClient({
    url: databaseUrl,
    authToken: process.env.TURSO_AUTH_TOKEN,
  });

  return tursoClient;
}

export function getAuthDb() {
  db ??= drizzle(getTursoClient(), {
    schema: appSchema,
  });

  return db;
}

export async function checkFridayAuthDatabase() {
  const startedAt = Date.now();
  const status = getFridayAuthConfigStatus();

  if (!status.databaseConfigured) {
    return {
      latencyMs: 0,
      message: "Turso is not configured for this build.",
      ok: false,
      status,
    };
  }

  try {
    await getTursoClient().execute("select 1");
    return {
      latencyMs: Date.now() - startedAt,
      message: "Friday account database is reachable.",
      ok: true,
      status,
    };
  } catch (error) {
    return {
      latencyMs: Date.now() - startedAt,
      message: error instanceof Error ? error.message : "Friday account database check failed.",
      ok: false,
      status,
    };
  }
}
