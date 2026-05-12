"use client";

import { Pause, Play, SkipBack, SkipForward, Sparkles, Volume2 } from "lucide-react";
import { LiquidGlassMaterial, type GlassShape, type GlassTone } from "@/liquidglass/fiday/components/liquid-glass-material";
import { Button } from "@/liquidglass/fiday/components/ui/button";

type MovieGlassHeroProps = {
  aberration: number;
  blur: number;
  debug: boolean;
  displacement: number;
  elasticity: number;
  saturation: number;
  shape: GlassShape;
  tone: GlassTone;
};

const sceneMedia = {
  colorful:
    "linear-gradient(140deg, rgba(255,92,119,.48), rgba(43,105,255,.32) 42%, rgba(255,207,93,.36)), url('https://images.unsplash.com/photo-1500530855697-b586d89ba3ee?auto=format&fit=crop&w=1200&q=82')",
  dark:
    "linear-gradient(150deg, rgba(4,8,16,.22), rgba(41,16,78,.36) 48%, rgba(12,35,45,.28)), url('https://images.unsplash.com/photo-1485470733090-0aae1788d5af?auto=format&fit=crop&w=1200&q=82')",
  light:
    "linear-gradient(140deg, rgba(255,255,255,.24), rgba(119,185,255,.2) 52%, rgba(255,214,170,.28)), url('https://images.unsplash.com/photo-1493246507139-91e8fad9978e?auto=format&fit=crop&w=1200&q=82')",
} satisfies Record<GlassTone, string>;

export function MovieGlassHero({
  aberration,
  blur,
  debug,
  displacement,
  elasticity,
  saturation,
  shape,
  tone,
}: MovieGlassHeroProps) {
  const isLight = tone === "light";

  return (
    <section className="movie-stage" data-tone={tone}>
      <div className="phone-frame">
        <div
          className="phone-screen"
          style={{ backgroundImage: sceneMedia[tone] }}
        >
          <div className="phone-status">
            <span>9:41</span>
            <span>5G</span>
          </div>

          <div className="movie-copy">
            <span className="movie-kicker">Now Playing</span>
            <h1>Silent Orbit</h1>
            <p>Directors Cut</p>
          </div>

          <LiquidGlassMaterial
            aberration={aberration}
            blur={blur}
            className="movie-title-glass"
            contentClassName="movie-title-glass__content"
            debug={debug}
            displacement={displacement}
            elasticity={elasticity}
            radius={34}
            saturation={saturation}
            shape="title"
            tone={tone}
          >
            <div className="movie-title-row">
              <span className="movie-orb">
                <Sparkles className="size-4" />
              </span>
              <div>
                <strong>Silent Orbit</strong>
                <span>Atmos audio / 2h 04m</span>
              </div>
            </div>
            <div className="movie-progress" aria-hidden="true">
              <span />
            </div>
          </LiquidGlassMaterial>

          <LiquidGlassMaterial
            aberration={aberration}
            blur={Math.max(10, blur - 4)}
            className="movie-control-glass"
            contentClassName="movie-control-glass__content"
            displacement={displacement * 0.78}
            elasticity={elasticity}
            radius={28}
            saturation={saturation}
            shape={shape === "control" ? "control" : "dock"}
            tone={tone}
          >
            <Button aria-label="Previous" size="icon" variant="ghost">
              <SkipBack className="size-4" />
            </Button>
            <Button aria-label={isLight ? "Pause preview" : "Play preview"} size="icon" variant="secondary">
              {isLight ? <Pause className="size-4" /> : <Play className="size-4" />}
            </Button>
            <Button aria-label="Next" size="icon" variant="ghost">
              <SkipForward className="size-4" />
            </Button>
            <span className="volume-pill">
              <Volume2 className="size-4" />
              <span />
            </span>
          </LiquidGlassMaterial>
        </div>
      </div>
    </section>
  );
}
