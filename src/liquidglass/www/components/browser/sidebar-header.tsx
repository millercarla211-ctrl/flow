"use client";

import { Cog, PanelLeft, ShieldAlert, ShieldBan, Trash2 } from "lucide-react";
import { Button } from "@/liquidglass/www/components/ui/button";

interface SidebarHeaderProps {
  sidebarExpanded: boolean;
  onToggleSidebar: () => void;
}

export function SidebarHeader({
  sidebarExpanded,
  onToggleSidebar,
}: SidebarHeaderProps) {
  return (
    <div className="grid h-11 shrink-0 grid-cols-2 gap-px">
      {sidebarExpanded ? (
        <>
          <div className="bg-card flex items-center gap-1 px-2">
            <Button
              variant="ghost"
              size="icon"
              className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
              onClick={onToggleSidebar}
            >
              <PanelLeft className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
            >
              <Cog className="h-4 w-4" />
            </Button>
          </div>
          <div className="bg-card flex items-center justify-end gap-1 px-2">
            <Button
              variant="ghost"
              size="icon"
              className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
            >
              <ShieldAlert className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
            >
              <ShieldBan className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        </>
      ) : (
        <div className="bg-card col-span-2 flex items-center justify-center">
          <Button
            variant="ghost"
            size="icon"
            className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
            onClick={onToggleSidebar}
          >
            <PanelLeft className="h-4 w-4" />
          </Button>
        </div>
      )}
    </div>
  );
}
