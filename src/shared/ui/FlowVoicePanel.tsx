import type React from "react";
import {
  AlertTriangle,
  ClipboardCheck,
  Cpu,
  HardDrive,
  Keyboard,
  Mic2,
  ShieldCheck,
  Sparkles,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { LiveWaveform } from "@/components/ui/live-waveform";
import { ShimmeringText } from "@/components/ui/shimmering-text";

type FlowReadinessState = "ready" | "warning" | "pending";

type ReadinessItem = {
  label: string;
  value: string;
  state: FlowReadinessState;
  icon: React.ReactNode;
  actionLabel?: string;
  onAction?: () => void;
};

export function FlowVoicePanel({
  modeLabel,
  headline = "Flow is listening locally",
  hint,
  modelLabel,
  modelReady = false,
  microphoneLabel,
  microphoneReady = false,
  shortcutLabel,
  shortcutReady = false,
  pasteLabel,
  pasteReady = true,
  aiLabel,
  aiReady = true,
  storageLabel,
  readinessPercent = 0,
  onOpenGeneralSettings,
  onOpenModels,
  onOpenAppSettings,
}: {
  modeLabel: string;
  headline?: string;
  hint?: string;
  modelLabel: string;
  modelReady?: boolean;
  microphoneLabel: string;
  microphoneReady?: boolean;
  shortcutLabel: string;
  shortcutReady?: boolean;
  pasteLabel: string;
  pasteReady?: boolean;
  aiLabel: string;
  aiReady?: boolean;
  storageLabel: string;
  readinessPercent?: number;
  onOpenGeneralSettings?: () => void;
  onOpenModels?: () => void;
  onOpenAppSettings?: () => void;
}) {
  const readinessState: FlowReadinessState =
    readinessPercent >= 100 ? "ready" : readinessPercent >= 60 ? "warning" : "pending";
  const readinessLabel =
    readinessState === "ready"
      ? "Ready"
      : readinessState === "warning"
        ? "Needs attention"
        : "Setup";
  const statusItems: ReadinessItem[] = [
    {
      label: "Model",
      value: modelLabel,
      state: modelReady ? "ready" : "warning",
      icon: <Cpu size={13} aria-hidden="true" />,
      actionLabel: modelReady ? undefined : "Models",
      onAction: modelReady ? undefined : onOpenModels,
    },
    {
      label: "Mic",
      value: microphoneLabel,
      state: microphoneReady ? "ready" : "warning",
      icon: <Mic2 size={13} aria-hidden="true" />,
      actionLabel: microphoneReady ? undefined : "Settings",
      onAction: microphoneReady ? undefined : onOpenGeneralSettings,
    },
    {
      label: "Shortcut",
      value: shortcutLabel,
      state: shortcutReady ? "ready" : "warning",
      icon: <Keyboard size={13} aria-hidden="true" />,
      actionLabel: shortcutReady ? undefined : "Settings",
      onAction: shortcutReady ? undefined : onOpenGeneralSettings,
    },
    {
      label: "Paste",
      value: pasteLabel,
      state: pasteReady ? "ready" : "warning",
      icon: <ClipboardCheck size={13} aria-hidden="true" />,
      actionLabel: pasteReady ? undefined : "Permissions",
      onAction: pasteReady ? undefined : onOpenAppSettings,
    },
    {
      label: "AI",
      value: aiLabel,
      state: aiReady ? "ready" : "pending",
      icon: <Sparkles size={13} aria-hidden="true" />,
      actionLabel: aiReady ? undefined : "Models",
      onAction: aiReady ? undefined : onOpenModels,
    },
  ];

  return (
    <Card className="flow-voice-panel mb-5 overflow-hidden">
      <CardContent className="space-y-3 p-3">
        <div className="flex items-center gap-4">
          <div className="flex size-10 items-center justify-center rounded-md border border-[var(--border)] bg-[var(--secondary)]">
            <Mic2 size={18} className="text-[var(--foreground)]" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2">
              <ShimmeringText
                text={headline}
                className="text-xs font-semibold"
                color="var(--muted-foreground)"
                shimmerColor="var(--foreground)"
              />
              <Badge
                variant="outline"
                className="border-[var(--border)] bg-[var(--secondary)] text-[var(--foreground)]"
              >
                {modeLabel}
              </Badge>
              <Badge
                variant="outline"
                className="border-[var(--border)] bg-[var(--surface-interactive)] text-[var(--muted-foreground)]"
              >
                {readinessPercent}% {readinessLabel}
              </Badge>
            </div>
            <div className="mt-1 flex min-w-0 items-center gap-1.5 text-[11px] text-[var(--muted-foreground)]">
              <HardDrive size={12} className="shrink-0" />
              <span className="truncate">{storageLabel}</span>
            </div>
            {hint && (
              <div className="mt-1 flex min-w-0 items-center gap-1.5 text-[11px] text-[var(--foreground)]">
                <Keyboard size={12} className="shrink-0 text-[var(--muted-foreground)]" />
                <span className="truncate">{hint}</span>
              </div>
            )}
          </div>
          <LiveWaveform
            processing
            active={false}
            height={40}
            barWidth={3}
            barGap={3}
            barRadius={2}
            className="hidden w-32 text-[var(--foreground)] sm:block"
            aria-hidden="true"
          />
        </div>

        <div className="grid grid-cols-2 gap-1.5 sm:grid-cols-5">
          {statusItems.map((item) => (
            <div
              key={item.label}
              className="min-w-0 rounded-md border border-[var(--border)] bg-[var(--secondary)]/70 px-2 py-1.5"
            >
              <div className="flex items-center justify-between gap-1.5">
                <div className="flex min-w-0 items-center gap-1.5 text-[10px] font-medium uppercase text-[var(--muted-foreground)]">
                  <span
                    className={
                      item.state === "ready"
                        ? "text-[var(--foreground)]"
                        : item.state === "warning"
                          ? "text-[var(--color-warning-strong)]"
                          : "text-[var(--muted-foreground)]"
                    }
                  >
                    {item.state === "ready" ? (
                      <ShieldCheck size={13} aria-hidden="true" />
                    ) : item.state === "warning" ? (
                      <AlertTriangle size={13} aria-hidden="true" />
                    ) : (
                      item.icon
                    )}
                  </span>
                  <span className="truncate">{item.label}</span>
                </div>
                {item.actionLabel && item.onAction && (
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={item.onAction}
                    className="h-5 rounded px-1.5 text-[10px] text-[var(--foreground)] hover:bg-[var(--surface-interactive)]"
                  >
                    {item.actionLabel}
                  </Button>
                )}
              </div>
              <p className="mt-1 truncate text-[11px] text-[var(--foreground)]" title={item.value}>
                {item.value}
              </p>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
