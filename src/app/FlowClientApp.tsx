"use client";

import { StrictMode, useEffect, useState } from "react";
import App from "./App";
import { AppProviders } from "./providers";

export default function FlowClientApp() {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) return null;

  return (
    <StrictMode>
      <AppProviders>
        <App />
      </AppProviders>
    </StrictMode>
  );
}
