import { Activity, Database, Download, Link2, RefreshCw, RotateCcw, Trash2, Upload } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  emitFridayStorageChange,
  FRIDAY_STORAGE_EVENT,
  useLocalSettings,
} from "../../hooks/useLocalPersistence";
import { checkFridayProviderHealth, type ProviderHealthResult } from "../../utils/providerHealth";
import { checkFridaySyncHealth, type FridaySyncHealthResult } from "../../utils/syncHealth";
import {
  pullFridayWorkspaceSnapshot,
  pushFridayWorkspaceSnapshot,
} from "../../utils/workspaceCloudSync";
import {
  buildFridayWorkspaceBackup,
  clearFridayRestoreCheckpoint,
  createFridayWorkspaceBackupFilename,
  formatFridayWorkspaceBackupSummary,
  formatFridayWorkspaceBackupStatus,
  getFridayWorkspaceBackupEntries,
  parseFridayWorkspaceBackup,
  readFridayRestoreCheckpoint,
  restoreFridayWorkspaceBackupToStorage,
  serializeFridayWorkspaceBackup,
  type FridayWorkspaceBackup,
} from "../../utils/workspaceBackup";
import { DEFAULT_CONNECTORS, STORAGE_KEYS, type ConnectorSettings } from "./types";

const CONNECTOR_OPTIONS: Array<[keyof ConnectorSettings, string, string]> = [
  ["localFiles", "Local files", "Allow Friday to use local project files you add."],
  ["webSearch", "Web search", "Disabled until you explicitly enable remote lookup."],
  ["aiGateway", "Cloud AI", "Use configured online providers such as Groq or AI Gateway."],
  ["mcpConnectors", "MCP connectors", "Future app connectors stay off until configured."],
];

function downloadWorkspaceBackup(backup: FridayWorkspaceBackup, prefix?: string) {
  const payload = serializeFridayWorkspaceBackup(backup);
  const url = URL.createObjectURL(new Blob([payload], { type: "application/json" }));
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = createFridayWorkspaceBackupFilename(backup, prefix);
  anchor.click();
  URL.revokeObjectURL(url);
}

type RestoreCheckpointState = {
  tone: "idle" | "ready" | "error";
  text: string;
};

