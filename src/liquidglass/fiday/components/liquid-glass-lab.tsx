"use client";

import {
  Aperture,
  Bug,
  Circle,
  Film,
  Layers,
  Move,
  RotateCcw,
  Sparkles,
  Waves,
} from "lucide-react";
import { useMemo, useState } from "react";
import { LiquidGlassMaterial, type GlassShape, type GlassTone } from "@/liquidglass/fiday/components/liquid-glass-material";
import { MovieGlassHero } from "@/liquidglass/fiday/components/movie-glass-hero";
import { Badge } from "@/liquidglass/fiday/components/ui/badge";
import { Button } from "@/liquidglass/fiday/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/liquidglass/fiday/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/liquidglass/fiday/components/ui/select";
import { Separator } from "@/liquidglass/fiday/components/ui/separator";
import { Slider } from "@/liquidglass/fiday/components/ui/slider";
import { Switch } from "@/liquidglass/fiday/components/ui/switch";
import { cn } from "@/liquidglass/fiday/lib/utils";

type GlassConfig = {
  aberration: number;
  blur: number;
  displacement: number;
  elasticity: number;
  saturation: number;
};

const defaultConfig: GlassConfig = {
  aberration: 2.4,
  blur: 18,
  displacement: 78,
  elasticity: 0.22,
  saturation: 158,
};

const shapeCopy = {
  control: {
    icon: Aperture,
    title: "Scene controls",
    description: "Floating control strip",
  },
  dock: {
    icon: Layers,
    title: "Glass dock",
    description: "Wide elastic material",
  },
  pill: {
    icon: Waves,
    title: "Action pill",
    description: "Compact liquid button",
  },
  title: {
    icon: Film,
    title: "Movie title",
    description: "Readable media overlay",
  },
} satisfies Record<GlassShape, { description: string; icon: typeof Layers; title: string }>;

const toneLabels = {
  colorful: "Colorful poster",
  dark: "Dark cinema",
  light: "Bright landscape",
} satisfies Record<GlassTone, string>;

function controlLabel(label: string, value: string | number) {
  return (
    <div className="flex items-center justify-between gap-3 text-xs">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-mono text-foreground">{value}</span>
    </div>
  );
}

export function LiquidGlassLab() {
  const [shape, setShape] = useState<GlassShape>("title");
  const [tone, setTone] = useState<GlassTone>("colorful");
  const [debug, setDebug] = useState(false);
  const [config, setConfig] = useState<GlassConfig>(defaultConfig);

  function updateConfig<K extends keyof GlassConfig>(key: K, value: GlassConfig[K]) {
    setConfig((current) => ({ ...current, [key]: value }));
  }

  const materialContent = useMemo(() => {
    if (shape === "pill") {
      return (
        <span className="sample-pill-content">
          <Sparkles className="size-4" />
          Play Trailer
        </span>
      );
    }

    if (shape === "title") {
      return (
        <div className="sample-title-content">
          <span>Featured Film</span>
          <strong>Silent Orbit</strong>
        </div>
      );
    }

    if (shape === "control") {
      return (
        <div className="sample-control-content">
          <Button aria-label="Previous material state" size="icon" variant="ghost">
            <Move className="size-4" />
          </Button>
          <span>Focus Layer</span>
          <Button aria-label="Activate material" size="sm" variant="secondary">
            Lift
          </Button>
        </div>
      );
    }

    return (
      <div className="sample-dock-content">
        {["Finder", "TV", "Music", "Photos"].map((item) => (
          <span key={item}>{item.slice(0, 1)}</span>
        ))}
      </div>
    );
  }, [shape]);

  return (
    <main className="liquid-page min-h-screen overflow-x-hidden bg-background text-foreground" data-tone={tone}>
      <GooeyDefs />

      <section className="liquid-shell">
        <header className="liquid-header">
          <div className="liquid-header__copy">
            <Badge className="rounded-full" variant="secondary">
              Apple-style Liquid Glass
            </Badge>
            <h1>Liquid Glass</h1>
            <p>
              A cinematic material study with edge lensing, clear watery rims,
              live highlights, adaptive tint, and gooey materialization.
            </p>
          </div>

          <div className="shape-switcher" aria-label="Material shape">
            {(Object.keys(shapeCopy) as GlassShape[]).map((nextShape) => {
              const Icon = shapeCopy[nextShape].icon;
              return (
                <Button
                  aria-pressed={shape === nextShape}
                  className={cn(shape === nextShape && "shape-switcher__button--active")}
                  key={nextShape}
                  onClick={() => setShape(nextShape)}
                  size="sm"
                  variant={shape === nextShape ? "secondary" : "ghost"}
                >
                  <Icon className="size-4" />
                  {shapeCopy[nextShape].title}
                </Button>
              );
            })}
          </div>
        </header>

        <section className="liquid-experience-grid">
          <MovieGlassHero
            aberration={config.aberration}
            blur={config.blur}
            debug={debug}
            displacement={config.displacement}
            elasticity={config.elasticity}
            saturation={config.saturation}
            shape={shape}
            tone={tone}
          />

          <section className="material-board" data-shape={shape}>
            <div className="material-board__backdrop" aria-hidden="true">
              <span className="poster-strip poster-strip--red" />
              <span className="poster-strip poster-strip--gold" />
              <span className="poster-strip poster-strip--cyan" />
              <span className="poster-card poster-card--one" />
              <span className="poster-card poster-card--two" />
            </div>

            <div className="material-board__content">
              <div>
                <span className="material-eyebrow">{toneLabels[tone]}</span>
                <h2>{shapeCopy[shape].title}</h2>
                <p>{shapeCopy[shape].description}</p>
              </div>

              <div className="morph-stage">
                <div className="morph-stage__gooey" aria-hidden="true">
                  <span className="morph-ghost morph-ghost--one" />
                  <span className="morph-ghost morph-ghost--two" />
                  <span className="morph-ghost morph-ghost--three" />
                </div>
                <LiquidGlassMaterial
                  aberration={config.aberration}
                  blur={config.blur}
                  className="sample-material"
                  contentClassName="sample-material__content"
                  debug={debug}
                  displacement={config.displacement}
                  elasticity={config.elasticity}
                  radius={shape === "title" ? 34 : shape === "dock" ? 42 : 999}
                  saturation={config.saturation}
                  shape={shape}
                  tone={tone}
                >
                  {materialContent}
                </LiquidGlassMaterial>
              </div>
            </div>
          </section>

          <aside className="control-column">
            <ControlPanel
              config={config}
              debug={debug}
              shape={shape}
              tone={tone}
              onConfigChange={updateConfig}
              onDebugChange={setDebug}
              onReset={() => setConfig(defaultConfig)}
              onShapeChange={setShape}
              onToneChange={setTone}
            />
          </aside>
        </section>
      </section>
    </main>
  );
}

