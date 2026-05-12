/* eslint-disable */
// @ts-nocheck
"use client";

import React, { ReactNode } from "react";
import { motion } from "framer-motion";
import { cn } from "@/liquidglass/www/lib/utils";

const SPAN_COUNT = 25;

// Pre-generate rainbow gradient for optimized animation
function generateRainbowGradient(count: number): string {
  const stops = Array.from({ length: count + 1 }, (_, i) => {
    const hue = (i / count) * 360;
    return `hsl(${hue}, 80%, 60%)`;
  });
  return `linear-gradient(90deg, ${stops.join(", ")})`;
}

const RAINBOW_GRADIENT = generateRainbowGradient(SPAN_COUNT);

interface HelloGlowProps {
  className?: string;
  children?: ReactNode;
}

export function HelloGlow({ className, children }: HelloGlowProps) {
  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: `
        .hello-glow-container {
          position: relative;
          min-width: 100%;
          min-height: 125px;
          border-radius: 12px;
        }

        .hello-glow-background {
          position: absolute;
          inset: 0;
          border-radius: 12px;
          background: ${RAINBOW_GRADIENT};
          background-size: 200% 100%;
          animation: hello-gradient-shift 6s linear infinite;
          filter: blur(6px);
        }

        .hello-glow-background::after {
          content: "";
          position: absolute;
          inset: -15px;
          background: ${RAINBOW_GRADIENT};
          background-size: 200% 100%;
          animation: hello-gradient-shift 6s linear infinite;
          filter: blur(18px);
          border-radius: 12px;
          z-index: -1;
          opacity: 0.5;
        }

        .hello-glow-content {
          position: relative;
          width: 100%;
          height: 100%;
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1;
        }

        @keyframes hello-gradient-shift {
          0% {
            background-position: 0% 0%;
          }
          100% {
            background-position: 200% 0%;
          }
        }
      ` }} />

      <motion.div
        className={cn("hello-glow-container", className)}
        transition={{ type: "spring", stiffness: 300, damping: 20 }}
      >
        <div className="hello-glow-background" />
        <div className="hello-glow-content">
          {children}
        </div>
      </motion.div>
    </>
  );
}
