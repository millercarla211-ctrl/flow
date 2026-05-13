import { Link2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { useLocalSettings } from "../../hooks/useLocalPersistence";
import { DEFAULT_CONNECTORS, STORAGE_KEYS, type ConnectorSettings } from "./types";

const CONNECTOR_OPTIONS: Array<[keyof ConnectorSettings, string, string]> = [
  ["localFiles", "Local files", "Allow Friday to use local project files you add."],
  ["webSearch", "Web search", "Disabled until you explicitly enable remote lookup."],
  ["aiGateway", "Cloud AI", "Use configured online providers such as Groq or AI Gateway."],
  ["mcpConnectors", "MCP connectors", "Future app connectors stay off until configured."],
];

export function ConnectorsWorkspace() {
  const { settings, updateSettings } = useLocalSettings<ConnectorSettings>(
    STORAGE_KEYS.connectors,
    DEFAULT_CONNECTORS,
  );
  const cloudEnvEnabled =
    process.env.NEXT_PUBLIC_FRIDAY_ENABLE_CLOUD_AI === "true" ||
    process.env.NEXT_PUBLIC_FRIDAY_ENABLE_GROQ_AI === "true";

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
