import { useState } from "react";
import { useLingui } from "@lingui/react/macro";
import { Check, Copy, Link2, Trash2 } from "lucide-react";
import DotMatrix from "../../../shared/ui/DotMatrix";
import type { FlowFetchLink } from "../../../types";
import { useCopyFlowFetchLink, useDeleteFlowFetchLink, useFlowFetchLinks } from "../queries";

const formatSeenTime = (value: string) => {
  const date = new Date(value);
  const deltaMs = Date.now() - date.getTime();
  if (Number.isNaN(deltaMs)) return "";
  const minutes = Math.max(0, Math.round(deltaMs / 60_000));
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return date.toLocaleDateString([], { month: "short", day: "numeric" });
};

const hostFromUrl = (url: string) => {
  try {
    return new URL(url).host.replace(/^www\./, "");
  } catch {
    return url;
  }
};

function LinkRow({
  link,
  copied,
  onCopy,
  onDelete,
}: {
  link: FlowFetchLink;
  copied: boolean;
  onCopy: (link: FlowFetchLink) => void;
  onDelete: (id: string) => void;
}) {
  return (
    <article className="group grid grid-cols-[1fr_auto] items-center gap-3 rounded-lg border border-border-primary bg-surface-surface p-3 transition-colors hover:border-border-secondary hover:bg-[var(--surface-interactive)]">
      <div className="min-w-0">
        <div className="flex items-center gap-2">
          <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md border border-border-primary bg-surface-elevated">
            <Link2 size={14} className="ui-color-muted" />
          </div>
          <div className="min-w-0">
            <div className="truncate ui-text-body-sm-strong ui-color-primary">{link.label}</div>
            <div className="truncate ui-text-meta ui-color-muted">{hostFromUrl(link.url)}</div>
          </div>
        </div>
        <div className="mt-2 truncate pl-9 ui-text-meta ui-color-disabled">{link.url}</div>
      </div>
      <div className="flex shrink-0 items-center gap-1">
        <span className="mr-2 hidden ui-text-micro ui-color-disabled sm:inline">
          {formatSeenTime(link.last_seen_at)}
        </span>
        <button
          type="button"
          onClick={() => onCopy(link)}
          className="ui-button-ghost h-8 w-8"
          aria-label="Copy link"
        >
          {copied ? <Check size={15} /> : <Copy size={15} />}
        </button>
        <button
          type="button"
          onClick={() => onDelete(link.id)}
          className="ui-button-ghost h-8 w-8"
          aria-label="Delete link"
        >
          <Trash2 size={15} />
        </button>
      </div>
    </article>
  );
}

export default function FlowFetchView({ isActive = true }: { isActive?: boolean }) {
  const { t } = useLingui();
  const linksQuery = useFlowFetchLinks(50, isActive);
  const copyMutation = useCopyFlowFetchLink();
  const deleteMutation = useDeleteFlowFetchLink();
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const links = linksQuery.data ?? [];

  const copyLink = async (link: FlowFetchLink) => {
    await copyMutation.mutateAsync(link.url);
    setCopiedId(link.id);
    window.setTimeout(() => setCopiedId(null), 1400);
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="mb-5">
        <div className="ui-text-section-label ui-color-muted">
          {t({ id: "flow_fetch.eyebrow", message: "Flow Fetch" })}
        </div>
        <h1 className="mt-1 ui-text-title font-medium ui-color-primary">
          {t({ id: "flow_fetch.title", message: "Recent copied links" })}
        </h1>
        <p className="mt-1 max-w-xl ui-text-body-sm ui-color-muted">
          {t({
            id: "flow_fetch.subtitle",
            message:
              "Flow watches for copied URLs locally, keeps them for 14 days, and lets you copy them back fast.",
          })}
        </p>
      </div>

      <section className="min-h-0 flex-1 rounded-lg border border-border-primary bg-surface-surface/60 p-2">
        {linksQuery.isLoading ? (
          <div className="flex h-52 items-center justify-center">
            <DotMatrix cols={10} rows={4} dotSize={2} gap={5} />
          </div>
        ) : links.length === 0 ? (
          <div className="flex h-72 flex-col items-center justify-center px-6 text-center">
            <div className="flex h-11 w-11 items-center justify-center rounded-full border border-border-primary bg-surface-elevated">
              <Link2 size={20} className="ui-color-muted" />
            </div>
            <div className="mt-4 ui-text-body-sm-strong ui-color-secondary">
              {t({ id: "flow_fetch.empty.title", message: "No copied links yet" })}
            </div>
            <div className="mt-1 max-w-sm ui-text-meta ui-color-muted">
              {t({
                id: "flow_fetch.empty.body",
                message: "Copy any http or https link and it will appear here automatically.",
              })}
            </div>
          </div>
        ) : (
          <div className="max-h-full space-y-2 overflow-y-auto">
            {links.map((link) => (
              <LinkRow
                key={link.id}
                link={link}
                copied={copiedId === link.id}
                onCopy={copyLink}
                onDelete={(id) => deleteMutation.mutate(id)}
              />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
