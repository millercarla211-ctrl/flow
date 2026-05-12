import { Keyboard, Mic2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { LiveWaveform } from "@/components/ui/live-waveform";
import { ShimmeringText } from "@/components/ui/shimmering-text";

export function FlowVoicePanel({
  modeLabel,
  headline = "Free, unlimited, fast local dictation",
  hint,
  badges = ["Completely free", "Unlimited"],
}: {
  modeLabel: string;
  headline?: string;
  hint?: string;
  badges?: string[];
}) {
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
              {badges.map((badge) => (
                <Badge
                  key={badge}
                  variant="outline"
                  className="border-[var(--border)] bg-[var(--surface-interactive)] text-[var(--muted-foreground)]"
                >
                  {badge}
                </Badge>
              ))}
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
      </CardContent>
    </Card>
  );
}
