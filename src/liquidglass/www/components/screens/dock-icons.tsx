"use client";

import {
  BookOpen,
  Calendar,
  Cloud,
  Code,
  File,
  FileText,
  Folder,
  Globe,
  Heart,
  Home,
  Image,
  type LucideIcon,
  Mail,
  MessageSquare,
  Music,
  Star,
  Sun,
  Terminal,
  Video,
  Zap,
} from "lucide-react";
import type { Screen } from "./types";

/** Icon names we use for custom screens. Must match keys in DOCK_ICON_MAP. */
export const DOCK_ICON_NAMES = [
  "BookOpen",
  "Calendar",
  "Cloud",
  "File",
  "FileText",
  "Folder",
  "Heart",
  "Image",
  "Mail",
  "MessageSquare",
  "Music",
  "Star",
  "Sun",
  "Video",
  "Zap",
] as const;

export type DockIconName = (typeof DOCK_ICON_NAMES)[number];

export const DOCK_ICON_MAP: Record<DockIconName, LucideIcon> = {
  BookOpen,
  Calendar,
  Cloud,
  File,
  FileText,
  Folder,
  Heart,
  Image,
  Mail,
  MessageSquare,
  Music,
  Star,
  Sun,
  Video,
  Zap,
};

export function getRandomDockIconName(): DockIconName {
  return DOCK_ICON_NAMES[Math.floor(Math.random() * DOCK_ICON_NAMES.length)];
}

export function getDockIconComponent(
  name: string | undefined,
): LucideIcon | null {
  if (!name || !(name in DOCK_ICON_MAP)) return null;
  return DOCK_ICON_MAP[name as DockIconName];
}

const TYPE_ICON_MAP: Record<string, LucideIcon> = {
  welcome: Home,
  terminal: Terminal,
  code: Code,
  browser: Globe,
};

/** Icon component for a screen (fixed type or custom dockIcon). */
export function getIconForScreen(screen: Screen): LucideIcon {
  if (screen.type === "custom" && screen.dockIcon) {
    const icon = getDockIconComponent(screen.dockIcon);
    if (icon) return icon;
  }
  return TYPE_ICON_MAP[screen.type] ?? File;
}
