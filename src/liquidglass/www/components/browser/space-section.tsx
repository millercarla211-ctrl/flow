"use client";

import { useSortable } from "@dnd-kit/sortable";
import {
  ChevronDown,
  ChevronRight,
  Folder,
  MoreHorizontal,
  Plus,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/liquidglass/www/components/ui/context-menu";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/liquidglass/www/components/ui/dropdown-menu";
import type { DropPosition, Workspace } from "./types";

interface SpaceSectionProps {
  workspace: Workspace | undefined;
  spaceCollapsed: boolean;
  isSpaceAreaHovered: boolean;
  dropdownOpen: boolean;
  overId: string | null;
  dropPosition: DropPosition;
  workspaces: Workspace[];
  activeWorkspace: string;
  onToggleCollapse: () => void;
  onSetSpaceAreaHovered: (hovered: boolean) => void;
  onSetDropdownOpen: (open: boolean) => void;
  onSetActiveWorkspace: (id: string) => void;
  onRenameWorkspace: () => void;
  onEditWorkspaceIcon: () => void;
  onUnloadSpace: () => void;
  onDeleteSpace: () => void;
  onCreateFolder: () => void;
  onOpenWorkspaceDialog: () => void;
  renderWorkspaceIcon: (
    workspace: Workspace,
    isActive: boolean,
  ) => React.ReactElement;
}

export function SpaceSection({
  workspace,
  spaceCollapsed,
  isSpaceAreaHovered,
  dropdownOpen,
  overId,
  dropPosition,
  workspaces,
  activeWorkspace,
  onToggleCollapse,
  onSetSpaceAreaHovered,
  onSetDropdownOpen,
  onSetActiveWorkspace,
  onRenameWorkspace,
  onEditWorkspaceIcon,
  onUnloadSpace,
  onDeleteSpace,
  onCreateFolder,
  onOpenWorkspaceDialog,
  renderWorkspaceIcon,
}: SpaceSectionProps) {
  const { setNodeRef } = useSortable({
    id: "space-section",
    data: { type: "space" },
  });

  return (
    <div
      className="group/spacearea shrink-0 px-2 pt-3 pb-2"
      onMouseEnter={() => onSetSpaceAreaHovered(true)}
      onMouseLeave={() => onSetSpaceAreaHovered(false)}
    >
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <div
            ref={setNodeRef}
            className="hover:bg-accent relative flex h-10 w-full cursor-pointer items-center rounded-md px-2 transition-colors"
          >
            <AnimatePresence>
              {(isSpaceAreaHovered || dropdownOpen) && (
                <motion.div
                  initial={{ width: 0, opacity: 0 }}
                  animate={{ width: "auto", opacity: 1 }}
                  exit={{ width: 0, opacity: 0 }}
                  transition={{ duration: 0.2 }}
                  className="shrink-0 overflow-hidden"
                  onClick={onToggleCollapse}
                >
                  <motion.div
                    animate={{ rotate: spaceCollapsed ? 0 : 180 }}
                    transition={{ duration: 0.2 }}
                  >
                    <ChevronDown className="text-muted-foreground h-4 w-4" />
                  </motion.div>
                </motion.div>
              )}
            </AnimatePresence>
            <motion.span
              className="text-muted-foreground min-w-0 flex-1 truncate text-left text-sm font-medium"
              animate={{ marginLeft: isSpaceAreaHovered ? 8 : 0 }}
              transition={{ duration: 0.2 }}
              onClick={onToggleCollapse}
            >
              {workspace?.name || "Space"}
            </motion.span>
            <AnimatePresence>
              {(isSpaceAreaHovered || dropdownOpen) && (
                <motion.div
                  initial={{ width: 0, opacity: 0 }}
                  animate={{ width: "auto", opacity: 1 }}
                  exit={{ width: 0, opacity: 0 }}
                  transition={{ duration: 0.2 }}
                  className="shrink-0 overflow-hidden"
                >
                  <DropdownMenu
                    modal={false}
                    open={dropdownOpen}
                    onOpenChange={onSetDropdownOpen}
                  >
                    <DropdownMenuTrigger asChild>
                      <button
                        className="hover:bg-accent rounded p-0.5"
                        onClick={(e) => e.stopPropagation()}
                        onContextMenu={(e) => e.stopPropagation()}
                      >
                        <MoreHorizontal className="text-muted-foreground h-4 w-4" />
                      </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent
                      side="right"
                      className="border-border bg-card w-56"
                    >
                      <DropdownMenuItem
                        onSelect={(e) => {
                          e.preventDefault();
                          onRenameWorkspace();
                        }}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        Change Name
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onSelect={(e) => {
                          e.preventDefault();
                          onEditWorkspaceIcon();
                        }}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        Change Icon
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onSelect={(e) => e.preventDefault()}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        Edit Theme
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onSelect={(e) => e.preventDefault()}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        Set Profile
                        <ChevronRight className="ml-auto h-4 w-4" />
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onSelect={(e) => {
                          e.preventDefault();
                          onUnloadSpace();
                        }}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        Unload Space
                      </DropdownMenuItem>
                      <div className="border-border my-1 border-t" />
                      <DropdownMenuItem
                        onSelect={(e) => e.preventDefault()}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        <div className="mr-2 h-4 w-4 flex items-center justify-center shrink-0">
                          ✓
                        </div>
                        <span className="max-w-[75%] truncate">
                          {workspace?.name || "Space"}
                        </span>
                      </DropdownMenuItem>
                      {workspaces
                        .filter((w) => w.id !== activeWorkspace)
                        .map((w) => (
                          <DropdownMenuItem
                            key={w.id}
                            onSelect={(e) => {
                              e.preventDefault();
                              onSetActiveWorkspace(w.id);
                            }}
                            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                          >
                            <div className="mr-2 h-4 w-4 flex items-center justify-center shrink-0">
                              {renderWorkspaceIcon(w, false)}
                            </div>
                            <span className="max-w-[75%] truncate">
                              {w.name}
                            </span>
                          </DropdownMenuItem>
                        ))}
                      <div className="border-border my-1 border-t" />
                      <DropdownMenuItem
                        onSelect={(e) => e.preventDefault()}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        Reorder Spaces
                      </DropdownMenuItem>
                      <div className="border-border my-1 border-t" />
                      <DropdownMenuItem
                        onSelect={(e) => {
                          e.preventDefault();
                          onCreateFolder();
                        }}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        <Folder className="mr-2 h-4 w-4" />
                        Create Folder
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onSelect={(e) => {
                          e.preventDefault();
                          onOpenWorkspaceDialog();
                        }}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        Create Space
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onSelect={(e) => {
                          e.preventDefault();
                          onDeleteSpace();
                        }}
                        className="text-destructive focus:bg-accent focus:text-destructive"
                      >
                        Delete Space
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </motion.div>
              )}
            </AnimatePresence>
            {overId === "space-section" && dropPosition === "before" && (
              <div className="absolute -top-1 left-0 right-0 z-100 flex items-center pointer-events-none">
                <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
                <div className="bg-primary h-0.5 flex-1" />
              </div>
            )}
            {overId === "space-section" && dropPosition === "after" && (
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
              onCreateFolder();
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            <Folder className="mr-2 h-4 w-4" />
            Create Folder
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e) => {
              e.preventDefault();
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            <Plus className="mr-2 h-4 w-4" />
            New Tab
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Collapse All Folders
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Expand All Folders
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    </div>
  );
}
