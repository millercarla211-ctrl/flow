"use client";

import * as LucideIcons from "lucide-react";
import { cn } from "@/liquidglass/www/lib/utils";
import type { Workspace } from "./types";

interface WorkspaceIconProps {
  workspace: Workspace;
  isActive?: boolean;
}

export function WorkspaceIcon({
  workspace,
  isActive = false,
}: WorkspaceIconProps) {
  if (workspace.icon.type === "emoji") {
    return (
      <div className="flex h-5 w-5 items-center justify-center">
        <span
          className={cn(
            "text-xs leading-none transition-all duration-300 ease-[cubic-bezier(0.165,0.84,0.44,1)]",
            isActive ? "" : "grayscale hover:grayscale-0",
          )}
        >
          {workspace.icon.value}
        </span>
      </div>
    );
  }

  if (workspace.icon.type === "icon") {
    const IconComponent = (LucideIcons as any)[workspace.icon.value];
    if (IconComponent) {
      return (
        <div className="flex h-5 w-5 items-center justify-center">
          <IconComponent
            className={cn(
              "h-3 w-3 transition-colors duration-300 ease-[cubic-bezier(0.165,0.84,0.44,1)]",
              isActive
                ? "text-primary"
                : "text-muted-foreground hover:text-accent-foreground",
            )}
          />
        </div>
      );
    }
  }

  return (
    <div className="flex h-5 w-5 items-center justify-center">
      <div
        className={cn(
          "h-2 w-2 rounded-full transition-all duration-300 ease-[cubic-bezier(0.165,0.84,0.44,1)]",
          isActive
            ? "bg-primary"
            : "bg-muted-foreground hover:bg-accent-foreground",
        )}
      />
    </div>
  );
}
