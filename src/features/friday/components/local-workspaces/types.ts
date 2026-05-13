import type { LocalRecord } from "../../hooks/useLocalPersistence";
import type { UIMessage } from "ai";

export type ResearchCitation = {
  id: string;
  label: string;
  kind: "note" | "file" | "instruction" | "memory";
  excerpt: string;
};

export type ResearchBrief = LocalRecord & {
  topic: string;
  sources: string[];
  plan: string[];
  status?: "Planned" | "Drafted";
  report?: string;
  citations?: ResearchCitation[];
  inspectedUrl?: boolean;
  lastModel?: string;
  lastTokensPerSecond?: number;
  lastTotalTimeMs?: number;
  projectId?: string;
  projectName?: string;
};

export type AgentTask = LocalRecord & {
  title: string;
  brief?: string;
  target: "browser" | "code" | "files";
  status: "Needs approval" | "Queued" | "Running" | "Completed" | "Blocked";
  plan?: string[];
  log?: string[];
  result?: string;
  inspectedWorkspace?: boolean;
  inspectedUrl?: boolean;
  lastModel?: string;
  lastTokensPerSecond?: number;
  lastTotalTimeMs?: number;
  projectId?: string;
  projectName?: string;
};

export type CanvasArtifact = LocalRecord & {
  title: string;
  kind: "Doc" | "Code" | "Markdown" | "UI";
  content: string;
  projectId?: string;
  projectName?: string;
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
  projectId?: string;
  projectName?: string;
};

export type FridayAutomation = LocalRecord & {
  title: string;
  cadence: string;
  instruction?: string;
  enabled: boolean;
  nextRunAt?: string;
  lastRunAt?: string;
  lastRunMode?: "Manual" | "Scheduled";
  lastResult?: string;
  runCount?: number;
  lastModel?: string;
  lastTokensPerSecond?: number;
  lastTotalTimeMs?: number;
  projectId?: string;
  projectName?: string;
};

export type FridayAskThread = LocalRecord & {
  title: string;
  modelKey: string;
  messageCount: number;
  messages: UIMessage[];
  lastModel?: string;
  lastTokensPerSecond?: number;
  lastTotalTimeMs?: number;
  projectId?: string;
  projectName?: string;
};

export type ProjectContextItem = LocalRecord & {
  projectId: string;
  projectName: string;
  label: string;
  kind: "note" | "file" | "instruction";
  content: string;
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
  projectContext: "friday.project-context.v1",
  memory: "friday.memory.v1",
  askThreads: "friday.ask-threads.v1",
  automations: "friday.automations.v1",
  connectors: "friday.connectors.v1",
} as const;

export const DEFAULT_CONNECTORS: ConnectorSettings = {
  localFiles: true,
  webSearch: false,
  aiGateway: true,
  mcpConnectors: false,
};
