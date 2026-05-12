/* eslint-disable */
// @ts-nocheck
"use client";

import { AnimatePresence, motion } from "motion/react";
import { memo, useCallback, useEffect, useRef, useState } from "react";

// ─── Constants ───────────────────────────────────────────────────────────────

const WAVE_LIFETIME_MS = 3_500;
const ANIMATION_DURATION_MS = 2_000;
const CANVAS_FPS = 15;
const CANVAS_FRAME_MS = 1000 / CANVAS_FPS;

const NUM_RINGS = 6;
const NUM_SPEED_LINES = 24;
const NUM_DOT_RINGS = 4;
const DOTS_PER_RING = 16;
const NUM_WEB_STRANDS = 12;
const NUM_CONNECTING_WEBS = 8;
const TAU = Math.PI * 2;

// White sticky web palette
const WHITE = { r: 255, g: 255, b: 255 } as const;
const LIGHT_GRAY = { r: 220, g: 220, b: 230 } as const;
const SILVER = { r: 192, g: 192, b: 192 } as const;
const PALE_BLUE = { r: 200, g: 220, b: 255 } as const;

// ─── Types ───────────────────────────────────────────────────────────────────

interface Point {
  readonly x: number;
  readonly y: number;
}

interface RGB {
  readonly r: number;
  readonly g: number;
  readonly b: number;
}

interface WaveInstance {
  readonly id: number;
  readonly x: number;
  readonly y: number;
  readonly pattern: number; // Random pattern variation 0-2
  readonly rotation: number; // Random rotation offset
  readonly scale: number; // Random size variation
}

interface SpidermanWavesModeProps {
  readonly active: boolean;
  readonly onTrigger?: () => void;
}

// ─── Easing ──────────────────────────────────────────────────────────────────

function easeOutCubic(t: number): number {
  return 1 - (1 - t) ** 3;
}

function easeOutQuart(t: number): number {
  return 1 - (1 - t) ** 4;
}

// ─── Pre-computed Flicker Table ──────────────────────────────────────────────

const FLICKER_SIZE = 64;
const FLICKER = Float32Array.from(
  { length: FLICKER_SIZE },
  () => 0.4 + Math.random() * 0.6,
);

// ─── Helpers ─────────────────────────────────────────────────────────────────

function rgba(c: RGB, a: number): string {
  return `rgba(${c.r},${c.g},${c.b},${a})`;
}

// ─── Canvas Renderer ─────────────────────────────────────────────────────────

/**
 * Draws sticky spider web wave effect with interconnected strands
 */
