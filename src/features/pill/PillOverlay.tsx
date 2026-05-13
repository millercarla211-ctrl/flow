import { useLingui } from "@lingui/react/macro";
import { AnimatePresence, motion } from "framer-motion";
import {
  ChevronUp,
  Check,
  Circle,
  Copy,
  FileText,
  Loader2,
  MessageSquareText,
  Pause,
  SendHorizontal,
  Settings,
  WandSparkles,
  X,
} from "lucide-react";
import React, { useRef, useEffect, useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useMachine } from "@xstate/react";
import { useSettings } from "../settings/queries";
import { pillMachine } from "./machine";
import type { PillStatus, StoredSettings } from "../../types";

/* ───────────────────────── Constants ───────────────────────── */

interface GridInfo {
  spacing: number;
  cols: number;
  rows: number;
  offsetX: number;
  offsetY: number;
}

const PILL_WIDTH = 34;
const PILL_HEIGHT = 13;
const IDLE_HANDLE_WIDTH = 24;
const IDLE_HANDLE_HEIGHT = 5;
const RECORDING_WIDTH = 156;
const RECORDING_HEIGHT = 42;
const POLISH_DOCK_WIDTH = 170;
const POLISH_DOCK_HEIGHT = 34;
const HOVER_ENTER_DELAY_MS = 130;
const HOVER_LEAVE_DELAY_MS = 620;
const HOVER_EXPAND_LOCK_MS = 850;
const INTERACTION_LOCK_MS = 1600;
const VOICE_BAR_HEIGHTS = [6, 11, 16, 9, 20, 13, 23, 15, 10, 18, 12, 21];
const POLISH_DOT_OPACITIES = [
  0.28, 0.36, 0.48, 0.62, 0.8, 0.72, 0.56, 0.44, 0.34, 0.26, 0.2, 0.18, 0.16,
];
const DOT_SPACING = 3;
const AUTO_TRANSFORM_PRESET_LABELS: Record<string, string> = {
  polish: "Polish",
  professional: "Professional",
  fix_grammar: "Grammar",
  shorter: "Shorter",
  turn_to_list: "List",
  terminal_command: "Terminal",
  vibe_coding: "Coding",
};
const DOT_RADIUS = {
  base: 0.9,
  icon: 1.2,
  wave: 1.0,
  loader: 1.0,
};

const EXPANDED_WIDTH = 218;
const EXPANDED_HEIGHT = 100;
const EXPANDED_BORDER_RADIUS = 999;
const DORMANT_AFTER_MS = 30_000;
const EXPANDED_TEXT_TOP_FADE =
  "linear-gradient(to bottom, var(--color-bg-primary) 0%, color-mix(in srgb, var(--color-bg-primary) 82%, transparent) 38%, color-mix(in srgb, var(--color-bg-primary) 38%, transparent) 74%, transparent 100%)";

const ICONS = {
  warning: [
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 1, 0, 0],
  ],
};

// Transition timing — fast-start smooth-end (Apple-style spring)
/* ───────────────────────── Color Palette ───────────────────────── */

interface PillColorPalette {
  base: string;
  highlight: string;
  error: string;
}

const FALLBACK_PILL_COLOR_PALETTE: PillColorPalette = {
  base: "40, 40, 40",
  highlight: "255, 255, 255",
  error: "239, 68, 68",
};

const readCssVar = (name: string, fallback: string): string => {
  if (typeof window === "undefined") return fallback;
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
};

const resolvePillColorPalette = (): PillColorPalette => ({
  base: readCssVar("--ui-pill-dot-base-rgb", FALLBACK_PILL_COLOR_PALETTE.base),
  highlight: readCssVar("--ui-pill-dot-highlight-rgb", FALLBACK_PILL_COLOR_PALETTE.highlight),
  error: readCssVar("--ui-pill-dot-error-rgb", FALLBACK_PILL_COLOR_PALETTE.error),
});

/* ───────────────────────── Drawing Helpers ───────────────────────── */

function getMaskOpacity(x: number, y: number, width: number, height: number): number {
  const radius = height / 2;
  const leftCenter = radius;
  const rightCenter = width - radius;
  let distToEdge: number;

  if (x < leftCenter) {
    const dist = Math.sqrt((x - leftCenter) ** 2 + (y - height / 2) ** 2);
    distToEdge = radius - dist;
  } else if (x > rightCenter) {
    const dist = Math.sqrt((x - rightCenter) ** 2 + (y - height / 2) ** 2);
    distToEdge = radius - dist;
  } else {
    distToEdge = Math.min(y, height - y);
  }

  return Math.max(0, Math.min(1, distToEdge / 15));
}

function isIconPixel(
  col: number,
  row: number,
  icon: number[][],
  centerCol: number,
  centerRow: number,
): boolean {
  const iconH = icon.length;
  const iconW = icon[0].length;
  const startCol = centerCol - Math.floor(iconW / 2);
  const startRow = centerRow - Math.floor(iconH / 2);
  const localCol = col - startCol;
  const localRow = row - startRow;

  if (localCol >= 0 && localCol < iconW && localRow >= 0 && localRow < iconH) {
    return icon[localRow][localCol] === 1;
  }
  return false;
}

