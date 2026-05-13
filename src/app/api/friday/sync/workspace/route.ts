import {
  getFridayWorkspaceSnapshot,
  requireFridayWorkspaceSyncSession,
  saveFridayWorkspaceSnapshot,
  validateFridayWorkspaceSyncPayload,
} from "@/server/friday/workspaceSync";

export const runtime = "nodejs";
export const maxDuration = 20;

export async function GET(request: Request) {
  const session = await requireFridayWorkspaceSyncSession(request);
  if (!session.ok) return session.response;

  const snapshot = await getFridayWorkspaceSnapshot(session.userId);
  if (!snapshot) {
    return Response.json({ ok: true, snapshot: null });
  }

  return Response.json({
    ok: true,
    snapshot: {
      payload: JSON.parse(snapshot.payload) as unknown,
      updatedAt: snapshot.updatedAt.toISOString(),
      version: snapshot.version,
    },
  });
}

export async function PUT(request: Request) {
  const session = await requireFridayWorkspaceSyncSession(request);
  if (!session.ok) return session.response;

  const payload = await request.json().catch(() => null);
  const validated = validateFridayWorkspaceSyncPayload(payload);

  if (!validated.ok) {
    return Response.json({ ok: false, message: validated.message }, { status: 400 });
  }

  const updatedAt = await saveFridayWorkspaceSnapshot(session.userId, validated.raw);

  return Response.json({
    ok: true,
    keyCount: Object.keys(validated.backup.keys).length,
    updatedAt,
  });
}

