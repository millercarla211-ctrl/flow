import { Link2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { useLocalSettings } from "../../hooks/useLocalPersistence";
import { DEFAULT_CONNECTORS, STORAGE_KEYS, type ConnectorSettings } from "./types";

const CONNECTOR_OPTIONS: Array<[keyof ConnectorSettings, string, string]> = [
  ["localFiles", "Local files", "Allow Friday to use local project files you add."],
  ["webSearch", "Web search", "Disabled until you explicitly enable remote lookup."],
  ["aiGateway", "AI Gateway", "Optional Vercel provider routing for cloud models."],
  ["mcpConnectors", "MCP connectors", "Future app connectors stay off until configured."],
];

export function ConnectorsWorkspace() {
  const { settings, updateSettings } = useLocalSettings<ConnectorSettings>(
    STORAGE_KEYS.connectors,
    DEFAULT_CONNECTORS,
  );

  return (
    <div className="grid gap-3 md:grid-cols-2">
      {CONNECTOR_OPTIONS.map(([key, title, body]) => {
        const enabled = settings[key];
        return (
          <button
            key={key}
            type="button"
            onClick={() => updateSettings({ [key]: !enabled } as Partial<ConnectorSettings>)}
            className="rounded-lg border border-[var(--border)] bg-[var(--background)] p-4 text-left transition-colors hover:bg-[var(--secondary)]"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="flex items-center gap-2 text-sm font-semibold text-[var(--foreground)]">
                <Link2 size={15} />
                {title}
              </div>
              <Badge variant="outline" className="border-[var(--border)]">
                {enabled ? "On" : "Off"}
              </Badge>
            </div>
            <p className="mt-2 text-xs leading-5 text-[var(--muted-foreground)]">{body}</p>
          </button>
        );
      })}
    </div>
  );
}
