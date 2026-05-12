"use client";

import {
  Box,
  Layers,
  Paintbrush,
  RectangleHorizontal,
  Settings,
  Sliders,
  Video,
  Wifi,
} from "lucide-react";
import { useState } from "react";
import { Button } from "@/liquidglass/www/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/liquidglass/www/components/ui/popover";
import { Slider } from "@/liquidglass/www/components/ui/slider";

type MediaType = "text" | "email" | "image" | "video" | "audio" | "live" | "3d";

interface MediaControlsProps {
  mediaType: MediaType;
}

// Image-specific controls
export function ImageControls() {
  const [ratio, setRatio] = useState("1:1");
  const [imageCount, setImageCount] = useState(1);
  const [open, setOpen] = useState(false);

  const ratios = ["1:1", "16:9", "9:16", "4:3", "3:4"];

  return (
    <div className="flex items-center gap-2">
      {/* Aspect Ratio Selector */}
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
            <RectangleHorizontal className="h-3.5 w-3.5" />
            <span className="hidden text-xs sm:inline">{ratio}</span>
          </Button>
        </PopoverTrigger>
        <PopoverContent side="top" align="start" className="w-48 p-2">
          <div className="space-y-1">
            {ratios.map((r) => (
              <Button
                key={r}
                variant={ratio === r ? "default" : "ghost"}
                size="sm"
                onClick={() => {
                  setRatio(r);
                  setOpen(false);
                }}
                className="w-full justify-start gap-2"
              >
                <RectangleHorizontal className="h-4 w-4" />
                <span className="text-sm">{r}</span>
              </Button>
            ))}
          </div>
        </PopoverContent>
      </Popover>

      {/* Image Count */}
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
            <Layers className="h-3.5 w-3.5" />
            <span className="hidden text-xs sm:inline">{imageCount}</span>
          </Button>
        </PopoverTrigger>
        <PopoverContent side="top" align="start" className="w-64 p-4">
          <div className="space-y-3">
            <div className="text-muted-foreground text-xs font-medium">
              Number of Images
            </div>
            <div className="flex items-center gap-3">
              <Slider
                value={[imageCount]}
                onValueChange={(v) => setImageCount(v[0])}
                min={1}
                max={10}
                step={1}
                className="flex-1"
              />
              <span className="text-foreground text-sm font-medium">
                {imageCount}
              </span>
            </div>
          </div>
        </PopoverContent>
      </Popover>

      {/* Image Settings */}
      <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
        <Settings className="h-3.5 w-3.5" />
        <span className="hidden text-xs sm:inline">Settings</span>
      </Button>
    </div>
  );
}

// Video-specific controls
function VideoControls() {
  const [duration, setDuration] = useState(5);

  return (
    <div className="flex items-center gap-2">
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
            <Video className="h-3.5 w-3.5" />
            <span className="hidden text-xs sm:inline">{duration}s</span>
          </Button>
        </PopoverTrigger>
        <PopoverContent side="top" align="start" className="w-64 p-4">
          <div className="space-y-3">
            <div className="text-muted-foreground text-xs font-medium">
              Video Duration
            </div>
            <div className="flex items-center gap-3">
              <Slider
                value={[duration]}
                onValueChange={(v) => setDuration(v[0])}
                min={1}
                max={60}
                step={1}
                className="flex-1"
              />
              <span className="text-foreground text-sm font-medium">
                {duration}s
              </span>
            </div>
          </div>
        </PopoverContent>
      </Popover>

      <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
        <RectangleHorizontal className="h-3.5 w-3.5" />
        <span className="hidden text-xs sm:inline">16:9</span>
      </Button>
    </div>
  );
}

// Audio-specific controls
function AudioControls() {
  return (
    <div className="flex items-center gap-2">
      <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
        <Sliders className="h-3.5 w-3.5" />
        <span className="hidden text-xs sm:inline">Voice</span>
      </Button>
    </div>
  );
}

// Live-specific controls
function LiveControls() {
  const [quality, setQuality] = useState("HD");

  return (
    <div className="flex items-center gap-2">
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
            <Wifi className="h-3.5 w-3.5" />
            <span className="hidden text-xs sm:inline">{quality}</span>
          </Button>
        </PopoverTrigger>
        <PopoverContent side="top" align="start" className="w-48 p-2">
          <div className="space-y-1">
            {["SD", "HD", "Full HD", "4K"].map((q) => (
              <Button
                key={q}
                variant={quality === q ? "default" : "ghost"}
                size="sm"
                onClick={() => setQuality(q)}
                className="w-full justify-start gap-2"
              >
                <Wifi className="h-4 w-4" />
                <span className="text-sm">{q}</span>
              </Button>
            ))}
          </div>
        </PopoverContent>
      </Popover>

      <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
        <Settings className="h-3.5 w-3.5" />
        <span className="hidden text-xs sm:inline">Stream</span>
      </Button>
    </div>
  );
}

// 3D-specific controls
function ThreeDControls() {
  const [format, setFormat] = useState("GLB");

  return (
    <div className="flex items-center gap-2">
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
            <Box className="h-3.5 w-3.5" />
            <span className="hidden text-xs sm:inline">{format}</span>
          </Button>
        </PopoverTrigger>
        <PopoverContent side="top" align="start" className="w-48 p-2">
          <div className="space-y-1">
            {["GLB", "GLTF", "OBJ", "FBX", "STL"].map((f) => (
              <Button
                key={f}
                variant={format === f ? "default" : "ghost"}
                size="sm"
                onClick={() => setFormat(f)}
                className="w-full justify-start gap-2"
              >
                <Box className="h-4 w-4" />
                <span className="text-sm">{f}</span>
              </Button>
            ))}
          </div>
        </PopoverContent>
      </Popover>

      <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
        <Paintbrush className="h-3.5 w-3.5" />
        <span className="hidden text-xs sm:inline">Texture</span>
      </Button>
    </div>
  );
}

export function MediaControls({ mediaType }: MediaControlsProps) {
  // Return different controls based on media type
  switch (mediaType) {
    case "image":
      return <ImageControls />;
    case "video":
      return <VideoControls />;
    case "audio":
      return <AudioControls />;
    case "live":
      return <LiveControls />;
    case "3d":
      return <ThreeDControls />;
    default:
      return null; // Text and email don't have additional controls
  }
}
