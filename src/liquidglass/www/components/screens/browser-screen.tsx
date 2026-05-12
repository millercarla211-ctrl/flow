"use client";

import { Globe } from "lucide-react";
import { motion } from "motion/react";

export function BrowserScreen() {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="flex h-full w-full flex-col rounded-[20px] border border-border bg-card shadow-2xl overflow-hidden"
    >
      <div className="flex items-center gap-2 border-b border-border p-3">
        <div className="h-3 w-3 rounded-full bg-red-500" />
        <div className="h-3 w-3 rounded-full bg-yellow-500" />
        <div className="h-3 w-3 rounded-full bg-green-500" />
        <div className="ml-2 flex flex-1 items-center gap-2 rounded-md bg-accent px-3 py-1.5">
          <Globe className="h-4 w-4 text-muted-foreground" />
          <span className="text-sm text-muted-foreground">
            https://zen-browser.app
          </span>
        </div>
      </div>
      <div className="flex flex-1 items-center justify-center">
        <div className="text-center space-y-4">
          <div className="mx-auto h-16 w-16 rounded-full bg-primary/10 flex items-center justify-center">
            <Globe className="h-8 w-8 text-primary" />
          </div>
          <h2 className="text-2xl font-bold text-foreground">Browser View</h2>
          <p className="text-muted-foreground">Your web browsing experience</p>
        </div>
      </div>
    </motion.div>
  );
}
