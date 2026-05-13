import { motion } from "framer-motion";
import { ArrowRight, Cpu, LockKeyhole, Route, Sparkles } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { FRIDAY_LOCAL_MODELS } from "@/features/ai";
import { FRIDAY_DASHBOARD_ORDER, FRIDAY_FEATURE_SPECS, type FridayAssistantView } from "../pageData";

export function FridayDashboard({
  onOpenView,
}: {
  onOpenView: (view: FridayAssistantView | "voice") => void;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto pb-6">
      <motion.section
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.24, ease: "easeOut" }}
        className="rounded-xl border border-[var(--border)] bg-[var(--card)] p-5"
      >
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="max-w-2xl">
            <div className="ui-text-section-label ui-color-muted">Friday workspace</div>
            <h1 className="mt-3 text-3xl font-semibold tracking-tight text-[var(--foreground)]">
              Your local-first AI workspace
            </h1>
            <p className="mt-3 text-sm leading-6 text-[var(--muted-foreground)]">
              Ask, research, build artifacts, run agents, manage projects, and keep the current
              WhisperFlow Beater voice stack available as a focused tool.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Badge variant="outline" className="border-[var(--border)] bg-[var(--secondary)]">
              Local by default
            </Badge>
            <Badge variant="outline" className="border-[var(--border)] bg-[var(--secondary)]">
              Gateway optional
            </Badge>
          </div>
        </div>
        <div className="mt-5 flex flex-wrap gap-2">
          <Button type="button" onClick={() => onOpenView("ask")}>
            Ask Friday
            <ArrowRight size={16} />
          </Button>
          <Button type="button" variant="outline" onClick={() => onOpenView("voice")}>
            Open Voice
          </Button>
        </div>
      </motion.section>

      <div className="grid gap-3 lg:grid-cols-3">
        {[
          {
            icon: <LockKeyhole size={18} />,
            label: "Privacy boundary",
            value: "No silent cloud calls",
          },
          {
            icon: <Route size={18} />,
            label: "Routing policy",
            value: "Helper, tool, daily, backup",
          },
          {
            icon: <Cpu size={18} />,
            label: "Daily model",
            value: "Qwen3.5 4B Revised",
          },
        ].map((item) => (
          <Card key={item.label} className="py-0">
            <CardContent className="flex items-center gap-3 p-4">
              <div className="flex size-9 items-center justify-center rounded-md border border-[var(--border)] bg-[var(--secondary)]">
                {item.icon}
              </div>
              <div className="min-w-0">
                <div className="text-[11px] uppercase tracking-[0.16em] text-[var(--muted-foreground)]">
                  {item.label}
                </div>
                <div className="mt-1 truncate text-sm font-semibold text-[var(--foreground)]">
                  {item.value}
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <section className="grid gap-3 lg:grid-cols-2">
        {FRIDAY_DASHBOARD_ORDER.map((id) => {
          const spec = FRIDAY_FEATURE_SPECS[id];
          return (
            <button
              key={spec.id}
              type="button"
              onClick={() => onOpenView(spec.id)}
              className="group rounded-xl border border-[var(--border)] bg-[var(--card)] p-4 text-left transition-colors hover:border-[var(--border-hover)] hover:bg-[var(--secondary)]"
            >
              <div className="flex items-start justify-between gap-3">
                <div>
                  <div className="ui-text-section-label ui-color-muted">{spec.eyebrow}</div>
                  <div className="mt-2 text-base font-semibold text-[var(--foreground)]">
                    {spec.title}
                  </div>
                </div>
                <Sparkles
                  size={16}
                  className="mt-1 text-[var(--muted-foreground)] transition-colors group-hover:text-[var(--foreground)]"
                />
              </div>
              <p className="mt-3 text-sm leading-6 text-[var(--muted-foreground)]">
                {spec.summary}
              </p>
              <div className="mt-4 flex items-center justify-between">
                <Badge variant="outline" className="border-[var(--border)]">
                  {spec.status}
                </Badge>
                <span className="text-xs text-[var(--foreground)]">{spec.primaryAction}</span>
              </div>
            </button>
          );
        })}
      </section>

      <Card className="py-0">
        <CardContent className="p-4">
          <div className="ui-text-section-label ui-color-muted">Local model policy</div>
          <div className="mt-3 grid gap-2 md:grid-cols-2">
            {FRIDAY_LOCAL_MODELS.map((model) => (
              <div
                key={model.key}
                className="rounded-lg border border-[var(--border)] bg-[var(--background)] p-3"
              >
                <div className="flex items-center justify-between gap-2">
                  <div className="text-sm font-semibold text-[var(--foreground)]">{model.label}</div>
                  <Badge variant="outline" className="border-[var(--border)] text-[10px]">
                    {model.speedLabel}
                  </Badge>
                </div>
                <p className="mt-2 text-xs leading-5 text-[var(--muted-foreground)]">
                  {model.description}
                </p>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
