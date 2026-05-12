"use client";

import { AnimatePresence, motion } from "motion/react";
import { memo, useCallback, useEffect, useRef, useState } from "react";

// ─── Constants ───────────────────────────────────────────────────────────────

const CRACK_LIFETIME_MS = 5_000;
const NUM_BRANCHES = 8;
const NUM_SHARDS = 8;
const INITIAL_RADIUS = 15;
const MAX_RADIUS = 500;
const LINE_STAGGER_MS = 15;
const CURVATURE = 0.3;
const DEG_TO_RAD = Math.PI / 180;
const BRANCH_ANGLE_STEP = 360 / (NUM_BRANCHES + 1);

// ─── Types ───────────────────────────────────────────────────────────────────

interface Point {
  readonly x: number;
  readonly y: number;
}

interface LinePath {
  readonly dl: number;
  readonly tx: number;
  readonly ty: number;
  readonly cpt: Point;
}

interface CrackLine {
  readonly p1: Point;
  readonly p2: Point;
  readonly path: LinePath;
  readonly level: number;
}

interface Shard {
  readonly index: number;
  readonly targetX: number;
  readonly targetY: number;
  readonly rotation: number;
  readonly delay: number;
}

interface CrackInstance {
  readonly id: number;
  readonly x: number;
  readonly y: number;
  readonly shards: readonly Shard[];
}

interface GunshotModeProps {
  readonly active: boolean;
  readonly onShoot?: () => void;
}

// ─── Geometry Helpers ────────────────────────────────────────────────────────

/**
 * Projects a point via 2D rotation of vector (r, r), producing a
 * spiral-like distribution at effective radius r√2 with a 45° offset.
 */
function projectPoint(center: Point, radius: number, angleDeg: number): Point {
  const rad = angleDeg * DEG_TO_RAD;
  const cos = Math.cos(rad);
  const sin = Math.sin(rad);
  return {
    x: center.x + radius * (cos - sin),
    y: center.y + radius * (sin + cos),
  };
}

/** Computes a curved path descriptor between two points with a randomised control point. */
function computeLinePath(p1: Point, p2: Point, curvature: number): LinePath {
  const cv = 5 * curvature;
  const dx = p2.x - p1.x;
  const dy = p2.y - p1.y;
  const dl = Math.sqrt(dx * dx + dy * dy);
  const invDl = dl > 0 ? 1 / dl : 0;

  const sx = dx * invDl;
  const sy = dy * invDl;
  const tx = dy * invDl;
  const ty = -dx * invDl;

  const midRatio = Math.random() * 0.5 + 0.3;
  const curveRange = Math.log(dl * Math.E) * cv;
  const curveOffset = Math.random() * curveRange - curveRange * 0.5;

  return {
    dl,
    tx,
    ty,
    cpt: {
      x: p1.x + sx * dl * midRatio + tx * curveOffset,
      y: p1.y + sy * dl * midRatio + ty * curveOffset,
    },
  };
}

// ─── Crack Generation ────────────────────────────────────────────────────────

interface BranchNode {
  readonly angle: number;
  readonly point: Point;
}

/**
 * Generates radial crack lines expanding outward from `center` in concentric
 * rings, with occasional tangential and diagonal cross-connections.
 */
function generateCrackLines(
  center: Point,
  viewW: number,
  viewH: number,
): CrackLine[] {
  const rings: (BranchNode | null)[][] = [];

  // Seed ring — starting points near the impact centre
  rings.push(
    Array.from({ length: NUM_BRANCHES }, (_, i) => {
      const angle = BRANCH_ANGLE_STEP * i + 10;
      return { angle, point: projectPoint(center, 5, angle) };
    }),
  );

  // Expand outward with geometrically increasing radius
  let radius = INITIAL_RADIUS;
  while (radius < MAX_RADIUS) {
    const prevRing = rings[rings.length - 1];
    const level = rings.length;

    rings.push(
      prevRing.map((prev) => {
        if (!prev) return null;
        const { x, y } = prev.point;
        if (x <= 0 || x >= viewW || y <= 0 || y >= viewH) return null;

        const jitter = (Math.random() * 10) / NUM_BRANCHES - 5 / NUM_BRANCHES;
        const angle = Math.min(prev.angle + jitter, 350);
        const rJitter = radius + ((Math.random() - 0.5) * radius) / level;
        return { angle, point: projectPoint(center, rJitter, angle) };
      }),
    );

    radius *= Math.random() * 1.5 + 1;
  }

  // Convert node pairs into renderable crack lines
  const lines: CrackLine[] = [];

  for (let l = 1; l < rings.length; l++) {
    for (let b = 0; b < NUM_BRANCHES; b++) {
      const prev = rings[l - 1][b];
      const curr = rings[l][b];
      if (!prev || !curr) continue;

      // Primary radial crack
      lines.push({
        p1: prev.point,
        p2: curr.point,
        path: computeLinePath(prev.point, curr.point, CURVATURE),
        level: l,
      });

      // Occasional tangential connection to the adjacent branch
      if (Math.random() < 0.2) {
        const neighbor = rings[l][(b + 1) % NUM_BRANCHES];
        if (neighbor) {
          lines.push({
            p1: curr.point,
            p2: neighbor.point,
            path: computeLinePath(curr.point, neighbor.point, CURVATURE),
            level: l,
          });
        }
      }

      // Rare diagonal connection to the next ring's adjacent branch
      if (l < rings.length - 1 && Math.random() < 0.1) {
        const diag = rings[l + 1][(b + 1) % NUM_BRANCHES];
        if (diag) {
          lines.push({
            p1: curr.point,
            p2: diag.point,
            path: computeLinePath(curr.point, diag.point, CURVATURE),
            level: l,
          });
        }
      }
    }
  }

  return lines;
}

