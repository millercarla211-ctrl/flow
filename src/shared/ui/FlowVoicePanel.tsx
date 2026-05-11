import { Mic2, Sparkles } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { LiveWaveform } from "@/components/ui/live-waveform";
import { ShimmeringText } from "@/components/ui/shimmering-text";

export function FlowVoicePanel({
  modeLabel,
  modelLabel = "Parakeet TDT INT8",
}: {
  modeLabel: string;
  modelLabel?: string;
}) {
  return (
    <Card className="flow-voice-panel mb-5 overflow-hidden">
      <CardContent className="flex items-center gap-4 p-3">
        <div className="flex size-10 items-center justify-center rounded-md border border-[var(--border)] bg-[var(--secondary)]">
          <Mic2 size={18} className="text-[var(--foreground)]" />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <ShimmeringText
              text="Flow is listening locally"
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
          </div>
          <div className="mt-1 flex items-center gap-1.5 text-[11px] text-[var(--muted-foreground)]">
            <Sparkles size={12} />
            <span className="truncate">{modelLabel}</span>
          </div>
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
      </CardContent>
    </Card>
  );
}
