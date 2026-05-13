import { STORAGE_KEYS } from "../components/local-workspaces/types";

export const FRIDAY_WORKSPACE_BACKUP_VERSION = 1;

export const FRIDAY_WORKSPACE_STORAGE_KEYS = [
  STORAGE_KEYS.askThreads,
  STORAGE_KEYS.research,
  STORAGE_KEYS.agents,
  STORAGE_KEYS.artifacts,
  STORAGE_KEYS.projects,
  STORAGE_KEYS.projectContext,
  STORAGE_KEYS.memory,
  STORAGE_KEYS.automations,
  STORAGE_KEYS.connectors,
] as const;

type FridayWorkspaceStorageKey = (typeof FRIDAY_WORKSPACE_STORAGE_KEYS)[number];

export type FridayWorkspaceBackup = {
  app: "Friday";
  version: typeof FRIDAY_WORKSPACE_BACKUP_VERSION;
  exportedAt: string;
  keys: Partial<Record<FridayWorkspaceStorageKey, unknown>>;
};

export type FridayWorkspaceBackupParseResult =
  | { ok: true; backup: FridayWorkspaceBackup }
  | { ok: false; message: string };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseStorageValue(raw: string): unknown {
  try {
    return JSON.parse(raw) as unknown;
  } catch {
    return raw;
  }
}

export function buildFridayWorkspaceBackup(
  readItem: (key: FridayWorkspaceStorageKey) => string | null,
  exportedAt = new Date().toISOString(),
): FridayWorkspaceBackup {
  const keys: FridayWorkspaceBackup["keys"] = {};

  for (const key of FRIDAY_WORKSPACE_STORAGE_KEYS) {
    const raw = readItem(key);
    if (raw !== null) {
      keys[key] = parseStorageValue(raw);
    }
  }

  return {
    app: "Friday",
    version: FRIDAY_WORKSPACE_BACKUP_VERSION,
    exportedAt,
    keys,
  };
}

export function serializeFridayWorkspaceBackup(backup: FridayWorkspaceBackup) {
  return JSON.stringify(backup, null, 2);
}

export function parseFridayWorkspaceBackup(raw: string): FridayWorkspaceBackupParseResult {
  let parsed: unknown;

  try {
    parsed = JSON.parse(raw) as unknown;
  } catch {
    return { ok: false, message: "That file is not valid JSON." };
  }

  if (!isRecord(parsed) || parsed.app !== "Friday") {
    return { ok: false, message: "This is not a Friday workspace backup." };
  }

  if (parsed.version !== FRIDAY_WORKSPACE_BACKUP_VERSION) {
    return { ok: false, message: "This backup version is not supported by this Friday build." };
  }

  if (typeof parsed.exportedAt !== "string" || !isRecord(parsed.keys)) {
    return { ok: false, message: "This Friday backup is missing required workspace data." };
  }

  const keys: FridayWorkspaceBackup["keys"] = {};
  for (const key of FRIDAY_WORKSPACE_STORAGE_KEYS) {
    if (Object.prototype.hasOwnProperty.call(parsed.keys, key)) {
      keys[key] = parsed.keys[key];
    }
  }

  return {
    ok: true,
    backup: {
      app: "Friday",
      version: FRIDAY_WORKSPACE_BACKUP_VERSION,
      exportedAt: parsed.exportedAt,
      keys,
    },
  };
}

export function getFridayWorkspaceBackupEntries(backup: FridayWorkspaceBackup) {
  return FRIDAY_WORKSPACE_STORAGE_KEYS.flatMap((key) =>
    Object.prototype.hasOwnProperty.call(backup.keys, key)
      ? [{ key, value: backup.keys[key] }]
      : [],
  );
}

