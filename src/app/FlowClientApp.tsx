"use client";

import { StrictMode } from "react";
import App from "./App";
import { AppProviders } from "./providers";

export default function FlowClientApp() {
  return (
    <StrictMode>
      <AppProviders>
        <App />
      </AppProviders>
    </StrictMode>
  );
}
