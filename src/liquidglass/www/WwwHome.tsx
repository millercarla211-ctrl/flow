"use client";

import {
  closestCenter,
  DndContext,
  type DragEndEvent,
  type DragOverEvent,
  DragOverlay,
  type DragStartEvent,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { VisuallyHidden } from "@radix-ui/react-visually-hidden";
import { Copy, Plus, X } from "lucide-react";
import { AnimatePresence } from "motion/react";
import { useEffect, useRef, useState } from "react";
import { BrowserContent } from "@/liquidglass/www/components/browser/browser-content";
import {
  MAX_LOGOS,
  MAX_VISIBLE_WORKSPACES,
} from "@/liquidglass/www/components/browser/constants";
import { SidebarCollapsed } from "@/liquidglass/www/components/browser/sidebar-collapsed";
import { SidebarExpanded } from "@/liquidglass/www/components/browser/sidebar-expanded";
import { SidebarHeader } from "@/liquidglass/www/components/browser/sidebar-header";
import type { DropPosition, Tab, TabFolder } from "@/liquidglass/www/components/browser/types";
import { useBrowserState } from "@/liquidglass/www/components/browser/use-browser-state";
import { getLogoComponentForTab } from "@/liquidglass/www/components/browser/utils";
import { WorkspaceIcon } from "@/liquidglass/www/components/browser/workspace-icon";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/liquidglass/www/components/ui/command";
import { Dialog, DialogContent, DialogTitle } from "@/liquidglass/www/components/ui/dialog";
import { WorkspaceDialog } from "@/liquidglass/www/components/workspace-dialog";
import { cn } from "@/liquidglass/www/lib/utils";

export default function Home() {
  const state = useBrowserState();
  const [activeId, setActiveId] = useState<string | null>(null);
  const [overId, setOverId] = useState<string | null>(null);
  const [dropPosition, setDropPosition] = useState<DropPosition>(null);
  const [logoContainerHovered, setLogoContainerHovered] = useState(false);
  const [isSpaceAreaHovered, setIsSpaceAreaHovered] = useState(false);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const scrollIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const [mediaProgress] = useState(33);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    }),
  );

  useEffect(() => {
    return () => {
      if (scrollIntervalRef.current) {
        clearInterval(scrollIntervalRef.current);
      }
    };
  }, []);

  function scrollWorkspaces(direction: "left" | "right"): void {
    if (direction === "left") {
      state.setWorkspaceScrollPosition((prev) => Math.max(0, prev - 1));
    } else {
      state.setWorkspaceScrollPosition((prev) =>
        Math.min(state.workspaces.length - MAX_VISIBLE_WORKSPACES, prev + 1),
      );
    }
  }

  function startScrolling(direction: "left" | "right"): void {
    scrollWorkspaces(direction);

    scrollIntervalRef.current = setInterval(() => {
      scrollWorkspaces(direction);
    }, 150);
  }

  function stopScrolling(): void {
    if (scrollIntervalRef.current) {
      clearInterval(scrollIntervalRef.current);
      scrollIntervalRef.current = null;
    }
  }

  const visibleWorkspaces = state.workspaces.slice(
    state.workspaceScrollPosition,
    state.workspaceScrollPosition + MAX_VISIBLE_WORKSPACES,
  );
  const canScrollLeft = state.workspaceScrollPosition > 0;
  const canScrollRight =
    state.workspaceScrollPosition <
    state.workspaces.length - MAX_VISIBLE_WORKSPACES;

  function handleDragStart(event: DragStartEvent): void {
    setActiveId(event.active.id as string);
  }

  function handleDragOver(event: DragOverEvent): void {
    const overId = event.over?.id as string | null;
    setOverId(overId);

    if (!overId || !event.over) {
      setDropPosition(null);
      setLogoContainerHovered(false);
      return;
    }

    const overData = event.over.data.current;
    const activeData = event.active.data.current;

    if (overId === "logo-container" || overData?.type === "logo-container") {
      if (activeData?.type === "tab" || activeData?.type === "folder") {
        setLogoContainerHovered(true);
        setDropPosition(null);
      } else {
        setLogoContainerHovered(false);
        setDropPosition(null);
      }
      return;
    } else {
      setLogoContainerHovered(false);
    }

    if (activeData?.type === "logo") {
      if (overData?.type === "logo") {
        const overRect = event.over.rect;
        const offsetX = event.delta.x;
        if (overRect && offsetX < 0) {
          setDropPosition("before");
        } else {
          setDropPosition("after");
        }
      } else if (overData?.type === "folder") {
        setDropPosition("inside");
      } else if (overData?.type === "tab") {
        const overRect = event.over.rect;
        const offsetY = event.delta.y;
        if (overRect && offsetY < 0) {
          setDropPosition("before");
        } else {
          setDropPosition("after");
        }
      } else {
        setDropPosition(null);
      }
      return;
    }

    if (overData?.type === "folder") {
      if (activeData?.type === "tab") {
        setDropPosition("inside");
      } else if (activeData?.type === "folder") {
        const overRect = event.over.rect;
        const offsetY = event.delta.y;
        if (overRect && offsetY < 0) {
          setDropPosition("before");
        } else {
          setDropPosition("after");
        }
      } else {
        setDropPosition(null);
      }
    } else if (overData?.type === "tab") {
      const overRect = event.over.rect;
      const offsetY = event.delta.y;
      if (overRect && offsetY < 0) {
        setDropPosition("before");
      } else {
        setDropPosition("after");
      }
    } else if (overData?.type === "logo") {
      const overRect = event.over.rect;
      const offsetX = event.delta.x;
      if (overRect && offsetX < 0) {
        setDropPosition("before");
      } else {
        setDropPosition("after");
      }
    } else {
      setDropPosition(null);
    }
  }

  function handleDragEnd(event: DragEndEvent): void {
    const { active, over } = event;
    setActiveId(null);
    setOverId(null);
    setDropPosition(null);
    setLogoContainerHovered(false);

    if (!over || active.id === over.id) return;

    const activeId = active.id as string;
    const overId = over.id as string;
    const activeData = active.data.current;
    const overData = over.data.current;

    // Handle logo reordering
    if (activeData?.type === "logo" && overData?.type === "logo") {
      const oldIndex = state.logos.findIndex(
        (l) => `logo-${l.id}` === activeId,
      );
      const newIndex = state.logos.findIndex((l) => `logo-${l.id}` === overId);
      if (oldIndex !== -1 && newIndex !== -1 && oldIndex !== newIndex) {
        const newLogos = [...state.logos];
        const [removed] = newLogos.splice(oldIndex, 1);
        newLogos.splice(newIndex, 0, removed);
        state.setLogos(newLogos);
      }
      return;
    }

    // Handle dragging logos out
    if (activeData?.type === "logo") {
      const logoId = activeId.replace("logo-", "");
      const logo = state.logos.find((l) => l.id === parseInt(logoId, 10));

      if (!logo) return;

      if (
        overId === "logo-container" ||
        overData?.type === "logo-container" ||
        overData?.type === "logo"
      ) {
        return;
      }

      state.setLogos(state.logos.filter((l) => l.id !== parseInt(logoId, 10)));

      const newTab: Tab = {
        id: Date.now().toString(),
        title: logo.title,
        url: "about:blank",
        workspaceId: state.activeWorkspace,
        folderId: overData?.type === "folder" ? overId : null,
      };

      if (overData?.type === "folder") {
        state.setFolders(
          state.folders.map((f) =>
            f.id === overId ? { ...f, tabs: [...f.tabs, newTab] } : f,
          ),
        );
      } else {
        state.setLooseTabs([...state.looseTabs, newTab]);
      }
      return;
    }

    // Handle dropping tabs/folders onto logo container
    if (overId === "logo-container" || overData?.type === "logo-container") {
      if (state.logos.length >= MAX_LOGOS) {
        alert("Logo container is full! Maximum 12 items allowed.");
        return;
      }

      if (activeData?.type === "folder") {
        const draggedFolder = state.folders.find((f) => f.id === activeId);
        if (draggedFolder) {
          const newLogo = {
            id: Date.now(),
            title: draggedFolder.name,
            component: getLogoComponentForTab({
              id: "",
              title: draggedFolder.name,
              url: "",
              workspaceId: "",
            }),
          };
          state.setLogos([...state.logos, newLogo]);
          return;
        }
      }

      let draggedTab = state.looseTabs.find((tab) => tab.id === activeId);
      let sourceFolder: TabFolder | undefined;

      if (!draggedTab) {
        for (const folder of state.folders) {
          const tab = folder.tabs.find((t) => t.id === activeId);
          if (tab) {
            draggedTab = tab;
            sourceFolder = folder;
            break;
          }
        }
      }

      if (draggedTab) {
        const newLogo = {
          id: Date.now(),
          title: draggedTab.title,
          component: getLogoComponentForTab(draggedTab),
        };

        state.setLogos([...state.logos, newLogo]);

        if (sourceFolder) {
          state.setFolders(
            state.folders.map((folder) =>
              folder.id === sourceFolder.id
                ? {
                    ...folder,
                    tabs: folder.tabs.filter((t) => t.id !== activeId),
                  }
                : folder,
            ),
          );
        } else {
          state.setLooseTabs(
            state.looseTabs.filter((tab) => tab.id !== activeId),
          );
        }
        return;
      }
    }
  }

  function addNewTab(): void {
    const newTab: Tab = {
      id: Date.now().toString(),
      title: "New Tab",
      url: "about:blank",
      workspaceId: state.activeWorkspace,
      folderId: null,
    };
    state.setLooseTabs([...state.looseTabs, newTab]);
  }

  function createFolder(): void {
    const folderCount = state.folders.filter(
      (f) => f.workspaceId === state.activeWorkspace,
    ).length;
    const newFolder: TabFolder = {
      id: `folder-${Date.now()}`,
      name: `Folder ${folderCount + 1}`,
      collapsed: false,
      workspaceId: state.activeWorkspace,
      parentId: null,
      tabs: [],
    };
    state.setFolders([...state.folders, newFolder]);
  }

  function toggleFolder(folderId: string): void {
    state.setFolders(
      state.folders.map((folder) =>
        folder.id === folderId
          ? { ...folder, collapsed: !folder.collapsed }
          : folder,
      ),
    );
  }

  function deleteFolder(folderId: string): void {
    const folder = state.folders.find((f) => f.id === folderId);
    if (folder) {
      const folderTabs = folder.tabs.map((tab) => ({ ...tab, folderId: null }));
      state.setLooseTabs([...state.looseTabs, ...folderTabs]);
    }
    state.setFolders(state.folders.filter((f) => f.id !== folderId));
  }

  function closeTab(tabId: string): void {
    state.setLooseTabs(state.looseTabs.filter((tab) => tab.id !== tabId));
    state.setFolders(
      state.folders.map((folder) => ({
        ...folder,
        tabs: folder.tabs.filter((tab) => tab.id !== tabId),
      })),
    );
  }

  function clearAllTabs(): void {
    state.setLooseTabs([]);
  }

  const activeWorkspaceFolders = state.folders.filter(
    (folder) =>
      folder.workspaceId === state.activeWorkspace && !folder.parentId,
  );
  const activeWorkspaceTabs = state.looseTabs.filter(
    (tab) => tab.workspaceId === state.activeWorkspace && !tab.folderId,
  );

  function renderWorkspaceIcon(workspace: any, isActive: boolean) {
    return <WorkspaceIcon workspace={workspace} isActive={isActive} />;
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDragEnd={handleDragEnd}
    >
      <div className="bg-background flex h-screen w-screen overflow-hidden">
        <div
          className={cn(
            "group bg-card flex h-full shrink-0 flex-col transition-all duration-200 select-none overflow-x-hidden",
            state.sidebarExpanded ? "w-[360px]" : "w-14",
          )}
        >
          <SidebarHeader
            sidebarExpanded={state.sidebarExpanded}
            onToggleSidebar={() =>
              state.setSidebarExpanded(!state.sidebarExpanded)
            }
          />

          {!state.sidebarExpanded && (
            <SidebarCollapsed
              workspaces={state.workspaces}
              activeWorkspace={state.activeWorkspace}
              visibleWorkspaces={visibleWorkspaces}
              canScrollLeft={canScrollLeft}
              canScrollRight={canScrollRight}
              downloadsOpen={state.downloadsOpen}
              plusMenuOpen={state.plusMenuOpen}
              onSetActiveWorkspace={state.setActiveWorkspace}
              onSetDownloadsOpen={state.setDownloadsOpen}
              onSetPlusMenuOpen={state.setPlusMenuOpen}
              onStartScrolling={startScrolling}
              onStopScrolling={stopScrolling}
              onCreateWorkspace={() =>
                state.handleCreateWorkspace({
                  name: "",
                  icon: { type: "dot", value: "" },
                  color:
                    state.COLORS[state.workspaces.length % state.COLORS.length],
                })
              }
              onOpenWorkspaceDialog={() => state.setWorkspaceDialogOpen(true)}
              onCreateFolder={createFolder}
              onAddNewTab={addNewTab}
            />
          )}

          <AnimatePresence>
            {state.sidebarExpanded && (
              <SidebarExpanded
                workspaces={state.workspaces}
                activeWorkspace={state.activeWorkspace}
                visibleWorkspaces={visibleWorkspaces}
                canScrollLeft={canScrollLeft}
                canScrollRight={canScrollRight}
                downloadsOpen={state.downloadsOpen}
                plusMenuOpen={state.plusMenuOpen}
                spaceCollapsed={state.spaceCollapsed}
                isSpaceAreaHovered={isSpaceAreaHovered}
                dropdownOpen={dropdownOpen}
                mediaPlaying={state.mediaPlaying}
                mediaProgress={mediaProgress}
                isMounted={state.isMounted}
                logos={state.logos}
                logoContainerHovered={logoContainerHovered}
                activeWorkspaceFolders={activeWorkspaceFolders}
                activeWorkspaceTabs={activeWorkspaceTabs}
                activeTab={state.activeTab}
                overId={overId}
                dropPosition={dropPosition}
                folders={state.folders}
                onSetActiveWorkspace={state.setActiveWorkspace}
                onSetDownloadsOpen={state.setDownloadsOpen}
                onSetPlusMenuOpen={state.setPlusMenuOpen}
                onSetSpaceCollapsed={state.setSpaceCollapsed}
                onSetSpaceAreaHovered={setIsSpaceAreaHovered}
                onSetDropdownOpen={setDropdownOpen}
                onSetMediaPlaying={state.setMediaPlaying}
                onStartScrolling={startScrolling}
                onStopScrolling={stopScrolling}
                onCreateWorkspace={() =>
                  state.handleCreateWorkspace({
                    name: "",
                    icon: { type: "dot", value: "" },
                    color:
                      state.COLORS[
                        state.workspaces.length % state.COLORS.length
                      ],
                  })
                }
                onOpenWorkspaceDialog={() => state.setWorkspaceDialogOpen(true)}
                onCreateFolder={createFolder}
                onAddNewTab={addNewTab}
                onClearAllTabs={clearAllTabs}
                onSetCommandOpen={state.setCommandOpen}
                onRemoveLogo={(logoId) =>
                  state.setLogos(state.logos.filter((l) => l.id !== logoId))
                }
                onRenameLogo={(logoId, newTitle) =>
                  state.setLogos(
                    state.logos.map((l) =>
                      l.id === logoId ? { ...l, title: newTitle } : l,
                    ),
                  )
                }
                onSetActiveTab={state.setActiveTab}
                onCloseTab={closeTab}
                onToggleFolder={toggleFolder}
                onDeleteFolder={deleteFolder}
                onRenameFolder={(folderId, newName) =>
                  state.setFolders(
                    state.folders.map((f) =>
                      f.id === folderId ? { ...f, name: newName } : f,
                    ),
                  )
                }
                onUnloadAllTabs={(folderId) =>
                  state.setFolders(
                    state.folders.map((f) =>
                      f.id === folderId ? { ...f, tabs: [] } : f,
                    ),
                  )
                }
                onCreateSubfolder={(parentId) => {
                  const subfolderCount = state.folders.filter(
                    (f) => f.parentId === parentId,
                  ).length;
                  const newSubfolder: TabFolder = {
                    id: `folder-${Date.now()}`,
                    name: `Subfolder ${subfolderCount + 1}`,
                    collapsed: false,
                    workspaceId: state.activeWorkspace,
                    parentId: parentId,
                    tabs: [],
                  };
                  state.setFolders([...state.folders, newSubfolder]);
                }}
                onUnpackFolder={(folderId) => {
                  const folder = state.folders.find((f) => f.id === folderId);
                  if (folder) {
                    const folderTabs = folder.tabs.map((tab) => ({
                      ...tab,
                      folderId: null,
                    }));
                    state.setLooseTabs([...state.looseTabs, ...folderTabs]);
                    deleteFolder(folderId);
                  }
                }}
                onRenameWorkspace={() => {
                  const currentWorkspace = state.workspaces.find(
                    (w) => w.id === state.activeWorkspace,
                  );
                  const newName = prompt(
                    "Enter new workspace name:",
                    currentWorkspace?.name || "",
                  );
                  if (newName?.trim()) {
                    state.setWorkspaces(
                      state.workspaces.map((w) =>
                        w.id === state.activeWorkspace
                          ? { ...w, name: newName.trim() }
                          : w,
                      ),
                    );
                  }
                }}
                onEditWorkspaceIcon={() => {
                  const currentWorkspace = state.workspaces.find(
                    (w) => w.id === state.activeWorkspace,
                  );
                  if (currentWorkspace) {
                    state.setWorkspaceEditMode({
                      workspaceId: currentWorkspace.id,
                      currentName: currentWorkspace.name,
                      currentIcon: currentWorkspace.icon,
                    });
                    state.setWorkspaceDialogOpen(true);
                  }
                }}
                onUnloadSpace={() => {
                  state.setFolders(
                    state.folders.filter(
                      (f) => f.workspaceId !== state.activeWorkspace,
                    ),
                  );
                  state.setLooseTabs(
                    state.looseTabs.filter(
                      (t) => t.workspaceId !== state.activeWorkspace,
                    ),
                  );
                }}
                onDeleteSpace={() => {
                  if (state.workspaces.length <= 1) {
                    alert("Cannot delete the last workspace!");
                    return;
                  }
                  const currentWorkspace = state.workspaces.find(
                    (w) => w.id === state.activeWorkspace,
                  );
                  if (
                    confirm(
                      `Are you sure you want to delete "${currentWorkspace?.name}"? All tabs and folders will be removed.`,
                    )
                  ) {
                    state.setFolders(
                      state.folders.filter(
                        (f) => f.workspaceId !== state.activeWorkspace,
                      ),
                    );
                    state.setLooseTabs(
                      state.looseTabs.filter(
                        (t) => t.workspaceId !== state.activeWorkspace,
                      ),
                    );
                    state.setWorkspaces(
                      state.workspaces.filter(
                        (w) => w.id !== state.activeWorkspace,
                      ),
                    );
                    const remainingWorkspaces = state.workspaces.filter(
                      (w) => w.id !== state.activeWorkspace,
                    );
                    if (remainingWorkspaces.length > 0) {
                      state.setActiveWorkspace(remainingWorkspaces[0].id);
                    }
                  }
                }}
                renderWorkspaceIcon={renderWorkspaceIcon}
                COLORS={state.COLORS}
              />
            )}
          </AnimatePresence>
        </div>

        <div className="flex min-w-0 flex-1 flex-col bg-card border-l">
          <BrowserContent sidebarExpanded={state.sidebarExpanded} />
        </div>
      </div>

      <WorkspaceDialog
        open={state.workspaceDialogOpen}
        onOpenChange={(open) => {
          state.setWorkspaceDialogOpen(open);
          if (!open) {
            state.setWorkspaceEditMode(undefined);
          }
        }}
        onCreateWorkspace={state.handleCreateWorkspace}
        editMode={state.workspaceEditMode}
        onUpdateWorkspace={state.handleUpdateWorkspace}
      />

      <Dialog open={state.commandOpen} onOpenChange={state.setCommandOpen}>
        <DialogContent className="overflow-hidden p-0 shadow-lg">
          <VisuallyHidden>
            <DialogTitle>Search and Command Palette</DialogTitle>
          </VisuallyHidden>
          <Command>
            <CommandInput placeholder="Search or enter address..." />
            <CommandList>
              <CommandEmpty>No results found.</CommandEmpty>
              <CommandGroup heading="Suggestions">
                <CommandItem
                  onSelect={() => {
                    addNewTab();
                    state.setCommandOpen(false);
                  }}
                >
                  <Plus className="mr-2 h-4 w-4" />
                  <span>New Tab</span>
                </CommandItem>
                <CommandItem
                  onSelect={() => {
                    state.setWorkspaceDialogOpen(true);
                    state.setCommandOpen(false);
                  }}
                >
                  <Copy className="mr-2 h-4 w-4" />
                  <span>Create Space</span>
                </CommandItem>
                <CommandItem
                  onSelect={() => {
                    clearAllTabs();
                    state.setCommandOpen(false);
                  }}
                >
                  <X className="mr-2 h-4 w-4" />
                  <span>Clear All Tabs</span>
                </CommandItem>
              </CommandGroup>
              <CommandGroup heading="Recent Tabs">
                {activeWorkspaceTabs.slice(0, 5).map((tab) => (
                  <CommandItem
                    key={tab.id}
                    onSelect={() => {
                      state.setActiveTab(tab.id);
                      state.setCommandOpen(false);
                    }}
                  >
                    <div className="bg-destructive mr-2 h-4 w-4 shrink-0 rounded-sm" />
                    <span className="max-w-[75%] truncate">{tab.title}</span>
                  </CommandItem>
                ))}
              </CommandGroup>
              <CommandGroup heading="Workspaces">
                {state.workspaces.map((workspace) => (
                  <CommandItem
                    key={workspace.id}
                    onSelect={() => {
                      state.setActiveWorkspace(workspace.id);
                      state.setCommandOpen(false);
                    }}
                  >
                    <div className="mr-2 flex h-4 w-4 items-center justify-center">
                      <WorkspaceIcon
                        workspace={workspace}
                        isActive={state.activeWorkspace === workspace.id}
                      />
                    </div>
                    <span className="max-w-[75%] truncate">
                      {workspace.name}
                    </span>
                  </CommandItem>
                ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </DialogContent>
      </Dialog>

      <DragOverlay>
        {activeId ? (
          <div className="bg-accent text-accent-foreground flex h-9 items-center gap-2 rounded-md px-2 shadow-lg">
            <div className="bg-destructive h-4 w-4 shrink-0 rounded-sm" />
            <span className="text-xs">
              {state.looseTabs.find((t) => t.id === activeId)?.title ||
                "Dragging..."}
            </span>
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}
