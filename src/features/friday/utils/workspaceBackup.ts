import { STORAGE_KEYS } from "../components/local-workspaces/types";

export const FRIDAY_WORKSPACE_BACKUP_VERSION = 1;
export const FRIDAY_RESTORE_CHECKPOINT_KEY = "friday.restore-checkpoint.v1";
export const FRIDAY_WORKSPACE_BACKUP_MAX_BYTES = 8 * 1024 * 1024;

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
type FridayWorkspaceStorage = Pick<Storage, "getItem" | "setItem">;
const FRIDAY_WORKSPACE_LIST_STORAGE_KEYS = FRIDAY_WORKSPACE_STORAGE_KEYS.filter(
  (key) => key !== STORAGE_KEYS.connectors,
);
const FRIDAY_CONNECTOR_KEYS = ["localFiles", "webSearch", "aiGateway", "mcpConnectors"] as const;
const ARTIFACT_KINDS = ["Doc", "Code", "Markdown", "UI"] as const;
const AGENT_TARGETS = ["browser", "code", "files"] as const;
const AGENT_STATUSES = ["Needs approval", "Queued", "Running", "Completed", "Blocked"] as const;
const CONTEXT_KINDS = ["note", "file", "instruction"] as const;
const MEMORY_SCOPES = ["Global", "Project", "Voice"] as const;
const RESEARCH_STATUSES = ["Planned", "Drafted"] as const;
const RESEARCH_CITATION_KINDS = ["note", "file", "instruction", "memory", "web"] as const;
const FRIDAY_WORKSPACE_SECTION_LABELS: Record<FridayWorkspaceStorageKey, string> = {
  [STORAGE_KEYS.askThreads]: "Ask threads",
  [STORAGE_KEYS.research]: "Research briefs",
  [STORAGE_KEYS.agents]: "Agent tasks",
  [STORAGE_KEYS.artifacts]: "Artifacts",
  [STORAGE_KEYS.projects]: "Projects",
  [STORAGE_KEYS.projectContext]: "Project context",
  [STORAGE_KEYS.memory]: "Memories",
  [STORAGE_KEYS.automations]: "Automations",
  [STORAGE_KEYS.connectors]: "Connector settings",
};

export type FridayWorkspaceBackup = {
  app: "Friday";
  version: typeof FRIDAY_WORKSPACE_BACKUP_VERSION;
  exportedAt: string;
  keys: Partial<Record<FridayWorkspaceStorageKey, unknown>>;
};

export type FridayWorkspaceBackupParseResult =
  | { ok: true; backup: FridayWorkspaceBackup }
  | { ok: false; message: string };
type FridayWorkspaceBackupParseError = Extract<
  FridayWorkspaceBackupParseResult,
  { ok: false }
>;

