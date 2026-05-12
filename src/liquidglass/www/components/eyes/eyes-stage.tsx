"use client";

import { useEyes } from "@/liquidglass/www/lib/hooks/use-eyes";
import { Eye } from "./eye";
import { GlassCrack } from "./glass-crack";
import { GunshotMode } from "./gunshot-mode";
import { EyesProvider } from "@/liquidglass/www/lib/eyes-context";
import { useState, useEffect } from "react";
import { generateCrackClipPath } from "@/liquidglass/www/lib/crack-path-generator";

interface EyesStageProps {
  children?: React.ReactNode;
  gunshotMode?: boolean;
}

export function EyesStage({ children, gunshotMode = false }: EyesStageProps) {
  const eyes = useEyes();
  const [showCrack, setShowCrack] = useState(false);
  const [crackPosition, setCrackPosition] = useState({ x: 0, y: 0 });
  const [eyeCrackPath, setEyeCrackPath] = useState<string | undefined>();

  useEffect(() => {
    if (eyes.current === "gunshot" && !gunshotMode) {
      const rect = document.querySelector(".eyes-container")?.getBoundingClientRect();
      if (rect) {
        setCrackPosition({
          x: rect.left + rect.width / 2,
          y: rect.top + rect.height / 2,
        });
      }
      setShowCrack(true);
      const timer = setTimeout(() => setShowCrack(false), 3500);
      return () => clearTimeout(timer);
    }
    
    // Generate crack pattern for cracked-happy animation
    if (eyes.current === "cracked-happy") {
      const crackPath = generateCrackClipPath(50, 40, 100, 100, 12);
      setEyeCrackPath(crackPath);
      const timer = setTimeout(() => setEyeCrackPath(undefined), 2000);
      return () => clearTimeout(timer);
    }
  }, [eyes.current, gunshotMode]);

  return (
    <EyesProvider play={eyes.play} current={eyes.current}>
      <div
        className="relative w-full max-w-md mx-auto cursor-pointer select-none eyes-container"
        {...eyes.stageProps}
      >
        <div
          className="relative w-full aspect-2/1 bg-transparent rounded-3xl flex items-center justify-center overflow-hidden"
          style={{ gap: eyes.gap }}
        >
          <div
            className="pointer-events-none absolute inset-0"
            style={{
              background: `radial-gradient(ellipse at ${50 + eyes.mouseOffset.x * 2}% ${50 + eyes.mouseOffset.y * 3}%, hsl(var(--foreground) / 0.015), transparent 60%)`,
            }}
          />
          <Eye style={eyes.leftStyle} crackPath={eyeCrackPath} />
          <Eye style={eyes.rightStyle} crackPath={eyeCrackPath} />
        </div>
      </div>
      {children}
      <GlassCrack show={showCrack} targetX={crackPosition.x} targetY={crackPosition.y} />
      <GunshotMode active={gunshotMode} onShoot={() => eyes.play("gunshot")} />
    </EyesProvider>
  );
}
