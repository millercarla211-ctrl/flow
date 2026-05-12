"use client";

import { Cloud, Laptop } from "lucide-react";
import { motion } from "motion/react";
import { useState } from "react";
import { Button } from "@/liquidglass/www/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/liquidglass/www/components/ui/popover";
import { cn } from "@/liquidglass/www/lib/utils";

type TargetType = "local" | "remote";

export function AITargetSwitcher() {
  const [target, setTarget] = useState<TargetType>("local");
  const [open, setOpen] = useState(false);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" size="sm" className="h-8 gap-2 px-3">
          <motion.div
            initial={false}
            animate={{ rotate: target === "remote" ? 180 : 0 }}
            transition={{ duration: 0.2 }}
          >
            {target === "local" ? (
              <Laptop className="h-3.5 w-3.5" />
            ) : (
              <Cloud className="h-3.5 w-3.5" />
            )}
          </motion.div>
          <span className="hidden text-xs sm:inline">
            {target === "local" ? "Local" : "Remote"}
          </span>
        </Button>
      </PopoverTrigger>
      <PopoverContent side="top" align="start" className="w-56 p-3">
        <div className="space-y-2">
          <div className="text-muted-foreground mb-2 text-xs font-medium">
            Target
          </div>
          <div className="bg-muted flex rounded-lg p-1">
            <button
              onClick={() => {
                setTarget("local");
                setOpen(false);
              }}
              className={cn(
                "flex flex-1 items-center justify-center gap-2 rounded-md px-3 py-2 text-sm transition-all",
                target === "local"
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              <Laptop className="h-4 w-4" />
              <span>Local</span>
            </button>
            <button
              onClick={() => {
                setTarget("remote");
                setOpen(false);
              }}
              className={cn(
                "flex flex-1 items-center justify-center gap-2 rounded-md px-3 py-2 text-sm transition-all",
                target === "remote"
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              <Cloud className="h-4 w-4" />
              <span>Remote</span>
            </button>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
