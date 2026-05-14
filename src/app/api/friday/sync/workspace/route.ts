import {
  getFridayWorkspaceSnapshot,
  parseStoredFridayWorkspaceSnapshot,
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

  const parsedSnapshot = parseStoredFridayWorkspaceSnapshot(snapshot.payload);
  if (!parsedSnapshot.ok) {
    return Response.json(
      { ok: false, message: parsedSnapshot.message },
      { status: 409 },
    );
  }

  return Response.json({
    ok: true,
    snapshot: {
      payload: parsedSnapshot.payload,
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
