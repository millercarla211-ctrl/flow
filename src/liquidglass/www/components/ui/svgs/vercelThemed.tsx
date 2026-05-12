"use client";

import type { SVGProps } from "react";
import { useTheme } from "next-themes";
import { useEffect, useState } from "react";

const VercelThemed = (props: SVGProps<SVGSVGElement>) => {
  const { resolvedTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const isDark = mounted && resolvedTheme === "dark";

  return (
    <svg 
      {...props} 
      viewBox="0 0 256 222" 
      preserveAspectRatio="xMidYMid"
      style={{ filter: isDark ? 'invert(1)' : 'none' }}
    >
      <path d="m128 0 128 221.705H0z" />
    </svg>
  );
};

export { VercelThemed };