function drawWaveFrame(
  ctx: CanvasRenderingContext2D,
  center: Point,
  progress: number,
  w: number,
  h: number,
  dpr: number,
  frame: number,
  pattern: number,
  rotation: number,
  scale: number,
): void {
  ctx.clearRect(0, 0, w * dpr, h * dpr);
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

  // Much smaller max radius
  const baseMaxR = Math.min(w, h) * 0.15 * scale;
  const maxR = baseMaxR;

  // Apply rotation
  ctx.translate(center.x, center.y);
  ctx.rotate(rotation);
  ctx.translate(-center.x, -center.y);

  // Pattern variations
  const numStrands = pattern === 0 ? 16 : pattern === 1 ? 20 : 24;
  const numRings = pattern === 0 ? 4 : pattern === 1 ? 5 : 6;
  const connectDensity = pattern === 0 ? 6 : pattern === 1 ? 8 : 10;

  // ── 1. Radial web strands with curves (air friction effect) ──────────────

  const strandAlpha = Math.max(0, 1 - progress * 1.2);
  if (strandAlpha > 0.01) {
    const step = TAU / numStrands;

    for (let i = 0; i < numStrands; i++) {
      const a = step * i;
      const cos = Math.cos(a);
      const sin = Math.sin(a);
      const inner = 10;
      const outer = inner + maxR * 0.9;
      const flicker = FLICKER[(frame * numStrands + i) & (FLICKER_SIZE - 1)];
      const thickness = 1.5 + 1.5;

      // Create flowing curve with multiple control points
      const dist = outer - inner;
      const cp1Dist = inner + dist * 0.33;
      const cp2Dist = inner + dist * 0.66;
      
      // Add wave-like oscillation to the curve
      const waveOffset1 = Math.sin(i * 0.5 + progress * 2) * maxR * 0.08;
      const waveOffset2 = Math.sin(i * 0.5 + progress * 2 + 1) * maxR * 0.12;
      
      const perpAngle = a + Math.PI / 2;
      
      const cp1X = center.x + Math.cos(a) * cp1Dist + Math.cos(perpAngle) * waveOffset1;
      const cp1Y = center.y + Math.sin(a) * cp1Dist + Math.sin(perpAngle) * waveOffset1;
      
      const cp2X = center.x + Math.cos(a) * cp2Dist + Math.cos(perpAngle) * waveOffset2;
      const cp2Y = center.y + Math.sin(a) * cp2Dist + Math.sin(perpAngle) * waveOffset2;

      ctx.globalAlpha = strandAlpha * flicker;
      ctx.strokeStyle = rgba(WHITE, 1);
      ctx.lineWidth = thickness;
      ctx.shadowBlur = 8;
      ctx.shadowColor = rgba(WHITE, strandAlpha * 0.6);

      ctx.beginPath();
      ctx.moveTo(center.x + cos * inner, center.y + sin * inner);
      ctx.bezierCurveTo(
        cp1X, cp1Y,
        cp2X, cp2Y,
        center.x + cos * outer, center.y + sin * outer
      );
      ctx.stroke();
    }
  }

  ctx.shadowBlur = 0;

  // ── 2. Curved concentric web rings (no perfect circles) ──────────────────

  const ringAlpha = Math.max(0, 1 - progress * 1.2);
  if (ringAlpha > 0.01) {
    for (let i = 0; i < numRings; i++) {
      const ringRadius = maxR * (0.25 + i * 0.15);

      ctx.globalAlpha = ringAlpha;
      ctx.strokeStyle = rgba(WHITE, 1);
      ctx.lineWidth = Math.max(1, (numRings - i) * 1.8);
      ctx.shadowBlur = 10;
      ctx.shadowColor = rgba(WHITE, ringAlpha * 0.7);

      // Draw organic curved ring (not a perfect circle)
      const segments = 32;
      const segmentStep = TAU / segments;
      
      ctx.beginPath();
      for (let s = 0; s <= segments; s++) {
        const angle = segmentStep * s;
        
        // Add organic variation to radius
        const radiusVariation = Math.sin(angle * 3 + i * 0.5) * ringRadius * 0.08;
        const waveVariation = Math.sin(angle * 5 + progress * 3) * ringRadius * 0.05;
        const r = ringRadius + radiusVariation + waveVariation;
        
        const x = center.x + Math.cos(angle) * r;
        const y = center.y + Math.sin(angle) * r;
        
        if (s === 0) {
          ctx.moveTo(x, y);
        } else {
          // Use quadratic curves between points for smooth flow
          const prevAngle = segmentStep * (s - 1);
          const prevR = ringRadius + 
            Math.sin(prevAngle * 3 + i * 0.5) * ringRadius * 0.08 +
            Math.sin(prevAngle * 5 + progress * 3) * ringRadius * 0.05;
          const prevX = center.x + Math.cos(prevAngle) * prevR;
          const prevY = center.y + Math.sin(prevAngle) * prevR;
          
          const cpX = (prevX + x) / 2;
          const cpY = (prevY + y) / 2;
          
          ctx.quadraticCurveTo(cpX, cpY, x, y);
        }
      }
      ctx.stroke();

      // Draw curved connecting web strands between rings
      if (i > 0) {
        const prevRingRadius = maxR * (0.25 + (i - 1) * 0.15);
        const connectAlpha = ringAlpha * 0.5;
        ctx.globalAlpha = connectAlpha;
        ctx.lineWidth = 1;
        ctx.shadowBlur = 6;

        const connectStep = TAU / connectDensity;
        for (let c = 0; c < connectDensity; c++) {
          const angle = connectStep * c + i * 0.3;
          
          // Calculate start and end points with organic variation
          const startRadiusVar = Math.sin(angle * 3 + (i-1) * 0.5) * prevRingRadius * 0.08;
          const endRadiusVar = Math.sin(angle * 3 + i * 0.5) * ringRadius * 0.08;
          
          const startR = prevRingRadius + startRadiusVar;
          const endR = ringRadius + endRadiusVar;
          
          const startX = center.x + Math.cos(angle) * startR;
          const startY = center.y + Math.sin(angle) * startR;
          const endX = center.x + Math.cos(angle) * endR;
          const endY = center.y + Math.sin(angle) * endR;

          // Add flowing curve to connecting strands
          const midDist = (startR + endR) / 2;
          const curveBias = (endR - startR) * 0.15;
          const perpAngle = angle + Math.PI / 2;
          const waveOffset = Math.sin(angle * 4 + progress * 2) * curveBias;
          
          const cp1X = center.x + Math.cos(angle) * (startR + (midDist - startR) * 0.4) + 
                       Math.cos(perpAngle) * (curveBias * 0.5 + waveOffset);
          const cp1Y = center.y + Math.sin(angle) * (startR + (midDist - startR) * 0.4) + 
                       Math.sin(perpAngle) * (curveBias * 0.5 + waveOffset);
          
          const cp2X = center.x + Math.cos(angle) * (startR + (midDist - startR) * 0.7) + 
                       Math.cos(perpAngle) * (curveBias * 0.3 - waveOffset * 0.5);
          const cp2Y = center.y + Math.sin(angle) * (startR + (midDist - startR) * 0.7) + 
                       Math.sin(perpAngle) * (curveBias * 0.3 - waveOffset * 0.5);

          ctx.beginPath();
          ctx.moveTo(startX, startY);
          ctx.bezierCurveTo(cp1X, cp1Y, cp2X, cp2Y, endX, endY);
          ctx.stroke();
        }
      }
    }
  }

  ctx.shadowBlur = 0;

  // ── 3. Central wave origin glow (organic shape) ───────────────────────────

  if (progress < 0.7) {
    const ba = Math.max(0, 1 - progress * 1.4) * 0.7;
    const br = 25 * scale;

    // Draw organic glow shape instead of perfect circle
    const glowSegments = 16;
    const glowStep = TAU / glowSegments;
    
    const g = ctx.createRadialGradient(center.x, center.y, 0, center.x, center.y, br * 1.5);
    g.addColorStop(0, `rgba(255,255,255,${ba})`);
    g.addColorStop(0.5, rgba(PALE_BLUE, ba * 0.5));
    g.addColorStop(1, rgba(WHITE, 0));

    ctx.globalAlpha = 1;
    ctx.fillStyle = g;
    ctx.shadowBlur = 15;
    ctx.shadowColor = rgba(WHITE, ba * 0.5);
    
    ctx.beginPath();
    for (let s = 0; s <= glowSegments; s++) {
      const angle = glowStep * s;
      const radiusVar = Math.sin(angle * 4 + progress * 5) * br * 0.2;
      const r = br + radiusVar;
      const x = center.x + Math.cos(angle) * r;
      const y = center.y + Math.sin(angle) * r;
      
      if (s === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }
    ctx.closePath();
    ctx.fill();
  }

  ctx.shadowBlur = 0;
  ctx.restore();
}

// ─── Data Generation ─────────────────────────────────────────────────────────

// Removed glitch bars generation

// ─── Hoisted Style Objects ───────────────────────────────────────────────────

const CANVAS_STYLE: React.CSSProperties = { mixBlendMode: "screen" };
const CURSOR_STYLE: React.CSSProperties = { willChange: "transform" };
const GLITCH_BG: React.CSSProperties = { mixBlendMode: "screen" };

// ─── SpiderCrosshair ─────────────────────────────────────────────────────────

const STRAND_TRANSFORMS = Array.from(
  { length: NUM_WEB_STRANDS },
  (_, i) => `translate(0,-50%) rotate(${(360 / NUM_WEB_STRANDS) * i}deg)`,
);

const SpiderCrosshair = memo(function SpiderCrosshair() {
  return (
    <>
      {/* Central pulse */}
      <div className="absolute left-1/2 top-1/2 h-2.5 w-2.5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-white shadow-[0_0_12px_rgba(255,255,255,0.9)]">
        <div className="absolute inset-0 animate-ping rounded-full bg-white opacity-70" />
      </div>

      {/* Web strands */}
      {STRAND_TRANSFORMS.map((transform, i) => (
        <div
          key={i}
          className="absolute left-1/2 top-1/2 h-px w-10 origin-left bg-linear-to-r from-white/90 to-transparent"
          style={{ transform }}
        />
      ))}

      {/* Inner ring */}
      <div className="absolute left-1/2 top-1/2 h-8 w-8 -translate-x-1/2 -translate-y-1/2 rounded-full border border-white/50" />

      {/* Outer ring */}
      <div className="absolute left-1/2 top-1/2 h-16 w-16 -translate-x-1/2 -translate-y-1/2 rounded-full border border-white/30" />
    </>
  );
});

// ─── WaveEffect ──────────────────────────────────────────────────────────────

interface WaveEffectProps {
  readonly x: number;
  readonly y: number;
  readonly pattern: number;
  readonly rotation: number;
  readonly scale: number;
}

const WaveEffect = memo(function WaveEffect({ x, y, pattern, rotation, scale }: WaveEffectProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const w = window.innerWidth;
    const h = window.innerHeight;
    canvas.width = w * dpr;
    canvas.height = h * dpr;

    const center: Point = { x, y };
    const t0 = performance.now();
    let rafId: number;
    let lastDraw = 0;
    let frame = 0;

    const tick = (now: number) => {
      const progress = Math.min(1, (now - t0) / ANIMATION_DURATION_MS);

      // Throttle draw rate for Spider-Verse choppy aesthetic
      if (now - lastDraw >= CANVAS_FRAME_MS) {
        lastDraw = now - ((now - lastDraw) % CANVAS_FRAME_MS);
        drawWaveFrame(ctx, center, progress, w, h, dpr, frame++, pattern, rotation, scale);
      }

      if (progress < 1) {
        rafId = requestAnimationFrame(tick);
      }
    };

    rafId = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafId);
  }, [x, y, pattern, rotation, scale]);

  const whiteGlow: React.CSSProperties = {
    background: `radial-gradient(circle at ${x}px ${y}px, ${rgba(WHITE, 0.3)} 0%, transparent 60%)`,
    mixBlendMode: "screen",
  };
  const silverGlow: React.CSSProperties = {
    background: `radial-gradient(circle at ${x}px ${y}px, ${rgba(SILVER, 0.25)} 0%, transparent 60%)`,
    mixBlendMode: "screen",
  };

  return (
    <motion.div
      initial={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.8 }}
      className="pointer-events-none fixed inset-0 z-50"
    >
      <canvas
        ref={canvasRef}
        className="absolute inset-0 h-full w-full"
        style={CANVAS_STYLE}
      />

      <motion.div
        className="absolute inset-0"
        initial={{ opacity: 0 }}
        animate={{ opacity: [0, 0.08, 0] }}
        transition={{ duration: 0.4, times: [0, 0.2, 1] }}
        style={whiteGlow}
      />

      <motion.div
        className="absolute inset-0"
        initial={{ opacity: 0 }}
        animate={{ opacity: [0, 0.05, 0] }}
        transition={{ duration: 0.4, times: [0, 0.2, 1] }}
        style={silverGlow}
      />
    </motion.div>
  );
});

