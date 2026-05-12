"use client";

import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import {
  ChevronLeft,
  ChevronRight,
  CircleDashed,
  Columns2,
  Copy,
  Folder,
  Grid3x3,
  Link,
  Play,
  Plus,
  Search,
  SkipBack,
  SkipForward,
  Volume2,
  X,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { Button } from "@/liquidglass/www/components/ui/button";
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
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/liquidglass/www/components/ui/popover";
import { ScrollArea } from "@/liquidglass/www/components/ui/scroll-area";
import { Youtube } from "@/liquidglass/www/components/ui/svgs/youtube";
import { cn } from "@/liquidglass/www/lib/utils";
import { DraggableTab } from "./draggable-tab";
import { DraggableTabInFolder } from "./draggable-tab-in-folder";
import { DroppableFolder } from "./droppable-folder";
import { LogoContainer } from "./logo-container";
import { SpaceSection } from "./space-section";
import type { DropPosition, SVGLogo, Tab, TabFolder, Workspace } from "./types";
import { WorkspaceIcon } from "./workspace-icon";

interface SidebarExpandedProps {
  workspaces: Workspace[];
  activeWorkspace: string;
  visibleWorkspaces: Workspace[];
  canScrollLeft: boolean;
  canScrollRight: boolean;
  downloadsOpen: boolean;
  plusMenuOpen: boolean;
  spaceCollapsed: boolean;
  isSpaceAreaHovered: boolean;
  dropdownOpen: boolean;
  mediaPlaying: boolean;
  mediaProgress: number;
  isMounted: boolean;
  logos: SVGLogo[];
  logoContainerHovered: boolean;
  activeWorkspaceFolders: TabFolder[];
  activeWorkspaceTabs: Tab[];
  activeTab: string;
  overId: string | null;
  dropPosition: DropPosition;
  folders: TabFolder[];
  onSetActiveWorkspace: (id: string) => void;
  onSetDownloadsOpen: (open: boolean) => void;
  onSetPlusMenuOpen: (open: boolean) => void;
  onSetSpaceCollapsed: (collapsed: boolean) => void;
  onSetSpaceAreaHovered: (hovered: boolean) => void;
  onSetDropdownOpen: (open: boolean) => void;
  onSetMediaPlaying: (playing: boolean) => void;
  onStartScrolling: (direction: "left" | "right") => void;
  onStopScrolling: () => void;
  onCreateWorkspace: () => void;
  onOpenWorkspaceDialog: () => void;
  onCreateFolder: () => void;
  onAddNewTab: () => void;
  onClearAllTabs: () => void;
  onSetCommandOpen: (open: boolean) => void;
  onRemoveLogo: (logoId: number) => void;
  onRenameLogo: (logoId: number, newTitle: string) => void;
  onSetActiveTab: (tabId: string) => void;
  onCloseTab: (tabId: string) => void;
  onToggleFolder: (folderId: string) => void;
  onDeleteFolder: (folderId: string) => void;
  onRenameFolder: (folderId: string, newName: string) => void;
  onUnloadAllTabs: (folderId: string) => void;
  onCreateSubfolder: (parentId: string) => void;
  onUnpackFolder: (folderId: string) => void;
  onRenameWorkspace: () => void;
  onEditWorkspaceIcon: () => void;
  onUnloadSpace: () => void;
  onDeleteSpace: () => void;
  renderWorkspaceIcon: (
    workspace: Workspace,
    isActive: boolean,
  ) => React.ReactElement;
  COLORS: string[];
}

export function SidebarExpanded(props: SidebarExpandedProps) {
  const { setNodeRef: setNewTabButtonRef } = useSortable({
    id: "new-tab-button",
    data: { type: "new-tab-button" },
  });

  const currentWorkspace = props.workspaces.find(
    (w) => w.id === props.activeWorkspace,
  );

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.15 }}
      className="flex flex-1 flex-col overflow-hidden overflow-x-hidden"
    >
      <div className="shrink-0 px-2 pt-1 pb-3">
        <button
          onClick={() => props.onSetCommandOpen(true)}
          className="bg-accent hover:bg-muted relative flex w-full items-center gap-2 rounded-lg px-3 py-2.5 transition-colors"
        >
          <Search className="text-muted-foreground h-4 w-4 shrink-0" />
          <span className="text-muted-foreground flex-1 text-left text-sm">
            Search or enter address...
          </span>
          <div className="flex items-center gap-1">
            <Link className="text-muted-foreground h-4 w-4" />
            <Grid3x3 className="text-muted-foreground h-4 w-4" />
          </div>
        </button>
      </div>

      <LogoContainer
        logos={props.logos}
        isMounted={props.isMounted}
        logoContainerHovered={props.logoContainerHovered}
        onRemoveLogo={props.onRemoveLogo}
        onRenameLogo={props.onRenameLogo}
        onAddNewTab={props.onAddNewTab}
        onCreateFolder={props.onCreateFolder}
        overId={props.overId}
        dropPosition={props.dropPosition}
      />

      <SpaceSection
        workspace={currentWorkspace}
        spaceCollapsed={props.spaceCollapsed}
        isSpaceAreaHovered={props.isSpaceAreaHovered}
        dropdownOpen={props.dropdownOpen}
        overId={props.overId}
        dropPosition={props.dropPosition}
        workspaces={props.workspaces}
        activeWorkspace={props.activeWorkspace}
        onToggleCollapse={() =>
          props.onSetSpaceCollapsed(!props.spaceCollapsed)
        }
        onSetSpaceAreaHovered={props.onSetSpaceAreaHovered}
        onSetDropdownOpen={props.onSetDropdownOpen}
        onSetActiveWorkspace={props.onSetActiveWorkspace}
        onRenameWorkspace={props.onRenameWorkspace}
        onEditWorkspaceIcon={props.onEditWorkspaceIcon}
        onUnloadSpace={props.onUnloadSpace}
        onDeleteSpace={props.onDeleteSpace}
        onCreateFolder={props.onCreateFolder}
        onOpenWorkspaceDialog={props.onOpenWorkspaceDialog}
        renderWorkspaceIcon={props.renderWorkspaceIcon}
      />

      <AnimatePresence>
        {!props.spaceCollapsed && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="overflow-hidden"
            onMouseEnter={() => props.onSetSpaceAreaHovered(true)}
            onMouseLeave={() => props.onSetSpaceAreaHovered(false)}
          >
            <div className="shrink-0 space-y-1 px-2 pb-3">
              {props.isMounted ? (
                <SortableContext
                  items={props.activeWorkspaceFolders.map((f) => f.id)}
                  strategy={verticalListSortingStrategy}
                >
                  {props.activeWorkspaceFolders.map((folder) => (
                    <div key={folder.id}>
                      <DroppableFolder
                        folder={folder}
                        overId={props.overId}
                        dropPosition={props.dropPosition}
                        onToggleFolder={props.onToggleFolder}
                        onDeleteFolder={props.onDeleteFolder}
                        onRenameFolder={props.onRenameFolder}
                        onUnloadAllTabs={props.onUnloadAllTabs}
                        onCreateSubfolder={props.onCreateSubfolder}
                        onUnpackFolder={props.onUnpackFolder}
                      />
                      <AnimatePresence>
                        {!folder.collapsed && folder.tabs.length > 0 && (
                          <motion.div
                            initial={{ height: 0, opacity: 0 }}
                            animate={{ height: "auto", opacity: 1 }}
                            exit={{ height: 0, opacity: 0 }}
                            transition={{ duration: 0.2 }}
                            className="ml-5 space-y-0.5"
                          >
                            <SortableContext
                              items={folder.tabs.map((t) => t.id)}
                              strategy={verticalListSortingStrategy}
                            >
                              {folder.tabs.map((tab) => (
                                <DraggableTabInFolder
                                  key={tab.id}
                                  tab={tab}
                                  overId={props.overId}
                                  dropPosition={props.dropPosition}
                                  onSetActiveTab={props.onSetActiveTab}
                                  onCloseTab={props.onCloseTab}
                                />
                              ))}
                            </SortableContext>
                          </motion.div>
                        )}
                      </AnimatePresence>
                    </div>
                  ))}
                </SortableContext>
              ) : (
                props.activeWorkspaceFolders.map((folder) => (
                  <div key={folder.id}>
                    <div className="text-muted-foreground hover:bg-accent hover:text-accent-foreground group/item relative flex h-10 w-full cursor-pointer items-center gap-2 rounded-md px-2 text-sm transition-colors overflow-hidden">
                      <Folder className="text-primary h-4 w-4 shrink-0" />
                      <span className="text-foreground min-w-0 flex-1 truncate pr-5 text-left text-sm">
                        {folder.name}
                      </span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      <div
        ref={setNewTabButtonRef}
        className="shrink-0 px-2 pt-3 pb-3 overflow-x-hidden"
      >
        <div className="border-border/50 relative border-t">
          <button
            onClick={props.onClearAllTabs}
            className="text-muted-foreground hover:text-accent-foreground absolute -top-2 right-0 cursor-pointer bg-card px-1 text-xs opacity-0 transition-opacity group-hover:opacity-100"
          >
            Clear
          </button>
        </div>
        <div className="pt-3">
          <Button
            onClick={props.onAddNewTab}
            className="border border-dashed px-2 text-muted-foreground hover:bg-accent hover:text-accent-foreground h-10 w-full justify-start gap-2 bg-transparent overflow-hidden"
            variant="ghost"
          >
            <Plus className="h-4 w-4 shrink-0" />
            <span className="min-w-0 flex-1 truncate text-sm text-left">
              New Tab
            </span>
          </Button>
        </div>
      </div>

      <ScrollArea className="flex-1 overflow-hidden overflow-x-hidden px-2">
        <div className="w-full max-w-full space-y-0.5 pb-4">
          {props.isMounted ? (
            <SortableContext
              items={props.activeWorkspaceTabs.map((t) => t.id)}
              strategy={verticalListSortingStrategy}
            >
              {props.activeWorkspaceTabs.map((tab) => (
                <DraggableTab
                  key={tab.id}
                  tab={tab}
                  activeTab={props.activeTab}
                  overId={props.overId}
                  dropPosition={props.dropPosition}
                  onSetActiveTab={props.onSetActiveTab}
                  onCloseTab={props.onCloseTab}
                  onAddNewTab={props.onAddNewTab}
                  onCreateFolder={props.onCreateFolder}
                />
              ))}
            </SortableContext>
          ) : (
            props.activeWorkspaceTabs.map((tab) => (
              <div
                key={tab.id}
                className="group/item relative flex h-9 w-full max-w-[95%] cursor-pointer items-center gap-2 rounded-md px-2 text-sm select-none text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground"
              >
                <div className="bg-destructive h-4 w-4 shrink-0 rounded-sm" />
                <span className="min-w-0 flex-1 truncate pr-6 text-xs">
                  {tab.title}
                </span>
              </div>
            ))
          )}
        </div>
      </ScrollArea>

      {props.mediaPlaying && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="border-border bg-accent/30 mx-auto w-[95%] shrink-0 rounded-md border-t p-3"
        >
          <div className="space-y-2">
            <div className="flex items-start justify-between gap-2">
              <div className="min-w-0 flex-1">
                <p className="text-secondary-foreground truncate text-xs font-medium">
                  Every Level Of Intelligence Explained in 5 Minutes
                </p>
                <p className="text-muted-foreground truncate text-xs">
                  Nick Explains
                </p>
              </div>
              <button
                onClick={() => props.onSetMediaPlaying(false)}
                className="text-muted-foreground hover:text-accent-foreground shrink-0"
              >
                <X className="h-3 w-3" />
              </button>
            </div>

            <div className="text-muted-foreground flex items-center gap-1 text-xs">
              <span>1:40</span>
              <div className="bg-muted relative h-1 flex-1 overflow-hidden rounded-full">
                <div
                  className="bg-primary h-full"
                  style={{ width: `${props.mediaProgress}%` }}
                />
              </div>
              <span>5:07</span>
            </div>

            <div className="flex items-center justify-between">
              <div className="bg-destructive flex h-6 w-6 items-center justify-center rounded">
                <Youtube className="text-destructive-foreground h-3 w-3" />
              </div>

              <div className="flex items-center gap-1">
                <button className="text-muted-foreground hover:text-accent-foreground">
                  <SkipBack className="h-4 w-4" />
                </button>
                <button className="text-accent-foreground hover:text-accent-foreground">
                  <Play className="h-4 w-4" />
                </button>
                <button className="text-muted-foreground hover:text-accent-foreground">
                  <SkipForward className="h-4 w-4" />
                </button>
              </div>

              <button className="text-muted-foreground hover:text-accent-foreground">
                <Volume2 className="h-4 w-4" />
              </button>
            </div>
          </div>
        </motion.div>
      )}

      <div className="border-border shrink-0">
        <motion.div
          className="flex items-center justify-between p-2"
          initial={{ y: 20, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 0.1 }}
        >
          <ContextMenu>
            <ContextMenuTrigger asChild>
              <motion.div
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
                drag
                dragConstraints={{
                  left: 0,
                  right: 0,
                  top: 0,
                  bottom: 0,
                }}
                dragElastic={0.1}
                transition={{
                  type: "spring",
                  stiffness: 400,
                  damping: 25,
                  mass: 0.5,
                }}
              >
                <Popover
                  open={props.downloadsOpen}
                  onOpenChange={props.onSetDownloadsOpen}
                >
                  <PopoverTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                    >
                      <CircleDashed className="h-4 w-4" />
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent
                    side="top"
                    align="start"
                    className="border-border bg-card w-80 p-4"
                  >
                    <div className="space-y-4">
                      <p className="text-muted-foreground text-sm">
                        No downloads for this session.
                      </p>
                      <div className="border-border border-t pt-4">
                        <button className="text-accent-foreground hover:text-accent-foreground text-sm underline">
                          Show all downloads
                        </button>
                      </div>
                    </div>
                  </PopoverContent>
                </Popover>
              </motion.div>
            </ContextMenuTrigger>
            <ContextMenuContent className="border-border bg-card">
              <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                Open Downloads
              </ContextMenuItem>
              <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                Clear Downloads
              </ContextMenuItem>
            </ContextMenuContent>
          </ContextMenu>

          <div className="relative flex w-38 shrink-0 items-center justify-center overflow-hidden">
            <motion.button
              whileTap={props.canScrollLeft ? { scale: 0.9 } : {}}
              onMouseDown={() =>
                props.canScrollLeft && props.onStartScrolling("left")
              }
              onMouseUp={props.onStopScrolling}
              onMouseLeave={props.onStopScrolling}
              onTouchStart={() =>
                props.canScrollLeft && props.onStartScrolling("left")
              }
              onTouchEnd={props.onStopScrolling}
              className={cn(
                "bg-background text-primary absolute left-0 z-10 flex h-6 w-6 items-center justify-center rounded-full border transition-all duration-200",
                props.canScrollLeft
                  ? "text-muted-foreground hover:text-accent-foreground cursor-pointer"
                  : "hidden!",
              )}
              aria-label="Scroll workspaces left"
              aria-disabled={!props.canScrollLeft}
            >
              <ChevronLeft className="h-3 w-3" />
            </motion.button>

            <div className="flex h-6 items-center justify-center px-2">
              <div className="flex items-center gap-1">
                <AnimatePresence mode="popLayout">
                  {props.visibleWorkspaces.map((workspace) => (
                    <ContextMenu key={workspace.id}>
                      <ContextMenuTrigger asChild>
                        <motion.button
                          layout
                          initial={{ opacity: 0, scale: 0.8 }}
                          animate={{ opacity: 1, scale: 1 }}
                          exit={{ opacity: 0, scale: 0.8 }}
                          transition={{
                            layout: {
                              type: "spring",
                              stiffness: 400,
                              damping: 30,
                            },
                            opacity: { duration: 0.2 },
                            scale: { duration: 0.2 },
                          }}
                          onClick={() =>
                            props.onSetActiveWorkspace(workspace.id)
                          }
                          className={cn(
                            "group flex h-4 w-4 shrink-0 items-center justify-center rounded-md",
                            props.activeWorkspace === workspace.id &&
                              "scale-110",
                          )}
                          title={workspace.name}
                          aria-label={`Switch to ${workspace.name} workspace`}
                          whileHover={{ scale: 1.25 }}
                          whileTap={{ scale: 0.95 }}
                          drag
                          dragConstraints={{
                            left: 0,
                            right: 0,
                            top: 0,
                            bottom: 0,
                          }}
                          dragElastic={0.2}
                        >
                          <WorkspaceIcon
                            workspace={workspace}
                            isActive={props.activeWorkspace === workspace.id}
                          />
                        </motion.button>
                      </ContextMenuTrigger>
                      <ContextMenuContent className="border-border bg-card w-56">
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Change Name
                        </ContextMenuItem>
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Change Icon
                        </ContextMenuItem>
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Edit Theme
                        </ContextMenuItem>
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Set Profile
                          <ChevronRight className="ml-auto h-4 w-4" />
                        </ContextMenuItem>
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Unload Space
                        </ContextMenuItem>
                        <div className="border-border my-1 border-t" />
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          <div className="mr-2 h-4 w-4 flex items-center justify-center shrink-0">
                            ✓
                          </div>
                          <span className="max-w-[75%] truncate">
                            {workspace.name}
                          </span>
                        </ContextMenuItem>
                        {props.workspaces
                          .filter((w) => w.id !== workspace.id)
                          .map((w) => (
                            <ContextMenuItem
                              key={w.id}
                              onClick={() => props.onSetActiveWorkspace(w.id)}
                              className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                            >
                              <div className="mr-2 h-4 w-4 flex items-center justify-center shrink-0">
                                <WorkspaceIcon workspace={w} isActive={false} />
                              </div>
                              <span className="max-w-[75%] truncate">
                                {w.name}
                              </span>
                            </ContextMenuItem>
                          ))}
                        <div className="border-border my-1 border-t" />
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Reorder Spaces
                        </ContextMenuItem>
                        <div className="border-border my-1 border-t" />
                        <ContextMenuItem
                          onClick={props.onOpenWorkspaceDialog}
                          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                        >
                          Create Space
                        </ContextMenuItem>
                        <ContextMenuItem className="text-destructive focus:bg-accent focus:text-destructive">
                          Delete Space
                        </ContextMenuItem>
                      </ContextMenuContent>
                    </ContextMenu>
                  ))}
                </AnimatePresence>
              </div>
            </div>

            <motion.button
              whileTap={props.canScrollRight ? { scale: 0.9 } : {}}
              onMouseDown={() =>
                props.canScrollRight && props.onStartScrolling("right")
              }
              onMouseUp={props.onStopScrolling}
              onMouseLeave={props.onStopScrolling}
              onTouchStart={() =>
                props.canScrollRight && props.onStartScrolling("right")
              }
              onTouchEnd={props.onStopScrolling}
              className={cn(
                "bg-background text-primary absolute right-0 z-50 flex h-6 w-6 items-center justify-center rounded-full border transition-all duration-200",
                props.canScrollRight
                  ? "text-muted-foreground hover:text-accent-foreground cursor-pointer"
                  : "hidden!",
              )}
              aria-label="Scroll workspaces right"
              aria-disabled={!props.canScrollRight}
            >
              <ChevronRight className="h-3 w-3" />
            </motion.button>
          </div>

          <motion.div
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
            drag
            dragConstraints={{ left: 0, right: 0, top: 0, bottom: 0 }}
            dragElastic={0.1}
            transition={{
              type: "spring",
              stiffness: 400,
              damping: 25,
              mass: 0.5,
            }}
          >
            <DropdownMenu
              open={props.plusMenuOpen}
              onOpenChange={props.onSetPlusMenuOpen}
            >
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                  onClick={(e) => {
                    e.stopPropagation();
                    props.onCreateWorkspace();
                  }}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    props.onSetPlusMenuOpen(true);
                  }}
                >
                  <Plus className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                side="top"
                align="end"
                sideOffset={5}
                className="border-border bg-card w-56"
              >
                <DropdownMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                  <Folder className="mr-2 h-4 w-4" />
                  Live Folder
                  <ChevronRight className="ml-auto h-4 w-4" />
                </DropdownMenuItem>
                <div className="border-border my-1 border-t" />
                <DropdownMenuItem
                  onClick={() => {
                    props.onOpenWorkspaceDialog();
                    props.onSetPlusMenuOpen(false);
                  }}
                  className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                >
                  <Copy className="mr-2 h-4 w-4" />
                  Create Space
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={() => {
                    props.onCreateFolder();
                    props.onSetPlusMenuOpen(false);
                  }}
                  className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                >
                  <Folder className="mr-2 h-4 w-4" />
                  Create Folder
                </DropdownMenuItem>
                <div className="border-border my-1 border-t" />
                <DropdownMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                  <Columns2 className="mr-2 h-4 w-4" />
                  New Split
                </DropdownMenuItem>
                <div className="border-border my-1 border-t" />
                <DropdownMenuItem
                  onClick={() => {
                    props.onAddNewTab();
                    props.onSetPlusMenuOpen(false);
                  }}
                  className="text-accent-foreground focus:bg-accent focus:text-accent-foreground text-left"
                >
                  <Plus className="mr-2 h-4 w-4" />
                  New Tab
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </motion.div>
        </motion.div>
      </div>
    </motion.div>
  );
}
