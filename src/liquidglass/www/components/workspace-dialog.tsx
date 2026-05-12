"use client";

import * as LucideIcons from "lucide-react";
import * as React from "react";
import { useState } from "react";
import { Button } from "@/liquidglass/www/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/liquidglass/www/components/ui/dialog";
import { Input } from "@/liquidglass/www/components/ui/input";
import { Label } from "@/liquidglass/www/components/ui/label";
import { ScrollArea } from "@/liquidglass/www/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/liquidglass/www/components/ui/tabs";
import { cn } from "@/liquidglass/www/lib/utils";

interface WorkspaceDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreateWorkspace: (workspace: {
    name: string;
    icon: { type: "emoji" | "icon" | "dot"; value: string };
    color: string;
  }) => void;
  editMode?: {
    workspaceId: string;
    currentName: string;
    currentIcon: { type: "emoji" | "icon" | "dot"; value: string };
  };
  onUpdateWorkspace?: (
    workspaceId: string,
    updates: {
      name?: string;
      icon?: { type: "emoji" | "icon" | "dot"; value: string };
    },
  ) => void;
}

const EMOJI_LIST = [
  "😊",
  "💼",
  "🎮",
  "💻",
  "📚",
  "🎨",
  "🎵",
  "🏠",
  "🚀",
  "⚡",
  "🔥",
  "💡",
  "🌟",
  "🎯",
  "📱",
  "🌈",
  "🎪",
  "🎭",
  "🎬",
  "📷",
  "🎸",
  "🎹",
  "🎤",
  "🎧",
  "📺",
  "🕹️",
  "🎲",
  "🧩",
  "🎰",
  "🏆",
  "🥇",
  "🎖️",
  "🏅",
  "⚽",
  "🏀",
  "🏈",
  "⚾",
  "🎾",
  "🏐",
  "🎳",
];

const ICON_NAMES = [
  "Home",
  "Briefcase",
  "Code",
  "Book",
  "Palette",
  "Music",
  "Gamepad2",
  "Rocket",
  "Zap",
  "Flame",
  "Lightbulb",
  "Star",
  "Target",
  "Smartphone",
  "Monitor",
  "Laptop",
  "Coffee",
  "Heart",
  "ShoppingCart",
  "Camera",
  "Film",
  "Headphones",
  "Mic",
  "Video",
  "Image",
  "FileText",
  "Folder",
  "Archive",
  "Database",
  "Server",
  "Cloud",
  "Globe",
  "Mail",
  "MessageSquare",
  "Phone",
  "Calendar",
  "Clock",
  "MapPin",
  "Navigation",
  "Compass",
];

