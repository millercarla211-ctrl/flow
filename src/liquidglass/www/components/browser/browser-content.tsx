"use client";

import { useState } from "react";
import { getRandomDockIconName } from "@/liquidglass/www/components/screens/dock-icons";
import { MacOSDock } from "@/liquidglass/www/components/screens/macos-dock";
import { ScreenCarousel } from "@/liquidglass/www/components/screens/screen-carousel";
import { ScreenGridDialog } from "@/liquidglass/www/components/screens/screen-grid-dialog";
import type { Screen } from "@/liquidglass/www/components/screens/types";

interface BrowserContentProps {
  sidebarExpanded: boolean;
}

const INITIAL_SCREENS: Screen[] = [
  { id: "welcome", type: "welcome", title: "Welcome", width: 0, height: 0 },
  { id: "terminal", type: "terminal", title: "Terminal", width: 0, height: 0 },
  { id: "code", type: "code", title: "Code Editor", width: 0, height: 0 },
  { id: "browser", type: "browser", title: "Browser", width: 0, height: 0 },
];

export function BrowserContent({ sidebarExpanded }: BrowserContentProps) {
  const [activeScreenId, setActiveScreenId] = useState<string>("welcome");
  const [screens, setScreens] = useState<Screen[]>(INITIAL_SCREENS);
  const [gridOpen, setGridOpen] = useState(false);

  const handleScreenResize = (id: string, width: number, height: number) => {
    setScreens((prev) =>
      prev.map((screen) =>
        screen.id === id ? { ...screen, width, height } : screen,
      ),
    );
  };

  const handleAddScreen = () => {
    const number = screens.length + 1;
    const newScreen: Screen = {
      id: `screen-${Date.now()}`,
      type: "custom",
      title: `Screen ${number}`,
      width: 0,
      height: 0,
      dockIcon: getRandomDockIconName(),
    };
    setScreens((prev) => [...prev, newScreen]);
    setActiveScreenId(newScreen.id);
  };

  return (
    <div className="relative flex h-full w-full items-center justify-center overflow-hidden">
      <MacOSDock
        screens={screens}
        activeScreenId={activeScreenId}
        onScreenChange={setActiveScreenId}
        onAddScreen={handleAddScreen}
        onToggleViewMode={() => setGridOpen(true)}
        sidebarExpanded={sidebarExpanded}
      />
      <ScreenCarousel
        activeScreenId={activeScreenId}
        screens={screens}
        onScreenChange={setActiveScreenId}
        onScreenResize={handleScreenResize}
        onScreensUpdate={setScreens}
        sidebarExpanded={sidebarExpanded}
      />
      <ScreenGridDialog
        open={gridOpen}
        onOpenChange={setGridOpen}
        screens={screens}
        activeScreenId={activeScreenId}
        onSelectScreen={setActiveScreenId}
      />
    </div>
  );
}
