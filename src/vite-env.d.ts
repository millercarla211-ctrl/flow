/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_FLOW_AUTH_BASE_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare module "*.po" {
  import type { Messages } from "@lingui/core";

  export const messages: Messages;
}
