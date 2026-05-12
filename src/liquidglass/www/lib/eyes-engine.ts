"use client";

import { DEFAULT_EYE_STATE, type Animation, type EyeState } from "./eyes-types";

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

function lerpColor(a: string, b: string, t: number): string {
  const parse = (hex: string) => {
    const h = hex.replace("#", "");
    return [
      parseInt(h.slice(0, 2), 16),
      parseInt(h.slice(2, 4), 16),
      parseInt(h.slice(4, 6), 16),
    ];
  };
  try {
    const [ar, ag, ab] = parse(a);
    const [br, bg, bb] = parse(b);
    const r = Math.round(lerp(ar, br, t));
    const g = Math.round(lerp(ag, bg, t));
    const bv = Math.round(lerp(ab, bb, t));
    return `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${bv.toString(16).padStart(2, "0")}`;
  } catch {
    return t < 0.5 ? a : b;
  }
}

function easeFunc(type: string, t: number): number {
  switch (type) {
    case "linear": return t;
    case "ease-in": return t * t;
    case "ease-out": return t * (2 - t);
    case "spring": return 1 - Math.cos(t * Math.PI * 1.2) ** 3 * Math.exp(-t * 4);
    default: return t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
  }
}

function mergeState(base: EyeState, partial: Partial<EyeState>): EyeState {
  return { ...base, ...partial };
}

function interpolateEyeState(a: EyeState, b: EyeState, t: number): EyeState {
  return {
    w: lerp(a.w, b.w, t),
    h: lerp(a.h, b.h, t),
    r: typeof a.r === "number" && typeof b.r === "number" ? lerp(a.r as number, b.r as number, t) : (t < 0.5 ? a.r : b.r),
    x: lerp(a.x, b.x, t),
    y: lerp(a.y, b.y, t),
    sx: lerp(a.sx, b.sx, t),
    sy: lerp(a.sy, b.sy, t),
    rot: lerp(a.rot, b.rot, t),
    color: lerpColor(a.color, b.color, t),
  };
}

export interface Frame {
  left: EyeState;
  right: EyeState;
  gap: number;
}

export function computeFrame(anim: Animation, timeMs: number): Frame {
  const kfs = anim.keyframes;
  const t = Math.min(timeMs, anim.duration);

  let prev = kfs[0];
  let next = kfs[kfs.length - 1];
  for (let i = 0; i < kfs.length - 1; i++) {
    if (t >= kfs[i].at && t <= kfs[i + 1].at) {
      prev = kfs[i];
      next = kfs[i + 1];
      break;
    }
  }

  const segDur = next.at - prev.at;
  const rawT = segDur > 0 ? (t - prev.at) / segDur : 1;
  const easedT = easeFunc(next.ease || "ease", rawT);

  const prevEyes = mergeState(DEFAULT_EYE_STATE, prev.eyes || {});
  const nextEyes = mergeState(DEFAULT_EYE_STATE, next.eyes || {});
  const baseInterp = interpolateEyeState(prevEyes, nextEyes, easedT);

  const prevLeft = mergeState(prevEyes, prev.left || {});
  const nextLeft = mergeState(nextEyes, next.left || {});
  const prevRight = mergeState(prevEyes, prev.right || {});
  const nextRight = mergeState(nextEyes, next.right || {});

  const left = prev.left || next.left
    ? interpolateEyeState(prevLeft, nextLeft, easedT)
    : baseInterp;
  const right = prev.right || next.right
    ? interpolateEyeState(prevRight, nextRight, easedT)
    : baseInterp;

  const gap = lerp(prev.gap ?? 60, next.gap ?? 60, easedT);

  return { left, right, gap };
}

export function eyeStateToCSS(state: EyeState): React.CSSProperties {
  return {
    width: state.w,
    height: state.h,
    borderRadius: typeof state.r === "number" ? state.r : state.r,
    transform: `translate(${state.x}px, ${state.y}px) scaleX(${state.sx}) scaleY(${state.sy}) rotate(${state.rot}deg)`,
    backgroundColor: state.color,
  };
}
