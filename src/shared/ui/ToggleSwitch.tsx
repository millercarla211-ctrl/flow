import { motion } from "framer-motion";

type ToggleSwitchSize = "sm" | "md";

type ToggleSwitchProps = {
  enabled: boolean;
  onToggle: () => void;
  ariaLabel: string;
  disabled?: boolean;
  size?: ToggleSwitchSize;
};

const sizeConfig = {
  sm: {
    track: "w-7 h-4",
    thumb: "top-[2px] w-3 h-3",
    onLeft: "calc(100% - 14px)",
    offLeft: "2px",
  },
  md: {
    track: "w-10 h-5",
    thumb: "top-0.5 w-4 h-4",
    onLeft: "calc(100% - 18px)",
    offLeft: "2px",
  },
} as const;

const ToggleSwitch = ({
  enabled,
  onToggle,
  ariaLabel,
  disabled = false,
  size = "sm",
}: ToggleSwitchProps) => {
  const config = sizeConfig[size];

  return (
    <button
      type="button"
      onClick={onToggle}
      role="switch"
      aria-checked={enabled}
      aria-label={ariaLabel}
      disabled={disabled}
      className={`${config.track} rounded-full transition-colors relative ${
        enabled ? "bg-cloud" : "bg-border-secondary"
      } ${disabled ? "opacity-50 cursor-not-allowed" : ""}`}
    >
      <motion.div
        className={`absolute ${config.thumb} rounded-full bg-white shadow-xs`}
        initial={false}
        animate={{ left: enabled ? config.onLeft : config.offLeft }}
        transition={{ type: "spring", stiffness: 500, damping: 30 }}
      />
    </button>
  );
};

export default ToggleSwitch;
