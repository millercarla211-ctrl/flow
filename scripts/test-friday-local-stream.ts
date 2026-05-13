import {
  collectLocalTextStream,
  createLocalAssistantDraft,
  createFridayGatewaySystemPrompt,
  resolveFridayModel,
  resolveFridayGatewayChatRequest,
} from "../src/features/ai";
import { createLocalAgentRun } from "../src/features/friday/utils/localAgentRunner";
import { rankAskContext } from "../src/features/friday/utils/localRetrieval";
import { createLocalResearchDraft } from "../src/features/friday/utils/localResearch";
import { parseFridayStreamPayload } from "../src/features/friday/utils/providerHealth";
import { buildProviderResearchPrompt } from "../src/features/friday/utils/providerResearch";

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

const retrievedContext = rankAskContext({
  query: "remember workspace automations",
  contextItems: [],
  memories: [
    {
      id: "memory_test",
      title: "Workspace automations",
      body: "Friday should keep scheduled follow-ups and local automations attached to the workspace.",
      scope: "Project",
      pinned: true,
      projectId: "project_test",
      projectName: "Friday OS",
      createdAt: timestamp,
      updatedAt: timestamp,
    },
  ],
  projectId: "project_test",
});

if (retrievedContext[0]?.kind !== "memory") {
  throw new Error("Friday Ask retrieval did not rank pinned project memory.");
}

const agentRun = createLocalAgentRun({
  id: "agent_test",
  title: "inspect the Ask page",
  target: "code",
  status: "Queued",
  createdAt: timestamp,
  updatedAt: timestamp,
});

if (agentRun.status !== "Completed" || !agentRun.result.includes("No remote provider")) {
  throw new Error("Friday local agent runner did not produce the expected guarded result.");
}

const gatewayDeniedByRequest = resolveFridayGatewayChatRequest(
  {
    allowCloud: false,
    model: "gateway-openai-gpt-5-4",
    messages: [
      {
        id: "msg_test",
        role: "user",
        parts: [{ type: "text", text: "hello" }],
      },
    ],
  },
  { cloudEnabled: true },
);

if (gatewayDeniedByRequest.ok || gatewayDeniedByRequest.status !== 403) {
  throw new Error("Friday gateway route did not require explicit cloud approval.");
}

const gatewayDeniedByBuild = resolveFridayGatewayChatRequest(
  {
    allowCloud: true,
    model: "gateway-openai-gpt-5-4",
    messages: [
      {
        id: "msg_test",
        role: "user",
        parts: [{ type: "text", text: "hello" }],
      },
    ],
  },
  { cloudEnabled: false },
);

if (gatewayDeniedByBuild.ok || gatewayDeniedByBuild.status !== 403) {
  throw new Error("Friday gateway route did not honor local-only build settings.");
}

const groqDeniedByBuild = resolveFridayGatewayChatRequest(
  {
    allowCloud: true,
    model: "groq-llama-3-1-8b-instant",
    messages: [
      {
        id: "msg_test",
        role: "user",
        parts: [{ type: "text", text: "hello" }],
      },
    ],
  },
  { groqEnabled: false },
);

if (groqDeniedByBuild.ok || groqDeniedByBuild.status !== 403) {
  throw new Error("Friday Groq route did not honor local-only build settings.");
}

const groqAllowed = resolveFridayGatewayChatRequest(
  {
    allowCloud: true,
    model: "groq-llama-3-1-8b-instant",
    messages: [
      {
        id: "msg_test",
        role: "user",
        parts: [{ type: "text", text: "hello" }],
      },
    ],
  },
  { groqEnabled: true },
);

if (!groqAllowed.ok || groqAllowed.provider !== "groq" || groqAllowed.modelId !== "llama-3.1-8b-instant") {
  throw new Error("Friday Groq route did not resolve the approved Groq model.");
}

const gatewayAllowed = resolveFridayGatewayChatRequest(
  {
    allowCloud: true,
    model: "gateway-openai-gpt-5-4",
    context: {
      projectName: "Friday OS",
      projectInstructions: "Keep provider use explicit.",
    },
    messages: [
      {
        id: "msg_test",
        role: "user",
        parts: [{ type: "text", text: "hello" }],
      },
    ],
  },
  { cloudEnabled: true },
);

if (!gatewayAllowed.ok || gatewayAllowed.modelId !== "openai/gpt-5.4") {
  throw new Error("Friday gateway route did not resolve the approved gateway model.");
}

const gatewayPrompt = createFridayGatewaySystemPrompt(gatewayAllowed.context);
if (!gatewayPrompt.includes("Active project: Friday OS")) {
  throw new Error("Friday gateway prompt did not include approved project context.");
}

const providerHealthText = parseFridayStreamPayload(
  [
    'data: {"type":"start"}',
    'data: {"type":"text-start","id":"txt-0"}',
    'data: {"type":"text-delta","id":"txt-0","delta":"Friday"}',
    'data: {"type":"text-delta","id":"txt-0","delta":" ready."}',
    "data: [DONE]",
  ].join("\n\n"),
);

if (providerHealthText.text !== "Friday ready.") {
  throw new Error("Friday provider health parser did not collect streamed text.");
}

const providerHealthError = parseFridayStreamPayload(
  'data: {"type":"error","errorText":"Provider unavailable"}\n\ndata: [DONE]\n',
);

if (providerHealthError.errorText !== "Provider unavailable") {
  throw new Error("Friday provider health parser did not expose streamed errors.");
}

const providerResearchPrompt = buildProviderResearchPrompt({
  id: "research_test",
  createdAt: timestamp,
  updatedAt: timestamp,
  topic: "local-first research",
  sources: ["Local files"],
  plan: ["Review local evidence.", "Write a cited answer."],
  citations: [
    {
      id: "source_test",
      label: "local-note.md",
      kind: "note",
      excerpt: "Friday should cite only approved local evidence.",
    },
  ],
});

if (
  !providerResearchPrompt.includes("Do not invent citations") ||
  !providerResearchPrompt.includes("[1] local-note.md")
) {
  throw new Error("Friday provider research prompt did not preserve citation boundaries.");
}

console.log(`Friday local stream smoke passed with ${model.label}.`);
