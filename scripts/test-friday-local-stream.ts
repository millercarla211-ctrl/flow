import {
  collectLocalTextStream,
  createLocalAssistantDraft,
  resolveFridayModel,
} from "../src/features/ai";
import { createLocalResearchDraft } from "../src/features/friday/utils/localResearch";

const model = resolveFridayModel("qwen35-4b-revised-q4km");
const draft = createLocalAssistantDraft("write a short Friday status", model, {
  projectName: "Friday OS",
  projectInstructions: "Keep the answer local-first.",
  contextItems: [
    {
      label: "Roadmap",
      kind: "note",
      content: "Connect Ask responses to workspace records.",
    },
  ],
});
const streamed = await collectLocalTextStream(draft);

if (!streamed.includes("Friday is running in local-first mode")) {
  throw new Error("Friday local stream did not include the expected local-first message.");
}

if (!streamed.includes(model.label)) {
  throw new Error("Friday local stream did not include the selected model label.");
}

if (!streamed.includes("Project: Friday OS")) {
  throw new Error("Friday local stream did not include active project context.");
}

const timestamp = new Date().toISOString();
const researchDraft = createLocalResearchDraft({
  topic: "workspace records",
  project: {
    id: "project_test",
    name: "Friday OS",
    instructions: "Keep research local.",
    modelKey: "qwen35-4b-revised-q4km",
    createdAt: timestamp,
    updatedAt: timestamp,
  },
  contextItems: [
    {
      id: "context_test",
      projectId: "project_test",
      projectName: "Friday OS",
      label: "workspace-records.md",
      kind: "file",
      content: "Workspace records connect Ask, Research, Memory, Artifacts, and Automations.",
      createdAt: timestamp,
      updatedAt: timestamp,
    },
  ],
  memories: [],
});

if (!researchDraft.report.includes("[1] workspace-records.md")) {
  throw new Error("Friday local research did not include a cited local source.");
}

console.log(`Friday local stream smoke passed with ${model.label}.`);
