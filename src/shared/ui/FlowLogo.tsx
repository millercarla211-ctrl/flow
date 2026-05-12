import { motion } from "framer-motion";

type FlowLogoSize = "sm" | "md" | "lg";

const FLOW_LOGO_SIZES: Record<FlowLogoSize, number> = {
  sm: 18,
  md: 34,
  lg: 54,
};

export const FlowLogo = ({ size = "md" }: { size?: FlowLogoSize }) => {
  const pixelSize = FLOW_LOGO_SIZES[size];

  return (
    <motion.svg
      aria-hidden="true"
      focusable="false"
      width={pixelSize}
      height={pixelSize}
      viewBox="0 0 64 64"
      className="flow-mark"
      initial={{ opacity: 0.88 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 1.8, repeat: Infinity, repeatType: "reverse" }}
    >
      <rect width="64" height="64" rx="16" fill="var(--flow-logo-bg)" />
      <path
        d="M15 13.5H52.5L48.2 22.2H27.3L23.7 27.2H44.3L40.2 35.6H23.7V50.5H14.2V20.5L15 13.5Z"
        fill="var(--flow-logo-fg)"
      />
      <path
        d="M27.3 22.2H48.2L46.8 25.1H25.3L27.3 22.2Z"
        fill="var(--flow-logo-bg)"
        opacity="0.34"
      />
      <path
        d="M38.5 40.3H50.8L48.5 44.8H35.9L38.5 40.3Z"
        fill="var(--flow-logo-fg)"
        opacity="0.74"
      />
    </motion.svg>
  );
};
