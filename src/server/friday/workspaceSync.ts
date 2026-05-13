import { eq } from "drizzle-orm";

import { getAuth } from "@/server/auth";
import { getAuthDb } from "@/server/auth/db";
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

export async function requireFridayWorkspaceSyncSession(
  request: Request,
): Promise<FridayWorkspaceSyncSession> {
  const session = await getAuth().api.getSession({
    headers: request.headers,
  });

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

