import {
  collectLocalTextStream,
  createLocalAssistantDraft,
  resolveFridayModel,
} from "../src/features/ai";

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

console.log(`Friday local stream smoke passed with ${model.label}.`);
