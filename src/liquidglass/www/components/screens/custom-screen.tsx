"use client";

import { motion } from "motion/react";
import { getDockIconComponent } from "./dock-icons";

interface CustomScreenProps {
  title: string;
  dockIcon?: string;
}

export function CustomScreen({ title, dockIcon }: CustomScreenProps) {
  const Icon = getDockIconComponent(dockIcon);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="flex h-full w-full flex-col items-center justify-center gap-4 rounded-[20px] border border-border bg-muted/30 p-8 shadow-2xl overflow-hidden"
    >
      {Icon && (
        <div className="flex h-16 w-16 items-center justify-center rounded-xl bg-muted">
          <Icon className="h-8 w-8 text-muted-foreground" />
        </div>
      )}
      <h2 className="text-xl font-semibold text-foreground">{title}</h2>
      <p className="text-sm text-muted-foreground">Empty screen</p>
    </motion.div>
  );
}
