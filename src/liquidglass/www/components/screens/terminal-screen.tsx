"use client";

import { motion } from "motion/react";

export function TerminalScreen() {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="flex h-full w-full flex-col rounded-[20px] border border-border bg-black/95 font-mono text-sm shadow-2xl overflow-hidden"
    >
      <div className="mb-2 flex items-center gap-2 p-4 pb-0">
        <div className="h-3 w-3 rounded-full bg-red-500" />
        <div className="h-3 w-3 rounded-full bg-yellow-500" />
        <div className="h-3 w-3 rounded-full bg-green-500" />
        <span className="ml-2 text-xs text-muted-foreground">Terminal</span>
      </div>
      <div className="flex-1 overflow-auto px-4 pb-4">
        <div className="space-y-1">
          <div className="text-green-400">
            <span className="text-blue-400">user@zen</span>
            <span className="text-white">:</span>
            <span className="text-cyan-400">~</span>
            <span className="text-white">$ </span>
            <span className="text-white">Welcome to Zen Browser Terminal</span>
          </div>
          <div className="text-muted-foreground">Type commands here...</div>
          <div className="mt-4 text-green-400">
            <span className="text-blue-400">user@zen</span>
            <span className="text-white">:</span>
            <span className="text-cyan-400">~</span>
            <span className="text-white">$ </span>
            <span className="animate-pulse">_</span>
          </div>
        </div>
      </div>
    </motion.div>
  );
}