export function WorkspaceDialog({
  open,
  onOpenChange,
  onCreateWorkspace,
  editMode,
  onUpdateWorkspace,
}: WorkspaceDialogProps): React.ReactElement {
  const [name, setName] = useState(editMode?.currentName || "");
  const [selectedIcon, setSelectedIcon] = useState<{
    type: "emoji" | "icon" | "dot";
    value: string;
  }>(editMode?.currentIcon || { type: "dot", value: "" });

  // Update state when editMode changes
  React.useEffect(() => {
    if (editMode) {
      setName(editMode.currentName);
      setSelectedIcon(editMode.currentIcon);
    } else {
      setName("");
      setSelectedIcon({ type: "dot", value: "" });
    }
  }, [editMode]);

  function handleSubmit(): void {
    if (editMode && onUpdateWorkspace) {
      // Edit mode
      onUpdateWorkspace(editMode.workspaceId, {
        name: name.trim() || editMode.currentName,
        icon: selectedIcon,
      });
    } else {
      // Create mode
      const workspaceName = name.trim() || `Workspace ${Date.now()}`;
      onCreateWorkspace({
        name: workspaceName,
        icon: selectedIcon,
        color: "hsl(var(--primary))",
      });
    }

    // Reset form
    setName("");
    setSelectedIcon({ type: "dot", value: "" });
    onOpenChange(false);
  }

  function renderIcon(
    type: "emoji" | "icon" | "dot",
    value: string,
  ): React.ReactElement {
    if (type === "emoji") {
      return <span className="text-base">{value}</span>;
    }
    if (type === "icon") {
      const IconComponent = (LucideIcons as any)[value];
      if (IconComponent) {
        return <IconComponent className="h-4 w-4" />;
      }
    }
    return <div className="h-2 w-2 rounded-full bg-primary" />;
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl border-zinc-800 bg-zinc-900 text-zinc-100">
        <DialogHeader>
          <DialogTitle className="text-zinc-100">
            {editMode ? "Edit Workspace" : "Create New Workspace"}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          {/* Preview */}
          <div className="flex items-center gap-3 rounded-lg border border-zinc-800 bg-zinc-800/50 p-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-md bg-zinc-800">
              {renderIcon(selectedIcon.type, selectedIcon.value)}
            </div>
            <div>
              <p className="text-sm font-medium text-zinc-300">Preview</p>
              <p className="text-xs text-zinc-500">
                {name || `Workspace ${Date.now().toString().slice(-3)}`}
              </p>
            </div>
          </div>

          {/* Name Input */}
          <div className="space-y-2">
            <Label htmlFor="name" className="text-zinc-300">
              Workspace Name {!editMode && "(Optional)"}
            </Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={
                editMode
                  ? "Enter workspace name"
                  : "Leave empty for auto-generated name"
              }
              className="border-zinc-800 bg-zinc-800 text-zinc-100 placeholder:text-zinc-500"
            />
            {!editMode && (
              <p className="text-xs text-zinc-500">
                If left empty, will be named "Workspace 1", "Workspace 2", etc.
              </p>
            )}
          </div>

          {/* Icon Selection */}
          <div className="space-y-2">
            <Label className="text-zinc-300">Icon</Label>
            <Tabs defaultValue="dot" className="w-full">
              <TabsList className="grid w-full grid-cols-3 bg-zinc-800">
                <TabsTrigger
                  value="dot"
                  onClick={() => setSelectedIcon({ type: "dot", value: "" })}
                  className="data-[state=active]:bg-zinc-700"
                >
                  Dot
                </TabsTrigger>
                <TabsTrigger
                  value="emoji"
                  className="data-[state=active]:bg-zinc-700"
                >
                  Emoji
                </TabsTrigger>
                <TabsTrigger
                  value="icon"
                  className="data-[state=active]:bg-zinc-700"
                >
                  Icon
                </TabsTrigger>
              </TabsList>

              <TabsContent value="dot" className="mt-4">
                <div className="rounded-lg border border-zinc-800 bg-zinc-800/50 p-4 text-center">
                  <p className="text-sm text-zinc-400">
                    A simple colored dot will be used
                  </p>
                </div>
              </TabsContent>

              <TabsContent value="emoji" className="mt-4">
                <ScrollArea className="h-48 rounded-lg border border-zinc-800 bg-zinc-800/50 p-4">
                  <div className="grid grid-cols-8 gap-2">
                    {EMOJI_LIST.map((emoji) => (
                      <button
                        type="button"
                        key={emoji}
                        onClick={() =>
                          setSelectedIcon({ type: "emoji", value: emoji })
                        }
                        className={cn(
                          "flex h-10 w-10 items-center justify-center rounded-md text-xl transition-colors hover:bg-zinc-700",
                          selectedIcon.type === "emoji" &&
                            selectedIcon.value === emoji &&
                            "bg-zinc-700 ring-2 ring-zinc-600",
                        )}
                      >
                        {emoji}
                      </button>
                    ))}
                  </div>
                </ScrollArea>
              </TabsContent>

              <TabsContent value="icon" className="mt-4">
                <ScrollArea className="h-48 rounded-lg border border-zinc-800 bg-zinc-800/50 p-4">
                  <div className="grid grid-cols-8 gap-2">
                    {ICON_NAMES.map((iconName) => {
                      const IconComponent = (LucideIcons as any)[iconName];
                      if (!IconComponent) return null;
                      return (
                        <button
                          type="button"
                          key={iconName}
                          onClick={() =>
                            setSelectedIcon({ type: "icon", value: iconName })
                          }
                          className={cn(
                            "flex h-10 w-10 items-center justify-center rounded-md transition-colors hover:bg-zinc-700",
                            selectedIcon.type === "icon" &&
                              selectedIcon.value === iconName &&
                              "bg-zinc-700 ring-2 ring-zinc-600",
                          )}
                          title={iconName}
                        >
                          <IconComponent className="h-4 w-4 text-zinc-300" />
                        </button>
                      );
                    })}
                  </div>
                </ScrollArea>
              </TabsContent>
            </Tabs>
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="ghost"
            onClick={() => onOpenChange(false)}
            className="text-zinc-400 hover:bg-zinc-800 hover:text-zinc-100"
          >
            Cancel
          </Button>
          <Button
            onClick={handleSubmit}
            className="bg-zinc-700 text-zinc-100 hover:bg-zinc-600"
          >
            {editMode ? "Save Changes" : "Create Workspace"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
