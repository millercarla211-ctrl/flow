"use client";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/liquidglass/www/components/ui/dialog";
import { cn } from "@/liquidglass/www/lib/utils";
import { getIconForScreen } from "./dock-icons";
import type { Screen } from "./types";

interface ScreenGridDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  screens: Screen[];
  activeScreenId: string;
  onSelectScreen: (screenId: string) => void;
}

export function ScreenGridDialog({
  open,
  onOpenChange,
  screens,
  activeScreenId,
  onSelectScreen,
}: ScreenGridDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl gap-4 sm:rounded-lg">
        <DialogHeader>
          <DialogTitle>Screens</DialogTitle>
        </DialogHeader>
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
          {screens.map((screen) => {
            const Icon = getIconForScreen(screen);
            const isActive = screen.id === activeScreenId;
            return (
              <button
                key={screen.id}
                type="button"
                onClick={() => {
                  onSelectScreen(screen.id);
                  onOpenChange(false);
                }}
                className={cn(
                  "flex flex-col items-center gap-2 rounded-lg border-2 p-4 transition-colors hover:bg-accent/50",
                  isActive
                    ? "border-primary bg-primary/10"
                    : "border-border bg-muted/30 hover:border-primary/50",
                )}
              >
                <div
                  className={cn(
                    "flex h-12 w-12 items-center justify-center rounded-md",
                    isActive
                      ? "bg-primary text-primary-foreground"
                      : "bg-muted",
                  )}
                >
                  <Icon className="h-6 w-6" />
                </div>
                <span className="truncate text-center text-sm font-medium max-w-full">
                  {screen.title}
                </span>
              </button>
            );
          })}
        </div>
      </DialogContent>
    </Dialog>
  );
}
