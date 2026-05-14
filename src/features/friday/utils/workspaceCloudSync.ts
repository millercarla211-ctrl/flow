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

type WorkspaceSyncOptions = {
  fetcher?: typeof fetch;
  route?: string;
  timeoutMs?: number;
};

function getBrowserLocalStorage() {
  return typeof window === "undefined" ? null : window.localStorage;
}

function createSyncFailure(error: unknown, fallback: string): FridayWorkspaceCloudSyncResult {
  const message =
    error instanceof DOMException && error.name === "AbortError"
      ? "Friday workspace sync timed out."
      : error instanceof Error
        ? error.message
        : fallback;

  return {
    ok: false,
    message,
  };
}

export async function pushFridayWorkspaceSnapshot({
  fetcher = fetch,
  route = "/api/friday/sync/workspace",
  storage,
  timeoutMs = 15_000,
}: {
  storage?: Storage | null;
} & WorkspaceSyncOptions = {}): Promise<FridayWorkspaceCloudSyncResult> {
  let timeout: ReturnType<typeof setTimeout> | null = null;

  try {
    const resolvedStorage = storage === undefined ? getBrowserLocalStorage() : storage;
    if (!resolvedStorage) {
      return {
        ok: false,
        message: "Local workspace storage is unavailable in this environment.",
      };
    }

    const controller = new AbortController();
    timeout = setTimeout(() => controller.abort(), timeoutMs);
    const backup = buildFridayWorkspaceBackup((key) => resolvedStorage.getItem(key));
    const response = await fetcher(route, {
      body: JSON.stringify(backup),
      headers: { "content-type": "application/json" },
      method: "PUT",
      signal: controller.signal,
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
  } catch (error) {
    return createSyncFailure(error, "Friday workspace sync upload failed.");
  } finally {
    if (timeout) clearTimeout(timeout);
  }
}

export async function pullFridayWorkspaceSnapshot({
  fetcher = fetch,
  route = "/api/friday/sync/workspace",
  timeoutMs = 15_000,
}: WorkspaceSyncOptions = {}): Promise<FridayWorkspaceCloudSyncResult> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetcher(route, {
      method: "GET",
      signal: controller.signal,
    });
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
  } catch (error) {
    return createSyncFailure(error, "Friday workspace sync download failed.");
  } finally {
    clearTimeout(timeout);
  }
}
