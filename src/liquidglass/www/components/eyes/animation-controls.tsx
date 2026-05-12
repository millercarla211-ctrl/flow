"use client";

import { Button } from "@/liquidglass/www/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/liquidglass/www/components/ui/card";
import { Badge } from "@/liquidglass/www/components/ui/badge";
import { ScrollArea } from "@/liquidglass/www/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/liquidglass/www/components/ui/tabs";
import { getAllAnimations } from "@/liquidglass/www/lib/eyes-registry";
import { useEyesControl } from "@/liquidglass/www/lib/eyes-context";
import type { Animation } from "@/liquidglass/www/lib/eyes-types";
import { useState } from "react";
import {
  Smile,
  Frown,
  Angry,
  Heart,
  Eye,
  Zap,
  Sparkles,
  Target,
  ArrowLeft,
  ArrowRight,
  RotateCcw,
  Skull,
  Moon,
  Search,
  Lightbulb,
  PartyPopper,
  Crosshair,
} from "lucide-react";

const iconMap: Record<string, React.ComponentType<{ className?: string }>> = {
  blink: Eye,
  happy: Smile,
  surprised: Sparkles,
  love: Heart,
  wink: Eye,
  sad: Frown,
  angry: Angry,
  sleepy: Moon,
  squint: Search,
  dizzy: RotateCcw,
  "look-left": ArrowLeft,
  "look-right": ArrowRight,
  shake: RotateCcw,
  glitch: Zap,
  bounce: Zap,
  roll: RotateCcw,
  scared: Sparkles,
  thinking: Lightbulb,
  excited: PartyPopper,
  gunshot: Skull,
};

interface AnimationControlsProps {
  onGunshotModeChange?: (active: boolean) => void;
  onSpidermanModeChange?: (active: boolean) => void;
}

export function AnimationControls({ onGunshotModeChange, onSpidermanModeChange }: AnimationControlsProps) {
  const { play, current } = useEyesControl();
  const animations = getAllAnimations();
  const [gunshotMode, setGunshotMode] = useState(false);
  const [spidermanMode, setSpidermanMode] = useState(false);

  const toggleGunshotMode = () => {
    const newMode = !gunshotMode;
    setGunshotMode(newMode);
    onGunshotModeChange?.(newMode);
    if (newMode) {
      play("gunshot");
      setSpidermanMode(false);
      onSpidermanModeChange?.(false);
    }
  };

  const toggleSpidermanMode = () => {
    const newMode = !spidermanMode;
    setSpidermanMode(newMode);
    onSpidermanModeChange?.(newMode);
    if (newMode) {
      setGunshotMode(false);
      onGunshotModeChange?.(false);
    }
  };

  const groupedAnimations = animations.reduce(
    (acc, anim) => {
      const category = anim.tags?.[0] || "other";
      if (!acc[category]) acc[category] = [];
      acc[category].push(anim);
      return acc;
    },
    {} as Record<string, Animation[]>,
  );

  const categories = [
    { key: "basic", label: "Basic", icon: Eye },
    { key: "emotion", label: "Emotions", icon: Smile },
    { key: "gesture", label: "Gestures", icon: Target },
    { key: "effect", label: "Effects", icon: Zap },
    { key: "viral", label: "Viral", icon: Sparkles },
  ];

  const renderAnimationButton = (anim: Animation) => {
    const Icon = iconMap[anim.name] || Eye;
    const isPlaying = current === anim.name;

    return (
      <Button
        key={anim.name}
        variant={isPlaying ? "default" : "outline"}
        size="sm"
        onClick={() => play(anim.name)}
        className="flex items-center gap-2 justify-start h-auto py-3 px-4"
        disabled={isPlaying}
      >
        <Icon className="h-4 w-4 shrink-0" />
        <div className="flex flex-col items-start gap-1 flex-1 min-w-0">
          <span className="text-sm font-medium truncate w-full text-left">
            {anim.name.split("-").map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" ")}
          </span>
          <span className="text-xs text-muted-foreground truncate w-full text-left">
            {anim.description}
          </span>
        </div>
        {anim.trigger && (
          <Badge variant="secondary" className="text-xs shrink-0">
            {anim.trigger}
          </Badge>
        )}
      </Button>
    );
  };

  return (
    <Card className="w-full max-w-md">
      <CardHeader className="pb-3">
        <CardTitle className="text-lg flex items-center gap-2">
          <Eye className="h-5 w-5" />
          Animation Controls
        </CardTitle>
        <CardDescription>Click any animation to play it</CardDescription>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="quick" className="w-full">
          <TabsList className="grid w-full grid-cols-3 mb-4">
            <TabsTrigger value="all">All</TabsTrigger>
            <TabsTrigger value="category">Category</TabsTrigger>
            <TabsTrigger value="quick">Quick</TabsTrigger>
          </TabsList>

          <TabsContent value="all" className="mt-0">
            <ScrollArea className="h-[400px] pr-4">
              <div className="flex flex-col gap-2">
                {animations.map(renderAnimationButton)}
              </div>
            </ScrollArea>
          </TabsContent>

          <TabsContent value="category" className="mt-0">
            <ScrollArea className="h-[400px] pr-4">
              <div className="space-y-4">
                {categories.map((cat) => {
                  const anims = groupedAnimations[cat.key] || [];
                  if (anims.length === 0) return null;

                  return (
                    <div key={cat.key} className="space-y-2">
                      <div className="flex items-center gap-2 text-sm font-semibold text-muted-foreground">
                        <cat.icon className="h-4 w-4" />
                        {cat.label}
                      </div>
                      <div className="flex flex-col gap-2">
                        {anims.map(renderAnimationButton)}
                      </div>
                    </div>
                  );
                })}
              </div>
            </ScrollArea>
          </TabsContent>

          <TabsContent value="quick" className="mt-0">
            <div className="grid grid-cols-2 gap-2">
              <Button
                variant="outline"
                onClick={() => play("blink")}
                className="h-20 flex flex-col gap-1"
              >
                <Eye className="h-5 w-5" />
                <span className="text-xs">Blink</span>
              </Button>
              <Button
                variant="outline"
                onClick={() => play("happy")}
                className="h-20 flex flex-col gap-1"
              >
                <Smile className="h-5 w-5" />
                <span className="text-xs">Happy</span>
              </Button>
              <Button
                variant="outline"
                onClick={() => play("love")}
                className="h-20 flex flex-col gap-1"
              >
                <Heart className="h-5 w-5" />
                <span className="text-xs">Love</span>
              </Button>
              <Button
                variant="outline"
                onClick={() => play("surprised")}
                className="h-20 flex flex-col gap-1"
              >
                <Sparkles className="h-5 w-5" />
                <span className="text-xs">Surprised</span>
              </Button>
              <Button
                variant="outline"
                onClick={() => play("angry")}
                className="h-20 flex flex-col gap-1"
              >
                <Angry className="h-5 w-5" />
                <span className="text-xs">Angry</span>
              </Button>
              <Button
                variant={gunshotMode ? "destructive" : "outline"}
                onClick={toggleGunshotMode}
                className="h-20 flex flex-col gap-1"
              >
                <Skull className="h-5 w-5" />
                <span className="text-xs">{gunshotMode ? "Exit Gunshot" : "Gunshot Mode"}</span>
              </Button>
              <Button
                variant={spidermanMode ? "default" : "outline"}
                onClick={toggleSpidermanMode}
                className="h-20 flex flex-col gap-1"
              >
                <Crosshair className="h-5 w-5" />
                <span className="text-xs">{spidermanMode ? "Exit Spider-Man" : "Spider-Man"}</span>
              </Button>
            </div>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
}
