"use client";

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { X } from "lucide-react";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/liquidglass/www/components/ui/context-menu";
import { cn } from "@/liquidglass/www/lib/utils";
import type { DropPosition, SVGLogo } from "./types";

interface DraggableLogoProps {
  logo: SVGLogo;
  overId: string | null;
  dropPosition: DropPosition;
  onRemoveLogo: (logoId: number) => void;
  onRenameLogo: (logoId: number, newTitle: string) => void;
}

export function DraggableLogo({
  logo,
  overId,
  dropPosition,
  onRemoveLogo,
  onRenameLogo,
}: DraggableLogoProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
    isOver,
  } = useSortable({
    id: `logo-${logo.id}`,
    data: { type: "logo", logo },
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const LogoComponent = logo.component;

  return (
    <ContextMenu modal={false}>
      <ContextMenuTrigger asChild>
        <div
          ref={setNodeRef}
          style={style}
          {...attributes}
          {...listeners}
          className={cn(
            "bg-background/90 hover:bg-accent relative flex h-16 cursor-grab items-center justify-center rounded-md transition-colors active:cursor-grabbing",
            isDragging && "opacity-50",
            isOver && "ring-2 ring-primary",
          )}
          title={logo.title}
        >
          <LogoComponent className="h-6 w-6" />
          {overId === `logo-${logo.id}` && dropPosition === "before" && (
            <div className="absolute -left-1 top-0 bottom-0 z-100 flex flex-col items-center pointer-events-none">
              <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
              <div className="bg-primary w-0.5 flex-1" />
            </div>
          )}
          {overId === `logo-${logo.id}` && dropPosition === "after" && (
            <div className="absolute -right-1 top-0 bottom-0 z-100 flex flex-col items-center pointer-events-none">
              <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
              <div className="bg-primary w-0.5 flex-1" />
            </div>
          )}
        </div>
      </ContextMenuTrigger>
      <ContextMenuContent className="border-border bg-card w-56">
        <ContextMenuItem
          onSelect={(e) => {
            e.preventDefault();
            const newTitle = prompt("Enter new title:", logo.title);
            if (newTitle?.trim()) {
              onRenameLogo(logo.id, newTitle.trim());
            }
          }}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Rename
        </ContextMenuItem>
        <div className="border-border my-1 border-t" />
        <ContextMenuItem
          onSelect={(e) => {
            e.preventDefault();
            if (confirm(`Remove "${logo.title}" from quick access?`)) {
              onRemoveLogo(logo.id);
            }
          }}
          className="text-destructive focus:bg-accent focus:text-destructive"
        >
          <X className="mr-2 h-4 w-4" />
          Remove
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
