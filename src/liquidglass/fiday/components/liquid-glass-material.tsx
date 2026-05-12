"use client";

import {
  type CSSProperties,
  type PointerEvent,
  type ReactNode,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from "react";
import { cn } from "@/liquidglass/fiday/lib/utils";

export type GlassTone = "dark" | "light" | "colorful";
export type GlassShape = "dock" | "pill" | "title" | "control";

type PointerState = {
  active: boolean;
  inside: boolean;
  x: number;
  y: number;
  dx: number;
  dy: number;
};

export type LiquidGlassMaterialProps = {
  aberration?: number;
  active?: boolean;
  ariaLabel?: string;
  blur?: number;
  children: ReactNode;
  className?: string;
  contentClassName?: string;
  debug?: boolean;
  displacement?: number;
  elasticity?: number;
  onClick?: () => void;
  radius?: number;
  saturation?: number;
  shape?: GlassShape;
  style?: CSSProperties;
  tone?: GlassTone;
};

const initialPointer: PointerState = {
  active: false,
  inside: false,
  x: 68,
  y: 24,
  dx: 0,
  dy: 0,
};

function encodeSvg(svg: string) {
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

function buildEdgeDisplacementMap({
  edge,
  height,
  radius,
  x,
  y,
  width,
}: {
  edge: number;
  height: number;
  radius: number;
  x: number;
  y: number;
  width: number;
}) {
  const safeWidth = Math.max(120, Math.round(width));
  const safeHeight = Math.max(64, Math.round(height));
  const safeEdge = Math.max(10, Math.min(edge, Math.min(safeWidth, safeHeight) / 3));
  const safeRadius = Math.max(12, Math.min(radius, Math.min(safeWidth, safeHeight) / 2));
  const innerRadius = Math.max(0, safeRadius - safeEdge * 0.68);
  const innerWidth = Math.max(1, safeWidth - safeEdge * 2);
  const innerHeight = Math.max(1, safeHeight - safeEdge * 2);

  return `
    <svg viewBox="0 0 ${safeWidth} ${safeHeight}" xmlns="http://www.w3.org/2000/svg">
      <defs>
        <linearGradient id="red" x1="100%" y1="8%" x2="0%" y2="92%">
          <stop offset="0%" stop-color="rgb(128,128,128)" stop-opacity="0.1"/>
          <stop offset="36%" stop-color="rgb(255,55,55)" stop-opacity="0.92"/>
          <stop offset="100%" stop-color="rgb(128,128,128)" stop-opacity="0.15"/>
        </linearGradient>
        <linearGradient id="blue" x1="5%" y1="0%" x2="95%" y2="100%">
          <stop offset="0%" stop-color="rgb(128,128,128)" stop-opacity="0.12"/>
          <stop offset="58%" stop-color="rgb(30,85,255)" stop-opacity="0.9"/>
          <stop offset="100%" stop-color="rgb(128,128,128)" stop-opacity="0.18"/>
        </linearGradient>
        <radialGradient id="splash" cx="${x}%" cy="${y}%" r="62%">
          <stop offset="0%" stop-color="rgb(255,255,255)" stop-opacity="0.95"/>
          <stop offset="28%" stop-color="rgb(180,225,255)" stop-opacity="0.5"/>
          <stop offset="100%" stop-color="rgb(128,128,128)" stop-opacity="0"/>
        </radialGradient>
        <filter id="soften" x="-20%" y="-20%" width="140%" height="140%">
          <feGaussianBlur stdDeviation="${Math.max(4, safeEdge * 0.22)}"/>
        </filter>
        <mask id="edgeMask">
          <rect width="${safeWidth}" height="${safeHeight}" rx="${safeRadius}" fill="white"/>
          <rect x="${safeEdge}" y="${safeEdge}" width="${innerWidth}" height="${innerHeight}" rx="${innerRadius}" fill="black" filter="url(#soften)"/>
        </mask>
      </defs>
      <rect width="${safeWidth}" height="${safeHeight}" rx="${safeRadius}" fill="rgb(128,128,128)" fill-opacity="0"/>
      <g mask="url(#edgeMask)">
        <rect width="${safeWidth}" height="${safeHeight}" rx="${safeRadius}" fill="rgb(128,128,128)" fill-opacity="0.34"/>
        <rect width="${safeWidth}" height="${safeHeight}" rx="${safeRadius}" fill="url(#red)"/>
        <rect width="${safeWidth}" height="${safeHeight}" rx="${safeRadius}" fill="url(#blue)" style="mix-blend-mode:screen"/>
        <rect width="${safeWidth}" height="${safeHeight}" rx="${safeRadius}" fill="url(#splash)" style="mix-blend-mode:screen"/>
      </g>
    </svg>
  `;
}

export function LiquidGlassMaterial({
  aberration = 2.2,
  active = false,
  ariaLabel,
  blur = 18,
  children,
  className,
  contentClassName,
  debug = false,
  displacement = 74,
  elasticity = 0.18,
  onClick,
  radius = 34,
  saturation = 152,
  shape = "dock",
  style,
  tone = "dark",
}: LiquidGlassMaterialProps) {
  const rawId = useId();
  const filterId = `lg-${rawId.replace(/[^a-zA-Z0-9_-]/g, "")}`;
  const rootRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 320, height: 96 });
  const [pointer, setPointer] = useState<PointerState>(initialPointer);
  const [reducedMotion, setReducedMotion] = useState(false);

  useEffect(() => {
    const node = rootRef.current;
    if (!node) {
      return;
    }

    const observer = new ResizeObserver(([entry]) => {
      const rect = entry.contentRect;
      setSize({
        width: Math.max(1, rect.width),
        height: Math.max(1, rect.height),
      });
    });

    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    const update = () => setReducedMotion(media.matches);
    update();
    media.addEventListener("change", update);
    return () => media.removeEventListener("change", update);
  }, []);

  const edgeMapSvg = useMemo(
    () =>
      buildEdgeDisplacementMap({
        edge: Math.min(size.width, size.height) * 0.24,
        height: size.height,
        radius,
        width: size.width,
        x: pointer.x,
        y: pointer.y,
      }),
    [pointer.x, pointer.y, radius, size.height, size.width],
  );

  const edgeMapUri = useMemo(() => encodeSvg(edgeMapSvg), [edgeMapSvg]);

  function updatePointer(event: PointerEvent<HTMLDivElement>, activeState?: boolean) {
    const rect = event.currentTarget.getBoundingClientRect();
    const x = ((event.clientX - rect.left) / rect.width) * 100;
    const y = ((event.clientY - rect.top) / rect.height) * 100;
    const dx = x - 50;
    const dy = y - 50;

    setPointer((current) => ({
      active: activeState ?? current.active,
      inside: true,
      x: Math.max(0, Math.min(100, x)),
      y: Math.max(0, Math.min(100, y)),
      dx,
      dy,
    }));
  }

  const isActive = active || pointer.active;
  const isAlive = pointer.inside || isActive;
  const stretch = isAlive && !reducedMotion ? Math.min(1, Math.hypot(pointer.dx, pointer.dy) / 76) : 0;
  const scaleX = 1 + Math.abs(pointer.dx / 100) * elasticity * 0.58 * stretch;
  const scaleY = 1 + Math.abs(pointer.dy / 100) * elasticity * 0.58 * stretch;
  const translateX = !reducedMotion ? pointer.dx * elasticity * 0.1 : 0;
  const translateY = !reducedMotion ? pointer.dy * elasticity * 0.1 : 0;

  const cssVars = {
    "--glass-aberration": aberration.toFixed(2),
    "--glass-active": isActive ? 1 : 0,
    "--glass-blur": `${blur}px`,
    "--glass-elastic-x": `${translateX.toFixed(2)}px`,
    "--glass-elastic-y": `${translateY.toFixed(2)}px`,
    "--glass-radius": `${radius}px`,
    "--glass-saturation": `${saturation}%`,
    "--glass-scale-x": scaleX.toFixed(4),
    "--glass-scale-y": scaleY.toFixed(4),
    "--glass-x": `${pointer.x}%`,
    "--glass-y": `${pointer.y}%`,
    ...style,
  } as CSSProperties;

  const backdropStyle = {
    WebkitBackdropFilter: `blur(var(--glass-blur)) saturate(var(--glass-saturation))`,
    backdropFilter: `blur(var(--glass-blur)) saturate(var(--glass-saturation))`,
    filter: `url(#${filterId})`,
  } as CSSProperties;

  return (
    <div
      aria-label={ariaLabel}
      className={cn("lg-material", className)}
      data-active={isActive}
      data-interactive={Boolean(onClick)}
      data-reduced-motion={reducedMotion}
      data-shape={shape}
      data-tone={tone}
      onBlur={() => setPointer((current) => ({ ...current, active: false, inside: false }))}
      onClick={onClick}
      onFocus={() => setPointer((current) => ({ ...current, active: true, inside: true }))}
      onPointerCancel={() => setPointer((current) => ({ ...current, active: false }))}
      onPointerDown={(event) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        updatePointer(event, true);
      }}
      onPointerEnter={(event) => updatePointer(event)}
      onPointerLeave={() => setPointer((current) => ({ ...current, active: false, inside: false }))}
      onPointerMove={updatePointer}
      onPointerUp={(event) => {
        if (event.currentTarget.hasPointerCapture(event.pointerId)) {
          event.currentTarget.releasePointerCapture(event.pointerId);
        }
        updatePointer(event, false);
      }}
      ref={rootRef}
      role={onClick ? "button" : undefined}
      style={cssVars}
      tabIndex={onClick ? 0 : undefined}
    >
      <svg aria-hidden="true" className="lg-material__filter" focusable="false">
        <defs>
          <filter
            colorInterpolationFilters="sRGB"
            height="170%"
            id={filterId}
            width="170%"
            x="-35%"
            y="-35%"
          >
            <feImage
              height="100%"
              href={edgeMapUri}
              preserveAspectRatio="none"
              result="MAP"
              width="100%"
              x="0"
              y="0"
            />
            <feDisplacementMap
              in="SourceGraphic"
              in2="MAP"
              result="RED_DISPLACED"
              scale={-displacement}
              xChannelSelector="R"
              yChannelSelector="B"
            />
            <feColorMatrix
              in="RED_DISPLACED"
              result="RED_CHANNEL"
              type="matrix"
              values="1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0"
            />
            <feDisplacementMap
              in="SourceGraphic"
              in2="MAP"
              result="GREEN_DISPLACED"
              scale={-displacement + aberration * 2}
              xChannelSelector="R"
              yChannelSelector="B"
            />
            <feColorMatrix
              in="GREEN_DISPLACED"
              result="GREEN_CHANNEL"
              type="matrix"
              values="0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 1 0"
            />
            <feDisplacementMap
              in="SourceGraphic"
              in2="MAP"
              result="BLUE_DISPLACED"
              scale={-displacement + aberration * 4}
              xChannelSelector="R"
              yChannelSelector="B"
            />
            <feColorMatrix
              in="BLUE_DISPLACED"
              result="BLUE_CHANNEL"
              type="matrix"
              values="0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 0 1 0"
            />
            <feBlend in="GREEN_CHANNEL" in2="BLUE_CHANNEL" mode="screen" result="GB" />
            <feBlend in="RED_CHANNEL" in2="GB" mode="screen" result="RGB" />
            <feGaussianBlur in="RGB" result="EDGE_SOFT" stdDeviation={Math.max(0.08, 0.5 - aberration * 0.08)} />
            <feComposite in="EDGE_SOFT" in2="MAP" operator="in" result="EDGE_ONLY" />
            <feComponentTransfer in="MAP" result="INVERTED_MAP">
              <feFuncA tableValues="1 0" type="table" />
            </feComponentTransfer>
            <feComposite in="SourceGraphic" in2="INVERTED_MAP" operator="in" result="CENTER" />
            <feComposite in="EDGE_ONLY" in2="CENTER" operator="over" />
          </filter>
        </defs>
      </svg>

      <div className="lg-material__surface">
        <span className="lg-material__warp" style={backdropStyle} />
        <span className="lg-material__tint" />
        <span className="lg-material__rim lg-material__rim--outer" />
        <span className="lg-material__rim lg-material__rim--inner" />
        <span className="lg-material__caustic" />
        <span className="lg-material__specular" />
        <span className="lg-material__splash" />
        <div className={cn("lg-material__content", contentClassName)}>{children}</div>
      </div>

      {debug ? (
        <div
          className="lg-material__debug-map"
          dangerouslySetInnerHTML={{ __html: edgeMapSvg }}
        />
      ) : null}
    </div>
  );
}
