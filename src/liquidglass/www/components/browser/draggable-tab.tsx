"use client";

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { ChevronRight, X } from "lucide-react";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/liquidglass/www/components/ui/context-menu";
import { cn } from "@/liquidglass/www/lib/utils";
import type { DropPosition, Tab } from "./types";

interface DraggableTabProps {
  tab: Tab;
  activeTab: string;
  overId: string | null;
  dropPosition: DropPosition;
  onSetActiveTab: (tabId: string) => void;
  onCloseTab: (tabId: string) => void;
  onAddNewTab: () => void;
  onCreateFolder: () => void;
}

export function DraggableTab({
  tab,
  activeTab,
  overId,
  dropPosition,
  onSetActiveTab,
  onCloseTab,
  onAddNewTab,
  onCreateFolder,
}: DraggableTabProps) {
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
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <div
          ref={setNodeRef}
          style={style}
          {...attributes}
          {...listeners}
          className={cn(
            "group/item relative flex h-9 max-w-[93.5%] cursor-grab items-center gap-2 rounded-md px-2 text-sm select-none active:cursor-grabbing transition-colors",
            activeTab === tab.id
              ? "bg-accent text-accent-foreground"
              : "text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground",
            isDragging && "opacity-50",
          )}
          onClick={() => onSetActiveTab(tab.id)}
        >
          <div className="bg-destructive h-4 w-4 shrink-0 rounded-sm" />
          <span className="min-w-0 flex-1 truncate pr-6 text-xs">
            {tab.title}
          </span>
          <button
            className="absolute right-2 flex h-3 w-3 shrink-0 cursor-pointer items-center justify-center opacity-0 transition-opacity group-hover/item:opacity-100"
            onClick={(e) => {
              e.stopPropagation();
              onCloseTab(tab.id);
            }}
          >
            <X className="h-3 w-3" />
          </button>
          {overId === tab.id && dropPosition === "before" && (
            <div className="absolute -top-1 left-0 right-0 z-[1000000000000000000000000] flex items-center">
              <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
              <div className="bg-primary h-0.5 flex-1" />
            </div>
          )}
          {overId === tab.id && dropPosition === "after" && (
            <div className="absolute -bottom-1 left-0 right-0 z-[1000000000000000000000000] flex items-center">
              <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
              <div className="bg-primary h-0.5 flex-1" />
            </div>
          )}
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
          New Tab Below
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => {
            e.preventDefault();
            onCreateFolder();
          }}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          New Folder
        </ContextMenuItem>
        <div className="border-border my-1 border-t" />
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Reload Tab
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Mute Tab
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Remove from Essentials
        </ContextMenuItem>
        <div className="border-border my-1 border-t" />
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Change Icon...
        </ContextMenuItem>
        <div className="border-border my-1 border-t" />
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Duplicate Tab
        </ContextMenuItem>
        <div className="border-border my-1 border-t" />
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Bookmark Tab...
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Move Tab
          <ChevronRight className="ml-auto h-4 w-4" />
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Share
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Open in New Container Tab
          <ChevronRight className="ml-auto h-4 w-4" />
        </ContextMenuItem>
        <div className="border-border my-1 border-t" />
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Select All Tabs
        </ContextMenuItem>
        <div className="border-border my-1 border-t" />
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-muted-foreground/50 focus:bg-accent focus:text-muted-foreground/50"
        >
          Close Duplicate Tabs
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Close Multiple Tabs
          <ChevronRight className="ml-auto h-4 w-4" />
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Reopen Closed Tab
        </ContextMenuItem>
        <div className="border-border my-1 border-t" />
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Replace Essential URL with Current
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Reset Essential Tab
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
