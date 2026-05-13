import { AgentsWorkspace } from "./local-workspaces/AgentsWorkspace";
import { AutomationsWorkspace } from "./local-workspaces/AutomationsWorkspace";
import { CanvasWorkspace } from "./local-workspaces/CanvasWorkspace";
import { ConnectorsWorkspace } from "./local-workspaces/ConnectorsWorkspace";
import { MemoryWorkspace } from "./local-workspaces/MemoryWorkspace";
import { ProjectsWorkspace } from "./local-workspaces/ProjectsWorkspace";
import { ResearchWorkspace } from "./local-workspaces/ResearchWorkspace";
import type { FridayAssistantView } from "../pageData";

export function FridayLocalWorkspace({ view }: { view: FridayAssistantView }) {
  if (view === "research") return <ResearchWorkspace />;
  if (view === "agents") return <AgentsWorkspace />;
  if (view === "canvas" || view === "artifacts") return <CanvasWorkspace />;
  if (view === "projects") return <ProjectsWorkspace />;
  if (view === "memory") return <MemoryWorkspace />;
  if (view === "connectors") return <ConnectorsWorkspace />;
  if (view === "automations") return <AutomationsWorkspace />;
  return null;
}
