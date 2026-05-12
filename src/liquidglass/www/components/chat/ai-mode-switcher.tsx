"use client";

import { Bug, GitBranch, MessageSquare, Sparkles } from "lucide-react";
import { motion } from "motion/react";
import { useState } from "react";
import { Button } from "@/liquidglass/www/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/liquidglass/www/components/ui/popover";
import { cn } from "@/liquidglass/www/lib/utils";

type AIMode = "agent" | "plan" | "ask" | "debug";

interface AIModeOption {
  id: AIMode;
  label: string;
  icon: React.ElementType;
  description: string;
}

const AI_MODES: AIModeOption[] = [
  {
    id: "agent",
    label: "Agent",
    icon: Sparkles,
    description: "Autonomous AI agent",
  },
  {
    id: "plan",
    label: "Plan Tree",
    icon: GitBranch,
    description: "Strategic planning mode",
  },
  {
    id: "ask",
    label: "Ask",
    icon: MessageSquare,
    description: "Question & answer mode",
  },
  {
    id: "debug",
    label: "Debug",
    icon: Bug,
    description: "Debug and troubleshoot",
  },
];

export function AIModeSwitcher() {
  const [mode, setMode] = useState<AIMode>("agent");
  const [open, setOpen] = useState(false);

  const currentMode = AI_MODES.find((m) => m.id === mode);
  const ModeIcon = currentMode?.icon || Sparkles;

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
          <ModeIcon className="h-3.5 w-3.5" />
          <span className="hidden text-xs sm:inline">{currentMode?.label}</span>
        </Button>
      </PopoverTrigger>
      <PopoverContent side="top" align="start" className="w-72 p-3">
        <div className="space-y-2">
          <div className="text-muted-foreground mb-3 text-xs font-medium">
            AI Mode
          </div>
          <div className="space-y-1">
            {AI_MODES.map((modeOption) => {
              const Icon = modeOption.icon;
              const isSelected = mode === modeOption.id;
              return (
                <motion.button
                  key={modeOption.id}
                  onClick={() => {
                    setMode(modeOption.id);
                    setOpen(false);
                  }}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  className={cn(
                    "flex w-full items-start gap-3 rounded-lg px-3 py-2.5 text-left transition-all",
                    isSelected
                      ? "bg-primary text-primary-foreground"
                      : "hover:bg-accent hover:text-accent-foreground",
                  )}
                >
                  <Icon className="mt-0.5 h-4 w-4 shrink-0" />
                  <div className="flex-1 space-y-0.5">
                    <div className="text-sm font-medium">
                      {modeOption.label}
                    </div>
                    <div
                      className={cn(
                        "text-xs",
                        isSelected
                          ? "text-primary-foreground/80"
                          : "text-muted-foreground",
                      )}
                    >
                      {modeOption.description}
                    </div>
                  </div>
                </motion.button>
              );
            })}
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