/** Prepares a canvas for drawing (clears, scales) and calls the render function. */
function renderToCanvas(
  canvas: HTMLCanvasElement | null,
  grid: GridInfo,
  palette: PillColorPalette,
  render: (
    ctx: CanvasRenderingContext2D,
    w: number,
    h: number,
    grid: GridInfo,
    palette: PillColorPalette,
  ) => void,
): void {
  const ctx = canvas?.getContext("2d");
  if (!canvas || !ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const w = canvas.width / dpr;
  const h = canvas.height / dpr;

  ctx.shadowBlur = 0;
  ctx.shadowColor = "transparent";
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  render(ctx, w, h, grid, palette);
}

/* ───────────────────────── Component ───────────────────────── */

interface PillOverlayProps {
  className?: string;
  style?: React.CSSProperties;
  sensitivity?: number;
  decay?: number;
}

interface ExpandedTextSegment {
  key: number;
  text: string;
  isWhitespace: boolean;
}

type TranscriptionCompletePayload = {
  transcript?: string;
  auto_paste?: boolean;
};

type WakeStatusPayload = {
  active?: boolean;
  message?: string;
  command?: string;
  transcript?: string;
  confidence?: number;
  speaker_score?: number;
};

type QuickActionStatus =
  | "copied"
  | "saved"
  | "opened"
  | "pasted"
  | "auto_on"
  | "auto_off"
  | "hidden"
  | "error"
  | null;

type FlowTip = {
  id: string;
  label: string;
  title: string;
  body: string;
  actionLabel: string;
  action: "app_settings";
} | null;

type PasteTextResult = {
  pasted: boolean;
  copied: boolean;
  message: string;
};

function getExpandedTextSegments(text: string): ExpandedTextSegment[] {
  let offset = 0;

  return text
    .split(/(\s+)/)
    .filter((segment) => segment !== "")
    .map((segment) => {
      const key = offset;
      offset += segment.length;
      return {
        key,
        text: segment,
        isWhitespace: /^\s+$/.test(segment),
      };
    });
}

const PillOverlay: React.FC<PillOverlayProps> = ({
  className = "",
  style = {},
  sensitivity = 3,
  decay = 0.85,
}) => {
  const { t } = useLingui();
  const [state, send] = useMachine(pillMachine);
  const pillStatus = state.value as PillStatus;
  const { spectrumBins, lastSpectrumAt, isErrorFlashing, isExpanded, expandedText } = state.context;
  const [isHovering, setIsHovering] = useState(false);
  const [isDormant, setIsDormant] = useState(false);
  const [showFirstTip, setShowFirstTip] = useState(false);
  const [flowTip, setFlowTip] = useState<FlowTip>(null);
  const [lastTranscript, setLastTranscript] = useState("");
  const [quickActionStatus, setQuickActionStatus] = useState<QuickActionStatus>(null);
  const autoTransformQuery = useSettings(
    (settings) => ({
      enabled: settings.auto_transform_enabled,
      presetId: settings.auto_transform_preset_id || "polish",
      llmReady:
        settings.llm_enabled &&
        settings.llm_provider !== "none" &&
        Boolean(settings.llm_model?.trim()),
    }),
    true,
  );

  // Primary canvas (small pill dots)
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<GridInfo>({
    spacing: DOT_SPACING,
    cols: 0,
    rows: 0,
    offsetX: 0,
    offsetY: 0,
  });
  const heightsRef = useRef<number[]>([]);

  // Background canvas (expanded pill dots)
  const bgCanvasRef = useRef<HTMLCanvasElement>(null);
  const bgContainerRef = useRef<HTMLDivElement>(null);
  const bgGridRef = useRef<GridInfo>({
    spacing: DOT_SPACING,
    cols: 0,
    rows: 0,
    offsetX: 0,
    offsetY: 0,
  });
  const bgHeightsRef = useRef<number[]>([]);
  const overlayRootRef = useRef<HTMLDivElement>(null);

  // Animation & audio state
  const animationRef = useRef<number | null>(null);
  const loaderTimeRef = useRef<number>(0);
  const lastCanvasDrawAtRef = useRef<number>(0);
  const colorPaletteRef = useRef<PillColorPalette>(FALLBACK_PILL_COLOR_PALETTE);
  const audioReferenceLevelRef = useRef<number>(0);
  const audioFrameCountRef = useRef<number>(0);
  const quickActionTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const quickStatusTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const flowTipTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const dormantTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hoverEnterTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hoverLeaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hoverLockUntilRef = useRef<number>(0);

  /** Render to both canvases (primary + background). */
  const renderBoth = useCallback(
    (
      render: (
        ctx: CanvasRenderingContext2D,
        w: number,
        h: number,
        grid: GridInfo,
        palette: PillColorPalette,
      ) => void,
    ) => {
      const palette = colorPaletteRef.current;
      renderToCanvas(canvasRef.current, gridRef.current, palette, render);
      renderToCanvas(bgCanvasRef.current, bgGridRef.current, palette, render);
    },
    [],
  );

  const refreshColorPalette = useCallback(() => {
    colorPaletteRef.current = resolvePillColorPalette();
  }, []);

  const clearHoverTimers = useCallback(() => {
    if (hoverEnterTimerRef.current) {
      clearTimeout(hoverEnterTimerRef.current);
      hoverEnterTimerRef.current = null;
    }
    if (hoverLeaveTimerRef.current) {
      clearTimeout(hoverLeaveTimerRef.current);
      hoverLeaveTimerRef.current = null;
    }
  }, []);

  const setHoverIntent = useCallback(
    (hovering: boolean) => {
      clearHoverTimers();
      if (hovering) {
        setIsDormant(false);
        hoverEnterTimerRef.current = setTimeout(() => {
          hoverLockUntilRef.current = Date.now() + HOVER_EXPAND_LOCK_MS;
          setIsHovering(true);
          hoverEnterTimerRef.current = null;
        }, HOVER_ENTER_DELAY_MS);
        return;
      }

      const lockRemaining = Math.max(0, hoverLockUntilRef.current - Date.now());
      const leaveDelay = Math.max(HOVER_LEAVE_DELAY_MS, lockRemaining);
      hoverLeaveTimerRef.current = setTimeout(() => {
        if (overlayRootRef.current?.matches(":hover")) {
          hoverLockUntilRef.current = Date.now() + HOVER_EXPAND_LOCK_MS;
          setIsHovering(true);
        } else {
          setIsHovering(false);
        }
        hoverLeaveTimerRef.current = null;
      }, leaveDelay);
    },
    [clearHoverTimers],
  );

  const keepOverlayInteractive = useCallback(() => {
    clearHoverTimers();
    setIsDormant(false);
    hoverLockUntilRef.current = Date.now() + INTERACTION_LOCK_MS;
    setIsHovering(true);
  }, [clearHoverTimers]);

  useEffect(() => () => clearHoverTimers(), [clearHoverTimers]);

  /* ── Draw: Processing (breathing wave) ── */

  const drawProcessingFrame = useCallback(
    (time: number) => {
      renderBoth((ctx, width, height, { cols, rows, spacing, offsetX, offsetY }, palette) => {
        const speed = 0.0015;
        const waveLength = cols * 0.4;
        const breathe = 0.5 + 0.5 * Math.sin(time * 0.001);

        for (let c = 0; c < cols; c++) {
          for (let r = 0; r < rows; r++) {
            const cx = offsetX + c * spacing + spacing / 2;
            const cy = offsetY + r * spacing + spacing / 2;
            const maskAlpha = getMaskOpacity(cx, cy, width, height);
            if (maskAlpha <= 0.05) continue;

            const distFromCenterY = Math.abs(cy - height / 2);
            const wavePhase = c / waveLength - time * speed;
            const wave = Math.sin(wavePhase * Math.PI * 2) * 0.5 + 0.5;

            const maxRadius = height * 0.4 * (0.6 + 0.4 * breathe);
            const activeRadius = wave * maxRadius;
            const isActive = distFromCenterY < activeRadius;

            ctx.beginPath();
            if (isActive) {
              const edgeFactor = 1 - distFromCenterY / (activeRadius + 0.5);
              const brightness = Math.pow(edgeFactor, 1.5) * (0.7 + 0.3 * wave);
              ctx.fillStyle = `rgba(${palette.highlight}, ${brightness * maskAlpha})`;
              if (brightness > 0.7) {
                ctx.shadowBlur = 3;
                ctx.shadowColor = `rgba(${palette.highlight}, 0.3)`;
              }
              ctx.arc(cx, cy, DOT_RADIUS.loader, 0, Math.PI * 2);
            } else {
              ctx.fillStyle = `rgba(${palette.base}, ${maskAlpha * 0.4})`;
              ctx.arc(cx, cy, DOT_RADIUS.base, 0, Math.PI * 2);
            }
            ctx.fill();

            if (isActive) {
              ctx.shadowBlur = 0;
              ctx.shadowColor = "transparent";
            }
          }
        }
      });
    },
    [renderBoth],
  );

  /* ── Draw: Error (flashing warning icon) ── */

  const drawErrorFrame = useCallback(
    (time: number) => {
      renderBoth((ctx, width, height, { cols, rows, spacing, offsetX, offsetY }, palette) => {
        const centerCol = Math.floor(cols / 2);
        const centerRow = Math.floor(rows / 2);
        const flash = Math.sin(time * 0.02 * Math.PI * 2);
        const intensity = 0.5 + 0.5 * Math.max(0, flash);

        for (let c = 0; c < cols; c++) {
          for (let r = 0; r < rows; r++) {
            const cx = offsetX + c * spacing + spacing / 2;
            const cy = offsetY + r * spacing + spacing / 2;
            const maskAlpha = getMaskOpacity(cx, cy, width, height);
            if (maskAlpha <= 0.05) continue;

            const isIcon = isIconPixel(c, r, ICONS.warning, centerCol, centerRow);

            ctx.beginPath();
            if (isIcon) {
              ctx.fillStyle = `rgba(${palette.error}, ${maskAlpha})`;
              ctx.shadowBlur = 6;
              ctx.shadowColor = `rgba(${palette.error}, 0.6)`;
              ctx.arc(cx, cy, DOT_RADIUS.icon, 0, Math.PI * 2);
            } else {
              ctx.fillStyle = `rgba(${palette.error}, ${intensity * maskAlpha * 0.6})`;
              ctx.shadowBlur = 0;
              ctx.shadowColor = "transparent";
              ctx.arc(cx, cy, DOT_RADIUS.base, 0, Math.PI * 2);
            }
            ctx.fill();
          }
        }
      });
    },
    [renderBoth],
  );

  /* ── Draw: Audio spectrum (listening waveform) ── */

  const drawAudioFrame = useCallback(
    (audioData: Uint8Array) => {
      // Compute audio normalization once (shared across both canvases)
      let normalizationFactor = 1;
      if (audioData.length > 0) {
        let framePeak = 0;
        for (let i = 0; i < audioData.length; i += 4) {
          if (audioData[i] > framePeak) framePeak = audioData[i];
        }

        const SIGNAL_FLOOR = 15;
        const TARGET_PEAK = 200;
        const MIN_REFERENCE = 40;

        audioFrameCountRef.current++;

        if (framePeak > SIGNAL_FLOOR) {
          const frameCount = audioFrameCountRef.current;
          const adaptUp = frameCount < 30 ? 0.3 : 0.1;
          const adaptDown = frameCount < 30 ? 0.05 : 0.005;

          if (framePeak > audioReferenceLevelRef.current) {
            audioReferenceLevelRef.current +=
              (framePeak - audioReferenceLevelRef.current) * adaptUp;
          } else {
            audioReferenceLevelRef.current +=
              (framePeak - audioReferenceLevelRef.current) * adaptDown;
          }
        }

        const effectiveRef = Math.max(audioReferenceLevelRef.current, MIN_REFERENCE);
        normalizationFactor = TARGET_PEAK / effectiveRef;
      }

      // Render to each canvas with its own height array
      const targets = [
        { canvas: canvasRef.current, grid: gridRef.current, heights: heightsRef },
        { canvas: bgCanvasRef.current, grid: bgGridRef.current, heights: bgHeightsRef },
      ] as const;

      const palette = colorPaletteRef.current;

      for (const { canvas, grid, heights } of targets) {
        renderToCanvas(
          canvas,
          grid,
          palette,
          (ctx, width, height, { cols, rows, spacing, offsetX, offsetY }) => {
            const centerCol = Math.floor(cols / 2);

            if (audioData.length > 0) {
              for (let i = 0; i <= centerCol; i++) {
                const distFromCenter = i / centerCol;
                const freqIndex = Math.floor(
                  audioData.length * 0.4 * (distFromCenter * distFromCenter),
                );
                let sample = audioData[freqIndex] || 0;
                if (audioData[freqIndex + 1]) sample = (sample + audioData[freqIndex + 1]) / 2;

                let val = ((sample * normalizationFactor) / 255) * sensitivity;
                if (distFromCenter < 0.2) val *= 1.25;
                val = Math.min(val, 1.0);

                const leftIdx = centerCol - i;
                if (leftIdx >= 0 && leftIdx < cols) {
                  if (val > heights.current[leftIdx]) {
                    heights.current[leftIdx] += (val - heights.current[leftIdx]) * 0.5;
                  } else {
                    heights.current[leftIdx] += (val - heights.current[leftIdx]) * (1 - decay);
                  }
                }

                const rightIdx = centerCol + i;
                if (rightIdx < cols && rightIdx !== leftIdx) {
                  heights.current[rightIdx] = heights.current[leftIdx];
                }
              }
            }

            for (let c = 0; c < cols; c++) {
              const amp = heights.current[c] || 0;
              const activeRadiusPixels = amp * (height * 0.45);

              for (let r = 0; r < rows; r++) {
                const cx = offsetX + c * spacing + spacing / 2;
                const cy = offsetY + r * spacing + spacing / 2;
                const maskAlpha = getMaskOpacity(cx, cy, width, height);
                if (maskAlpha <= 0.05) continue;

                const distFromCenterY = Math.abs(cy - height / 2);
                const isWaveActive =
                  activeRadiusPixels > 0.5 && distFromCenterY < activeRadiusPixels;

                ctx.beginPath();
                if (isWaveActive) {
                  const waveEdgeDist = 1 - distFromCenterY / (activeRadiusPixels + 0.1);
                  const brightness = 0.5 + waveEdgeDist * 0.5;
                  ctx.fillStyle = `rgba(${palette.highlight}, ${brightness * maskAlpha})`;
                  ctx.shadowBlur = brightness > 0.8 ? 4 : 0;
                  ctx.shadowColor =
                    brightness > 0.8 ? `rgba(${palette.highlight}, 0.4)` : "transparent";
                  ctx.arc(cx, cy, DOT_RADIUS.wave, 0, Math.PI * 2);
                } else {
                  ctx.fillStyle = `rgba(${palette.base}, ${maskAlpha})`;
                  ctx.shadowBlur = 0;
                  ctx.shadowColor = "transparent";
                  ctx.arc(cx, cy, DOT_RADIUS.base, 0, Math.PI * 2);
                }
                ctx.fill();
              }
            }
          },
        );
      }
    },
    [decay, sensitivity],
  );

  /* ── Draw: Idle (static base dots) ── */

  const drawBaseDots = useCallback(() => {
    renderBoth((ctx, width, height, { cols, rows, spacing, offsetX, offsetY }, palette) => {
      for (let c = 0; c < cols; c++) {
        for (let r = 0; r < rows; r++) {
          const cx = offsetX + c * spacing + spacing / 2;
          const cy = offsetY + r * spacing + spacing / 2;
          const maskAlpha = getMaskOpacity(cx, cy, width, height);
          if (maskAlpha <= 0.05) continue;

          ctx.beginPath();
          ctx.fillStyle = `rgba(${palette.base}, ${maskAlpha})`;
          ctx.arc(cx, cy, DOT_RADIUS.base, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    });
  }, [renderBoth]);

  /* ── Draw: Static icon (error settled state) ── */

  const drawStaticIcon = useCallback(
    (icon: number[][], color: string, glowColor?: string) => {
      renderBoth((ctx, width, height, { cols, rows, spacing, offsetX, offsetY }, palette) => {
        const centerCol = Math.floor(cols / 2);
        const centerRow = Math.floor(rows / 2);

        for (let c = 0; c < cols; c++) {
          for (let r = 0; r < rows; r++) {
            const cx = offsetX + c * spacing + spacing / 2;
            const cy = offsetY + r * spacing + spacing / 2;
            const maskAlpha = getMaskOpacity(cx, cy, width, height);
            if (maskAlpha <= 0.05) continue;

            const isIcon = isIconPixel(c, r, icon, centerCol, centerRow);

            ctx.beginPath();
            if (isIcon) {
              ctx.fillStyle = `rgba(${color}, ${maskAlpha})`;
              ctx.shadowBlur = glowColor ? 8 : 0;
              ctx.shadowColor = glowColor ? `rgba(${glowColor}, 0.5)` : "transparent";
              ctx.arc(cx, cy, DOT_RADIUS.icon, 0, Math.PI * 2);
            } else {
              ctx.fillStyle = `rgba(${palette.base}, ${maskAlpha})`;
              ctx.shadowBlur = 0;
              ctx.shadowColor = "transparent";
              ctx.arc(cx, cy, DOT_RADIUS.base, 0, Math.PI * 2);
            }
            ctx.fill();
          }
        }
      });
    },
    [renderBoth],
  );

  /* ───────────────────────── Animation Loop ───────────────────────── */

  const stopAllAnimations = useCallback(() => {
    if (animationRef.current) {
      cancelAnimationFrame(animationRef.current);
      animationRef.current = null;
    }
  }, []);

  // Stable refs for the animation loop
  const drawAudioFrameRef = useRef(drawAudioFrame);
  const drawProcessingFrameRef = useRef(drawProcessingFrame);
  const drawErrorFrameRef = useRef(drawErrorFrame);
  const drawBaseDotsRef = useRef(drawBaseDots);
  const drawStaticIconRef = useRef(drawStaticIcon);

  useEffect(() => {
    drawAudioFrameRef.current = drawAudioFrame;
    drawProcessingFrameRef.current = drawProcessingFrame;
    drawErrorFrameRef.current = drawErrorFrame;
    drawBaseDotsRef.current = drawBaseDots;
    drawStaticIconRef.current = drawStaticIcon;
  }, [drawAudioFrame, drawProcessingFrame, drawErrorFrame, drawBaseDots, drawStaticIcon]);

  const spectrumBinsRef = useRef(spectrumBins);
  const lastSpectrumAtRef = useRef(lastSpectrumAt);

  useEffect(() => {
    spectrumBinsRef.current = spectrumBins;
    lastSpectrumAtRef.current = lastSpectrumAt;
  }, [spectrumBins, lastSpectrumAt]);

  useEffect(() => {
    stopAllAnimations();

    if (pillStatus === "idle") {
      drawBaseDotsRef.current();
      return;
    }

    if (pillStatus === "listening") {
      audioReferenceLevelRef.current = 0;
      audioFrameCountRef.current = 0;
      heightsRef.current.fill(0);
    }

    loaderTimeRef.current = 0;
    lastCanvasDrawAtRef.current = 0;
    let animationStartTime: number | null = null;
    const emptySpectrum = new Uint8Array(128);

    const tick = (frameTime: number) => {
      if (animationStartTime === null) {
        animationStartTime = frameTime;
      }
      loaderTimeRef.current = frameTime - animationStartTime;
      const frameInterval = pillStatus === "listening" ? 34 : pillStatus === "processing" ? 50 : 80;
      if (loaderTimeRef.current - lastCanvasDrawAtRef.current < frameInterval) {
        animationRef.current = requestAnimationFrame(tick);
        return;
      }
      lastCanvasDrawAtRef.current = loaderTimeRef.current;

      switch (pillStatus) {
        case "listening": {
          const now = performance.now();
          const audioData =
            now - lastSpectrumAtRef.current > 250 ? emptySpectrum : spectrumBinsRef.current;
          drawAudioFrameRef.current(audioData);
          break;
        }
        case "processing":
          drawProcessingFrameRef.current(loaderTimeRef.current);
          break;
        case "error":
          if (isErrorFlashing) {
            drawErrorFrameRef.current(loaderTimeRef.current);
          }
          break;
      }

      animationRef.current = requestAnimationFrame(tick);
    };

    animationRef.current = requestAnimationFrame(tick);
    return () => stopAllAnimations();
  }, [pillStatus, isErrorFlashing, stopAllAnimations]);

  useEffect(() => {
    if (pillStatus === "error" && !isErrorFlashing) {
      stopAllAnimations();
      const errorColor = colorPaletteRef.current.error;
      drawStaticIconRef.current(ICONS.warning, errorColor, errorColor);
    }
  }, [pillStatus, isErrorFlashing, stopAllAnimations]);

  /* ───────────────────────── Window / Keyboard ───────────────────────── */

  const hideWindow = useCallback(async () => {
    setFlowTip(null);
    try {
      await invoke("cancel_recording");
    } catch (err) {
      console.error("Failed to hide window:", err);
    }
  }, []);

  const toggleRecording = useCallback(
    async (event: React.MouseEvent<HTMLButtonElement>) => {
      event.preventDefault();
      event.stopPropagation();
      keepOverlayInteractive();
      try {
        await invoke("toggle_recording");
      } catch (err) {
        console.error("Failed to toggle recording:", err);
      }
    },
    [keepOverlayInteractive],
  );

  useEffect(() => {
    const storageKey = "flow_pill_first_tip_seen_v1";
    try {
      if (window.localStorage.getItem(storageKey)) {
        return;
      }
      window.localStorage.setItem(storageKey, "1");
      setShowFirstTip(true);
      const timeout = window.setTimeout(() => setShowFirstTip(false), 8000);
      return () => window.clearTimeout(timeout);
    } catch {
      return;
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    const unlisteners: UnlistenFn[] = [];

    const clearQuickTimers = () => {
      if (quickActionTimerRef.current) {
        clearTimeout(quickActionTimerRef.current);
        quickActionTimerRef.current = null;
      }
      if (quickStatusTimerRef.current) {
        clearTimeout(quickStatusTimerRef.current);
        quickStatusTimerRef.current = null;
      }
      if (flowTipTimerRef.current) {
        clearTimeout(flowTipTimerRef.current);
        flowTipTimerRef.current = null;
      }
    };

    const wakeVisible = () => {
      setIsDormant(false);
      if (dormantTimerRef.current) {
        clearTimeout(dormantTimerRef.current);
        dormantTimerRef.current = null;
      }
    };

    listen<TranscriptionCompletePayload>("transcription:complete", (event) => {
      if (cancelled) return;
      const transcript = event.payload?.transcript?.trim() ?? "";
      if (!transcript) {
        return;
      }
      clearQuickTimers();
      wakeVisible();
      setLastTranscript(transcript);
      setQuickActionStatus(null);
      quickActionTimerRef.current = setTimeout(() => {
        setLastTranscript("");
        setQuickActionStatus(null);
        setFlowTip(null);
        quickActionTimerRef.current = null;
      }, 15000);
    }).then((fn) => {
      if (cancelled) fn();
      else unlisteners.push(fn);
    });

    listen("recording:start", () => {
      if (cancelled) return;
      clearQuickTimers();
      wakeVisible();
      setLastTranscript("");
      setQuickActionStatus(null);
      setFlowTip(null);
    }).then((fn) => {
      if (cancelled) fn();
      else unlisteners.push(fn);
    });

    listen<WakeStatusPayload>("wake:status", (event) => {
      if (cancelled || !event.payload?.active) return;
      wakeVisible();
    }).then((fn) => {
      if (cancelled) fn();
      else unlisteners.push(fn);
    });

    return () => {
      cancelled = true;
      clearQuickTimers();
      if (dormantTimerRef.current) {
        clearTimeout(dormantTimerRef.current);
        dormantTimerRef.current = null;
      }
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (dormantTimerRef.current) {
      clearTimeout(dormantTimerRef.current);
      dormantTimerRef.current = null;
    }

    const hasFollowUpContent = Boolean(lastTranscript.trim());
    if (pillStatus !== "idle" || isHovering || isExpanded || hasFollowUpContent || showFirstTip) {
      setIsDormant(false);
      return;
    }

    dormantTimerRef.current = setTimeout(() => {
      setIsDormant(true);
      dormantTimerRef.current = null;
    }, DORMANT_AFTER_MS);

    return () => {
      if (dormantTimerRef.current) {
        clearTimeout(dormantTimerRef.current);
        dormantTimerRef.current = null;
      }
    };
  }, [isExpanded, isHovering, lastTranscript, pillStatus, showFirstTip]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && pillStatus === "error") {
        e.preventDefault();
        send({ type: "DISMISS" });
        hideWindow();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [pillStatus, send, hideWindow]);

  /* ───────────────────────── Canvas Setup ───────────────────────── */

  const setupCanvas = useCallback(() => {
    refreshColorPalette();
    const dpr = window.devicePixelRatio || 1;

    const setupSingle = (
      container: HTMLDivElement | null,
      canvas: HTMLCanvasElement | null,
      gRef: React.MutableRefObject<GridInfo>,
      hRef: React.MutableRefObject<number[]>,
    ) => {
      if (!canvas || !container) return;
      const rect = container.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) return;

      canvas.width = rect.width * dpr;
      canvas.height = rect.height * dpr;
      canvas.style.width = `${rect.width}px`;
      canvas.style.height = `${rect.height}px`;

      const ctx = canvas.getContext("2d");
      if (ctx) ctx.scale(dpr, dpr);

      const cols = Math.floor(rect.width / DOT_SPACING);
      const rows = Math.floor(rect.height / DOT_SPACING);
      gRef.current = {
        spacing: DOT_SPACING,
        cols,
        rows,
        offsetX: (rect.width - cols * DOT_SPACING) / 2,
        offsetY: (rect.height - rows * DOT_SPACING) / 2,
      };

      if (hRef.current.length !== cols) {
        hRef.current = new Array(cols).fill(0);
      }
    };

    setupSingle(containerRef.current, canvasRef.current, gridRef, heightsRef);
    setupSingle(bgContainerRef.current, bgCanvasRef.current, bgGridRef, bgHeightsRef);

    if (pillStatus === "idle") {
      drawBaseDotsRef.current();
    }
  }, [drawBaseDots, refreshColorPalette, pillStatus]);

  useEffect(() => {
    refreshColorPalette();
  }, [refreshColorPalette]);

  useEffect(() => {
    const observer = new ResizeObserver(setupCanvas);
    if (containerRef.current) observer.observe(containerRef.current);
    if (bgContainerRef.current) observer.observe(bgContainerRef.current);
    setupCanvas();
    return () => observer.disconnect();
  }, [setupCanvas]);

  /* ───────────────────────── Render ───────────────────────── */

  const getStatusMessage = (s: PillStatus) => {
    switch (s) {
      case "listening":
        return t({
          id: "pill.status.listening",
          message: "Listening...",
        });
      case "processing":
        return t({
          id: "pill.status.processing",
          message: "Processing...",
        });
      case "error":
        return t({
          id: "pill.status.error",
          message: "Error occurred",
        });
      default:
        return "";
    }
  };

  const statusMessage = getStatusMessage(pillStatus);
  const idleMessage = t({ id: "pill.status.ready", message: "Ready" });
  const visibleStatusMessage = statusMessage || idleMessage;
  const isActiveStatus = pillStatus !== "idle";
  const hasTranscriptPreview = isExpanded && (expandedText || statusMessage).trim().length > 0;
  const hasFollowUpContent = Boolean(lastTranscript.trim());
  const overlayDormant =
    isDormant && !isHovering && !isActiveStatus && !isExpanded && !hasFollowUpContent;
  const showRecordingUi = !overlayDormant && (isHovering || isActiveStatus || isExpanded);
  const showPolishDock = showRecordingUi && !isActiveStatus && !hasTranscriptPreview;
  const shellWidth = hasTranscriptPreview
    ? EXPANDED_WIDTH
    : showPolishDock
      ? POLISH_DOCK_WIDTH
      : showRecordingUi
        ? RECORDING_WIDTH
        : PILL_WIDTH;
  const shellHeight = hasTranscriptPreview
    ? EXPANDED_HEIGHT
    : showPolishDock
      ? POLISH_DOCK_HEIGHT
      : showRecordingUi
        ? RECORDING_HEIGHT
        : PILL_HEIGHT;
  const shellRadius = EXPANDED_BORDER_RADIUS;
  const displayText = expandedText || visibleStatusMessage;
  const transcriptAvailable = Boolean(lastTranscript.trim());
  const autoTransformEnabled = autoTransformQuery.data?.enabled ?? false;
  const autoTransformPresetId = autoTransformQuery.data?.presetId ?? "polish";
  const autoTransformLabel = AUTO_TRANSFORM_PRESET_LABELS[autoTransformPresetId] ?? "Transform";
  const autoTransformReady = autoTransformQuery.data?.llmReady ?? false;
  const transformVerb = (autoTransformLabel || "Polish").toLowerCase();
  const polishShortcutLabel = "Win Alt 1";
  const showAutoTransformControl =
    showRecordingUi && !isActiveStatus && Boolean(autoTransformQuery.data);
  const showFlowTip = false;
  const showPolishTip = false;
  const showQuickActions =
    !showPolishDock &&
    !isActiveStatus &&
    !showFlowTip &&
    (transcriptAvailable || showAutoTransformControl);
  const showFirstUseTip =
    showFirstTip &&
    !isActiveStatus &&
    !hasTranscriptPreview &&
    !overlayDormant &&
    !showFlowTip &&
    !showQuickActions &&
    !showPolishTip;
  const showIdleHandle = !overlayDormant && !showRecordingUi && !hasTranscriptPreview;
  const bubbleTitle =
    pillStatus === "error"
      ? t({ id: "pill.action.dismiss", message: "Dismiss" })
      : t({ id: "pill.action.stop", message: "Stop dictation" });
  const springTransition = {
    type: "spring" as const,
    stiffness: 520,
    damping: 38,
    mass: 0.7,
  };
  const softSpringTransition = {
    type: "spring" as const,
    stiffness: 360,
    damping: 32,
    mass: 0.8,
  };
  const bgOpacityTransition = hasTranscriptPreview
    ? "opacity 0.6s ease 0.12s"
    : "opacity 0.25s ease";
  const darkIdleShell = "var(--border)";
  const darkIdleBorder = "var(--color-bg-primary)";
  const darkIdleHandle = "var(--color-border-secondary)";
  const quickStatusLabel =
    quickActionStatus === "copied"
      ? "Copied"
      : quickActionStatus === "saved"
        ? "Saved"
        : quickActionStatus === "opened"
          ? "Opened"
          : quickActionStatus === "pasted"
            ? "Pasted"
            : quickActionStatus === "auto_on"
              ? "Auto on"
              : quickActionStatus === "auto_off"
                ? "Auto off"
                : quickActionStatus === "hidden"
                  ? "Hidden"
                  : quickActionStatus === "error"
                    ? "Try again"
                    : "";

  const flashQuickStatus = (_status: QuickActionStatus) => {
    setQuickActionStatus(null);
    if (quickStatusTimerRef.current) {
      clearTimeout(quickStatusTimerRef.current);
      quickStatusTimerRef.current = null;
    }
  };

  const handleBubbleAction = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    keepOverlayInteractive();
    if (pillStatus === "error") {
      send({ type: "DISMISS" });
    }
    await hideWindow();
  };

  const dismissFlowTip = (event?: React.MouseEvent<HTMLButtonElement>) => {
    event?.preventDefault();
    event?.stopPropagation();
    if (flowTipTimerRef.current) {
      clearTimeout(flowTipTimerRef.current);
      flowTipTimerRef.current = null;
    }
    setFlowTip(null);
  };

  const handleFlowTipAction = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    const action = flowTip?.action;
    dismissFlowTip();

    if (action === "app_settings") {
      try {
        await invoke("open_app_settings");
      } catch (error) {
        console.error("Failed to open app settings:", error);
      }
    }
  };

  const handleCopyLastTranscript = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    keepOverlayInteractive();
    const text = lastTranscript.trim();
    if (!text) return;

    try {
      await invoke("copy_last_transcript");
      flashQuickStatus("copied");
    } catch (error) {
      console.error("Failed to copy last transcript:", error);
      flashQuickStatus("error");
    }
  };

  const handlePasteLastTranscript = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    keepOverlayInteractive();
    if (!lastTranscript.trim()) return;

    try {
      const result = await invoke<PasteTextResult>("paste_last_transcript");
      flashQuickStatus(result.copied ? "copied" : "pasted");
    } catch (error) {
      console.error("Failed to paste last transcript:", error);
      flashQuickStatus("error");
    }
  };

  const handleSaveLastTranscript = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    keepOverlayInteractive();
    const text = lastTranscript.trim();
    if (!text) return;

    try {
      await invoke("create_scratchpad_entry", { body: text, source: "overlay" });
      flashQuickStatus("saved");
    } catch (error) {
      console.error("Failed to save last transcript:", error);
      flashQuickStatus("error");
    }
  };

  const handleTransformLastTranscript = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    keepOverlayInteractive();
    const text = lastTranscript.trim();
    if (!text) return;

    try {
      await invoke("open_transforms_view", { text });
      flashQuickStatus("opened");
    } catch (error) {
      console.error("Failed to open transform view:", error);
      flashQuickStatus("error");
    }
  };

  const handleToggleAutoTransform = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    keepOverlayInteractive();

    const nextEnabled = !autoTransformEnabled;
    if (nextEnabled && !autoTransformReady) {
      try {
        await invoke("open_llm_cleanup_settings");
        flashQuickStatus("opened");
      } catch (error) {
        console.error("Failed to open model settings:", error);
        flashQuickStatus("error");
      }
      return;
    }

    try {
      await invoke<StoredSettings>("set_auto_transform_setting", {
        enabled: nextEnabled,
        presetId: autoTransformPresetId,
      });
      flashQuickStatus(nextEnabled ? "auto_on" : "auto_off");
    } catch (error) {
      console.error("Failed to toggle Auto Transform:", error);
      flashQuickStatus("error");
    }
  };

  const handlePauseFlow = useCallback(async () => {
    setIsDormant(false);
    setIsHovering(false);
    flashQuickStatus("hidden");
    try {
      await invoke("pause_flow_temporarily", { seconds: 300 });
    } catch (error) {
      console.error("Failed to pause Friday:", error);
      flashQuickStatus("error");
    }
  }, []);

  useEffect(() => {
    const needsTipFrame = showFlowTip || showPolishTip || showQuickActions;
    invoke("set_pill_overlay_tip_frame", { expanded: needsTipFrame }).catch((error) => {
      console.error("Failed to resize pill overlay for tip:", error);
    });

    return () => {
      if (needsTipFrame) {
        invoke("set_pill_overlay_tip_frame", { expanded: false }).catch(() => {});
      }
    };
  }, [showFlowTip, showPolishTip, showQuickActions]);

  const renderRecordIcon = () => {
    if (pillStatus === "processing") {
      return <Loader2 size={14} className="animate-spin" aria-hidden="true" />;
    }
    if (pillStatus === "listening" || pillStatus === "error") {
      return <Pause size={14} aria-hidden="true" />;
    }
    return <Circle size={12} fill="currentColor" strokeWidth={0} aria-hidden="true" />;
  };

  return (
    <div
      ref={overlayRootRef}
      className={`relative w-full h-full flex flex-col justify-end select-none ${className}`}
      style={style}
      onContextMenu={(e) => {
        e.preventDefault();
        void handlePauseFlow();
      }}
      onMouseEnter={() => {
        setIsDormant(false);
        setHoverIntent(true);
      }}
      onMouseLeave={() => setHoverIntent(false)}
      onFocus={() => {
        setIsDormant(false);
        setHoverIntent(true);
      }}
      onBlur={() => setHoverIntent(false)}
    >
      <div className="sr-only" role="status" aria-live="polite">
        {visibleStatusMessage}
      </div>
      <div className="relative flex items-end justify-center pb-[7px]">
        <AnimatePresence>
          {showFlowTip && flowTip && (
            <motion.div
              key={flowTip.id}
              initial={{ opacity: 0, y: 12, scale: 0.94, filter: "blur(6px)" }}
              animate={{ opacity: 1, y: 0, scale: 1, filter: "blur(0px)" }}
              exit={{ opacity: 0, y: 8, scale: 0.97, filter: "blur(5px)" }}
              transition={softSpringTransition}
              className="absolute bottom-[54px] z-40 w-[300px] rounded-[28px] border p-5"
              style={{
                color: "#ffffff",
                backgroundColor: "rgba(0,0,0,0.96)",
                borderColor: "rgba(255,255,255,0.08)",
                boxShadow:
                  "0 24px 70px rgba(0,0,0,0.58), 0 0 0 1px rgba(255,255,255,0.04), inset 0 1px 0 rgba(255,255,255,0.08)",
                backdropFilter: "blur(22px)",
                fontFamily: "var(--font-sans), Inter, ui-sans-serif, system-ui, sans-serif",
              }}
            >
              <button
                type="button"
                aria-label="Close tip"
                onClick={dismissFlowTip}
                className="absolute right-4 top-4 flex h-8 w-8 items-center justify-center rounded-full border"
                style={{
                  color: "rgba(255,255,255,0.78)",
                  backgroundColor: "rgba(255,255,255,0.02)",
                  borderColor: "rgba(255,255,255,0.12)",
                }}
              >
                <X size={16} aria-hidden="true" />
              </button>

              <div
                className="inline-flex h-7 items-center gap-1.5 rounded-md px-2.5"
                style={{
                  color: "#1d1128",
                  backgroundColor: "#eac7ff",
                  fontSize: 12,
                  fontWeight: 650,
                  letterSpacing: 0,
                }}
              >
                <Settings size={12} aria-hidden="true" />
                {flowTip.label}
              </div>
              <div className="mt-4 pr-8">
                <h3 className="text-[15px] font-semibold leading-5 tracking-normal">
                  {flowTip.title}
                </h3>
                <p className="mt-2 text-[13px] font-medium leading-5 tracking-normal text-white/58">
                  {flowTip.body}
                </p>
              </div>
              <div className="mt-5 flex justify-end">
                <button
                  type="button"
                  onClick={handleFlowTipAction}
                  className="h-10 rounded-xl px-5 text-[13px] font-semibold tracking-normal"
                  style={{
                    color: "#1b1b1b",
                    backgroundColor: "#f6f1ea",
                    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.7)",
                  }}
                >
                  {flowTip.actionLabel}
                </button>
              </div>
            </motion.div>
          )}
          {showPolishTip && (
            <motion.div
              key="polish-tip"
              initial={{ opacity: 0, y: 8, scale: 0.96, filter: "blur(5px)" }}
              animate={{ opacity: 1, y: 0, scale: 1, filter: "blur(0px)" }}
              exit={{ opacity: 0, y: 5, scale: 0.98, filter: "blur(4px)" }}
              transition={softSpringTransition}
              className="absolute bottom-[46px] z-30 whitespace-nowrap rounded-full px-4 py-2 text-center"
              style={{
                color: "#ffffff",
                backgroundColor: "rgba(0,0,0,0.94)",
                boxShadow: "0 16px 42px rgba(0,0,0,0.45), inset 0 1px 0 rgba(255,255,255,0.08)",
                backdropFilter: "blur(18px)",
                fontFamily: "var(--font-mono), 'JetBrains Mono', ui-monospace, monospace",
                fontSize: 13,
                fontWeight: 500,
                lineHeight: "17px",
                letterSpacing: 0,
              }}
            >
              Click or press {polishShortcutLabel} to {transformVerb}
            </motion.div>
          )}
          {showQuickActions && (
            <motion.div
              key="quick-actions"
              initial={{ opacity: 0, y: 10, scale: 0.95, filter: "blur(4px)" }}
              animate={{ opacity: 1, y: 0, scale: 1, filter: "blur(0px)" }}
              exit={{ opacity: 0, y: 7, scale: 0.97, filter: "blur(4px)" }}
              transition={softSpringTransition}
              className="absolute bottom-[36px] z-20 flex items-center gap-1 rounded-full border px-1.5 py-1"
              onPointerEnter={keepOverlayInteractive}
              onPointerDownCapture={keepOverlayInteractive}
              style={{
                color: "var(--color-text-primary)",
                backgroundColor: "color-mix(in srgb, var(--color-bg-primary) 94%, transparent)",
                borderColor: "var(--ui-pill-shell-border)",
                boxShadow: "0 14px 45px rgba(0,0,0,0.36), inset 0 1px 0 rgba(255,255,255,0.08)",
                backdropFilter: "blur(18px)",
              }}
            >
              {quickActionStatus ? (
                <span
                  className="flex h-7 items-center gap-1.5 rounded-full px-2.5"
                  style={{
                    color:
                      quickActionStatus === "error"
                        ? "var(--color-error)"
                        : "var(--color-text-primary)",
                    fontFamily: "var(--font-mono), 'JetBrains Mono', ui-monospace, monospace",
                    fontSize: 11,
                    letterSpacing: 0,
                  }}
                >
                  {quickActionStatus === "error" ? (
                    <X size={12} aria-hidden="true" />
                  ) : (
                    <Check size={12} aria-hidden="true" />
                  )}
                  {quickStatusLabel}
                </span>
              ) : (
                <>
                  {showAutoTransformControl && (
                    <button
                      type="button"
                      aria-label={
                        autoTransformEnabled
                          ? `Disable Auto Transform (${autoTransformLabel})`
                          : `Enable Auto Transform (${autoTransformLabel})`
                      }
                      title={
                        autoTransformEnabled
                          ? `Auto Transform on: ${autoTransformLabel}`
                          : autoTransformReady
                            ? `Auto Transform off: ${autoTransformLabel}`
                            : "Configure text enhancement to use Auto Transform"
                      }
                      onClick={handleToggleAutoTransform}
                      className="flex h-7 items-center gap-1.5 rounded-full border px-2.5"
                      style={{
                        backgroundColor: autoTransformEnabled
                          ? "color-mix(in srgb, var(--surface-interactive-strong) 86%, transparent)"
                          : "color-mix(in srgb, var(--color-bg-secondary) 88%, transparent)",
                        borderColor: autoTransformEnabled
                          ? "var(--color-border-hover)"
                          : "var(--ui-pill-shell-border)",
                        color: autoTransformEnabled
                          ? "var(--color-text-primary)"
                          : "var(--color-text-secondary)",
                        boxShadow: "inset 0 1px 0 rgba(255,255,255,0.1)",
                        fontFamily: "var(--font-mono), 'JetBrains Mono', ui-monospace, monospace",
                        fontSize: 10,
                        letterSpacing: 0,
                      }}
                    >
                      <WandSparkles size={12} aria-hidden="true" />
                      <span>{autoTransformEnabled ? autoTransformLabel : "Auto"}</span>
                    </button>
                  )}
                  {transcriptAvailable && (
                    <>
                      <button
                        type="button"
                        aria-label="Paste transcript"
                        title="Paste transcript"
                        onClick={handlePasteLastTranscript}
                        className="flex h-7 w-7 items-center justify-center rounded-full border"
                        style={{
                          backgroundColor:
                            "color-mix(in srgb, var(--color-bg-secondary) 88%, transparent)",
                          borderColor: "var(--ui-pill-shell-border)",
                          boxShadow: "inset 0 1px 0 rgba(255,255,255,0.1)",
                        }}
                      >
                        <SendHorizontal size={13} aria-hidden="true" />
                      </button>
                      <button
                        type="button"
                        aria-label="Copy transcript"
                        title="Copy transcript"
                        onClick={handleCopyLastTranscript}
                        className="flex h-7 w-7 items-center justify-center rounded-full border"
                        style={{
                          backgroundColor:
                            "color-mix(in srgb, var(--color-bg-secondary) 88%, transparent)",
                          borderColor: "var(--ui-pill-shell-border)",
                          boxShadow: "inset 0 1px 0 rgba(255,255,255,0.1)",
                        }}
                      >
                        <Copy size={13} aria-hidden="true" />
                      </button>
                      <button
                        type="button"
                        aria-label="Save transcript"
                        title="Save transcript"
                        onClick={handleSaveLastTranscript}
                        className="flex h-7 w-7 items-center justify-center rounded-full border"
                        style={{
                          backgroundColor:
                            "color-mix(in srgb, var(--color-bg-secondary) 88%, transparent)",
                          borderColor: "var(--ui-pill-shell-border)",
                          boxShadow: "inset 0 1px 0 rgba(255,255,255,0.1)",
                        }}
                      >
                        <FileText size={13} aria-hidden="true" />
                      </button>
                      <button
                        type="button"
                        aria-label="Transform transcript"
                        title="Transform transcript"
                        onClick={handleTransformLastTranscript}
                        className="flex h-7 w-7 items-center justify-center rounded-full border"
                        style={{
                          backgroundColor:
                            "color-mix(in srgb, var(--color-bg-secondary) 88%, transparent)",
                          borderColor: "var(--ui-pill-shell-border)",
                          boxShadow: "inset 0 1px 0 rgba(255,255,255,0.1)",
                        }}
                      >
                        <WandSparkles size={13} aria-hidden="true" />
                      </button>
                    </>
                  )}
                </>
              )}
            </motion.div>
          )}
          {showFirstUseTip && (
            <motion.div
              key="first-tip"
              initial={{ opacity: 0, y: 10, scale: 0.96, filter: "blur(4px)" }}
              animate={{ opacity: 1, y: 0, scale: 1, filter: "blur(0px)" }}
              exit={{ opacity: 0, y: 6, scale: 0.98, filter: "blur(4px)" }}
              transition={softSpringTransition}
              className="absolute bottom-[56px] max-w-[274px] rounded-full border px-3 py-2 text-center"
              style={{
                color: "var(--color-text-secondary)",
                backgroundColor: "color-mix(in srgb, var(--color-bg-primary) 94%, transparent)",
                borderColor: "var(--ui-pill-shell-border)",
                boxShadow: "0 14px 45px rgba(0,0,0,0.36), inset 0 1px 0 rgba(255,255,255,0.08)",
                backdropFilter: "blur(18px)",
                fontFamily: "var(--font-mono), 'JetBrains Mono', ui-monospace, monospace",
                fontSize: 11,
                lineHeight: "16px",
                letterSpacing: 0,
              }}
            >
              Hold your shortcut to dictate. Hover this line for controls.
            </motion.div>
          )}
        </AnimatePresence>
        {/* Pill shell — transitions between basic and dynamic modes */}
        <motion.div
          className={`relative overflow-hidden flex flex-col ${isErrorFlashing ? "animate-shake" : ""}`}
          onPointerEnter={keepOverlayInteractive}
          onPointerDownCapture={keepOverlayInteractive}
          animate={{
            width: shellWidth,
            height: shellHeight,
            borderRadius: shellRadius,
            opacity: overlayDormant ? 0 : 1,
            scale: overlayDormant ? 0.74 : isHovering && !isActiveStatus ? 1.02 : 1,
            y: overlayDormant ? 8 : 0,
            filter: overlayDormant ? "blur(6px)" : "blur(0px)",
          }}
          transition={springTransition}
          style={{
            backgroundColor: showPolishDock
              ? "transparent"
              : showRecordingUi || hasTranscriptPreview
                ? "color-mix(in srgb, var(--color-bg-primary) 92%, transparent)"
                : darkIdleShell,
            borderColor: showPolishDock
              ? "transparent"
              : showRecordingUi || hasTranscriptPreview
                ? "var(--ui-pill-shell-border)"
                : darkIdleBorder,
            borderWidth: 1,
            borderStyle: "solid",
            boxShadow: showPolishDock
              ? "none"
              : showRecordingUi || hasTranscriptPreview
                ? "0 18px 55px rgba(0,0,0,0.42), 0 0 0 1px rgba(255,255,255,0.04), inset 0 1px 0 rgba(255,255,255,0.08)"
                : "0 10px 24px rgba(0,0,0,0.62), 0 0 10px color-mix(in srgb, var(--border) 34%, transparent), inset 0 1px 0 rgba(255,255,255,0.035)",
            backdropFilter: showPolishDock ? "none" : "blur(18px)",
            padding: showIdleHandle ? 4 : 0,
          }}
        >
          <motion.div
            aria-hidden="true"
            className="absolute left-1/2 top-1/2 z-[4] -translate-x-1/2 -translate-y-1/2 rounded-full"
            animate={{
              opacity: showIdleHandle ? 1 : 0,
              scale: showIdleHandle ? 1 : 0.72,
              width: IDLE_HANDLE_WIDTH,
              height: IDLE_HANDLE_HEIGHT,
            }}
            transition={softSpringTransition}
            style={{
              backgroundColor: darkIdleHandle,
              boxShadow:
                "0 0 10px color-mix(in srgb, var(--color-border-secondary) 42%, transparent)",
            }}
          />
          <div
            aria-hidden="true"
            className="absolute inset-0 pointer-events-none"
            style={{
              background:
                "radial-gradient(circle at 50% 0%, rgba(255,255,255,0.11), transparent 45%), linear-gradient(180deg, rgba(255,255,255,0.06), transparent 58%)",
            }}
          />
          {/* Expanded content area — positioned above background dots */}
          <div
            className="pill-expanded-content relative z-10"
            style={{
              flex: hasTranscriptPreview ? 1 : 0,
              opacity: hasTranscriptPreview ? 1 : 0,
              overflow: "hidden",
              minHeight: 0,
              transition: "opacity 0.35s ease 0.12s, flex 0.5s cubic-bezier(0.32, 0.72, 0, 1)",
            }}
          >
            <div
              className="h-full w-full flex flex-col"
              style={{
                padding: hasTranscriptPreview ? "14px 16px 56px" : "0 16px",
                transition: "padding 0.5s cubic-bezier(0.32, 0.72, 0, 1)",
                position: "relative",
              }}
            >
              <div
                aria-hidden="true"
                className="absolute left-0 right-0 top-0 pointer-events-none z-20"
                style={{
                  height: 30,
                  background: EXPANDED_TEXT_TOP_FADE,
                  opacity: hasTranscriptPreview ? 1 : 0,
                  transition: "opacity 0.24s ease",
                }}
              />

              <div className="flex-1 w-full overflow-hidden flex flex-col justify-end relative z-10">
                <motion.div layout="position" className="w-full flex flex-col justify-end">
                  <p
                    style={{
                      margin: 0,
                      fontSize: "12px",
                      lineHeight: "1.5",
                      fontFamily: "var(--font-mono), 'JetBrains Mono', ui-monospace, monospace",
                      color: "var(--color-text-primary)",
                      fontWeight: 400,
                      letterSpacing: 0,
                      textAlign: "left",
                      width: "100%",
                      wordBreak: "break-word",
                    }}
                  >
                    {displayText
                      ? getExpandedTextSegments(displayText).map(({ key, text, isWhitespace }) => {
                          if (isWhitespace) {
                            return (
                              <motion.span
                                key={key}
                                layout="position"
                                transition={{
                                  layout: { type: "spring", bounce: 0, duration: 0.4 },
                                }}
                                style={{ display: "inline-block", whiteSpace: "pre" }}
                              >
                                {text}
                              </motion.span>
                            );
                          }
                          return (
                            <motion.span
                              key={key}
                              layout="position"
                              initial={{ opacity: 0, filter: "blur(4px)", y: 8 }}
                              animate={{ opacity: 1, filter: "blur(0px)", y: 0 }}
                              transition={{
                                opacity: { duration: 0.4, ease: "easeOut" },
                                filter: { duration: 0.4, ease: "easeOut" },
                                y: { duration: 0.4, ease: "easeOut" },
                                layout: { type: "spring", bounce: 0, duration: 0.4 },
                              }}
                              style={{
                                display: "inline-block",
                                willChange: "transform, opacity, filter",
                              }}
                            >
                              {text}
                            </motion.span>
                          );
                        })
                      : null}
                  </p>
                </motion.div>
              </div>
            </div>
          </div>

          {/* Background canvas — full expanded size, fades in at low opacity */}
          <div
            className="absolute inset-0 pointer-events-none flex items-center justify-center z-[1]"
            style={{
              opacity: hasTranscriptPreview ? 0.08 : 0,
              transition: bgOpacityTransition,
            }}
          >
            <div
              ref={bgContainerRef}
              className="relative overflow-hidden rounded-[inherit]"
              style={{ width: EXPANDED_WIDTH, height: EXPANDED_HEIGHT }}
            >
              <canvas
                ref={bgCanvasRef}
                className="absolute inset-0 w-full h-full block"
                role="img"
                aria-label={t({
                  id: "pill.background_visualizer",
                  message: "Background audio visualizer",
                })}
              />
            </div>
          </div>

          {/* Primary canvas — small pill dots, fades out when expanded */}
          <div className="absolute bottom-4 left-14 h-[22px] w-[74px] pointer-events-none overflow-hidden rounded-full opacity-0">
            <div
              ref={containerRef}
              className="relative overflow-hidden rounded-full"
              style={{ width: 74, height: 22 }}
            >
              <canvas
                ref={canvasRef}
                className="absolute inset-0 w-full h-full block"
                role="img"
                aria-label={t({
                  id: "pill.visualizer",
                  message: "Audio visualizer",
                })}
              />
            </div>
          </div>
          <motion.div
            className={`absolute inset-x-0 bottom-0 z-[3] flex h-full items-center ${
              showPolishDock ? "gap-1.5 px-[2px]" : "gap-2 px-[5px]"
            }`}
            onPointerEnter={keepOverlayInteractive}
            onPointerDownCapture={keepOverlayInteractive}
            animate={{
              opacity: showRecordingUi || isActiveStatus ? 1 : 0,
              y: showRecordingUi || isActiveStatus ? 0 : 4,
            }}
            transition={softSpringTransition}
            style={{ pointerEvents: showRecordingUi || isActiveStatus ? "auto" : "none" }}
          >
            {showPolishDock ? (
              <>
                <motion.button
                  type="button"
                  aria-label={`Start dictation and ${transformVerb}`}
                  title={`Click or press ${polishShortcutLabel} to ${transformVerb}`}
                  onClick={toggleRecording}
                  whileHover={{ scale: 1.035 }}
                  whileTap={{ scale: 0.94 }}
                  className="flex h-[30px] w-[74px] shrink-0 items-center justify-center rounded-full border"
                  style={{
                    backgroundColor: "rgba(0,0,0,0.9)",
                    borderColor: "rgba(255,255,255,0.08)",
                    boxShadow: "0 10px 28px rgba(0,0,0,0.42), inset 0 1px 0 rgba(255,255,255,0.08)",
                    color: "rgba(255,255,255,0.88)",
                    backdropFilter: "blur(16px)",
                  }}
                >
                  <span className="flex items-center justify-center gap-[2px]" aria-hidden="true">
                    {POLISH_DOT_OPACITIES.map((opacity, index) => (
                      <motion.span
                        key={`polish-dot-${index}`}
                        className="h-[2px] w-[2px] rounded-full"
                        animate={{ opacity: [opacity * 0.55, opacity, opacity * 0.55] }}
                        transition={{
                          duration: 1.35,
                          repeat: Infinity,
                          repeatType: "mirror",
                          ease: "easeInOut",
                          delay: index * 0.035,
                        }}
                        style={{ backgroundColor: "currentColor" }}
                      />
                    ))}
                  </span>
                </motion.button>

                <div
                  className="flex h-[30px] shrink-0 items-center rounded-full border p-[2px]"
                  style={{
                    backgroundColor: "rgba(0,0,0,0.86)",
                    borderColor: "rgba(255,255,255,0.08)",
                    boxShadow: "0 10px 26px rgba(0,0,0,0.38), inset 0 1px 0 rgba(255,255,255,0.08)",
                    backdropFilter: "blur(16px)",
                  }}
                >
                  <motion.button
                    type="button"
                    aria-label={
                      autoTransformEnabled
                        ? `Disable Auto Transform (${autoTransformLabel})`
                        : `Enable Auto Transform (${autoTransformLabel})`
                    }
                    title={
                      autoTransformEnabled
                        ? `Auto Transform on: ${autoTransformLabel}`
                        : autoTransformReady
                          ? `Auto Transform off: ${autoTransformLabel}`
                          : "Configure text enhancement to use Auto Transform"
                    }
                    onClick={handleToggleAutoTransform}
                    whileTap={{ scale: 0.92 }}
                    className="flex h-[24px] w-[24px] items-center justify-center rounded-full"
                    style={{
                      color: autoTransformEnabled
                        ? "#ffffff"
                        : "color-mix(in srgb, var(--color-text-primary) 70%, transparent)",
                      backgroundColor: autoTransformEnabled
                        ? "rgba(255,255,255,0.1)"
                        : "transparent",
                    }}
                  >
                    <WandSparkles size={13} aria-hidden="true" />
                  </motion.button>
                  <motion.button
                    type="button"
                    aria-label="Open transform options"
                    title="Open transform options"
                    onClick={handleTransformLastTranscript}
                    disabled={!transcriptAvailable}
                    whileTap={{ scale: transcriptAvailable ? 0.92 : 1 }}
                    className="flex h-[24px] w-[22px] items-center justify-center rounded-full"
                    style={{
                      color: transcriptAvailable
                        ? "color-mix(in srgb, var(--color-text-primary) 74%, transparent)"
                        : "color-mix(in srgb, var(--color-text-primary) 26%, transparent)",
                      cursor: transcriptAvailable ? "pointer" : "default",
                    }}
                  >
                    <ChevronUp size={13} aria-hidden="true" />
                  </motion.button>
                </div>

                <motion.button
                  type="button"
                  aria-label={transcriptAvailable ? "Paste transcript" : "No transcript yet"}
                  title={transcriptAvailable ? "Paste transcript" : "No transcript yet"}
                  onClick={transcriptAvailable ? handlePasteLastTranscript : undefined}
                  disabled={!transcriptAvailable}
                  whileTap={{ scale: transcriptAvailable ? 0.92 : 1 }}
                  className="flex h-[30px] w-[30px] shrink-0 items-center justify-center rounded-full border"
                  style={{
                    color: transcriptAvailable
                      ? "color-mix(in srgb, var(--color-text-primary) 82%, transparent)"
                      : "color-mix(in srgb, var(--color-text-primary) 32%, transparent)",
                    backgroundColor: "rgba(0,0,0,0.86)",
                    borderColor: "rgba(255,255,255,0.08)",
                    boxShadow: "0 10px 26px rgba(0,0,0,0.38), inset 0 1px 0 rgba(255,255,255,0.08)",
                    backdropFilter: "blur(16px)",
                    cursor: transcriptAvailable ? "pointer" : "default",
                  }}
                >
                  <MessageSquareText size={13} aria-hidden="true" />
                </motion.button>
              </>
            ) : (
              <>
                <motion.button
                  type="button"
                  aria-label={pillStatus === "listening" ? "Pause dictation" : "Start dictation"}
                  title={pillStatus === "listening" ? "Pause dictation" : "Start dictation"}
                  onClick={toggleRecording}
                  whileTap={{ scale: 0.92 }}
                  className="flex h-[34px] w-[34px] shrink-0 items-center justify-center rounded-full border"
                  style={{
                    backgroundColor:
                      pillStatus === "listening"
                        ? "var(--color-error)"
                        : "color-mix(in srgb, var(--color-bg-secondary) 88%, transparent)",
                    color: pillStatus === "listening" ? "white" : "var(--color-text-primary)",
                    borderColor: "var(--ui-pill-shell-border)",
                    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.14)",
                  }}
                >
                  {renderRecordIcon()}
                </motion.button>

                <div className="relative min-w-0 flex-1 px-0.5">
                  <div
                    aria-hidden="true"
                    className="relative flex h-7 items-center justify-center gap-[3px] overflow-hidden rounded-full"
                  >
                    {VOICE_BAR_HEIGHTS.map((height, index) => {
                      const activeHeight = Math.max(5, height);
                      const idleHeight = Math.max(4, Math.round(height * 0.44));
                      return (
                        <motion.span
                          key={`voice-bar-${index}`}
                          className="block w-[2.5px] rounded-full"
                          animate={{
                            height:
                              pillStatus === "listening" || pillStatus === "processing"
                                ? [
                                    idleHeight,
                                    activeHeight,
                                    Math.max(6, activeHeight - 5),
                                    idleHeight,
                                  ]
                                : [
                                    idleHeight,
                                    Math.max(idleHeight + 4, Math.round(activeHeight * 0.7)),
                                    idleHeight,
                                  ],
                            opacity:
                              pillStatus === "error"
                                ? 0.28
                                : pillStatus === "listening"
                                  ? [0.62, 1, 0.78, 0.62]
                                  : [0.52, 0.9, 0.52],
                          }}
                          transition={{
                            duration: pillStatus === "listening" ? 0.82 : 1.8,
                            repeat: Infinity,
                            repeatType: "mirror",
                            ease: "easeInOut",
                            delay: index * 0.045,
                          }}
                          style={{
                            backgroundColor: "var(--color-text-primary)",
                            boxShadow:
                              "0 0 10px color-mix(in srgb, var(--color-text-primary) 42%, transparent)",
                          }}
                        />
                      );
                    })}
                  </div>
                </div>

                <motion.button
                  type="button"
                  aria-label={bubbleTitle}
                  title={bubbleTitle}
                  onClick={handleBubbleAction}
                  whileTap={{ scale: 0.92 }}
                  className="flex h-[34px] w-[34px] shrink-0 items-center justify-center rounded-full border"
                  style={{
                    color: pillStatus === "error" ? "white" : "var(--color-text-primary)",
                    backgroundColor:
                      pillStatus === "error"
                        ? "var(--color-error)"
                        : "color-mix(in srgb, var(--color-bg-secondary) 88%, transparent)",
                    borderColor: "var(--ui-pill-shell-border)",
                    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.14)",
                  }}
                >
                  <X size={15} aria-hidden="true" />
                </motion.button>
              </>
            )}
          </motion.div>
        </motion.div>
      </div>
    </div>
  );
};

export default PillOverlay;
