import { createClient } from "@libsql/client";
import { drizzle } from "drizzle-orm/libsql";
import { authSchema } from "./schema";

const databaseUrl = process.env.TURSO_DATABASE_URL;

if (!databaseUrl) {
  throw new Error("TURSO_DATABASE_URL is required for Friday auth.");
}

export const tursoClient = createClient({
  url: databaseUrl,
  authToken: process.env.TURSO_AUTH_TOKEN,
});

export const db = drizzle(tursoClient, {
  schema: authSchema,
});
