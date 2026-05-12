"use client";

import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { Folder, Plus } from "lucide-react";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/liquidglass/www/components/ui/context-menu";
import { cn } from "@/liquidglass/www/lib/utils";
import { DraggableLogo } from "./draggable-logo";
import type { SVGLogo } from "./types";

interface LogoContainerProps {
  logos: SVGLogo[];
  isMounted: boolean;
  logoContainerHovered: boolean;
  onRemoveLogo: (logoId: number) => void;
  onRenameLogo: (logoId: number, newTitle: string) => void;
  onAddNewTab: () => void;
  onCreateFolder: () => void;
  overId: string | null;
  dropPosition: "before" | "after" | "inside" | null;
}

export function LogoContainer({
  logos,
  isMounted,
  logoContainerHovered,
  onRemoveLogo,
  onRenameLogo,
  onAddNewTab,
  onCreateFolder,
  overId,
  dropPosition,
}: LogoContainerProps) {
  const { setNodeRef } = useSortable({
    id: "logo-container",
    data: { type: "logo-container" },
  });

  return (
    <ContextMenu modal={false}>
      <ContextMenuTrigger asChild>
        <div
          ref={setNodeRef}
          className={cn(
            "shrink-0 p-2 transition-all duration-200 rounded-md",
            logoContainerHovered && "bg-primary/20 ring-2 ring-primary",
          )}
        >
          <div className="mx-auto grid w-[95%] grid-cols-4 gap-2">
            {isMounted ? (
              <SortableContext
                items={logos.map((l) => `logo-${l.id}`)}
                strategy={verticalListSortingStrategy}
              >
                {logos.map((logo) => (
                  <DraggableLogo
                    key={logo.id}
                    logo={logo}
                    overId={overId}
                    dropPosition={dropPosition}
                    onRemoveLogo={onRemoveLogo}
                    onRenameLogo={onRenameLogo}
                  />
                ))}
              </SortableContext>
            ) : (
              logos.map((logo) => {
                const LogoComponent = logo.component;
                return (
                  <div
                    key={logo.id}
                    className="bg-background/90 hover:bg-accent flex h-16 items-center justify-center rounded-md transition-colors"
                    title={logo.title}
                  >
                    <LogoComponent className="h-6 w-6" />
                  </div>
                );
              })
            )}
          </div>
        </div>
      </ContextMenuTrigger>
      <ContextMenuContent className="border-border bg-card w-56">
        <ContextMenuItem
          onSelect={(e) => {
            e.preventDefault();
            onAddNewTab();
          }}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          <Plus className="mr-2 h-4 w-4" />
          New Tab Below
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => {
            e.preventDefault();
            onCreateFolder();
          }}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          <Folder className="mr-2 h-4 w-4" />
          New Folder
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
