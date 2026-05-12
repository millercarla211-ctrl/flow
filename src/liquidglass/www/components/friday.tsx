/* eslint-disable */
// @ts-nocheck
"use client";

import * as React from "react";
import { Button } from "@/liquidglass/www/components/ui/button";
import { cn } from "@/liquidglass/www/lib/utils";

const FRIDAY_Z = 9990;
const SLIDE_DURATION_MS = 750;
const SPAN_COUNT = 25;
const BORDER_THICKNESS = 10;
const CORNER_SIZE = 20;
const GLOW_SPREAD = 8;
const GLOW_INTENSITY = 12;

function generateRainbowGradient(count: number): string {
  const stops = Array.from({ length: count + 1 }, (_, i) => {
    const hue = (i / count) * 360;
    return `hsl(${hue}, 85%, 55%)`;
  });
  return `linear-gradient(90deg, ${stops.join(", ")})`;
}

const RAINBOW_GRADIENT = generateRainbowGradient(SPAN_COUNT);

interface FridayProps {
  sidebarWidth?: number;
  className?: string;
}

export function Friday({ sidebarWidth = 56, className }: FridayProps) {
  const [isActive, setIsActive] = React.useState(false);
  const [phase, setPhase] = React.useState<
    "idle" | "entering" | "active" | "exiting"
  >("idle");

  const scrollOrigin = React.useRef(0);
  const rafRef = React.useRef<number>(0);

  React.useEffect(() => {
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    };
  }, []);

  const smoothScrollTo = React.useCallback(
    (target: number, duration: number): Promise<void> => {
      return new Promise((resolve) => {
        const start = window.scrollY;
        const delta = target - start;
        if (Math.abs(delta) < 1) {
          resolve();
          return;
        }
        const startTime = performance.now();

        function tick(now: number) {
          const elapsed = now - startTime;
          const progress = Math.min(elapsed / duration, 1);
          const eased =
            progress < 0.5
              ? 4 * progress * progress * progress
              : 1 - Math.pow(-2 * progress + 2, 3) / 2;

          window.scrollTo(0, start + delta * eased);

          if (progress < 1) {
            rafRef.current = requestAnimationFrame(tick);
          } else {
            resolve();
          }
        }
        rafRef.current = requestAnimationFrame(tick);
      });
    },
    []
  );

  const springScrollTo = React.useCallback(
    (target: number): Promise<void> => {
      return new Promise((resolve) => {
        let current = window.scrollY;
        let velocity = 0;
        const stiffness = 0.03;
        const damping = 0.2;

        function tick() {
          const displacement = target - current;
          velocity += displacement * stiffness - velocity * damping;
          current += velocity;

          window.scrollTo(0, current);

          if (Math.abs(current - target) < 0.5 && Math.abs(velocity) < 0.5) {
            window.scrollTo(0, target);
            resolve();
          } else {
            rafRef.current = requestAnimationFrame(tick);
          }
        }
        rafRef.current = requestAnimationFrame(tick);
      });
    },
    []
  );

  const runEntrySequence = React.useCallback(async () => {
    scrollOrigin.current = window.scrollY;
    setPhase("entering");

    const scrollAmount = window.innerHeight * 0.1;
    const maxScroll =
      document.documentElement.scrollHeight - window.innerHeight;
    const scrollTarget = Math.min(
      scrollOrigin.current + scrollAmount,
      maxScroll
    );

    await smoothScrollTo(scrollTarget, SLIDE_DURATION_MS * 0.9);
    await new Promise((r) => setTimeout(r, SLIDE_DURATION_MS * 0.1));

    setPhase("active");

    const origin = scrollOrigin.current;
    await springScrollTo(origin);
    await springScrollTo(origin - 30);
    await springScrollTo(origin);
  }, [smoothScrollTo, springScrollTo]);

  const handleToggle = React.useCallback(() => {
    if (!isActive) {
      setIsActive(true);
      runEntrySequence();
    } else {
      setIsActive(false);
      setPhase("idle");
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    }
  }, [isActive, runEntrySequence]);

  const showBottom = phase === "entering" || phase === "active";
  const showRight = phase === "entering" || phase === "active";
  const showLeft = phase === "entering" || phase === "active";
  const showTop = phase === "active";

  const BT = BORDER_THICKNESS;

  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: `
        .friday-border {
          position: fixed;
          opacity: 0;
          visibility: hidden;
          transition:
            opacity 0.4s cubic-bezier(0.4, 0, 0.2, 1),
            visibility 0s linear 0.4s;
          z-index: ${FRIDAY_Z - 1};
          pointer-events: none;
          will-change: opacity;
        }

        .friday-border.visible {
          opacity: 1;
          visibility: visible;
          transition:
            opacity 0.4s cubic-bezier(0.4, 0, 0.2, 1),
            visibility 0s linear 0s;
        }

        /* ===== HORIZONTAL BORDERS ===== */
        .friday-border-top,
        .friday-border-bottom {
          left: 0;
          right: 0;
          height: ${BT}px;
        }

        .friday-border-top { 
          top: 0;
          clip-path: polygon(0 0, 100% 0, calc(100% - ${BT}px) 100%, 0 100%);
        }
        
        .friday-border-bottom { 
          bottom: 0;
          clip-path: polygon(${BT}px 0, 100% 0, 100% 100%, 0 100%);
        }

        .friday-border-top::before,
        .friday-border-bottom::before {
          content: "";
          position: absolute;
          inset: 0;
          background: ${RAINBOW_GRADIENT};
          background-size: 200% 100%;
          animation: friday-gradient-shift 3s linear infinite;
        }

        /* Tight inner glow - close to the border */
        .friday-border-top::after,
        .friday-border-bottom::after {
          content: "";
          position: absolute;
          left: 0;
          right: 0;
          background: ${RAINBOW_GRADIENT};
          background-size: 200% 100%;
          animation: friday-gradient-shift 3s linear infinite;
          filter: blur(${GLOW_SPREAD}px);
          opacity: 0.7;
          will-change: transform;
          z-index: -1;
        }

        .friday-border-top::after {
          top: 0;
          height: ${GLOW_INTENSITY}px;
        }

        .friday-border-bottom::after {
          bottom: 0;
          height: ${GLOW_INTENSITY}px;
        }

        /* ===== VERTICAL BORDERS ===== */
        .friday-border-left,
        .friday-border-right {
          width: ${BT}px;
        }

        .friday-border-left { 
          left: 0;
          top: 0;
          bottom: 0;
          clip-path: polygon(0 0, 100% 0, 100% calc(100% - ${BT}px), 0 100%);
        }
        
        .friday-border-right { 
          right: 0;
          top: 0;
          bottom: 0;
          clip-path: polygon(0 ${BT}px, 100% 0, 100% 100%, 0 100%);
        }

        .friday-border-left::before,
        .friday-border-right::before {
          content: "";
          position: absolute;
          inset: 0;
          background: ${RAINBOW_GRADIENT.replace("90deg", "180deg")};
          background-size: 100% 200%;
          animation: friday-gradient-shift-v 3s linear infinite;
        }

        /* Tight inner glow for vertical borders */
        .friday-border-left::after,
        .friday-border-right::after {
          content: "";
          position: absolute;
          top: 0;
          bottom: 0;
          background: ${RAINBOW_GRADIENT.replace("90deg", "180deg")};
          background-size: 100% 200%;
          animation: friday-gradient-shift-v 3s linear infinite;
          filter: blur(${GLOW_SPREAD}px);
          opacity: 0.7;
          will-change: transform;
          z-index: -1;
        }

        .friday-border-left::after {
          left: 0;
          width: ${GLOW_INTENSITY}px;
        }

        .friday-border-right::after {
          right: 0;
          width: ${GLOW_INTENSITY}px;
        }

        /* ===== OUTER GLOW LAYER (secondary) ===== */
        .friday-glow {
          position: fixed;
          pointer-events: none;
          z-index: ${FRIDAY_Z - 2};
          opacity: 0;
          visibility: hidden;
          transition:
            opacity 0.4s cubic-bezier(0.4, 0, 0.2, 1),
            visibility 0s linear 0.4s;
        }

        .friday-glow.visible {
          opacity: 1;
          visibility: visible;
          transition:
            opacity 0.4s cubic-bezier(0.4, 0, 0.2, 1),
            visibility 0s linear 0s;
        }

        .friday-glow-top,
        .friday-glow-bottom {
          left: 0;
          right: 0;
          height: ${BT + GLOW_INTENSITY * 2}px;
        }

        .friday-glow-top {
          top: -${GLOW_INTENSITY}px;
        }

        .friday-glow-bottom {
          bottom: -${GLOW_INTENSITY}px;
        }

        .friday-glow-left,
        .friday-glow-right {
          top: 0;
          bottom: 0;
          width: ${BT + GLOW_INTENSITY * 2}px;
        }

        .friday-glow-left {
          left: -${GLOW_INTENSITY}px;
        }

        .friday-glow-right {
          right: -${GLOW_INTENSITY}px;
        }

        .friday-glow-top::before,
        .friday-glow-bottom::before {
          content: "";
          position: absolute;
          inset: 0;
          background: ${RAINBOW_GRADIENT};
          background-size: 200% 100%;
          animation: friday-gradient-shift 3s linear infinite;
          filter: blur(${GLOW_INTENSITY + 4}px);
          opacity: 0.35;
        }

        .friday-glow-left::before,
        .friday-glow-right::before {
          content: "";
          position: absolute;
          inset: 0;
          background: ${RAINBOW_GRADIENT.replace("90deg", "180deg")};
          background-size: 100% 200%;
          animation: friday-gradient-shift-v 3s linear infinite;
          filter: blur(${GLOW_INTENSITY + 4}px);
          opacity: 0.35;
        }

        @keyframes friday-gradient-shift {
          0%   { background-position: 0% 0%; }
          100% { background-position: 200% 0%; }
        }

        @keyframes friday-gradient-shift-v {
          0%   { background-position: 0% 0%; }
          100% { background-position: 0% 200%; }
        }
      ` }} />

      <div
        className={cn(
          "fixed bottom-32 z-50 flex justify-center transition-all duration-200",
          className
        )}
        style={{ left: `${sidebarWidth}px`, right: 0 }}
      >
        <Button
          onClick={handleToggle}
          variant="outline"
          size="sm"
          className={cn(
            "rounded-md border px-4 py-2 text-sm font-medium shadow-sm transition-colors",
            isActive
              ? "border-destructive/50 bg-destructive text-destructive-foreground hover:bg-destructive/90"
              : "border-input bg-background text-foreground hover:bg-accent hover:text-accent-foreground"
          )}
          aria-label={
            isActive
              ? "Deactivate Friday Effect"
              : "Activate Friday Effect"
          }
        >
          {isActive ? "Deactivate Friday" : "Activate Friday"}
        </Button>
      </div>

      {/* BORDER LINES */}
      <div
        className={showTop ? "friday-border friday-border-top visible" : "friday-border friday-border-top"}
        aria-hidden="true"
      />
      <div
        className={showBottom ? "friday-border friday-border-bottom visible" : "friday-border friday-border-bottom"}
        aria-hidden="true"
      />
      <div
        className={showLeft ? "friday-border friday-border-left visible" : "friday-border friday-border-left"}
        style={{ transitionDelay: showLeft ? "0.3s" : "0s" }}
        aria-hidden="true"
      />
      <div
        className={showRight ? "friday-border friday-border-right visible" : "friday-border friday-border-right"}
        style={{
          transitionDelay: showRight ? `${SLIDE_DURATION_MS / 2000}s` : "0s",
        }}
        aria-hidden="true"
      />

      {/* OUTER GLOW LAYERS */}
      <div
        className={showTop ? "friday-glow friday-glow-top visible" : "friday-glow friday-glow-top"}
        aria-hidden="true"
      />
      <div
        className={showBottom ? "friday-glow friday-glow-bottom visible" : "friday-glow friday-glow-bottom"}
        aria-hidden="true"
      />
      <div
        className={showLeft ? "friday-glow friday-glow-left visible" : "friday-glow friday-glow-left"}
        style={{ transitionDelay: showLeft ? "0.3s" : "0s" }}
        aria-hidden="true"
      />
      <div
        className={showRight ? "friday-glow friday-glow-right visible" : "friday-glow friday-glow-right"}
        style={{
          transitionDelay: showRight ? `${SLIDE_DURATION_MS / 2000}s` : "0s",
        }}
        aria-hidden="true"
      />
    </>
  );
}

Friday.displayName = "Friday";