import { eq } from "drizzle-orm";

import { getAuth } from "@/server/auth";
import { getAuthDb, getFridayAuthConfigStatus } from "@/server/auth/db";
import { fridayWorkspaceSnapshot } from "@/server/auth/schema";
import { parseFridayWorkspaceBackup } from "@/features/friday/utils/workspaceBackup";

const MAX_WORKSPACE_SYNC_BYTES = 1_500_000;

export type FridayWorkspaceSyncSession =
  | {
      ok: true;
      userId: string;
    }
  | {
      ok: false;
      response: Response;
    };

function getWorkspaceSyncUnavailableResponse() {
  const requirements = getFridayAuthConfigStatus();
  const missing = [
    requirements.authUrlConfigured ? null : "BETTER_AUTH_URL",
    requirements.databaseConfigured ? null : "TURSO_DATABASE_URL",
    requirements.tokenConfigured ? null : "TURSO_AUTH_TOKEN",
  ].filter(Boolean);

  if (missing.length === 0) return null;

  return Response.json(
    {
      ok: false,
      message: `Friday workspace sync is in local-only mode. Configure ${missing.join(
        ", ",
      )} to enable account sync.`,
      requirements,
    },
    { status: 503 },
  );
}

export async function requireFridayWorkspaceSyncSession(
  request: Request,
): Promise<FridayWorkspaceSyncSession> {
  const unavailableResponse = getWorkspaceSyncUnavailableResponse();
  if (unavailableResponse) {
    return {
      ok: false,
      response: unavailableResponse,
    };
  }

  let session: Awaited<ReturnType<ReturnType<typeof getAuth>["api"]["getSession"]>>;
  try {
    session = await getAuth().api.getSession({
      headers: request.headers,
    });
  } catch (error) {
    return {
      ok: false,
      response: Response.json(
        {
          ok: false,
          message:
            error instanceof Error
              ? error.message
              : "Friday workspace sync session check failed.",
        },
        { status: 503 },
      ),
    };
  }

  if (!session?.user.id) {
    return {
      ok: false,
      response: Response.json(
        { ok: false, message: "Sign in before using Friday workspace sync." },
        { status: 401 },
      ),
    };
  }

  return { ok: true, userId: session.user.id };
}

export function validateFridayWorkspaceSyncPayload(payload: unknown) {
  const raw = JSON.stringify(payload);
  if (raw.length > MAX_WORKSPACE_SYNC_BYTES) {
    return {
      ok: false as const,
      message: "Friday workspace sync payload is too large.",
    };
  }

  const parsed = parseFridayWorkspaceBackup(raw);
  if (!parsed.ok) {
    return {
      ok: false as const,
      message: parsed.message,
    };
  }

  return {
    backup: parsed.backup,
    ok: true as const,
    raw,
  };
}

export async function getFridayWorkspaceSnapshot(userId: string) {
  const [snapshot] = await getAuthDb()
    .select()
    .from(fridayWorkspaceSnapshot)
    .where(eq(fridayWorkspaceSnapshot.userId, userId))
    .limit(1);

  return snapshot ?? null;
}

export function parseStoredFridayWorkspaceSnapshot(payload: string) {
  try {
    const parsed = JSON.parse(payload) as unknown;
    const validated = parseFridayWorkspaceBackup(JSON.stringify(parsed));
    if (!validated.ok) {
      return {
        message: validated.message,
        ok: false as const,
      };
    }

    return {
      ok: true as const,
      payload: parsed,
    };
  } catch {
    return {
      message: "Stored Friday workspace snapshot is corrupted.",
      ok: false as const,
    };
  }
}

export async function saveFridayWorkspaceSnapshot(userId: string, payload: string) {
  const now = new Date();

  await getAuthDb()
    .insert(fridayWorkspaceSnapshot)
    .values({
      createdAt: now,
      payload,
      updatedAt: now,
      userId,
      version: 1,
    })
    .onConflictDoUpdate({
      target: fridayWorkspaceSnapshot.userId,
      set: {
        payload,
        updatedAt: now,
        version: 1,
      },
    });

  return now.toISOString();
}