// ─── Shard Generation ────────────────────────────────────────────────────────

function generateShards(x: number, y: number): Shard[] {
  const step = (Math.PI * 2) / NUM_SHARDS;
  return Array.from({ length: NUM_SHARDS }, (_, i) => {
    const angle = step * i + (Math.random() - 0.5) * 0.5;
    const dist = 50 + Math.random() * 100;
    return {
      index: i,
      targetX: x + Math.cos(angle) * dist,
      targetY: y + Math.sin(angle) * dist,
      rotation: Math.random() * 360,
      delay: Math.random() * 0.3,
    };
  });
}

// ─── Canvas Rendering ────────────────────────────────────────────────────────

/** Batch-draws a set of crack lines with glow and glass-reflection highlights. */
function drawCrackLines(ctx: CanvasRenderingContext2D, lines: CrackLine[]): void {
  ctx.save();

  for (const { p1, p2, path } of lines) {
    const { tx, ty, dl, cpt } = path;
    const halfW = dl * 0.25;

    // Crack stroke with outer glow
    ctx.globalAlpha = 0.3;
    ctx.strokeStyle = "rgba(255, 255, 255, 0.5)";
    ctx.lineWidth = 2;
    ctx.shadowBlur = 8;
    ctx.shadowColor = "rgba(255, 255, 255, 0.3)";
    ctx.beginPath();
    ctx.moveTo(p1.x, p1.y);
    ctx.quadraticCurveTo(cpt.x, cpt.y, p2.x, p2.y);
    ctx.stroke();

    // Glass-reflection band along the crack
    ctx.shadowBlur = 0;
    ctx.globalAlpha = 0.15;
    const grad = ctx.createLinearGradient(
      p1.x + halfW * tx,
      p1.y + halfW * ty,
      p1.x - halfW * tx,
      p1.y - halfW * ty,
    );
    grad.addColorStop(0, "rgba(255, 255, 255, 0)");
    grad.addColorStop(0.5, "rgba(255, 255, 255, 0.2)");
    grad.addColorStop(1, "rgba(255, 255, 255, 0)");
    ctx.fillStyle = grad;
    ctx.beginPath();
    ctx.moveTo(p1.x + halfW * tx, p1.y + halfW * ty);
    ctx.lineTo(p2.x + halfW * tx, p2.y + halfW * ty);
    ctx.lineTo(p2.x - halfW * tx, p2.y - halfW * ty);
    ctx.lineTo(p1.x - halfW * tx, p1.y - halfW * ty);
    ctx.closePath();
    ctx.fill();
  }

  ctx.restore();
}

// ─── Static styles (hoisted to avoid allocations per render) ─────────────────

const CANVAS_STYLE: React.CSSProperties = {
  mixBlendMode: "overlay",
  filter: "blur(0.5px)",
};

const SHARD_STYLE: React.CSSProperties = {
  background:
    "linear-gradient(135deg, rgba(255,255,255,0.15), rgba(255,255,255,0.05))",
  border: "1px solid rgba(255, 255, 255, 0.2)",
  clipPath: "polygon(50% 0%, 100% 38%, 82% 100%, 18% 100%, 0% 38%)",
};

const CROSSHAIR_STYLE: React.CSSProperties = { willChange: "transform" };

// ─── Crosshair ───────────────────────────────────────────────────────────────

const Crosshair = memo(function Crosshair() {
  return (
    <>
      <div className="absolute left-1/2 top-1/2 h-3 w-3 -translate-x-1/2 -translate-y-1/2 rounded-full bg-red-600 shadow-[0_0_20px_rgba(220,38,38,1)]">
        <div className="absolute inset-0 animate-ping rounded-full bg-red-600 opacity-75" />
      </div>
      <div className="absolute left-1/2 top-1/2 h-px w-16 -translate-x-1/2 -translate-y-1/2 bg-red-500/70" />
      <div className="absolute left-1/2 top-1/2 h-16 w-px -translate-x-1/2 -translate-y-1/2 bg-red-500/70" />
      <div className="absolute left-1/2 top-1/2 h-20 w-20 -translate-x-1/2 -translate-y-1/2 rounded-full border border-red-500/50" />
    </>
  );
});

