import type { LocalRecord } from "../../hooks/useLocalPersistence";

export type ResearchBrief = LocalRecord & {
  topic: string;
  sources: string[];
  plan: string[];
};

export type AgentTask = LocalRecord & {
  title: string;
  target: "browser" | "code" | "files";
  status: "Queued" | "Needs approval";
};

export type CanvasArtifact = LocalRecord & {
  title: string;
  kind: "Doc" | "Code" | "Markdown" | "UI";
  content: string;
};

export type FridayProject = LocalRecord & {
  name: string;
  instructions: string;
  modelKey: string;
};

export type FridayMemory = LocalRecord & {
  title: string;
  body: string;
  scope: "Global" | "Project" | "Voice";
  pinned: boolean;
};

export type FridayAutomation = LocalRecord & {
  title: string;
  cadence: string;
  enabled: boolean;
};

export type ConnectorSettings = {
  localFiles: boolean;
  webSearch: boolean;
  aiGateway: boolean;
  mcpConnectors: boolean;
};

export const STORAGE_KEYS = {
  research: "friday.research.v1",
  agents: "friday.agents.v1",
  artifacts: "friday.artifacts.v1",
  projects: "friday.projects.v1",
  memory: "friday.memory.v1",
  automations: "friday.automations.v1",
  connectors: "friday.connectors.v1",
} as const;

export const DEFAULT_CONNECTORS: ConnectorSettings = {
  localFiles: true,
  webSearch: false,
  aiGateway: false,
  mcpConnectors: false,
};
