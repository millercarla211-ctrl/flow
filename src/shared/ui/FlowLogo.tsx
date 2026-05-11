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
      <rect width="64" height="64" rx="18" fill="var(--flow-logo-bg)" />
      <path
        d="M14 14H52L48.2 24H25.5L21.6 28.6H43.8L40.4 38H21.6V51H14V14Z"
        fill="var(--flow-logo-fg)"
      />
      <path d="M25.5 24H48.2L46.8 27.5H22.6L25.5 24Z" fill="var(--flow-logo-bg)" opacity="0.26" />
    </motion.svg>
  );
};
