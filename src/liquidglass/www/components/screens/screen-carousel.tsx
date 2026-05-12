/* eslint-disable */
// @ts-nocheck
"use client";

import { animate, motion, useMotionValue } from "motion/react";
import { Resizable } from "re-resizable";
import { useEffect, useRef, useState } from "react";
import { BrowserScreen } from "./browser-screen";
import { CodeScreen } from "./code-screen";
import { CustomScreen } from "./custom-screen";
import { TerminalScreen } from "./terminal-screen";
import type { Screen } from "./types";
import { WelcomeScreen } from "./welcome-screen";

interface ScreenCarouselProps {
  activeScreenId: string;
  screens: Screen[];
  onScreenChange: (screenId: string) => void;
  onScreenResize: (id: string, width: number, height: number) => void;
  onScreensUpdate: (screens: Screen[]) => void;
  sidebarExpanded: boolean;
}

export function ScreenCarousel({
  activeScreenId,
  screens,
  onScreenChange,
  onScreenResize,
  onScreensUpdate,
  sidebarExpanded,
}: ScreenCarouselProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [containerSize, setContainerSize] = useState({ width: 0, height: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  // Counter to force re-renders during resize for live animation
  // biome-ignore lint/correctness/noUnusedVariables: Used to trigger re-renders
  const [resizeCounter, setResizeCounter] = useState(0);
  const [resizeDirection, setResizeDirection] = useState<
    "left" | "right" | "both" | null
  >(null);
  const resizeSidesRef = useRef<{ left: boolean; right: boolean }>({
    left: false,
    right: false,
  });
  const manuallyResizedScreensRef = useRef<Set<string>>(new Set());
  const initializedRef = useRef(false);
  const previousContainerSizeRef = useRef({ width: 0, height: 0 });
  const resizingDimensionsRef = useRef<{
    width: number;
    height: number;
  } | null>(null);
  const x = useMotionValue(0);

  const GAP = 8;
  const activeIndex = screens.findIndex((s) => s.id === activeScreenId);
  const safeActiveIndex = activeIndex >= 0 ? activeIndex : 0;

  // Track container size with ResizeObserver so we update when the sidebar
  // is toggled (flex layout changes) or the window is resized.
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const updateSize = () => {
      const width = el.offsetWidth;
      const height = el.offsetHeight;
      setContainerSize((prev) =>
        prev.width === width && prev.height === height
          ? prev
          : { width, height },
      );
    };

    updateSize();
    const ro = new ResizeObserver(updateSize);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // Sync screen dimensions: fill any uninitialized (0,0) screens and
  // recalculate all when viewport/container size changes.
  useEffect(() => {
    if (containerSize.width <= 0 || containerSize.height <= 0) return;

    const hasUninitialized = screens.some(
      (s) => s.width === 0 || s.height === 0,
    );
    const sizeChanged =
      Math.abs(containerSize.width - previousContainerSizeRef.current.width) >
        10 ||
      Math.abs(containerSize.height - previousContainerSizeRef.current.height) >
        10;

    if (hasUninitialized) {
      // Newly added screens or first load: set full container size for any with 0,0
      if (!initializedRef.current) initializedRef.current = true;
      previousContainerSizeRef.current = { ...containerSize };
      onScreensUpdate(
        screens.map((screen) =>
          screen.width === 0 || screen.height === 0
            ? {
                ...screen,
                width: containerSize.width,
                height: containerSize.height,
              }
            : screen,
        ),
      );
    } else if (sizeChanged) {
      previousContainerSizeRef.current = { ...containerSize };
      // Sidebar toggle / viewport resize: update each screen based on whether it was manually resized
      onScreensUpdate(
        screens.map((screen) => {
          const wasManuallyResized = manuallyResizedScreensRef.current.has(
            screen.id,
          );
          if (wasManuallyResized) {
            // Keep current size, just ensure within bounds
            return {
              ...screen,
              width: Math.min(screen.width, containerSize.width),
              height: Math.min(screen.height, containerSize.height),
            };
          } else {
            // Reset to full width
            return {
              ...screen,
              width: containerSize.width,
              height: containerSize.height,
            };
          }
        }),
      );
    }
  }, [containerSize, screens, onScreensUpdate]);

  // Position active screen with directional gravity based on last resize direction
  useEffect(() => {
    if (!isDragging && !isResizing && containerSize.width > 0) {
      const activeScreen = screens[safeActiveIndex];
      const activeScreenWidth = activeScreen?.width || containerSize.width;
      const leftEdgeOfActive = safeActiveIndex * (containerSize.width + GAP);
      const availableSpace = containerSize.width - activeScreenWidth;

      let targetX: number;

      // Apply gravity based on last resize direction
      if (resizeDirection === "both") {
        // Was resized from both sides - center to show both adjacent screens
        targetX = -leftEdgeOfActive + availableSpace / 2;
      } else if (resizeDirection === "left") {
        // Was resizing from left - keep screen pushed right
        targetX = -leftEdgeOfActive + availableSpace;
      } else if (resizeDirection === "right") {
        // Was resizing from right - keep screen pushed left
        targetX = -leftEdgeOfActive;
      } else {
        // Default: push to left (for initial state or after screen change)
        targetX = -leftEdgeOfActive;
      }

      animate(x, targetX, {
        type: "spring",
        stiffness: 300,
        damping: 30,
      });
    }
  }, [
    safeActiveIndex,
    containerSize.width,
    isDragging,
    isResizing,
    resizeDirection,
    screens,
    x,
  ]);

  const handleDragEnd = (_event: any, info: any) => {
    if (isResizing) return;
    setIsDragging(false);
    const offset = info.offset.x;
    const velocity = info.velocity.x;

    let newIndex = safeActiveIndex;

    if (Math.abs(velocity) > 500) {
      newIndex = velocity > 0 ? safeActiveIndex - 1 : safeActiveIndex + 1;
    } else if (Math.abs(offset) > containerSize.width / 4) {
      newIndex = offset > 0 ? safeActiveIndex - 1 : safeActiveIndex + 1;
    }

    newIndex = Math.max(0, Math.min(screens.length - 1, newIndex));

    if (newIndex !== safeActiveIndex) {
      onScreenChange(screens[newIndex].id);
      // Reset resize direction and tracking when changing screens
      setResizeDirection(null);
      resizeSidesRef.current = { left: false, right: false };
    }
  };

  const renderScreen = (screen: Screen) => {
    switch (screen.type) {
      case "terminal":
        return <TerminalScreen />;
      case "code":
        return <CodeScreen />;
      case "browser":
        return <BrowserScreen />;
      case "welcome":
        return <WelcomeScreen sidebarExpanded={sidebarExpanded} />;
      case "custom":
        return <CustomScreen title={screen.title} dockIcon={screen.dockIcon} />;
      default:
        return <WelcomeScreen sidebarExpanded={sidebarExpanded} />;
    }
  };

  return (
    <div ref={containerRef} className="relative h-full w-full overflow-hidden">
      <motion.div
        className="flex h-full"
        style={{ x }}
        // Drag functionality commented out to allow text selection
        // drag={!isResizing && "x"}
        // dragConstraints={{
        //   left: -(screens.length - 1) * (containerSize.width + GAP),
        //   right: containerSize.width,
        // }}
        // dragElastic={0.1}
        // onDragStart={() => {
        //   if (!isResizing) setIsDragging(true);
        // }}
        // onDragEnd={handleDragEnd}
      >
        {/* Render last screen before first for circular wrapping */}
        {safeActiveIndex === 0 &&
          resizeDirection &&
          (resizeDirection === "left" || resizeDirection === "both") &&
          screens.length > 1 && (
            <div
              key={`wrap-last-${screens[screens.length - 1].id}`}
              className="relative flex items-center justify-start shrink-0"
              style={{
                width: containerSize.width,
                minWidth: containerSize.width,
                height: containerSize.height,
                position: "absolute",
                left: -(containerSize.width + GAP),
              }}
            >
              <div
                className="h-full w-full cursor-pointer"
                style={{
                  width: screens[screens.length - 1].width,
                  height: screens[screens.length - 1].height,
                }}
                onClick={() => onScreenChange(screens[screens.length - 1].id)}
              >
                {renderScreen(screens[screens.length - 1])}
              </div>
            </div>
          )}

        {screens.map((screen, index) => {
          const isActive = index === safeActiveIndex;

          // Use resizing dimensions if currently resizing this screen
          const screenWidth =
            isActive && isResizing && resizingDimensionsRef.current
              ? resizingDimensionsRef.current.width
              : screen.width;
          const screenHeight =
            isActive && isResizing && resizingDimensionsRef.current
              ? resizingDimensionsRef.current.height
              : screen.height;

          // Wrapper width: active screen uses its actual width (live during resize), others use full width
          const wrapperWidth = isActive ? screenWidth : containerSize.width;

          return (
            <div
              key={screen.id}
              className="relative flex items-center justify-start shrink-0"
              style={{
                width: wrapperWidth,
                minWidth: wrapperWidth,
                height: containerSize.height,
                marginRight: GAP,
              }}
            >
              {isActive ? (
                <Resizable
                  size={{ width: screenWidth, height: screenHeight }}
                  onResizeStart={() => {
                    console.log(
                      "onResizeStart - resizeSides:",
                      resizeSidesRef.current,
                    );
                    setIsResizing(true);
                    manuallyResizedScreensRef.current.add(screen.id);
                    resizingDimensionsRef.current = {
                      width: screen.width,
                      height: screen.height,
                    };
                  }}
                  onResize={(_e, direction, _ref, d) => {
                    const newWidth = screen.width + d.width;
                    const newHeight = screen.height + d.height;

                    // Track which sides have been resized
                    const dir = direction as string;
                    const isResizingLeft = dir.includes("left");
                    const isResizingRight = dir.includes("right");

                    if (isResizingLeft) {
                      resizeSidesRef.current.left = true;
                    }
                    if (isResizingRight) {
                      resizeSidesRef.current.right = true;
                    }

                    console.log(
                      "onResize - direction:",
                      dir,
                      "resizeSides:",
                      resizeSidesRef.current,
                    );

                    // Determine overall direction
                    if (
                      resizeSidesRef.current.left &&
                      resizeSidesRef.current.right
                    ) {
                      setResizeDirection("both");
                      console.log("Setting direction to BOTH");
                    } else if (isResizingLeft && !isResizingRight) {
                      setResizeDirection("left");
                    } else if (isResizingRight && !isResizingLeft) {
                      setResizeDirection("right");
                    }

                    // Store dimensions in ref to avoid state updates during resize
                    resizingDimensionsRef.current = {
                      width: newWidth,
                      height: newHeight,
                    };

                    // Force re-render to update wrapper width live
                    setResizeCounter((prev) => prev + 1);

                    // Directional gravity based on resize direction
                    const leftEdgeOfActive =
                      safeActiveIndex * (containerSize.width + GAP);
                    const availableSpace = containerSize.width - newWidth;

                    let targetX: number;

                    if (
                      resizeSidesRef.current.left &&
                      resizeSidesRef.current.right
                    ) {
                      // Resized from both sides - center to show both adjacent screens
                      targetX = -leftEdgeOfActive + availableSpace / 2;
                    } else if (isResizingLeft && !isResizingRight) {
                      // Resizing from left - push screen to the right to reveal left screen
                      targetX = -leftEdgeOfActive + availableSpace;
                    } else if (isResizingRight && !isResizingLeft) {
                      // Resizing from right - push screen to the left to reveal right screen
                      targetX = -leftEdgeOfActive;
                    } else {
                      // Resizing from corners or top/bottom - keep centered
                      targetX = -leftEdgeOfActive + availableSpace / 2;
                    }

                    x.set(targetX);
                  }}
                  onResizeStop={(_e, _direction, _ref, d) => {
                    const finalWidth = screen.width + d.width;
                    const finalHeight = screen.height + d.height;

                    console.log(
                      "onResizeStop - final resizeSides:",
                      resizeSidesRef.current,
                    );

                    // Update state only when resize is complete
                    onScreenResize(screen.id, finalWidth, finalHeight);
                    resizingDimensionsRef.current = null;
                    setIsResizing(false);
                    setResizeCounter(0);
                  }}
                  minWidth={400}
                  minHeight={300}
                  maxWidth={containerSize.width}
                  maxHeight={containerSize.height}
                  className="relative"
                  enable={{
                    top: true,
                    right: true,
                    bottom: true,
                    left: true,
                    topRight: true,
                    bottomRight: true,
                    bottomLeft: true,
                    topLeft: true,
                  }}
                >
                  <motion.div
                    initial={{ scale: 0.95, opacity: 0 }}
                    animate={{ scale: 1, opacity: 1 }}
                    className="h-full w-full"
                  >
                    {renderScreen(screen)}
                  </motion.div>
                </Resizable>
              ) : (
                <div
                  className="h-full w-full cursor-pointer"
                  style={{ width: screenWidth, height: screenHeight }}
                  onClick={() => onScreenChange(screen.id)}
                >
                  {renderScreen(screen)}
                </div>
              )}
            </div>
          );
        })}

        {/* Render first screen after last for circular wrapping */}
        {safeActiveIndex === screens.length - 1 &&
          resizeDirection &&
          (resizeDirection === "right" || resizeDirection === "both") &&
          screens.length > 1 && (
            <div
              key={`wrap-first-${screens[0].id}`}
              className="relative flex items-center justify-start shrink-0"
              style={{
                width: containerSize.width,
                minWidth: containerSize.width,
                height: containerSize.height,
                marginLeft: GAP,
              }}
            >
              <div
                className="h-full w-full cursor-pointer"
                style={{ width: screens[0].width, height: screens[0].height }}
                onClick={() => onScreenChange(screens[0].id)}
              >
                {renderScreen(screens[0])}
              </div>
            </div>
          )}
      </motion.div>
    </div>
  );
}
