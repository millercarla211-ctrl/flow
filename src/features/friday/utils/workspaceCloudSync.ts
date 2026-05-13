import {
  buildFridayWorkspaceBackup,
  getFridayWorkspaceBackupEntries,
  parseFridayWorkspaceBackup,
  type FridayWorkspaceBackup,
} from "./workspaceBackup";

export type FridayWorkspaceCloudSyncResult =
  | {
      ok: true;
      message: string;
      keyCount: number;
      payload?: FridayWorkspaceBackup;
      updatedAt?: string;
    }
  | {
      ok: false;
      message: string;
    };

export async function pushFridayWorkspaceSnapshot({
  route = "/api/friday/sync/workspace",
  storage = window.localStorage,
}: {
  route?: string;
  storage?: Storage;
} = {}): Promise<FridayWorkspaceCloudSyncResult> {
  const backup = buildFridayWorkspaceBackup((key) => storage.getItem(key));
  const response = await fetch(route, {
    body: JSON.stringify(backup),
    headers: { "content-type": "application/json" },
    method: "PUT",
  });
  const body = (await response.json().catch(() => null)) as {
    keyCount?: number;
    message?: string;
    ok?: boolean;
    updatedAt?: string;
  } | null;

  if (!response.ok || !body?.ok) {
    return {
      ok: false,
      message: body?.message || `Workspace sync returned ${response.status}.`,
    };
  }

  return {
    ok: true,
    keyCount: body.keyCount ?? getFridayWorkspaceBackupEntries(backup).length,
    message: "Friday workspace snapshot uploaded.",
    updatedAt: body.updatedAt,
  };
}

export async function pullFridayWorkspaceSnapshot({
  route = "/api/friday/sync/workspace",
}: {
  route?: string;
} = {}): Promise<FridayWorkspaceCloudSyncResult> {
  const response = await fetch(route);
  const body = (await response.json().catch(() => null)) as {
    message?: string;
    ok?: boolean;
    snapshot?: {
      payload?: unknown;
      updatedAt?: string;
    } | null;
  } | null;

  if (!response.ok || !body?.ok) {
    return {
      ok: false,
      message: body?.message || `Workspace sync returned ${response.status}.`,
    };
  }

  if (!body.snapshot?.payload) {
    return {
      ok: false,
      message: "No Friday workspace snapshot is stored for this account yet.",
    };
  }

  const parsed = parseFridayWorkspaceBackup(JSON.stringify(body.snapshot.payload));
  if (!parsed.ok) {
    return {
      ok: false,
      message: parsed.message,
    };
  }

  return {
    ok: true,
    keyCount: getFridayWorkspaceBackupEntries(parsed.backup).length,
    message: "Friday workspace snapshot downloaded.",
    payload: parsed.backup,
    updatedAt: body.snapshot.updatedAt,
  };
}

