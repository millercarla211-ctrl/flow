export type FridayToolStatus = "available" | "approval-required" | "planned";

export type FridayToolDefinition = {
  key: string;
  label: string;
  scope: "local" | "connector" | "gateway";
  status: FridayToolStatus;
  description: string;
};

export const FRIDAY_TOOL_DEFINITIONS: FridayToolDefinition[] = [
  {
    key: "local-files",
    label: "Local files",
    scope: "local",
    status: "available",
    description: "Read user-selected local files and project context after explicit selection.",
  },
  {
    key: "voice-transcription",
    label: "Voice transcription",
    scope: "local",
    status: "available",
    description: "Use the WhisperFlow Beater STT/TTS pipeline as a first-class Friday input.",
  },
  {
    key: "web-research",
    label: "Web research",
    scope: "connector",
    status: "approval-required",
    description: "Future cited search with source controls and a visible research plan.",
  },
  {
    key: "browser-agent",
    label: "Browser agent",
    scope: "connector",
    status: "planned",
    description: "Future browser execution with approval gates and a persistent run log.",
  },
  {
    key: "gateway-models",
    label: "Gateway models",
    scope: "gateway",
    status: "approval-required",
    description: "Optional Vercel AI Gateway access when local-only mode is disabled.",
  },
];