export function ConnectorsWorkspace() {
  const backupInputRef = useRef<HTMLInputElement>(null);
  const [providerHealth, setProviderHealth] = useState<ProviderHealthResult | null>(null);
  const [isCheckingProvider, setIsCheckingProvider] = useState(false);
  const [syncHealth, setSyncHealth] = useState<FridaySyncHealthResult | null>(null);
  const [isCheckingSync, setIsCheckingSync] = useState(false);
  const [isSyncingWorkspace, setIsSyncingWorkspace] = useState(false);
  const [workspaceSyncMessage, setWorkspaceSyncMessage] = useState<{
    tone: "success" | "error";
    text: string;
  } | null>(null);
  const [backupMessage, setBackupMessage] = useState<{
    tone: "success" | "error" | "idle";
    text: string;
  } | null>(null);
  const [restoreCheckpoint, setRestoreCheckpoint] = useState<RestoreCheckpointState>({
    tone: "idle",
    text: "No restore checkpoint saved yet.",
  });
  const { settings, updateSettings } = useLocalSettings<ConnectorSettings>(
    STORAGE_KEYS.connectors,
    DEFAULT_CONNECTORS,
  );
  const cloudEnvEnabled =
    process.env.NEXT_PUBLIC_FRIDAY_ENABLE_CLOUD_AI === "true" ||
    process.env.NEXT_PUBLIC_FRIDAY_ENABLE_GROQ_AI === "true";
  const providerCheckDisabled = !settings.aiGateway || !cloudEnvEnabled || isCheckingProvider;
  const hasRestoreCheckpoint = restoreCheckpoint.tone === "ready";
  const canClearRestoreCheckpoint = restoreCheckpoint.tone !== "idle";

  const refreshRestoreCheckpoint = useCallback(() => {
    const parsed = readFridayRestoreCheckpoint(window.localStorage);

    if (parsed.ok) {
      setRestoreCheckpoint({
        tone: "ready",
        text: formatFridayWorkspaceBackupStatus(parsed.backup, "Available checkpoint"),
      });
      return;
    }

    const hasMissingCheckpoint = parsed.message.includes("No Friday restore checkpoint");
    setRestoreCheckpoint({
      tone: hasMissingCheckpoint ? "idle" : "error",
      text: parsed.message,
    });
  }, []);

  useEffect(() => {
    refreshRestoreCheckpoint();

    const onChange = () => refreshRestoreCheckpoint();
    window.addEventListener(FRIDAY_STORAGE_EVENT, onChange);
    window.addEventListener("storage", onChange);
    return () => {
      window.removeEventListener(FRIDAY_STORAGE_EVENT, onChange);
      window.removeEventListener("storage", onChange);
    };
  }, [refreshRestoreCheckpoint]);

  const runProviderCheck = async () => {
    if (providerCheckDisabled) return;
    setIsCheckingProvider(true);
    const result = await checkFridayProviderHealth();
    setProviderHealth(result);
    setIsCheckingProvider(false);
  };

  const runSyncCheck = async () => {
    if (isCheckingSync) return;
    setIsCheckingSync(true);
    const result = await checkFridaySyncHealth();
    setSyncHealth(result);
    setIsCheckingSync(false);
  };

  const exportWorkspaceBackup = () => {
    const backup = buildFridayWorkspaceBackup((key) => window.localStorage.getItem(key));
    downloadWorkspaceBackup(backup);

    const count = getFridayWorkspaceBackupEntries(backup).length;
    setBackupMessage({
      tone: "success",
      text: `${count} local section${count === 1 ? "" : "s"} exported: ${formatFridayWorkspaceBackupSummary(backup)}.`,
    });
  };

  const exportRestoreCheckpoint = () => {
    const parsed = readFridayRestoreCheckpoint(window.localStorage);
    if (!parsed.ok) {
      setBackupMessage({ tone: "error", text: parsed.message });
      return;
    }

    downloadWorkspaceBackup(parsed.backup, "friday-restore-checkpoint");
    setBackupMessage({
      tone: "success",
      text: `Restore checkpoint exported: ${formatFridayWorkspaceBackupStatus(parsed.backup, "Checkpoint")}`,
    });
  };

  const importWorkspaceBackup = async (file: File | undefined) => {
    if (!file) return;

    const parsed = parseFridayWorkspaceBackup(await file.text());
    if (!parsed.ok) {
      setBackupMessage({ tone: "error", text: parsed.message });
      return;
    }

    const { checkpoint, entries } = restoreFridayWorkspaceBackupToStorage({
      backup: parsed.backup,
      emitChange: emitFridayStorageChange,
      storage: window.localStorage,
    });

    setBackupMessage({
      tone: "success",
      text: `${entries.length} local section${entries.length === 1 ? "" : "s"} restored: ${formatFridayWorkspaceBackupSummary(parsed.backup)}. Safety checkpoint saved: ${formatFridayWorkspaceBackupSummary(checkpoint)}.`,
    });
  };

  const restoreSafetyCheckpoint = () => {
    const parsed = readFridayRestoreCheckpoint(window.localStorage);
    if (!parsed.ok) {
      setBackupMessage({ tone: "error", text: parsed.message });
      return;
    }

    const { checkpoint, entries } = restoreFridayWorkspaceBackupToStorage({
      backup: parsed.backup,
      emitChange: emitFridayStorageChange,
      storage: window.localStorage,
    });

    setBackupMessage({
      tone: "success",
      text: `${entries.length} local section${entries.length === 1 ? "" : "s"} restored from checkpoint: ${formatFridayWorkspaceBackupSummary(parsed.backup)}. New safety checkpoint saved: ${formatFridayWorkspaceBackupSummary(checkpoint)}.`,
    });
    refreshRestoreCheckpoint();
  };

  const clearRestoreCheckpoint = () => {
    clearFridayRestoreCheckpoint(window.localStorage);
    emitFridayStorageChange();
    refreshRestoreCheckpoint();
    setBackupMessage({
      tone: "success",
      text: "Restore checkpoint cleared from this browser profile.",
    });
  };

  const pushWorkspaceSnapshot = async () => {
    if (isSyncingWorkspace) return;
    setIsSyncingWorkspace(true);
    try {
      const result = await pushFridayWorkspaceSnapshot();
      setWorkspaceSyncMessage({
        tone: result.ok ? "success" : "error",
        text: result.ok
          ? `${result.keyCount} local section${result.keyCount === 1 ? "" : "s"} uploaded.`
          : result.message,
      });
    } finally {
      setIsSyncingWorkspace(false);
    }
  };

  const pullWorkspaceSnapshot = async () => {
    if (isSyncingWorkspace) return;
    setIsSyncingWorkspace(true);
    try {
      const result = await pullFridayWorkspaceSnapshot();

      if (result.ok && result.payload) {
        const { checkpoint, entries } = restoreFridayWorkspaceBackupToStorage({
          backup: result.payload,
          emitChange: emitFridayStorageChange,
          storage: window.localStorage,
        });
        setWorkspaceSyncMessage({
          tone: "success",
          text: `${entries.length} local section${
            entries.length === 1 ? "" : "s"
          } restored from sync: ${formatFridayWorkspaceBackupSummary(result.payload)}. Safety checkpoint saved: ${formatFridayWorkspaceBackupSummary(checkpoint)}.`,
        });
      } else {
        setWorkspaceSyncMessage({
          tone: "error",
          text: result.message,
        });
      }
    } finally {
      setIsSyncingWorkspace(false);
    }
  };

  return (
    <div className="space-y-3">
      <div className="rounded-lg border border-[var(--border)] bg-[var(--secondary)] p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <div className="text-sm font-semibold text-[var(--foreground)]">
              Local-first privacy boundary
            </div>
            <p className="mt-2 text-xs leading-5 text-[var(--muted-foreground)]">
              Friday will not call online AI unless the environment allows it and Cloud AI is
              enabled here. Groq runs through the server so the API key stays out of the client.
            </p>
          </div>
          <Badge variant="outline" className="border-[var(--border)]">
            {settings.aiGateway && cloudEnvEnabled ? "Cloud allowed" : "Local only"}
          </Badge>
        </div>
      </div>

      <div className="rounded-lg border border-[var(--border)] bg-[var(--background)] p-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="min-w-0">
            <div className="flex items-center gap-2 text-sm font-semibold text-[var(--foreground)]">
              <Database size={15} />
              Account sync health
            </div>
            <p className="mt-2 max-w-2xl text-xs leading-5 text-[var(--muted-foreground)]">
              Checks the hosted sync boundary without exposing Turso tokens or Better Auth secrets.
              Local Friday keeps working when this is off.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Badge
              variant="outline"
              className={
                syncHealth?.status === "ready"
                  ? "border-emerald-500/40 text-emerald-300"
                  : syncHealth?.status === "error"
                    ? "border-red-500/40 text-red-300"
                    : "border-[var(--border)]"
              }
            >
              {isCheckingSync
                ? "Checking"
                : syncHealth?.status === "ready"
                  ? "Ready"
                  : syncHealth?.status === "partial"
                    ? "Partial"
                    : syncHealth?.status === "local-only"
                      ? "Local only"
                      : syncHealth?.status === "error"
                        ? "Error"
                        : "Not checked"}
            </Badge>
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={isCheckingSync}
              onClick={() => void runSyncCheck()}
            >
              <RefreshCw size={14} className={isCheckingSync ? "animate-spin" : undefined} />
              Test sync
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={isSyncingWorkspace}
              onClick={() => void pushWorkspaceSnapshot()}
            >
              <Upload size={14} />
              Push
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={isSyncingWorkspace}
              onClick={() => void pullWorkspaceSnapshot()}
            >
              <Download size={14} />
              Pull
            </Button>
          </div>
        </div>
        <div className="mt-3 grid gap-2 text-xs text-[var(--muted-foreground)] md:grid-cols-3">
          <div className="rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3">
            <div className="ui-text-section-label ui-color-muted">Auth URL</div>
            <div className="mt-1 text-[var(--foreground)]">
              {syncHealth
                ? syncHealth.requirements.authUrlConfigured
                  ? "Configured"
                  : "Missing"
                : "Waiting"}
            </div>
          </div>
          <div className="rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3">
            <div className="ui-text-section-label ui-color-muted">Database</div>
            <div className="mt-1 text-[var(--foreground)]">
              {syncHealth
                ? syncHealth.requirements.databaseConfigured
                  ? "Configured"
                  : "Missing"
                : "Waiting"}
            </div>
          </div>
          <div className="rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3">
            <div className="ui-text-section-label ui-color-muted">Token</div>
            <div className="mt-1 text-[var(--foreground)]">
              {syncHealth
                ? syncHealth.requirements.tokenConfigured
                  ? "Configured"
                  : "Missing"
                : "Waiting"}
            </div>
          </div>
        </div>
        <div className="mt-3 rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3 text-xs leading-5 text-[var(--muted-foreground)]">
          {workspaceSyncMessage
            ? workspaceSyncMessage.text
            : syncHealth
            ? `${syncHealth.message} Checked in ${syncHealth.latencyMs} ms.`
            : "Run this before relying on account sync across machines."}
        </div>
      </div>

      <div className="rounded-lg border border-[var(--border)] bg-[var(--background)] p-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="min-w-0">
            <div className="flex items-center gap-2 text-sm font-semibold text-[var(--foreground)]">
              <Activity size={15} />
              Provider health
            </div>
            <p className="mt-2 max-w-2xl text-xs leading-5 text-[var(--muted-foreground)]">
              Checks the Friday chat route, provider env, stream format, and first model tokens.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Badge
              variant="outline"
              className={
                providerHealth?.status === "ready"
                  ? "border-emerald-500/40 text-emerald-300"
                  : providerHealth?.status === "error"
                    ? "border-red-500/40 text-red-300"
                    : "border-[var(--border)]"
              }
            >
              {isCheckingProvider
                ? "Checking"
                : providerHealth?.status === "ready"
                  ? "Ready"
                  : providerHealth?.status === "blocked"
                    ? "Blocked"
                    : providerHealth?.status === "error"
                      ? "Error"
                      : "Not checked"}
            </Badge>
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={providerCheckDisabled}
              onClick={() => void runProviderCheck()}
            >
              <RefreshCw size={14} className={isCheckingProvider ? "animate-spin" : undefined} />
              Test stream
            </Button>
          </div>
        </div>
        <div className="mt-3 grid gap-2 text-xs text-[var(--muted-foreground)] md:grid-cols-3">
          <div className="rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3">
            <div className="ui-text-section-label ui-color-muted">Route</div>
            <div className="mt-1 truncate text-[var(--foreground)]">
              {providerHealth?.route ?? "/api/friday/chat"}
            </div>
          </div>
          <div className="rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3">
            <div className="ui-text-section-label ui-color-muted">Latency</div>
            <div className="mt-1 text-[var(--foreground)]">
              {providerHealth ? `${providerHealth.latencyMs} ms` : "Waiting"}
            </div>
          </div>
          <div className="rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3">
            <div className="ui-text-section-label ui-color-muted">Model</div>
            <div className="mt-1 truncate text-[var(--foreground)]">
              {providerHealth?.modelKey ?? "Default provider"}
            </div>
          </div>
        </div>
        <div className="mt-3 rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3 text-xs leading-5 text-[var(--muted-foreground)]">
          {providerHealth
            ? `${providerHealth.message}${providerHealth.preview ? ` ${providerHealth.preview}` : ""}`
            : providerCheckDisabled
              ? "Enable Cloud AI and configure the provider env before testing a remote stream."
              : "Run a test before relying on cloud chat for Ask, Research, or Agents."}
        </div>
      </div>

      <div className="rounded-lg border border-[var(--border)] bg-[var(--background)] p-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="min-w-0">
            <div className="text-sm font-semibold text-[var(--foreground)]">
              Local workspace backup
            </div>
            <p className="mt-2 max-w-2xl text-xs leading-5 text-[var(--muted-foreground)]">
              Export or restore Friday projects, memories, chats, research, artifacts,
              automations, agents, and connector settings from this browser profile.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Badge
              variant="outline"
              className={
                backupMessage?.tone === "success"
                  ? "border-emerald-500/40 text-emerald-300"
                  : backupMessage?.tone === "error"
                    ? "border-red-500/40 text-red-300"
                    : "border-[var(--border)]"
              }
            >
              {backupMessage?.tone === "success"
                ? "Ready"
                : backupMessage?.tone === "error"
                  ? "Needs check"
                  : "Local"}
            </Badge>
            <Button
              type="button"
              size="sm"
              variant="outline"
              aria-label="Export Friday workspace backup"
              title="Export Friday workspace backup"
              onClick={exportWorkspaceBackup}
            >
              <Download size={14} />
              Export
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              aria-label="Import Friday workspace backup"
              title="Import Friday workspace backup"
              onClick={() => backupInputRef.current?.click()}
            >
              <Upload size={14} />
              Import
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              aria-label="Restore the saved Friday checkpoint"
              title={hasRestoreCheckpoint ? "Restore the saved Friday checkpoint" : "No restore checkpoint saved yet"}
              disabled={!hasRestoreCheckpoint}
              onClick={restoreSafetyCheckpoint}
            >
              <RotateCcw size={14} />
              Restore checkpoint
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              aria-label="Export the saved Friday checkpoint"
              title={hasRestoreCheckpoint ? "Export the saved Friday checkpoint" : "No restore checkpoint saved yet"}
              disabled={!hasRestoreCheckpoint}
              onClick={exportRestoreCheckpoint}
            >
              <Download size={14} />
              Export checkpoint
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              aria-label="Clear the saved Friday checkpoint"
              title={canClearRestoreCheckpoint ? "Clear the saved Friday checkpoint" : "No restore checkpoint saved yet"}
              disabled={!canClearRestoreCheckpoint}
              onClick={clearRestoreCheckpoint}
            >
              <Trash2 size={14} />
              Clear checkpoint
            </Button>
            <input
              ref={backupInputRef}
              type="file"
              accept="application/json,.json"
              className="hidden"
              onChange={(event) => {
                void importWorkspaceBackup(event.target.files?.[0]);
                event.currentTarget.value = "";
              }}
            />
          </div>
        </div>
        <div
          aria-live="polite"
          className="mt-3 rounded-md border border-[var(--border)] bg-[var(--secondary)] p-3 text-xs leading-5 text-[var(--muted-foreground)]"
          role="status"
        >
          {backupMessage?.text ??
            "Backups stay on your machine. Import only restores known Friday workspace keys."}
        </div>
        <div
          aria-live="polite"
          className={
            "mt-2 rounded-md border p-3 text-xs leading-5 " +
            (restoreCheckpoint.tone === "ready"
              ? "border-emerald-500/30 bg-emerald-500/5 text-emerald-200"
              : restoreCheckpoint.tone === "error"
                ? "border-red-500/30 bg-red-500/5 text-red-200"
                : "border-[var(--border)] bg-[var(--secondary)] text-[var(--muted-foreground)]")
          }
          role={restoreCheckpoint.tone === "error" ? "alert" : "status"}
        >
          {restoreCheckpoint.text}
        </div>
      </div>

      <div className="grid gap-3 md:grid-cols-2">
        {CONNECTOR_OPTIONS.map(([key, title, body]) => {
          const enabled = settings[key];
          const blocked = key === "aiGateway" && !cloudEnvEnabled;
          return (
            <button
              key={key}
              type="button"
              disabled={blocked}
              onClick={() => updateSettings({ [key]: !enabled } as Partial<ConnectorSettings>)}
              className="rounded-lg border border-[var(--border)] bg-[var(--background)] p-4 text-left transition-colors hover:bg-[var(--secondary)] disabled:cursor-not-allowed disabled:opacity-55"
            >
              <div className="flex items-center justify-between gap-3">
                <div className="flex items-center gap-2 text-sm font-semibold text-[var(--foreground)]">
                  <Link2 size={15} />
                  {title}
                </div>
                <Badge variant="outline" className="border-[var(--border)]">
                  {blocked ? "Needs env" : enabled ? "On" : "Off"}
                </Badge>
              </div>
              <p className="mt-2 text-xs leading-5 text-[var(--muted-foreground)]">{body}</p>
            </button>
          );
        })}
      </div>
    </div>
  );
}