export type FridayWorkspaceRestoreResult = {
  checkpoint: FridayWorkspaceBackup;
  entries: ReturnType<typeof getFridayWorkspaceBackupEntries>;
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isStringArray(value: unknown) {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

function isKnownValue<T extends readonly string[]>(values: T, value: unknown): value is T[number] {
  return typeof value === "string" && values.includes(value);
}

function parseStorageValue(raw: string): unknown {
  try {
    return JSON.parse(raw) as unknown;
  } catch {
    return raw;
  }
}

function validateLocalRecord(section: string, value: unknown) {
  if (!isRecord(value)) return `${section} records must be objects.`;
  if (
    typeof value.id !== "string" ||
    typeof value.createdAt !== "string" ||
    typeof value.updatedAt !== "string"
  ) {
    return `${section} records must include id, createdAt, and updatedAt strings.`;
  }
  return null;
}

function validateOptionalCitations(value: unknown) {
  if (value === undefined) return null;
  if (!Array.isArray(value)) return "Friday research citations must be a list.";

  for (const citation of value) {
    if (!isRecord(citation)) return "Friday research citation records must be objects.";
    if (
      typeof citation.id !== "string" ||
      typeof citation.label !== "string" ||
      typeof citation.excerpt !== "string" ||
      !isKnownValue(RESEARCH_CITATION_KINDS, citation.kind)
    ) {
      return "Friday research citations must include id, label, kind, and excerpt.";
    }
  }

  return null;
}

function validateBackupRecord(key: FridayWorkspaceStorageKey, value: unknown) {
  const baseMessage = validateLocalRecord(key, value);
  if (baseMessage) return baseMessage;
  if (!isRecord(value)) return `${key} records must be objects.`;

  if (key === STORAGE_KEYS.askThreads) {
    return typeof value.title === "string" &&
      typeof value.modelKey === "string" &&
      typeof value.messageCount === "number" &&
      Array.isArray(value.messages)
      ? null
      : "Friday Ask thread records are malformed.";
  }

  if (key === STORAGE_KEYS.research) {
    if (
      typeof value.topic !== "string" ||
      !isStringArray(value.sources) ||
      !isStringArray(value.plan)
    ) {
      return "Friday research records are malformed.";
    }
    if (value.status !== undefined && !isKnownValue(RESEARCH_STATUSES, value.status)) {
      return "Friday research status is not supported.";
    }
    return validateOptionalCitations(value.citations);
  }

  if (key === STORAGE_KEYS.agents) {
    return typeof value.title === "string" &&
      isKnownValue(AGENT_TARGETS, value.target) &&
      isKnownValue(AGENT_STATUSES, value.status)
      ? null
      : "Friday agent records are malformed.";
  }

  if (key === STORAGE_KEYS.artifacts) {
    return typeof value.title === "string" &&
      isKnownValue(ARTIFACT_KINDS, value.kind) &&
      typeof value.content === "string"
      ? null
      : "Friday artifact records are malformed.";
  }

  if (key === STORAGE_KEYS.projects) {
    return typeof value.name === "string" &&
      typeof value.instructions === "string" &&
      typeof value.modelKey === "string"
      ? null
      : "Friday project records are malformed.";
  }

  if (key === STORAGE_KEYS.projectContext) {
    return typeof value.projectId === "string" &&
      typeof value.projectName === "string" &&
      typeof value.label === "string" &&
      isKnownValue(CONTEXT_KINDS, value.kind) &&
      typeof value.content === "string"
      ? null
      : "Friday project context records are malformed.";
  }

  if (key === STORAGE_KEYS.memory) {
    return typeof value.title === "string" &&
      typeof value.body === "string" &&
      isKnownValue(MEMORY_SCOPES, value.scope) &&
      typeof value.pinned === "boolean"
      ? null
      : "Friday memory records are malformed.";
  }

  if (key === STORAGE_KEYS.automations) {
    return typeof value.title === "string" &&
      typeof value.cadence === "string" &&
      typeof value.enabled === "boolean"
      ? null
      : "Friday automation records are malformed.";
  }

  return null;
}

function validateBackupSection(key: FridayWorkspaceStorageKey, value: unknown) {
  if (key === STORAGE_KEYS.connectors) {
    if (!isRecord(value)) return "Friday connector settings must be an object.";

    const hasInvalidConnectorValue = FRIDAY_CONNECTOR_KEYS.some(
      (connectorKey) =>
        Object.prototype.hasOwnProperty.call(value, connectorKey) &&
        typeof value[connectorKey] !== "boolean",
    );

    return hasInvalidConnectorValue
      ? "Friday connector settings must use boolean values."
      : null;
  }

  if (FRIDAY_WORKSPACE_LIST_STORAGE_KEYS.includes(key) && !Array.isArray(value)) {
    return `Friday backup section ${key} must be a list.`;
  }

  if (Array.isArray(value)) {
    for (const [index, item] of value.entries()) {
      const recordMessage = validateBackupRecord(key, item);
      if (recordMessage) {
        return `${recordMessage} Check ${key} item ${index + 1}.`;
      }
    }
  }

  return null;
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

export function validateFridayWorkspaceBackupSize(
  byteSize: number,
  maxBytes = FRIDAY_WORKSPACE_BACKUP_MAX_BYTES,
): FridayWorkspaceBackupParseError | null {
  if (byteSize <= 0) {
    return {
      ok: false,
      message: "This Friday backup file is empty.",
    };
  }

  if (byteSize <= maxBytes) return null;

  const maxMegabytes = (maxBytes / 1024 / 1024).toLocaleString("en", {
    maximumFractionDigits: 1,
  });
  return {
    ok: false,
    message: `This Friday backup is too large to import safely. Choose a JSON file under ${maxMegabytes} MB.`,
  };
}

export function validateFridayWorkspaceBackupFileMetadata({
  name,
  size,
  type,
}: {
  name: string;
  size: number;
  type?: string;
}): FridayWorkspaceBackupParseError | null {
  const sizeError = validateFridayWorkspaceBackupSize(size);
  if (sizeError) return sizeError;

  const hasJsonExtension = name.toLowerCase().endsWith(".json");
  const hasJsonMimeType = type ? type.toLowerCase().includes("json") : false;
  if (hasJsonExtension || hasJsonMimeType) return null;

  return {
    ok: false,
    message: "Choose a .json Friday workspace backup file.",
  };
}

export function createFridayWorkspaceBackupFilename(
  backup: FridayWorkspaceBackup,
  prefix = "friday-workspace",
) {
  const exportedAt = new Date(backup.exportedAt);
  const dateSegment = Number.isNaN(exportedAt.getTime())
    ? backup.exportedAt
        .replace(/[^a-z0-9-]+/gi, "-")
        .replace(/^-+|-+$/g, "")
        .slice(0, 32) || "unknown-date"
    : exportedAt.toISOString().slice(0, 10);

  return `${prefix}-${dateSegment}.json`;
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
      const validationMessage = validateBackupSection(key, parsed.keys[key]);
      if (validationMessage) {
        return { ok: false, message: validationMessage };
      }

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

export function readFridayRestoreCheckpoint(
  storage: Pick<Storage, "getItem">,
): FridayWorkspaceBackupParseResult {
  const raw = storage.getItem(FRIDAY_RESTORE_CHECKPOINT_KEY);
  if (!raw) {
    return { ok: false, message: "No Friday restore checkpoint is saved yet." };
  }

  return parseFridayWorkspaceBackup(raw);
}

export function clearFridayRestoreCheckpoint(storage: Pick<Storage, "removeItem">) {
  storage.removeItem(FRIDAY_RESTORE_CHECKPOINT_KEY);
}

export function getFridayWorkspaceBackupEntries(backup: FridayWorkspaceBackup) {
  return FRIDAY_WORKSPACE_STORAGE_KEYS.flatMap((key) =>
    Object.prototype.hasOwnProperty.call(backup.keys, key)
      ? [{ key, value: backup.keys[key] }]
      : [],
  );
}

function getBackupSectionItemCount(value: unknown) {
  if (Array.isArray(value)) return value.length;
  if (isRecord(value)) return Object.keys(value).length;
  return 1;
}

export function summarizeFridayWorkspaceBackup(backup: FridayWorkspaceBackup) {
  return getFridayWorkspaceBackupEntries(backup).map((entry) => ({
    ...entry,
    count: getBackupSectionItemCount(entry.value),
    label: FRIDAY_WORKSPACE_SECTION_LABELS[entry.key],
  }));
}

export function formatFridayWorkspaceBackupSummary(backup: FridayWorkspaceBackup) {
  const summary = summarizeFridayWorkspaceBackup(backup);
  if (summary.length === 0) return "No Friday workspace sections.";

  return summary
    .map((entry) => `${entry.label}: ${entry.count}`)
    .join(", ");
}

export function formatFridayWorkspaceBackupTimestamp(backup: FridayWorkspaceBackup) {
  const exportedAt = new Date(backup.exportedAt);
  if (Number.isNaN(exportedAt.getTime())) return backup.exportedAt;

  return exportedAt
    .toISOString()
    .replace(/\.\d{3}Z$/, " UTC")
    .replace("T", " ");
}

export function formatFridayWorkspaceBackupStatus(
  backup: FridayWorkspaceBackup,
  label = "Backup",
) {
  return `${label} saved ${formatFridayWorkspaceBackupTimestamp(backup)}: ${formatFridayWorkspaceBackupSummary(backup)}.`;
}

export function formatFridayWorkspaceExportStatus(backup: FridayWorkspaceBackup) {
  const sectionCount = getFridayWorkspaceBackupEntries(backup).length;
  const sectionLabel = sectionCount === 1 ? "section" : "sections";

  return `${sectionCount} local ${sectionLabel} exported. ${formatFridayWorkspaceBackupStatus(backup)}`;
}

export function formatFridayWorkspaceRestoreStatus({
  action,
  backup,
  checkpoint,
  entries,
}: {
  action: string;
  backup: FridayWorkspaceBackup;
  checkpoint: FridayWorkspaceBackup;
  entries: FridayWorkspaceRestoreResult["entries"];
}) {
  const sectionLabel = entries.length === 1 ? "section" : "sections";

  return `${entries.length} local ${sectionLabel} ${action}: ${formatFridayWorkspaceBackupSummary(
    backup,
  )}. ${formatFridayWorkspaceBackupStatus(checkpoint, "Safety checkpoint")}`;
}

export function formatFridayRestoreCheckpointClearMessage(
  result: FridayWorkspaceBackupParseResult,
) {
  if (result.ok) {
    return `Restore checkpoint cleared: ${formatFridayWorkspaceBackupStatus(result.backup, "Checkpoint")}`;
  }

  if (result.message.includes("No Friday restore checkpoint")) {
    return "No restore checkpoint was saved.";
  }

  return `Invalid restore checkpoint cleared: ${result.message}`;
}

export function restoreFridayWorkspaceBackupToStorage({
  backup,
  checkpointAt = new Date().toISOString(),
  emitChange,
  storage,
}: {
  backup: FridayWorkspaceBackup;
  checkpointAt?: string;
  emitChange?: (key?: FridayWorkspaceStorageKey) => void;
  storage: FridayWorkspaceStorage;
}): FridayWorkspaceRestoreResult {
  const checkpoint = buildFridayWorkspaceBackup((key) => storage.getItem(key), checkpointAt);
  storage.setItem(FRIDAY_RESTORE_CHECKPOINT_KEY, serializeFridayWorkspaceBackup(checkpoint));

  const entries = getFridayWorkspaceBackupEntries(backup);
  for (const entry of entries) {
    storage.setItem(entry.key, JSON.stringify(entry.value));
    emitChange?.(entry.key);
  }
  emitChange?.();

  return { checkpoint, entries };
}
