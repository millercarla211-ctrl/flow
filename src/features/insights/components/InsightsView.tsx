import type { ReactNode } from "react";
import { useLingui } from "@lingui/react/macro";
import { motion } from "framer-motion";
import {
  Activity,
  BarChart3,
  CheckCircle2,
  Clock3,
  Flame,
  Gauge,
  Mic2,
  Sparkles,
  Timer,
  Zap,
} from "lucide-react";
import DotMatrix from "../../../shared/ui/DotMatrix";
import type { InsightBreakdown, InsightsSummary } from "../../../types";
import { useInsights } from "../queries";

const numberFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });

const compactFormatter = new Intl.NumberFormat(undefined, {
  notation: "compact",
  maximumFractionDigits: 1,
});

function formatNumber(value: number) {
  return numberFormatter.format(Math.round(value || 0));
}

function formatCompact(value: number) {
  return compactFormatter.format(Math.round(value || 0));
}

function formatDuration(seconds: number) {
  const minutes = Math.round((seconds || 0) / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const remaining = minutes % 60;
  return remaining > 0 ? `${hours}h ${remaining}m` : `${hours}h`;
}

function formatMilliseconds(milliseconds: number) {
  if (!milliseconds || milliseconds <= 0) return "n/a";
  if (milliseconds < 1000) return `${Math.round(milliseconds)}ms`;
  if (milliseconds < 10000) return `${(milliseconds / 1000).toFixed(1)}s`;
  return `${Math.round(milliseconds / 1000)}s`;
}

function MetricTile({
  icon,
  label,
  value,
  detail,
}: {
  icon: ReactNode;
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <article className="rounded-lg border border-border-primary bg-surface-surface p-4">
      <div className="flex items-center justify-between gap-3">
        <div className="ui-text-meta-strong ui-color-muted">{label}</div>
        <div className="flex h-8 w-8 items-center justify-center rounded-full border border-border-primary bg-surface-elevated ui-color-muted">
          {icon}
        </div>
      </div>
      <div className="mt-4 ui-text-title font-medium ui-color-primary">{value}</div>
      <div className="mt-1 ui-text-meta ui-color-disabled">{detail}</div>
    </article>
  );
}

function PerformanceStat({
  icon,
  label,
  value,
  detail,
}: {
  icon: ReactNode;
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <div className="min-w-0 rounded-md border border-border-primary bg-surface-elevated px-3 py-3">
      <div className="flex items-center justify-between gap-3">
        <div className="truncate ui-text-meta-strong ui-color-muted">{label}</div>
        <div className="shrink-0 ui-color-disabled">{icon}</div>
      </div>
      <div className="mt-2 ui-text-body-sm-strong ui-color-primary">{value}</div>
      <div className="mt-0.5 truncate ui-text-micro ui-color-disabled">{detail}</div>
    </div>
  );
}

function BreakdownList({
  title,
  items,
  empty,
}: {
  title: string;
  items: InsightBreakdown[];
  empty: string;
}) {
  const maxWords = Math.max(1, ...items.map((item) => item.words));

  return (
    <section className="rounded-lg border border-border-primary bg-surface-surface p-4">
      <div className="mb-4 ui-text-body-sm-strong ui-color-primary">{title}</div>
      {items.length === 0 ? (
        <div className="flex h-28 items-center justify-center rounded-md border border-border-primary bg-surface-elevated ui-text-meta ui-color-muted">
          {empty}
        </div>
      ) : (
        <div className="space-y-3">
          {items.map((item) => (
            <div key={item.label}>
              <div className="mb-1 flex items-center justify-between gap-3">
                <span className="truncate ui-text-meta-strong ui-color-secondary">
                  {item.label}
                </span>
                <span className="shrink-0 ui-text-micro ui-color-disabled">
                  {formatCompact(item.words)} words
                </span>
              </div>
              <div className="h-1.5 overflow-hidden rounded-full bg-surface-elevated">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${Math.max(7, (item.words / maxWords) * 100)}%` }}
                  transition={{ type: "spring", stiffness: 170, damping: 24 }}
                  className="h-full rounded-full bg-[var(--color-content-primary)]"
                />
              </div>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

function ActivityChart({ summary }: { summary: InsightsSummary }) {
  const maxWords = Math.max(1, ...summary.daily.map((day) => day.words));

  return (
    <section className="rounded-lg border border-border-primary bg-surface-surface p-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <div className="ui-text-body-sm-strong ui-color-primary">Activity</div>
          <div className="mt-1 ui-text-meta ui-color-muted">
            {formatNumber(summary.average_words_per_day)} words per day average
          </div>
        </div>
        <div className="rounded-full border border-border-primary bg-surface-elevated px-3 py-1 ui-text-micro ui-color-muted">
          Last {summary.days} days
        </div>
      </div>

      <div className="mt-5 flex h-40 items-end gap-1.5">
        {summary.daily.map((day, index) => {
          const height = day.words === 0 ? 4 : Math.max(10, (day.words / maxWords) * 100);
          const showLabel = index === 0 || index === summary.daily.length - 1 || index % 7 === 0;

          return (
            <div key={day.date} className="flex min-w-0 flex-1 flex-col items-center gap-2">
              <div className="flex h-32 w-full items-end">
                <motion.div
                  initial={{ height: 4, opacity: 0.6 }}
                  animate={{ height: `${height}%`, opacity: day.words > 0 ? 1 : 0.45 }}
                  transition={{ type: "spring", stiffness: 140, damping: 22, delay: index * 0.01 }}
                  title={`${day.label}: ${formatNumber(day.words)} words`}
                  className="mx-auto w-full max-w-5 rounded-t-full rounded-b-sm border border-border-secondary bg-[var(--surface-interactive-strong)]"
                />
              </div>
              <div className="h-3 ui-text-micro ui-color-disabled">
                {showLabel ? day.label : ""}
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}

function EmptyInsights() {
  return (
    <div className="flex h-72 flex-col items-center justify-center rounded-lg border border-border-primary bg-surface-surface px-6 text-center">
      <div className="flex h-11 w-11 items-center justify-center rounded-full border border-border-primary bg-surface-elevated">
        <BarChart3 size={20} className="ui-color-muted" />
      </div>
      <div className="mt-4 ui-text-body-sm-strong ui-color-secondary">No insight history yet</div>
      <div className="mt-1 max-w-sm ui-text-meta ui-color-muted">
        Dictate a few notes and Flow will turn your local transcript history into streaks, pace, and
        model usage.
      </div>
    </div>
  );
}

export default function InsightsView({ isActive = true }: { isActive?: boolean }) {
  const { t } = useLingui();
  const insightsQuery = useInsights(30, isActive);
  const summary = insightsQuery.data;

  const localDetail = summary ? `${Math.round(summary.local_percent)}% local engine usage` : "";
  const pasteSuccessValue = summary?.auto_paste_attempts
    ? `${Math.round(summary.auto_paste_success_percent)}%`
    : "n/a";

  if (insightsQuery.isLoading) {
    return (
      <div className="flex h-full min-h-0 flex-col">
        <div className="mb-5">
          <div className="ui-text-section-label ui-color-muted">
            {t({ id: "insights.eyebrow", message: "Insights" })}
          </div>
          <h1 className="mt-1 ui-text-title font-medium ui-color-primary">
            {t({ id: "insights.title", message: "Your voice activity" })}
          </h1>
        </div>
        <div className="flex h-72 items-center justify-center rounded-lg border border-border-primary bg-surface-surface">
          <DotMatrix cols={12} rows={5} dotSize={2} gap={5} animated />
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col overflow-y-auto pb-6">
      <div className="mb-5">
        <div className="ui-text-section-label ui-color-muted">
          {t({ id: "insights.eyebrow", message: "Insights" })}
        </div>
        <h1 className="mt-1 ui-text-title font-medium ui-color-primary">
          {t({ id: "insights.title", message: "Your voice activity" })}
        </h1>
        <p className="mt-1 max-w-xl ui-text-body-sm ui-color-muted">
          {t({
            id: "insights.subtitle",
            message:
              "Track local words, streaks, speech pace, cleanup usage, and which models are doing the work.",
          })}
        </p>
      </div>

      {insightsQuery.error && (
        <div className="mb-4 rounded-lg border border-[var(--color-error)]/30 bg-[var(--color-error)]/10 px-4 py-3 ui-text-body-sm ui-color-error-tint">
          {insightsQuery.error instanceof Error
            ? insightsQuery.error.message
            : String(insightsQuery.error)}
        </div>
      )}

      {!summary || summary.total_transcriptions === 0 ? (
        <EmptyInsights />
      ) : (
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-3 xl:grid-cols-4">
            <MetricTile
              icon={<Mic2 size={16} />}
              label="Total words"
              value={formatCompact(summary.total_words)}
              detail={`${formatNumber(summary.total_transcriptions)} local transcripts`}
            />
            <MetricTile
              icon={<Flame size={16} />}
              label="Current streak"
              value={`${formatNumber(summary.current_streak_days)} day${
                summary.current_streak_days === 1 ? "" : "s"
              }`}
              detail={`${formatNumber(summary.words_today)} words today`}
            />
            <MetricTile
              icon={<Gauge size={16} />}
              label="Speech pace"
              value={`${formatNumber(summary.average_words_per_minute)} wpm`}
              detail={`${formatDuration(summary.total_audio_seconds)} captured audio`}
            />
            <MetricTile
              icon={<Sparkles size={16} />}
              label="Local first"
              value={`${Math.round(summary.local_percent)}%`}
              detail={localDetail}
            />
          </div>

          <ActivityChart summary={summary} />

          <section className="rounded-lg border border-border-primary bg-surface-surface p-4">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div>
                <div className="ui-text-body-sm-strong ui-color-primary">Dictation performance</div>
                <div className="mt-1 ui-text-meta ui-color-muted">
                  Timing from the last {formatNumber(summary.timed_transcriptions)} completed runs
                </div>
              </div>
              <div className="rounded-full border border-border-primary bg-surface-elevated px-3 py-1 ui-text-micro ui-color-muted">
                {formatNumber(summary.auto_paste_attempts)} paste attempts
              </div>
            </div>

            <div className="mt-4 grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
              <PerformanceStat
                icon={<Zap size={14} />}
                label="Recognition"
                value={formatMilliseconds(summary.average_stt_elapsed_ms)}
                detail="average STT time"
              />
              <PerformanceStat
                icon={<Timer size={14} />}
                label="End-to-end"
                value={formatMilliseconds(summary.average_total_elapsed_ms)}
                detail="recording to ready"
              />
              <PerformanceStat
                icon={<CheckCircle2 size={14} />}
                label="Auto-paste"
                value={pasteSuccessValue}
                detail="successful inserts"
              />
              <PerformanceStat
                icon={<Clock3 size={14} />}
                label="Cleanup"
                value={formatMilliseconds(summary.average_cleanup_elapsed_ms)}
                detail={`${formatNumber(summary.paste_fallback_count)} paste fallbacks`}
              />
            </div>
          </section>

          <div className="grid gap-4 lg:grid-cols-2">
            <BreakdownList title="Top modes" items={summary.top_modes} empty="No modes yet" />
            <BreakdownList
              title="Speech models"
              items={summary.top_models}
              empty="No model usage yet"
            />
          </div>

          <section className="grid gap-3 rounded-lg border border-border-primary bg-surface-surface p-4 sm:grid-cols-3">
            <div>
              <div className="flex items-center gap-2 ui-text-meta-strong ui-color-muted">
                <Activity size={14} />
                This week
              </div>
              <div className="mt-2 ui-text-title font-medium ui-color-primary">
                {formatCompact(summary.words_this_week)}
              </div>
              <div className="mt-1 ui-text-meta ui-color-disabled">words since Monday</div>
            </div>
            <div>
              <div className="flex items-center gap-2 ui-text-meta-strong ui-color-muted">
                <BarChart3 size={14} />
                Best day
              </div>
              <div className="mt-2 ui-text-title font-medium ui-color-primary">
                {formatCompact(summary.best_day_words)}
              </div>
              <div className="mt-1 ui-text-meta ui-color-disabled">
                {summary.best_day_label || "No active day yet"}
              </div>
            </div>
            <div>
              <div className="flex items-center gap-2 ui-text-meta-strong ui-color-muted">
                <Clock3 size={14} />
                Cleanup
              </div>
              <div className="mt-2 ui-text-title font-medium ui-color-primary">
                {Math.round(summary.cleanup_percent)}%
              </div>
              <div className="mt-1 ui-text-meta ui-color-disabled">LLM-polished transcripts</div>
            </div>
          </section>
        </div>
      )}
    </div>
  );
}