// ─── CrackEffect ─────────────────────────────────────────────────────────────

interface CrackEffectProps {
  readonly x: number;
  readonly y: number;
  readonly shards: readonly Shard[];
}

const CrackEffect = memo(function CrackEffect({
  x,
  y,
  shards,
}: CrackEffectProps) {
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
    ctx.scale(dpr, dpr);

    const lines = generateCrackLines({ x, y }, w, h);

    // Group lines by level for staggered batch drawing
    const byLevel = new Map<number, CrackLine[]>();
    for (const line of lines) {
      const group = byLevel.get(line.level);
      if (group) {
        group.push(line);
      } else {
        byLevel.set(line.level, [line]);
      }
    }

    const timers: number[] = [];
    for (const [level, group] of byLevel) {
      timers.push(
        window.setTimeout(
          () => drawCrackLines(ctx, group),
          level * LINE_STAGGER_MS,
        ),
      );
    }

    return () => timers.forEach(clearTimeout);
  }, [x, y]);

  const vignetteStyle: React.CSSProperties = {
    background: `radial-gradient(circle at ${x}px ${y}px, transparent 0%, rgba(0,0,0,0.05) 50%, rgba(0,0,0,0.1) 100%)`,
  };

  return (
    <motion.div
      initial={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 1 }}
      className="pointer-events-none fixed inset-0 z-50"
    >
      <canvas
        ref={canvasRef}
        className="absolute inset-0 h-full w-full"
        style={CANVAS_STYLE}
      />

      {shards.map((shard) => (
        <motion.div
          key={shard.index}
          className="absolute h-8 w-8"
          initial={{ x, y, opacity: 0.6, scale: 1, rotate: 0 }}
          animate={{
            x: shard.targetX,
            y: shard.targetY + 200,
            opacity: 0,
            scale: 0.3,
            rotate: shard.rotation,
          }}
          transition={{ duration: 1.5, delay: shard.delay, ease: "easeIn" }}
          style={SHARD_STYLE}
        />
      ))}

      <motion.div
        className="absolute inset-0"
        initial={{ opacity: 0 }}
        animate={{ opacity: [0, 0.08, 0.06] }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.5 }}
        style={vignetteStyle}
      />
    </motion.div>
  );
});

// ─── GunshotMode (public API) ────────────────────────────────────────────────

export function GunshotMode({ active, onShoot }: GunshotModeProps) {
  const [cracks, setCracks] = useState<CrackInstance[]>([]);
  const crosshairRef = useRef<HTMLDivElement>(null);
  const nextId = useRef(0);
  const removalTimers = useRef(new Set<number>());

  // Position crosshair via direct DOM mutation — zero React re-renders
  useEffect(() => {
    if (!active) return;

    const onMove = (e: MouseEvent) => {
      const el = crosshairRef.current;
      if (el) {
        el.style.transform = `translate3d(${e.clientX}px,${e.clientY}px,0)`;
      }
    };

    window.addEventListener("mousemove", onMove, { passive: true });
    return () => window.removeEventListener("mousemove", onMove);
  }, [active]);

  // Flush cracks & timers on deactivation
  useEffect(() => {
    if (!active) {
      setCracks([]);
      removalTimers.current.forEach(clearTimeout);
      removalTimers.current.clear();
    }
  }, [active]);

  // Cleanup on unmount
  useEffect(() => {
    const timers = removalTimers.current;
    return () => timers.forEach(clearTimeout);
  }, []);

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      if (!active) return;

      const id = ++nextId.current;
      const { clientX: x, clientY: y } = e;

      setCracks((prev) => [
        ...prev,
        { id, x, y, shards: generateShards(x, y) },
      ]);
      onShoot?.();

      const timer = window.setTimeout(() => {
        setCracks((prev) => prev.filter((c) => c.id !== id));
        removalTimers.current.delete(timer);
      }, CRACK_LIFETIME_MS);
      removalTimers.current.add(timer);
    },
    [active, onShoot],
  );

  if (!active) return null;

  return (
    <>
      <div
        ref={crosshairRef}
        className="pointer-events-none fixed left-0 top-0 z-60"
        style={CROSSHAIR_STYLE}
      >
        <Crosshair />
      </div>

      <div
        className="fixed inset-0 z-55 cursor-none"
        onClick={handleClick}
      />

      <AnimatePresence>
        {cracks.map(({ id, x, y, shards }) => (
          <CrackEffect key={id} x={x} y={y} shards={shards} />
        ))}
      </AnimatePresence>
    </>
  );
}