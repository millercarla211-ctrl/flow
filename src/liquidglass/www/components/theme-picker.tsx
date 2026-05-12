/* eslint-disable */
// @ts-nocheck
"use client";

import { motion } from "motion/react";
import { useState, useRef, useCallback, useEffect } from "react";
import { useTheme } from "next-themes";
import { Card } from "@/liquidglass/www/components/ui/card";
import { Button } from "@/liquidglass/www/components/ui/button";
import { Sparkles, Sun, Moon, Plus, Minus, RotateCcw, ChevronLeft, ChevronRight } from "lucide-react";
import { cn } from "@/liquidglass/www/lib/utils";

interface ColorDot {
  id: number;
  x: number;
  y: number;
  color: string;
  isPrimary: boolean;
}

const PICKER_SIZE = 500;
const MAX_DOTS = 3;

// Preset color palettes (12 colors per page, 4 pages)
const colorPages = [
  [
    { r: 255, g: 179, b: 217 }, // Pink
    { r: 217, g: 179, b: 255 }, // Purple
    { r: 179, g: 217, b: 255 }, // Light Blue
    { r: 179, g: 255, b: 217 }, // Mint
    { r: 255, g: 255, b: 179 }, // Yellow
    { r: 255, g: 217, b: 179 }, // Peach
    { r: 255, g: 179, b: 179 }, // Coral
    { r: 217, g: 217, b: 217 }, // Gray
    { r: 255, g: 200, b: 220 }, // Light Pink
    { r: 200, g: 220, b: 255 }, // Periwinkle
    { r: 220, g: 255, b: 200 }, // Light Green
    { r: 255, g: 240, b: 200 }, // Cream
  ],
  [
    { r: 255, g: 107, b: 107 }, // Red
    { r: 255, g: 159, b: 64 }, // Orange
    { r: 255, g: 205, b: 86 }, // Gold
    { r: 75, g: 192, b: 192 }, // Teal
    { r: 54, g: 162, b: 235 }, // Blue
    { r: 153, g: 102, b: 255 }, // Violet
    { r: 201, g: 203, b: 207 }, // Silver
    { r: 99, g: 99, b: 99 }, // Dark Gray
    { r: 255, g: 87, b: 87 }, // Bright Red
    { r: 87, g: 255, b: 87 }, // Bright Green
    { r: 87, g: 87, b: 255 }, // Bright Blue
    { r: 255, g: 87, b: 255 }, // Magenta
  ],
  [
    { r: 139, g: 69, b: 19 }, // Brown
    { r: 184, g: 134, b: 11 }, // Dark Gold
    { r: 85, g: 107, b: 47 }, // Olive
    { r: 47, g: 79, b: 79 }, // Dark Slate
    { r: 72, g: 61, b: 139 }, // Dark Slate Blue
    { r: 128, g: 0, b: 128 }, // Purple
    { r: 220, g: 20, b: 60 }, // Crimson
    { r: 255, g: 140, b: 0 }, // Dark Orange
    { r: 34, g: 139, b: 34 }, // Forest Green
    { r: 0, g: 128, b: 128 }, // Teal Dark
    { r: 70, g: 130, b: 180 }, // Steel Blue
    { r: 123, g: 104, b: 238 }, // Medium Slate Blue
  ],
  [
    { r: 25, g: 25, b: 25 }, // Almost Black
    { r: 50, g: 50, b: 50 }, // Dark Charcoal
    { r: 75, g: 75, b: 75 }, // Charcoal
    { r: 100, g: 100, b: 100 }, // Gray
    { r: 150, g: 150, b: 150 }, // Medium Gray
    { r: 200, g: 200, b: 200 }, // Light Gray
    { r: 230, g: 230, b: 230 }, // Very Light Gray
    { r: 245, g: 245, b: 245 }, // Off White
    { r: 64, g: 0, b: 0 }, // Dark Red
    { r: 0, g: 64, b: 0 }, // Dark Green
    { r: 0, g: 0, b: 64 }, // Dark Blue
    { r: 64, g: 64, b: 0 }, // Dark Yellow
  ],
];

