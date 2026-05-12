"use client";

import { motion } from "motion/react";

export function CodeScreen() {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="flex h-full w-full flex-col rounded-[20px] border border-border bg-[#1e1e1e] font-mono text-sm shadow-2xl overflow-hidden"
    >
      <div className="mb-2 flex items-center gap-2 p-4 pb-0">
        <div className="h-3 w-3 rounded-full bg-red-500" />
        <div className="h-3 w-3 rounded-full bg-yellow-500" />
        <div className="h-3 w-3 rounded-full bg-green-500" />
        <span className="ml-2 text-xs text-muted-foreground">Code Editor</span>
      </div>
      <div className="flex-1 overflow-auto px-4 pb-4">
        <div className="space-y-1 text-sm">
          <div>
            <span className="text-purple-400">import</span>
            <span className="text-white"> {"{"} </span>
            <span className="text-blue-300">useState</span>
            <span className="text-white"> {"}"} </span>
            <span className="text-purple-400">from</span>
            <span className="text-orange-300"> "react"</span>
            <span className="text-white">;</span>
          </div>
          <div className="mt-4">
            <span className="text-purple-400">export</span>
            <span className="text-purple-400"> function</span>
            <span className="text-yellow-300"> App</span>
            <span className="text-white">() {"{"}</span>
          </div>
          <div className="ml-4">
            <span className="text-purple-400">const</span>
            <span className="text-white"> [</span>
            <span className="text-blue-300">count</span>
            <span className="text-white">, </span>
            <span className="text-blue-300">setCount</span>
            <span className="text-white">] = </span>
            <span className="text-yellow-300">useState</span>
            <span className="text-white">(</span>
            <span className="text-orange-300">0</span>
            <span className="text-white">);</span>
          </div>
          <div className="mt-4 ml-4">
            <span className="text-purple-400">return</span>
            <span className="text-white"> (</span>
          </div>
          <div className="ml-8">
            <span className="text-gray-500">&lt;</span>
            <span className="text-green-400">div</span>
            <span className="text-gray-500">&gt;</span>
          </div>
          <div className="ml-12">
            <span className="text-gray-500">&lt;</span>
            <span className="text-green-400">h1</span>
            <span className="text-gray-500">&gt;</span>
            <span className="text-white">Zen Browser Code Editor</span>
            <span className="text-gray-500">&lt;/</span>
            <span className="text-green-400">h1</span>
            <span className="text-gray-500">&gt;</span>
          </div>
          <div className="ml-8">
            <span className="text-gray-500">&lt;/</span>
            <span className="text-green-400">div</span>
            <span className="text-gray-500">&gt;</span>
          </div>
          <div className="ml-4">
            <span className="text-white">);</span>
          </div>
          <div>
            <span className="text-white">{"}"}</span>
          </div>
        </div>
      </div>
    </motion.div>
  );
}
