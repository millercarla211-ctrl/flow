"use client";

import type { CSSProperties } from "react";

interface EyeProps {
  style: CSSProperties;
  className?: string;
  crackPath?: string;
}

export function Eye({ style, className, crackPath }: EyeProps) {
  // Determine which clip-path to use
  let clipPath: string | undefined;
  
  if (crackPath) {
    // Use crack pattern
    clipPath = `path('${crackPath}')`;
  } else if (style.backgroundColor === "#ff2d55") {
    // Use heart shape for love animation - no clipping, just the shape itself
    clipPath = undefined;
  }
  
  return (
    <div
      className={`relative transition-[width,height,border-radius] duration-60 ease-out ${className ?? ""}`}
      style={{
        ...style,
        willChange: "transform, width, height, border-radius, background-color",
        clipPath,
        // Use mask for heart shape instead of clip-path to avoid cutting
        ...(style.backgroundColor === "#ff2d55" && {
          maskImage: "url(\"data:image/svg+xml,%3Csvg viewBox='0 0 100 100' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M 50,25 C 50,20 45,8 30,8 C 15,8 5,20 5,35 C 5,55 30,75 50,92 C 70,75 95,55 95,35 C 95,20 85,8 70,8 C 55,8 50,20 50,25 Z' fill='black'/%3E%3C/svg%3E\")",
          maskSize: "contain",
          maskRepeat: "no-repeat",
          maskPosition: "center",
        }),
      }}
    />
  );
}
