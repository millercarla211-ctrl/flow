import { useLingui } from "@lingui/react/macro";
import { useMemo } from "react";
import { motion, type Variants, type Easing } from "framer-motion";
import { Check, Loader2 } from "lucide-react";
import DotMatrix from "../../../shared/ui/DotMatrix";

export const GlimpseLogo = ({ size = "md" }: { size?: "sm" | "md" | "lg" }) => {
  const sizes = {
    sm: { dot: 5, gap: 4 },
    md: { dot: 10, gap: 7 },
    lg: { dot: 14, gap: 10 },
  }[size];

  const dotColors = [
    "var(--color-cloud)",
    "var(--color-local)",
    "var(--color-local)",
    "var(--color-cloud)",
  ];

  const D = sizes.dot + sizes.gap;
  const r = sizes.dot / 2;

  const coords = [
    { cx: r, cy: r },             // 0: TL
    { cx: r + D, cy: r },         // 1: TR
    { cx: r, cy: r + D },         // 2: BL
    { cx: r + D, cy: r + D },     // 3: BR
  ];

  const cloudConnectorAnim = {
    cx: [coords[0].cx, coords[3].cx, coords[3].cx, coords[0].cx, coords[0].cx],
    cy: [coords[0].cy, coords[3].cy, coords[3].cy, coords[0].cy, coords[0].cy],
  };

  const localConnectorAnim = {
    cx: [coords[1].cx, coords[1].cx, coords[2].cx, coords[2].cx, coords[1].cx],
    cy: [coords[1].cy, coords[1].cy, coords[2].cy, coords[2].cy, coords[1].cy],
  };

  const gridSize = sizes.dot * 2 + sizes.gap;
  const stdDev = sizes.dot * 0.35; 
  const colorMatrixValues = `1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 18 -7`;

  return (
    <div className="relative flex items-center justify-center" style={{ width: gridSize, height: gridSize }}>
      <svg
        width={gridSize}
        height={gridSize}
        viewBox={`0 0 ${gridSize} ${gridSize}`}
        style={{ overflow: "visible" }}
      >
        <defs>
          <filter id={`goo-${size}`} x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur in="SourceGraphic" stdDeviation={stdDev} result="blur" />
            <feColorMatrix in="blur" mode="matrix" values={colorMatrixValues} result="goo" />
            <feBlend in="SourceGraphic" in2="goo" />
          </filter>
        </defs>
        <g filter={`url(#goo-${size})`}>
          {/* Static dots */}
          {coords.map((coord, i) => (
            <circle
              key={`static-${i}`}
              cx={coord.cx}
              cy={coord.cy}
              r={r}
              fill={dotColors[i]}
            />
          ))}

          {/* Cloud connector (moves TL <-> BR) */}
          <motion.circle
            r={r}
            fill="var(--color-cloud)"
            animate={cloudConnectorAnim}
            transition={{
              duration: 4,
              ease: "easeInOut",
              times: [0, 0.25, 0.5, 0.75, 1],
              repeat: Infinity,
            }}
          />

          {/* Local connector (moves TR <-> BL) */}
          <motion.circle
            r={r}
            fill="var(--color-local)"
            animate={localConnectorAnim}
            transition={{
              duration: 4,
              ease: "easeInOut",
              times: [0, 0.25, 0.5, 0.75, 1],
              repeat: Infinity,
            }}
          />
        </g>
      </svg>
    </div>
  );
};

export const StepIndicator = ({
  currentStep,
  total,
}: {
  currentStep: number;
  total: number;
}) => (
  <div className="flex items-center gap-1.5">
    {Array.from({ length: total }).map((_, i) => (
      <motion.div
        key={i}
        className="h-1.5 rounded-full bg-amber-400"
        animate={{
          width: i === currentStep ? 20 : 6,
          opacity: i <= currentStep ? 1 : 0.25,
        }}
        transition={{ duration: 0.25 }}
      />
    ))}
  </div>
);

export const StatusBadge = ({
  granted,
  checking,
}: {
  granted: boolean;
  checking?: boolean;
}) => {
  const { t } = useLingui();

  if (checking) {
    return (
      <span className="inline-flex items-center gap-1.5 ui-text-label text-content-muted">
        <Loader2 size={11} className="animate-spin" />
        {t({
          id: "onboarding.status.checking",
          message: "Checking...",
        })}
      </span>
    );
  }

  if (granted) {
    return (
      <motion.span
        className="inline-flex items-center gap-1 ui-text-label font-medium ui-color-success-strong"
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
      >
        <Check size={12} />
        {t({
          id: "onboarding.status.enabled",
          message: "Enabled",
        })}
      </motion.span>
    );
  }

  return (
    <span className="ui-text-label text-content-muted">
      {t({
        id: "onboarding.status.not_enabled",
        message: "Not enabled",
      })}
    </span>
  );
};

export const ModelProgress = ({
  percent,
  status,
}: {
  percent: number;
  status: string;
}) => {
  const cols = 40;
  const rows = 2;
  const totalDots = cols * rows;
  const activeCount = Math.round((percent / 100) * totalDots);

  const activeDots = useMemo(() => {
    const dots: number[] = [];
    for (let i = 0; i < activeCount && i < totalDots; i++) {
      dots.push(i);
    }
    return dots;
  }, [activeCount, totalDots]);

  const color =
    status === "error"
      ? "var(--color-error)"
      : status === "complete"
        ? "var(--color-success)"
        : "var(--color-cloud)";

  return (
    <DotMatrix
      rows={rows}
      cols={cols}
      activeDots={activeDots}
      dotSize={2}
      gap={2}
      color={color}
      className={status === "downloading" ? "opacity-80" : "opacity-60"}
      morphOnActive={true}
      activeScale={1.0}
    />
  );
};

type ApplePermissionIconProps = {
  size?: number;
  className?: string;
};

export const AppleAccessibilityIcon = ({
  size = 32,
  className,
}: ApplePermissionIconProps) => (
  <svg
    aria-hidden="true"
    viewBox="0 0 24 24"
    width={size}
    height={size}
    className={className}
    fill="none"
  >
    <path
      d="M12 5.6a2.1 2.1 0 1 0 0-4.2 2.1 2.1 0 0 0 0 4.2Z"
      fill="currentColor"
    />
    <path
      d="M4.35 7.2a.9.9 0 0 1 1.02-.75 42.5 42.5 0 0 0 13.26 0 .9.9 0 1 1 .27 1.78c-1.9.29-3.9.47-5.9.52v3.12l3.1 7.52a.9.9 0 1 1-1.67.68L12 14.16l-2.43 5.91a.9.9 0 0 1-1.67-.68l3.1-7.52V8.75a44.34 44.34 0 0 1-5.9-.52.9.9 0 0 1-.75-1.03Z"
      fill="currentColor"
    />
  </svg>
);

export type StepMotionProps = {
  custom: 1 | -1;
  variants: Variants;
  animate: string;
  exit: string;
  transition: { duration: number; ease: Easing };
};
