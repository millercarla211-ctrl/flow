export type FridayAssistantView =
  | "ask"
  | "research"
  | "agents"
  | "canvas"
  | "projects"
  | "memory"
  | "connectors"
  | "artifacts"
  | "automations";

export type FridayFeatureSpec = {
  id: FridayAssistantView;
  eyebrow: string;
  title: string;
  summary: string;
  status: "Ready" | "Local draft" | "Planned";
  primaryAction: string;
  capabilities: string[];
  dataBoundary: string;
  emptyState: string;
};

export const FRIDAY_FEATURE_SPECS: Record<FridayAssistantView, FridayFeatureSpec> = {
  ask: {
    id: "ask",
    eyebrow: "Assistant",
    title: "Ask Friday",
    summary: "A streaming assistant surface with local model routing and provider controls.",
    status: "Ready",
    primaryAction: "Start a chat",
    capabilities: ["Streaming responses", "Model picker", "Source and tool toggles"],
    dataBoundary: "Defaults to local model roles. Gateway models remain disabled until cloud mode is enabled.",
    emptyState: "Start with a question, a coding task, or a rewrite request.",
  },
  research: {
    id: "research",
    eyebrow: "Research",
    title: "Cited Research",
    summary: "Plan-first research with source scopes, progress logs, citations, and export-ready reports.",
    status: "Local draft",
    primaryAction: "Create plan",
    capabilities: ["Editable research plan", "Source scope controls", "Citation-ready report layout"],
    dataBoundary: "Local notes and files are separate from future web or premium source connectors.",
    emptyState: "Create a research brief and choose whether Friday can use web, files, or local notes.",
  },
  agents: {
    id: "agents",
    eyebrow: "Agents",
    title: "Task Agents",
    summary: "Browser, code, and tool task runners with approval-first execution.",
    status: "Local draft",
    primaryAction: "Prepare task",
    capabilities: ["Tool approval queue", "Run log", "Local Tauri command boundary"],
    dataBoundary: "Agent actions start as local commands and require explicit approval before external work.",
    emptyState: "Queue a task for coding, browser checks, or file-work automation.",
  },
  canvas: {
    id: "canvas",
    eyebrow: "Canvas",
    title: "Artifact Canvas",
    summary: "Side-by-side workspace for docs, code, markdown, UI snippets, and generated artifacts.",
    status: "Ready",
    primaryAction: "Open canvas",
    capabilities: ["Editable artifact area", "Preview pane", "Version-aware drafts"],
    dataBoundary: "Canvas drafts stay local until exported or shared through a configured connector.",
    emptyState: "Create a document, code snippet, markdown plan, or UI prototype.",
  },
  projects: {
    id: "projects",
    eyebrow: "Projects",
    title: "Friday Projects",
    summary: "Persistent workspaces with files, instructions, memories, threads, and model preferences.",
    status: "Local draft",
    primaryAction: "New project",
    capabilities: ["Project instructions", "File set", "Thread memory"],
    dataBoundary: "Project memory is local-first and should not sync until account sync is configured.",
    emptyState: "Create a workspace for a product, research topic, or coding sprint.",
  },
  memory: {
    id: "memory",
    eyebrow: "Memory",
    title: "Memory Control",
    summary: "Inspect, edit, pin, and disable memories that Friday uses across projects.",
    status: "Ready",
    primaryAction: "Review memory",
    capabilities: ["Pinned facts", "Project-scoped memory", "Local privacy controls"],
    dataBoundary: "Memory reads from local data only unless sync is enabled.",
    emptyState: "Add a memory, edit it in place, or pin it into active project context.",
  },
  connectors: {
    id: "connectors",
    eyebrow: "Connectors",
    title: "Apps and Providers",
    summary: "Manage local files, future MCP connectors, web search, and optional cloud providers.",
    status: "Local draft",
    primaryAction: "Add connector",
    capabilities: ["Local files", "Web search settings", "Future MCP/provider slots"],
    dataBoundary: "Every connector is opt-in. Friday must not call remote providers silently.",
    emptyState: "Connectors are ready for local files and future provider setup.",
  },
  artifacts: {
    id: "artifacts",
    eyebrow: "Artifacts",
    title: "Live Outputs",
    summary: "Collect generated apps, reports, code edits, tables, diagrams, and reusable cards.",
    status: "Ready",
    primaryAction: "Create artifact",
    capabilities: ["Preview cards", "Export targets", "Artifact history"],
    dataBoundary: "Artifacts are stored locally and can be exported manually.",
    emptyState: "Generated outputs will appear here after Ask, Research, Agents, or Canvas work.",
  },
  automations: {
    id: "automations",
    eyebrow: "Automations",
    title: "Scheduled Work",
    summary: "Local reminders, background jobs, recurring checks, and future async agent runs.",
    status: "Ready",
    primaryAction: "Schedule job",
    capabilities: ["Local reminders", "Due task runner", "Pause and review controls"],
    dataBoundary: "Automations should run locally unless a cloud workspace is explicitly configured.",
    emptyState: "Create a reminder, recurring research check, or local maintenance task.",
  },
};

export const FRIDAY_DASHBOARD_ORDER: FridayAssistantView[] = [
  "ask",
  "research",
  "agents",
  "canvas",
  "projects",
  "connectors",
];
