"use client";

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { ChevronRight, Folder, FolderOpen, X } from "lucide-react";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/liquidglass/www/components/ui/context-menu";
import { cn } from "@/liquidglass/www/lib/utils";
import type { DropPosition, TabFolder } from "./types";

interface DroppableFolderProps {
  folder: TabFolder;
  overId: string | null;
  dropPosition: DropPosition;
  onToggleFolder: (folderId: string) => void;
  onDeleteFolder: (folderId: string) => void;
  onRenameFolder: (folderId: string, newName: string) => void;
  onUnloadAllTabs: (folderId: string) => void;
  onCreateSubfolder: (parentId: string) => void;
  onUnpackFolder: (folderId: string) => void;
}

export function DroppableFolder({
  folder,
  overId,
  dropPosition,
  onToggleFolder,
  onDeleteFolder,
  onRenameFolder,
  onUnloadAllTabs,
  onCreateSubfolder,
  onUnpackFolder,
}: DroppableFolderProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: folder.id,
    data: { type: "folder", folder },
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const isDropTarget = overId === folder.id && dropPosition === "inside";

  return (
    <ContextMenu modal={false}>
      <ContextMenuTrigger asChild>
        <div
          ref={setNodeRef}
          style={style}
          {...attributes}
          {...listeners}
          className={cn(
            "text-muted-foreground hover:bg-accent hover:text-accent-foreground group/item relative flex h-10 w-full max-w-[95%] cursor-grab items-center gap-2 rounded-md px-2 text-sm transition-colors active:cursor-grabbing",
            isDropTarget && "bg-accent text-accent-foreground",
            isDragging && "opacity-50",
          )}
          onClick={() => onToggleFolder(folder.id)}
          onContextMenu={(e) => e.stopPropagation()}
        >
          {folder.collapsed ? (
            <Folder className="text-primary h-4 w-4 shrink-0" />
          ) : (
            <FolderOpen className="text-primary h-4 w-4 shrink-0" />
          )}
          <span className="text-foreground min-w-0 flex-1 truncate pr-6 text-sm">
            {folder.name}
          </span>
          <button
            className="absolute right-2 flex h-4 w-4 shrink-0 cursor-pointer items-center justify-center opacity-0 transition-opacity group-hover/item:opacity-100"
            onClick={(e) => {
              e.stopPropagation();
              onDeleteFolder(folder.id);
            }}
          >
            <X className="h-3 w-3" />
          </button>
          {overId === folder.id && dropPosition === "before" && (
            <div className="absolute -top-1 left-0 right-0 z-100 flex items-center pointer-events-none">
              <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
              <div className="bg-primary h-0.5 flex-1" />
            </div>
          )}
          {overId === folder.id && dropPosition === "after" && (
            <div className="absolute -bottom-1 left-0 right-0 z-100 flex items-center pointer-events-none">
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
            const newName = prompt("Enter new folder name:", folder.name);
            if (newName?.trim()) {
              onRenameFolder(folder.id, newName.trim());
            }
          }}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Rename Folder
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Change Icon...
        </ContextMenuItem>
        <div className="border-border my-1 border-t" />
        <ContextMenuItem
          onSelect={(e) => {
            e.preventDefault();
            onUnloadAllTabs(folder.id);
          }}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Unload All Tabs
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => {
            e.preventDefault();
            onCreateSubfolder(folder.id);
          }}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          New Subfolder
        </ContextMenuItem>
        <div className="border-border my-1 border-t" />
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Change Space...
          <ChevronRight className="ml-auto h-4 w-4" />
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => e.preventDefault()}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Convert folder to Space
        </ContextMenuItem>
        <div className="border-border my-1 border-t" />
        <ContextMenuItem
          onSelect={(e) => {
            e.preventDefault();
            onUnpackFolder(folder.id);
          }}
          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          Unpack Folder
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={(e) => {
            e.preventDefault();
            onDeleteFolder(folder.id);
          }}
          className="text-destructive focus:bg-accent focus:text-destructive"
        >
          Delete Folder
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
