"use client";

import { createContext, useContext, type ReactNode } from "react";

interface EyesContextType {
  play: (name: string) => void;
  current: string | null;
}

const EyesContext = createContext<EyesContextType | null>(null);

export function EyesProvider({ children, play, current }: { children: ReactNode; play: (name: string) => void; current: string | null }) {
  return <EyesContext.Provider value={{ play, current }}>{children}</EyesContext.Provider>;
}

export function useEyesControl() {
  const context = useContext(EyesContext);
  if (!context) {
    throw new Error("useEyesControl must be used within EyesProvider");
  }
  return context;
}
