"use client";

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { X } from "lucide-react";
import { cn } from "@/liquidglass/www/lib/utils";
import type { DropPosition, Tab } from "./types";

interface DraggableTabInFolderProps {
  tab: Tab;
  overId: string | null;
  dropPosition: DropPosition;
  onSetActiveTab: (tabId: string) => void;
  onCloseTab: (tabId: string) => void;
}

export function DraggableTabInFolder({
  tab,
  overId,
  dropPosition,
  onSetActiveTab,
  onCloseTab,
}: DraggableTabInFolderProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: tab.id,
    data: { type: "tab", tab },
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      className={cn(
        "group/welcome text-muted-foreground hover:bg-accent hover:text-accent-foreground relative flex h-9 w-full max-w-[95%] cursor-grab items-center gap-2 rounded-md px-2 text-sm transition-colors active:cursor-grabbing",
        isDragging && "opacity-50",
      )}
      onClick={() => onSetActiveTab(tab.id)}
    >
      <div className="bg-primary h-4 w-4 shrink-0 rounded-sm" />
      <span className="text-foreground min-w-0 flex-1 truncate pr-6 text-xs">
        {tab.title}
      </span>
      <button
        className="absolute right-2 flex h-3 w-3 shrink-0 cursor-pointer items-center justify-center opacity-0 transition-opacity group-hover/welcome:opacity-100"
        onClick={(e) => {
          e.stopPropagation();
          onCloseTab(tab.id);
        }}
      >
        <X className="h-3 w-3" />
      </button>
      {overId === tab.id && dropPosition === "before" && (
        <div className="absolute -top-1 left-0 right-0 z-100 flex items-center pointer-events-none">
          <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
          <div className="bg-primary h-0.5 flex-1" />
        </div>
      )}
      {overId === tab.id && dropPosition === "after" && (
        <div className="absolute -bottom-1 left-0 right-0 z-100 flex items-center pointer-events-none">
          <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
          <div className="bg-primary h-0.5 flex-1" />
        </div>
      )}
    </div>
  );
}
