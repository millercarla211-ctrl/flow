import { drizzleAdapter } from "@better-auth/drizzle-adapter";
import { betterAuth } from "better-auth";
import { getAuthDb } from "./db";
import { authSchema } from "./schema";

function parseTrustedOrigins(): string[] {
  const raw =
    process.env.BETTER_AUTH_TRUSTED_ORIGINS ??
    process.env.FLOW_AUTH_TRUSTED_ORIGINS ??
    "http://localhost:1420,http://localhost:5173,http://localhost:8735,http://localhost:3000,tauri://localhost,http://tauri.localhost";

  return raw
    .split(",")
    .map((origin) => origin.trim())
    .filter(Boolean);
}

function createFridayAuth() {
  return betterAuth({
    appName: "Friday",
    baseURL: process.env.BETTER_AUTH_URL,
    secret: process.env.BETTER_AUTH_SECRET ?? "flow-development-secret-change-me",
    trustedOrigins: parseTrustedOrigins(),
    database: drizzleAdapter(getAuthDb(), {
      provider: "sqlite",
      schema: authSchema,
      camelCase: true,
    }),
    emailAndPassword: {
      enabled: true,
      minPasswordLength: 8,
    },
  });
}

let authInstance: ReturnType<typeof createFridayAuth> | null = null;

export function getAuth(): ReturnType<typeof createFridayAuth> {
  if (!authInstance) {
    authInstance = createFridayAuth();
  }

  return authInstance;
}

export type FlowAuth = ReturnType<typeof getAuth>;