export function ThemePicker() {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);
  const [dots, setDots] = useState<ColorDot[]>([]);
  const [draggingDot, setDraggingDot] = useState<number | null>(null);
  const [opacity, setOpacity] = useState(0.5);
  const [texture, setTexture] = useState(0);
  const [colorPage, setColorPage] = useState(0);
  const [isDraggingTexture, setIsDraggingTexture] = useState(false);
  const [isPickerHovered, setIsPickerHovered] = useState(false);
  const [hasUserInteracted, setHasUserInteracted] = useState(false);
  const pickerRef = useRef<HTMLDivElement>(null);
  const textureRef = useRef<HTMLDivElement>(null);
  const sliderRef = useRef<HTMLDivElement>(null);

  // Prevent hydration mismatch
  useEffect(() => {
    setMounted(true);
  }, []);

  // Apply theme colors to CSS variables
  const applyThemeColors = useCallback(() => {
    if (dots.length === 0) return;

    const root = document.documentElement;
    
    // Calculate relative luminance for WCAG contrast
    const getLuminance = (r: number, g: number, b: number) => {
      const toLinear = (c: number) => {
        const normalized = c / 255;
        return normalized <= 0.03928 
          ? normalized / 12.92 
          : Math.pow((normalized + 0.055) / 1.055, 2.4);
      };
      
      const rLin = toLinear(r);
      const gLin = toLinear(g);
      const bLin = toLinear(b);
      
      return 0.2126 * rLin + 0.7152 * gLin + 0.0722 * bLin;
    };
    
    // Get contrasting text color (black or white) based on background luminance
    const getContrastColor = (luminance: number) => {
      // WCAG recommends 0.5 as threshold for contrast (adjusted for better visibility)
      return luminance > 0.5 
        ? { l: 0.145, c: 0, h: 0 } // Dark text for light backgrounds
        : { l: 0.985, c: 0, h: 0 }; // Light text for dark backgrounds
    };
    
    // Convert RGB to OKLCH
    const rgbToOklch = (r: number, g: number, b: number) => {
      // Normalize RGB to 0-1
      const rNorm = r / 255;
      const gNorm = g / 255;
      const bNorm = b / 255;
      
      // Convert to linear RGB
      const toLinear = (c: number) => c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
      const rLin = toLinear(rNorm);
      const gLin = toLinear(gNorm);
      const bLin = toLinear(bNorm);
      
      // Calculate lightness (simplified)
      const lightness = 0.2126 * rLin + 0.7152 * gLin + 0.0722 * bLin;
      
      // Calculate chroma and hue (simplified)
      const a = rNorm - gNorm;
      const b2 = gNorm - bNorm;
      const chroma = Math.sqrt(a * a + b2 * b2) * 0.4;
      let hue = Math.atan2(b2, a) * 180 / Math.PI;
      if (hue < 0) hue += 360;
      
      return { l: lightness, c: chroma, h: hue };
    };

    // Extract RGB from color string
    const extractRGB = (color: string) => {
      const match = color.match(/rgb\((\d+),\s*(\d+),\s*(\d+)\)/);
      if (match) {
        return { r: parseInt(match[1]), g: parseInt(match[2]), b: parseInt(match[3]) };
      }
      return { r: 128, g: 128, b: 128 };
    };

    // Get colors from dots
    const primaryDot = dots.find(d => d.isPrimary) || dots[0];
    const primaryRGB = extractRGB(primaryDot.color);
    const primaryOKLCH = rgbToOklch(primaryRGB.r, primaryRGB.g, primaryRGB.b);
    const primaryLuminance = getLuminance(primaryRGB.r, primaryRGB.g, primaryRGB.b);
    
    // Generate complementary colors
    const isDark = theme === 'dark';
    
    // Primary color - use the actual color from picker
    const primaryL = primaryOKLCH.l;
    root.style.setProperty('--primary', `oklch(${primaryL.toFixed(3)} ${primaryOKLCH.c.toFixed(3)} ${primaryOKLCH.h.toFixed(1)})`);
    
    // Primary foreground (contrasting text color based on primary color luminance)
    const primaryForeground = getContrastColor(primaryLuminance);
    root.style.setProperty('--primary-foreground', `oklch(${primaryForeground.l.toFixed(3)} ${primaryForeground.c.toFixed(3)} ${primaryForeground.h.toFixed(1)})`);
    
    // Accent color (slightly adjusted)
    const accentL = isDark ? Math.min(primaryL + 0.1, 0.9) : Math.max(primaryL - 0.1, 0.2);
    root.style.setProperty('--accent', `oklch(${accentL.toFixed(3)} ${primaryOKLCH.c.toFixed(3)} ${primaryOKLCH.h.toFixed(1)})`);
    root.style.setProperty('--accent-foreground', `oklch(${primaryForeground.l.toFixed(3)} ${primaryForeground.c.toFixed(3)} ${primaryForeground.h.toFixed(1)})`);
    
    // Secondary color (desaturated)
    const secondaryC = primaryOKLCH.c * 0.3;
    const secondaryL = isDark ? 0.269 : 0.97;
    root.style.setProperty('--secondary', `oklch(${secondaryL.toFixed(3)} ${secondaryC.toFixed(3)} ${primaryOKLCH.h.toFixed(1)})`);
    
    // Secondary foreground
    const secondaryForeground = isDark ? { l: 0.985, c: 0, h: 0 } : { l: 0.205, c: 0, h: 0 };
    root.style.setProperty('--secondary-foreground', `oklch(${secondaryForeground.l.toFixed(3)} ${secondaryForeground.c.toFixed(3)} ${secondaryForeground.h.toFixed(1)})`);
    
    // Muted color with opacity effect - MORE VISIBLE
    const mutedL = isDark ? 0.269 : 0.97;
    const mutedChroma = primaryOKLCH.c * 0.1 * opacity; // Opacity directly affects saturation
    root.style.setProperty('--muted', `oklch(${mutedL.toFixed(3)} ${mutedChroma.toFixed(3)} ${primaryOKLCH.h.toFixed(1)})`);
    
    // Muted foreground - affected by opacity MORE VISIBLY
    const mutedForegroundL = isDark ? (0.5 + opacity * 0.208) : (0.8 - opacity * 0.244); // 0.5-0.708 dark, 0.556-0.8 light
    const mutedForegroundChroma = primaryOKLCH.c * 0.05 * opacity;
    root.style.setProperty('--muted-foreground', `oklch(${mutedForegroundL.toFixed(3)} ${mutedForegroundChroma.toFixed(3)} ${primaryOKLCH.h.toFixed(1)})`);
    
    // Border color with texture AND opacity
    const borderL = isDark ? 0.275 : 0.922;
    const borderChroma = (primaryOKLCH.c * 0.05 + (texture * 0.03)) * opacity; // Opacity affects border
    root.style.setProperty('--border', `oklch(${borderL.toFixed(3)} ${borderChroma.toFixed(3)} ${primaryOKLCH.h.toFixed(1)})`);
    
    // Ring color affected by opacity
    const ringL = isDark ? 0.556 : 0.708;
    const ringChroma = primaryOKLCH.c * 0.2 * opacity;
    root.style.setProperty('--ring', `oklch(${ringL.toFixed(3)} ${ringChroma.toFixed(3)} ${primaryOKLCH.h.toFixed(1)})`);
    
    // Card colors with texture AND opacity
    const cardL = isDark ? 0.205 : 1;
    const cardChroma = (primaryOKLCH.c * 0.02 + (texture * 0.05)) * opacity;
    root.style.setProperty('--card', `oklch(${cardL.toFixed(3)} ${cardChroma.toFixed(3)} ${primaryOKLCH.h.toFixed(1)})`);
    const cardForegroundL = isDark ? 0.985 : 0.145;
    root.style.setProperty('--card-foreground', `oklch(${cardForegroundL.toFixed(3)} 0 0)`);
    
    // Popover colors with opacity
    const popoverL = isDark ? 0.269 : 1;
    const popoverChroma = primaryOKLCH.c * 0.02 * opacity;
    root.style.setProperty('--popover', `oklch(${popoverL.toFixed(3)} ${popoverChroma.toFixed(3)} ${primaryOKLCH.h.toFixed(1)})`);
    root.style.setProperty('--popover-foreground', `oklch(${cardForegroundL.toFixed(3)} 0 0)`);
    
    // Apply opacity and texture with visible effects
    root.style.setProperty('--theme-opacity', opacity.toString());
    root.style.setProperty('--theme-texture', texture.toString());
    
    // Apply texture as a filter to cards and UI elements
    const noiseIntensity = texture * 0.15; // 0 to 0.15
    const grainFilter = texture > 0 
      ? `contrast(${1 + texture * 0.1}) brightness(${1 - texture * 0.05})` 
      : 'none';
    root.style.setProperty('--theme-filter', grainFilter);
    
    console.log('Theme applied:', { 
      primaryOKLCH, 
      primaryLuminance,
      primaryForeground,
      opacity, 
      texture, 
      isDark,
      noiseIntensity,
      grainFilter,
      mutedForegroundL,
      cardChroma,
      mutedChroma,
      mutedForegroundChroma,
      borderChroma,
      ringChroma,
      popoverChroma,
      contrastText: primaryLuminance > 0.5 ? 'dark' : 'light'
    });
  }, [dots, opacity, texture, theme]);

  // Auto-apply theme when values change - ONLY if user has interacted
  useEffect(() => {
    if (mounted && dots.length > 0 && hasUserInteracted) {
      applyThemeColors();
    }
  }, [dots, opacity, texture, mounted, applyThemeColors, hasUserInteracted]);

  // HSL to RGB conversion
  const hslToRgb = (h: number, s: number, l: number): [number, number, number] => {
    const hueToRgb = (p: number, q: number, t: number) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1 / 6) return p + (q - p) * 6 * t;
      if (t < 1 / 2) return q;
      if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
      return p;
    };

    if (s === 0) {
      const val = Math.round(l * 255);
      return [val, val, val];
    }

    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    const r = hueToRgb(p, q, h + 1 / 3);
    const g = hueToRgb(p, q, h);
    const b = hueToRgb(p, q, h - 1 / 3);

    return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
  };

  // Get color from position in circular picker
  const getColorFromPosition = (x: number, y: number): string => {
    const centerX = PICKER_SIZE / 2;
    const centerY = PICKER_SIZE / 2;
    const radius = PICKER_SIZE / 2;

    const distance = Math.sqrt((x - centerX) ** 2 + (y - centerY) ** 2);
    let angle = Math.atan2(y - centerY, x - centerX);
    angle = (angle * 180) / Math.PI;
    if (angle < 0) angle += 360;

    const normalizedDistance = 1 - Math.min(distance / radius, 1);
    const hue = angle / 360;
    const saturation = normalizedDistance;
    const lightness = 0.5;

    const [r, g, b] = hslToRgb(hue, saturation, lightness);
    return `rgb(${r}, ${g}, ${b})`;
  };

  // Handle click on circular picker
  const handlePickerClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!pickerRef.current || draggingDot !== null) return;

    const rect = pickerRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const color = getColorFromPosition(x, y);

    setHasUserInteracted(true); // Mark that user has interacted

    if (dots.length === 0) {
      setDots([{ id: 0, x, y, color, isPrimary: true }]);
    } else if (dots.length < MAX_DOTS) {
      setDots([...dots, { id: dots.length, x, y, color, isPrimary: false }]);
    } else {
      setDots(dots.map(dot => 
        dot.isPrimary ? { ...dot, x, y, color } : dot
      ));
    }
  };

  // Handle dot drag
  const handleDotMouseDown = (e: React.MouseEvent, dotId: number) => {
    e.stopPropagation();
    setDraggingDot(dotId);
  };

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (draggingDot === null || !pickerRef.current) return;

    const rect = pickerRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const color = getColorFromPosition(x, y);

    setDots(dots.map(dot => 
      dot.id === draggingDot ? { ...dot, x, y, color } : dot
    ));
  }, [draggingDot, dots]);

  const handleMouseUp = useCallback(() => {
    setDraggingDot(null);
  }, []);

  useEffect(() => {
    if (draggingDot !== null) {
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
      return () => {
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
      };
    }
  }, [draggingDot, handleMouseMove, handleMouseUp]);

  // Texture control - fixed rotation calculation
  const handleTextureMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsDraggingTexture(true);
    updateTextureFromMouse(e.clientX, e.clientY);
  };

  const updateTextureFromMouse = (clientX: number, clientY: number) => {
    if (!textureRef.current) return;

    const rect = textureRef.current.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    // Calculate angle from center
    let angle = Math.atan2(clientY - centerY, clientX - centerX);
    // Convert to degrees and normalize to 0-360
    let degrees = (angle * 180) / Math.PI + 90;
    if (degrees < 0) degrees += 360;
    
    // Convert to 0-1 range and snap to 16 steps
    let textureValue = degrees / 360;
    textureValue = Math.round(textureValue * 16) / 16;
    if (textureValue >= 1) textureValue = 0;

    setTexture(textureValue);
  };

  const handleTextureMouseMove = useCallback((e: MouseEvent) => {
    if (!isDraggingTexture) return;
    updateTextureFromMouse(e.clientX, e.clientY);
  }, [isDraggingTexture]);

  const handleTextureMouseUp = useCallback(() => {
    setIsDraggingTexture(false);
  }, []);

  useEffect(() => {
    if (isDraggingTexture) {
      document.addEventListener("mousemove", handleTextureMouseMove);
      document.addEventListener("mouseup", handleTextureMouseUp);
      return () => {
        document.removeEventListener("mousemove", handleTextureMouseMove);
        document.removeEventListener("mouseup", handleTextureMouseUp);
      };
    }
  }, [isDraggingTexture, handleTextureMouseMove, handleTextureMouseUp]);

  // Add/remove dots
  const addDot = () => {
    if (dots.length >= MAX_DOTS) return;
    const centerX = PICKER_SIZE / 2;
    const centerY = PICKER_SIZE / 2;
    const color = getColorFromPosition(centerX, centerY);
    setDots([...dots, { id: dots.length, x: centerX, y: centerY, color, isPrimary: false }]);
  };

  const removeDot = () => {
    if (dots.length === 0) return;
    setDots(dots.slice(0, -1));
  };

  // Apply preset color
  const applyPresetColor = (rgb: { r: number; g: number; b: number }) => {
    const color = `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`;
    
    setHasUserInteracted(true); // Mark that user has interacted
    
    if (dots.length === 0) {
      const centerX = PICKER_SIZE / 2;
      const centerY = PICKER_SIZE / 2;
      setDots([{ id: 0, x: centerX, y: centerY, color, isPrimary: true }]);
    } else {
      setDots(dots.map(dot => 
        dot.isPrimary ? { ...dot, color } : dot
      ));
    }
  };



  return (
    <Card className="w-full max-w-2xl bg-background border p-0 overflow-hidden">
      {/* Circular Gradient Picker */}
      <motion.div 
        className="flex justify-center"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <div
          ref={pickerRef}
          className="relative overflow-hidden cursor-crosshair w-full min-w-full border bg-background transition-all duration-300"
          style={{
            width: PICKER_SIZE,
            height: PICKER_SIZE,
          }}
          onClick={handlePickerClick}
          onMouseEnter={() => setIsPickerHovered(true)}
          onMouseLeave={() => setIsPickerHovered(false)}
        >
          {/* Dot pattern on background - HARDCODED for testing */}
          <div 
            className="absolute inset-0 pointer-events-none z-0"
            style={{
              backgroundImage: "radial-gradient(circle, #666666 1px, transparent 1px)",
              backgroundSize: "10px 10px",
            }}
          />

          {/* Theme Mode Selector - Inside picker at top */}
          {mounted && (
            <motion.div 
              className="absolute top-4 left-1/2 -translate-x-1/2 flex items-center gap-2 z-20"
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2, duration: 0.4 }}
            >
              <Button
                variant="ghost"
                size="icon"
                className={cn(
                  "h-11 w-11 rounded-xl transition-all backdrop-blur-md",
                  theme === "system" ? "bg-background/90 shadow-lg" : "bg-background/50 hover:bg-background/70"
                )}
                onClick={(e) => { e.stopPropagation(); setTheme("system"); }}
              >
                <Sparkles className="h-5 w-5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className={cn(
                  "h-11 w-11 rounded-xl transition-all backdrop-blur-md",
                  theme === "light" ? "bg-background/90 shadow-lg" : "bg-background/50 hover:bg-background/70"
                )}
                onClick={(e) => { e.stopPropagation(); setTheme("light"); }}
              >
                <Sun className="h-5 w-5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className={cn(
                  "h-11 w-11 rounded-xl transition-all backdrop-blur-md",
                  theme === "dark" ? "bg-background/90 shadow-lg" : "bg-background/50 hover:bg-background/70"
                )}
                onClick={(e) => { e.stopPropagation(); setTheme("dark"); }}
              >
                <Moon className="h-5 w-5" />
              </Button>
            </motion.div>
          )}

          {/* Dots */}
          {dots.map((dot, index) => (
            <motion.div
              key={dot.id}
              className={cn(
                "absolute rounded-full border-white cursor-move shadow-lg",
                dot.isPrimary ? "w-12 h-12 border-[5px] z-10" : "w-10 h-10 border-4"
              )}
              style={{
                backgroundColor: dot.color,
              }}
              initial={{ 
                opacity: 0, 
                scale: 0,
                x: dot.x - (dot.isPrimary ? 24 : 20),
                y: dot.y - (dot.isPrimary ? 24 : 20),
              }}
              animate={{ 
                opacity: 1,
                scale: draggingDot === dot.id ? 1.15 : 1,
                x: dot.x - (dot.isPrimary ? 24 : 20),
                y: dot.y - (dot.isPrimary ? 24 : 20),
                boxShadow: draggingDot === dot.id 
                  ? "0 20px 25px -5px rgb(0 0 0 / 0.3), 0 8px 10px -6px rgb(0 0 0 / 0.3)"
                  : "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)",
              }}
              transition={{
                opacity: { duration: 0.3, delay: index * 0.1 },
                scale: { 
                  type: "spring",
                  stiffness: 500,
                  damping: 35,
                },
                x: {
                  type: "spring",
                  stiffness: 600,
                  damping: 40,
                  mass: 0.5,
                },
                y: {
                  type: "spring",
                  stiffness: 600,
                  damping: 40,
                  mass: 0.5,
                },
                boxShadow: { duration: 0.2 },
              }}
              onMouseDown={(e) => handleDotMouseDown(e, dot.id)}
              whileHover={{ 
                scale: draggingDot === dot.id ? 1.15 : 1.12,
                transition: { 
                  type: "spring",
                  stiffness: 400,
                  damping: 25,
                }
              }}
            />
          ))}

          {/* Add/Remove/Reset Controls */}
          <motion.div 
            className="absolute bottom-4 left-1/2 -translate-x-1/2 flex gap-2 z-20"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3, duration: 0.4 }}
          >
            <Button
              variant="secondary"
              size="icon"
              className="h-9 w-9 rounded-xl bg-background/80 backdrop-blur hover:bg-background/90"
              onClick={(e) => { e.stopPropagation(); addDot(); }}
              disabled={dots.length >= MAX_DOTS}
            >
              <Plus className="h-4 w-4" />
            </Button>
            <Button
              variant="secondary"
              size="icon"
              className="h-9 w-9 rounded-xl bg-background/80 backdrop-blur hover:bg-background/90"
              onClick={(e) => { e.stopPropagation(); removeDot(); }}
              disabled={dots.length === 0}
            >
              <Minus className="h-4 w-4" />
            </Button>
            <Button
              variant="secondary"
              size="icon"
              className="h-9 w-9 rounded-xl bg-background/80 backdrop-blur hover:bg-background/90"
              onClick={(e) => { e.stopPropagation(); setDots([]); }}
            >
              <RotateCcw className="h-4 w-4" />
            </Button>
          </motion.div>
        </div>
      </motion.div>

      {/* Color Palette */}
      <motion.div 
        className="w-full py-4 border-t"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.4, duration: 0.4 }}
      >
        <div className="flex items-center gap-3 px-4">
          <Button
            variant="ghost"
            size="icon"
            className="h-9 w-9 shrink-0"
            onClick={() => setColorPage((colorPage - 1 + colorPages.length) % colorPages.length)}
          >
            <ChevronLeft className="h-5 w-5" />
          </Button>
          
          <motion.div 
            className="flex-1 grid grid-cols-12 gap-2"
            key={colorPage}
            initial={{ opacity: 0, x: colorPage > 0 ? 20 : -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.3 }}
          >
            {colorPages[colorPage].map((color, index) => {
              // Create more vibrant gradient variations
              const lighterR = Math.min(255, Math.round(color.r + (255 - color.r) * 0.4));
              const lighterG = Math.min(255, Math.round(color.g + (255 - color.g) * 0.4));
              const lighterB = Math.min(255, Math.round(color.b + (255 - color.b) * 0.4));
              
              const darkerR = Math.max(0, Math.round(color.r * 0.6));
              const darkerG = Math.max(0, Math.round(color.g * 0.6));
              const darkerB = Math.max(0, Math.round(color.b * 0.6));
              
              return (
                <motion.button
                  key={index}
                  initial={{ scale: 0, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  transition={{ 
                    delay: index * 0.03,
                    type: "spring",
                    stiffness: 400,
                    damping: 25,
                  }}
                  whileHover={{ 
                    scale: 1.15, 
                    y: -3,
                    transition: { duration: 0.2 }
                  }}
                  whileTap={{ scale: 0.95 }}
                  onClick={() => applyPresetColor(color)}
                  className="aspect-square rounded-full border-2 border-border shadow-sm transition-shadow hover:shadow-md"
                  style={{ 
                    background: `linear-gradient(135deg, rgb(${lighterR}, ${lighterG}, ${lighterB}) 0%, rgb(${color.r}, ${color.g}, ${color.b}) 50%, rgb(${darkerR}, ${darkerG}, ${darkerB}) 100%)`
                  }}
                />
              );
            })}
          </motion.div>

          <Button
            variant="ghost"
            size="icon"
            className="h-9 w-9 shrink-0"
            onClick={() => setColorPage((colorPage + 1) % colorPages.length)}
          >
            <ChevronRight className="h-5 w-5" />
          </Button>
        </div>
      </motion.div>

      {/* Opacity Slider with Snake Wave and Texture Control */}
      <motion.div 
        className="py-4 space-y-3 border-t"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.5, duration: 0.4 }}
      >
        <div className="flex items-center gap-4 px-6">
          {/* Opacity value and label */}
          <div className="flex flex-col items-center gap-1 shrink-0 w-20">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Opacity</label>
            <span className="text-sm font-mono text-foreground font-bold">{Math.round(opacity * 100)}%</span>
          </div>

          {/* Snake Wave Slider */}
          <div className="relative flex-1 h-20">
            {/* Background track */}
            <div className="absolute top-1/2 -translate-y-1/2 w-full h-3 bg-muted/50 rounded-full" />
            
            {/* Snake Wave */}
            <div className="absolute inset-0 pointer-events-none flex items-center">
              <svg 
                width="100%" 
                height="60" 
                viewBox="0 0 1000 60"
                preserveAspectRatio="none"
                className="block"
              >
                {/* Border stroke */}
                <path
                  d="M 0,30 Q 25,10 50,30 T 100,30 Q 125,10 150,30 T 200,30 Q 225,10 250,30 T 300,30 Q 325,10 350,30 T 400,30 Q 425,10 450,30 T 500,30 Q 525,10 550,30 T 600,30 Q 625,10 650,30 T 700,30 Q 725,10 750,30 T 800,30 Q 825,10 850,30 T 900,30 Q 925,10 950,30 T 1000,30"
                  fill="none"
                  stroke="#888888"
                  strokeWidth="6"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
                {/* Main stroke */}
                <path
                  d="M 0,30 Q 25,10 50,30 T 100,30 Q 125,10 150,30 T 200,30 Q 225,10 250,30 T 300,30 Q 325,10 350,30 T 400,30 Q 425,10 450,30 T 500,30 Q 525,10 550,30 T 600,30 Q 625,10 650,30 T 700,30 Q 725,10 750,30 T 800,30 Q 825,10 850,30 T 900,30 Q 925,10 950,30 T 1000,30"
                  fill="none"
                  stroke="#000000"
                  strokeWidth="3"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            </div>

            {/* Slider Input */}
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={opacity}
              onChange={(e) => setOpacity(parseFloat(e.target.value))}
              className="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
            />

            {/* Slider Thumb with animation */}
            <motion.div 
              className="absolute top-1/2 -translate-y-1/2 w-6 h-12 bg-foreground rounded-full pointer-events-none shadow-lg z-20"
              style={{ 
                left: `calc(${opacity * 100}% - 12px)`,
              }}
            />
          </div>

          {/* Circular Texture Control */}
          <div className="flex flex-col items-center gap-1 shrink-0">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Texture</label>
            <span className="text-sm font-mono text-foreground font-bold">{Math.round(texture * 100)}%</span>
            <motion.div 
              ref={textureRef}
              className="relative w-24 h-24 rounded-full border-2 border-border bg-background flex items-center justify-center cursor-pointer mt-1"
              onMouseDown={handleTextureMouseDown}
              whileHover={{ 
                scale: 1.05,
                borderColor: "hsl(var(--foreground) / 0.3)",
                transition: { 
                  type: "spring",
                  stiffness: 400,
                  damping: 20,
                }
              }}
              whileTap={{ 
                scale: 0.95,
                transition: { 
                  type: "spring",
                  stiffness: 500,
                  damping: 25,
                }
              }}
            >
            {/* Inner circle */}
            <div 
              className="absolute inset-4 rounded-full border border-border/30"
            />
            
            {/* Texture dots around circle */}
            {Array.from({ length: 16 }).map((_, i) => {
              const angle = (i / 16) * Math.PI * 2 - Math.PI / 2;
              const active = i / 16 <= texture;
              return (
                <div
                  key={i}
                  className={cn(
                    "absolute w-1.5 h-1.5 rounded-full transition-all duration-150",
                    active ? "bg-foreground opacity-100 scale-110" : "bg-muted-foreground opacity-30 scale-100"
                  )}
                  style={{
                    left: `${50 + Math.cos(angle) * 40}%`,
                    top: `${50 + Math.sin(angle) * 40}%`,
                    transform: `translate(-50%, -50%)`,
                  }}
                />
              );
            })}
            
            {/* Texture handler */}
            <div 
              className="absolute w-1.5 h-7 bg-foreground rounded-full pointer-events-none shadow-sm"
              style={{
                left: "50%",
                top: "12%",
                transform: `translateX(-50%) rotate(${texture * 360}deg)`,
                transformOrigin: "center 36px",
              }}
            />
          </motion.div>
          </div>
        </div>
      </motion.div>
    </Card>
  );
}
