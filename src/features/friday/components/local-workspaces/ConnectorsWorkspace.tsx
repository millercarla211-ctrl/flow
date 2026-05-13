import { Activity, Link2, RefreshCw } from "lucide-react";
import { useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useLocalSettings } from "../../hooks/useLocalPersistence";
import { checkFridayProviderHealth, type ProviderHealthResult } from "../../utils/providerHealth";
import { DEFAULT_CONNECTORS, STORAGE_KEYS, type ConnectorSettings } from "./types";

const CONNECTOR_OPTIONS: Array<[keyof ConnectorSettings, string, string]> = [
  ["localFiles", "Local files", "Allow Friday to use local project files you add."],
  ["webSearch", "Web search", "Disabled until you explicitly enable remote lookup."],
  ["aiGateway", "Cloud AI", "Use configured online providers such as Groq or AI Gateway."],
  ["mcpConnectors", "MCP connectors", "Future app connectors stay off until configured."],
];

export function ConnectorsWorkspace() {
  const [providerHealth, setProviderHealth] = useState<ProviderHealthResult | null>(null);
  const [isCheckingProvider, setIsCheckingProvider] = useState(false);
  const { settings, updateSettings } = useLocalSettings<ConnectorSettings>(
    STORAGE_KEYS.connectors,
    DEFAULT_CONNECTORS,
  );
  const cloudEnvEnabled =
    process.env.NEXT_PUBLIC_FRIDAY_ENABLE_CLOUD_AI === "true" ||
    process.env.NEXT_PUBLIC_FRIDAY_ENABLE_GROQ_AI === "true";
  const providerCheckDisabled = !settings.aiGateway || !cloudEnvEnabled || isCheckingProvider;

  const runProviderCheck = async () => {
    if (providerCheckDisabled) return;
    setIsCheckingProvider(true);
    const result = await checkFridayProviderHealth();
    setProviderHealth(result);
    setIsCheckingProvider(false);
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
