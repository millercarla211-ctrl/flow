"use client";

import { Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import * as React from "react";
import { Button } from "@/liquidglass/www/components/ui/button";

export function ThemeSwitcher() {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = React.useState(false);

  React.useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    return null;
  }

  return (
    <div className="flex items-center justify-center gap-2">
      <Button
        variant={theme === "light" ? "default" : "outline"}
        size="lg"
        onClick={() => setTheme("light")}
        className="gap-2"
      >
        <Sun className="h-5 w-5" />
        Light
      </Button>
      <Button
        variant={theme === "dark" ? "default" : "outline"}
        size="lg"
        onClick={() => setTheme("dark")}
        className="gap-2"
      >
        <Moon className="h-5 w-5" />
        Dark
      </Button>
    </div>
  );
}