function GooeyDefs() {
  return (
    <svg aria-hidden="true" className="gooey-defs" focusable="false">
      <defs>
        <filter id="gooey-materialize">
          <feGaussianBlur in="SourceGraphic" result="blur" stdDeviation="10" />
          <feColorMatrix
            in="blur"
            result="goo"
            type="matrix"
            values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 22 -10"
          />
          <feComposite in="SourceGraphic" in2="goo" operator="atop" />
        </filter>
      </defs>
    </svg>
  );
}

function ControlPanel({
  config,
  debug,
  shape,
  tone,
  onConfigChange,
  onDebugChange,
  onReset,
  onShapeChange,
  onToneChange,
}: {
  config: GlassConfig;
  debug: boolean;
  shape: GlassShape;
  tone: GlassTone;
  onConfigChange: <K extends keyof GlassConfig>(key: K, value: GlassConfig[K]) => void;
  onDebugChange: (value: boolean) => void;
  onReset: () => void;
  onShapeChange: (value: GlassShape) => void;
  onToneChange: (value: GlassTone) => void;
}) {
  return (
    <Card className="control-card border-foreground/10 bg-background/80 shadow-2xl shadow-foreground/10 backdrop-blur-xl">
      <CardHeader className="space-y-3">
        <div className="flex items-center justify-between gap-3">
          <div>
            <CardTitle className="text-xl tracking-normal">Material</CardTitle>
            <CardDescription>Optics, shape, and scene response.</CardDescription>
          </div>
          <Button aria-label="Reset material" onClick={onReset} size="icon" variant="ghost">
            <RotateCcw className="size-4" />
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-5">
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <span className="text-xs text-muted-foreground">Shape</span>
            <Select onValueChange={(value) => onShapeChange(value as GlassShape)} value={shape}>
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="title">Movie title</SelectItem>
                <SelectItem value="dock">Dock</SelectItem>
                <SelectItem value="pill">Pill</SelectItem>
                <SelectItem value="control">Control</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <span className="text-xs text-muted-foreground">Scene</span>
            <Select onValueChange={(value) => onToneChange(value as GlassTone)} value={tone}>
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="colorful">Colorful</SelectItem>
                <SelectItem value="dark">Dark</SelectItem>
                <SelectItem value="light">Light</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div className="flex items-center justify-between rounded-md border border-border bg-muted/40 px-3 py-2">
          <span className="inline-flex items-center gap-2 text-sm">
            <Bug className="size-4" /> Debug map
          </span>
          <Switch checked={debug} onCheckedChange={onDebugChange} />
        </div>

        <div className="flex items-center justify-between rounded-md border border-border bg-muted/40 px-3 py-2">
          <span className="inline-flex items-center gap-2 text-sm">
            <Circle className="size-4" /> Touch glow
          </span>
          <span className="text-xs text-muted-foreground">Always on</span>
        </div>

        <Separator />

        <div className="space-y-4">
          <SliderControl
            label="Rim refraction"
            max={140}
            min={20}
            onChange={(value) => onConfigChange("displacement", value)}
            step={1}
            value={config.displacement}
          />
          <SliderControl
            label="Frost"
            max={32}
            min={6}
            onChange={(value) => onConfigChange("blur", value)}
            step={1}
            value={config.blur}
          />
          <SliderControl
            label="Saturation"
            max={220}
            min={100}
            onChange={(value) => onConfigChange("saturation", value)}
            step={1}
            value={config.saturation}
          />
          <SliderControl
            label="Aberration"
            max={6}
            min={0}
            onChange={(value) => onConfigChange("aberration", value)}
            step={0.1}
            value={config.aberration}
          />
          <SliderControl
            label="Elasticity"
            max={0.45}
            min={0}
            onChange={(value) => onConfigChange("elasticity", value)}
            step={0.01}
            value={config.elasticity}
          />
        </div>
      </CardContent>
    </Card>
  );
}

function SliderControl({
  label,
  max,
  min,
  onChange,
  step,
  value,
}: {
  label: string;
  max: number;
  min: number;
  onChange: (value: number) => void;
  step: number;
  value: number;
}) {
  return (
    <div className="space-y-2">
      {controlLabel(label, Number.isInteger(value) ? value : value.toFixed(2))}
      <Slider
        max={max}
        min={min}
        onValueChange={([next]) => onChange(next)}
        step={step}
        value={[value]}
      />
    </div>
  );
}
