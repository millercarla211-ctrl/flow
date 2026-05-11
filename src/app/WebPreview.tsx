import { Download, HardDrive, Mic2, ShieldCheck, Zap } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { LiveWaveform } from "@/components/ui/live-waveform";
import { ShimmeringText } from "@/components/ui/shimmering-text";
import { FlowLogo } from "@/shared/ui/FlowLogo";

const DETAILS = [
  {
    icon: HardDrive,
    label: "Local models",
    value: "Parakeet + Whisper Turbo",
  },
  {
    icon: Zap,
    label: "Startup",
    value: "STT warmup enabled",
  },
  {
    icon: ShieldCheck,
    label: "Storage",
    value: "G-drive first",
  },
];

export default function WebPreview() {
  return (
    <main className="min-h-screen bg-background text-foreground">
      <div className="mx-auto flex min-h-screen w-full max-w-5xl flex-col px-6 py-8">
        <header className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <FlowLogo size="md" />
            <span className="flow-brand-word text-2xl text-foreground">Flow</span>
          </div>
          <Badge variant="outline" className="border-border text-foreground">
            Desktop local STT
          </Badge>
        </header>

        <section className="grid flex-1 items-center gap-6 py-12 lg:grid-cols-[1.05fr_0.95fr]">
          <div className="space-y-6">
            <div className="space-y-3">
              <ShimmeringText
                text="Private dictation, warmed before you speak."
                className="text-3xl font-semibold leading-tight sm:text-5xl"
                spread={1.2}
              />
              <p className="max-w-xl text-sm leading-6 text-muted-foreground sm:text-base">
                Flow is the local-first speech workspace: fast Parakeet transcription, native paste,
                local model storage, and optional cloud providers only when you choose them.
              </p>
            </div>

            <div className="flex flex-wrap gap-3">
              <Button asChild>
                <a href="https://github.com/essencefromexistence/flow">
                  <Download />
                  Source
                </a>
              </Button>
              <Button variant="outline" asChild>
                <a href="https://github.com/elevenlabs/ui">ElevenLabs UI</a>
              </Button>
            </div>
          </div>

          <Card className="overflow-hidden">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Mic2 className="size-4 text-foreground" />
                Local STT pipeline
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-5">
              <LiveWaveform
                processing
                active={false}
                height={88}
                barWidth={4}
                barGap={4}
                barRadius={3}
                className="text-foreground"
              />
              <div className="grid gap-3">
                {DETAILS.map((item) => (
                  <div
                    key={item.label}
                    className="flex items-center justify-between rounded-md border border-border bg-secondary/40 px-3 py-2"
                  >
                    <span className="flex items-center gap-2 text-xs text-muted-foreground">
                      <item.icon className="size-3.5" />
                      {item.label}
                    </span>
                    <span className="text-xs text-foreground">{item.value}</span>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </section>
      </div>
    </main>
  );
}