// ─── SpidermanWavesMode (Public API) ─────────────────────────────────────────

export function SpidermanWavesMode({ active, onTrigger }: SpidermanWavesModeProps) {
  const [waves, setWaves] = useState<WaveInstance[]>([]);
  const crosshairRef = useRef<HTMLDivElement>(null);
  const nextId = useRef(0);
  const timers = useRef(new Set<number>());

  useEffect(() => {
    if (!active) return;

    const onMove = (e: MouseEvent) => {
      const el = crosshairRef.current;
      if (el) el.style.transform = `translate3d(${e.clientX}px,${e.clientY}px,0)`;
    };

    window.addEventListener("mousemove", onMove, { passive: true });
    return () => window.removeEventListener("mousemove", onMove);
  }, [active]);

  useEffect(() => {
    if (!active) {
      setWaves([]);
      timers.current.forEach(clearTimeout);
      timers.current.clear();
    }
  }, [active]);

  useEffect(() => {
    const t = timers.current;
    return () => t.forEach(clearTimeout);
  }, []);

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      if (!active) return;

      const id = ++nextId.current;
      const { clientX: x, clientY: y } = e;

      // Generate random wave characteristics
      const pattern = Math.floor(Math.random() * 3); // 0, 1, or 2
      const rotation = Math.random() * TAU;
      const scale = 0.7 + Math.random() * 0.6; // 0.7 to 1.3

      setWaves((prev) => [
        ...prev,
        { id, x, y, pattern, rotation, scale },
      ]);
      onTrigger?.();

      const timer = window.setTimeout(() => {
        setWaves((prev) => prev.filter((w) => w.id !== id));
        timers.current.delete(timer);
      }, WAVE_LIFETIME_MS);
      timers.current.add(timer);
    },
    [active, onTrigger],
  );

  if (!active) return null;

  return (
    <>
      <div
        ref={crosshairRef}
        className="pointer-events-none fixed left-0 top-0 z-60"
        style={CURSOR_STYLE}
      >
        <SpiderCrosshair />
      </div>

      <div className="fixed inset-0 z-55 cursor-none" onClick={handleClick} />

      <AnimatePresence>
        {waves.map(({ id, x, y, pattern, rotation, scale }) => (
          <WaveEffect key={id} x={x} y={y} pattern={pattern} rotation={rotation} scale={scale} />
        ))}
      </AnimatePresence>
    </>
  );
}
