import {
  collectLocalTextStream,
  createLocalAssistantDraft,
  createFridayGatewaySystemPrompt,
  resolveFridayModel,
  resolveFridayGatewayChatRequest,
} from "../src/features/ai";
import { createLocalAgentRun } from "../src/features/friday/utils/localAgentRunner";
import {
  createAutomationFallbackResult,
  createAutomationPrompt,
  isAutomationDue,
  nextScheduledAutomationRun,
  selectNextDueAutomation,
} from "../src/features/friday/utils/localAutomation";
import { rankAskContext } from "../src/features/friday/utils/localRetrieval";
import { createLocalResearchDraft } from "../src/features/friday/utils/localResearch";
import {
  checkFridayProviderHealth,
  parseFridayStreamPayload,
} from "../src/features/friday/utils/providerHealth";
import { buildProviderResearchPrompt } from "../src/features/friday/utils/providerResearch";
import { checkFridaySyncHealth } from "../src/features/friday/utils/syncHealth";
import {
  extractHtmlTitle,
  extractReadableHtmlText,
  inspectWebSource,
  normalizeWebInspectionUrl,
} from "../src/features/friday/utils/webInspection";
import { parseDuckDuckGoLiteResults, searchWebSources } from "../src/features/friday/utils/webSearch";
import {
  pullFridayWorkspaceSnapshot,
  pushFridayWorkspaceSnapshot,
} from "../src/features/friday/utils/workspaceCloudSync";
import {
  buildFridayWorkspaceBackup,
  getFridayWorkspaceBackupEntries,
  parseFridayWorkspaceBackup,
  serializeFridayWorkspaceBackup,
} from "../src/features/friday/utils/workspaceBackup";
import { getFridayAuthConfigStatus } from "../src/server/auth/db";
import {
  parseStoredFridayWorkspaceSnapshot,
  requireFridayWorkspaceSyncSession,
  validateFridayWorkspaceSyncPayload,
} from "../src/server/friday/workspaceSync";

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
function createTestStorage(seed: Record<string, string>): Storage {
  const store = new Map(Object.entries(seed));

  return {
    get length() {
      return store.size;
    },
    clear() {
      store.clear();
    },
    getItem(key: string) {
      return store.get(key) ?? null;
    },
    key(index: number) {
      return Array.from(store.keys())[index] ?? null;
    },
    removeItem(key: string) {
      store.delete(key);
    },
    setItem(key: string, value: string) {
      store.set(key, value);
    },
  };
}

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

const scopedAgentRun = createLocalAgentRun(
  {
    id: "agent_scoped_test",
    title: "fix src/features/friday/components/FridayAskView.tsx",
    brief: "Must preserve the model picker. Verify typecheck passes.",
    target: "code",
    status: "Queued",
    createdAt: timestamp,
    updatedAt: timestamp,
  },
  {
    projectName: "Friday OS",
    projectInstructions: "Keep Friday local-first.",
    contextItems: [
      {
        label: "agent-guidance.md",
        kind: "note",
        content: "Agent plans should use approved project context before execution.",
      },
    ],
  },
);

if (
  !scopedAgentRun.result.includes("Detected scope: src/features/friday/components/FridayAskView.tsx") ||
  !scopedAgentRun.result.includes("Acceptance checklist") ||
  !scopedAgentRun.result.includes("Project: Friday OS")
) {
  throw new Error("Friday scoped agent runner did not preserve path, acceptance, and project context.");
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

const readyProviderHealth = await checkFridayProviderHealth({
  fetcher: async (_input, init) => {
    if (init?.method !== "POST") {
      throw new Error("Friday provider health used the wrong HTTP method.");
    }

    return new Response(
      [
        'data: {"type":"text-start","id":"txt-0"}',
        'data: {"type":"text-delta","id":"txt-0","delta":"Friday provider ready."}',
        "data: [DONE]",
      ].join("\n\n"),
      { status: 200 },
    );
  },
  modelKey: "groq-llama-3-1-8b-instant",
});

if (readyProviderHealth.status !== "ready" || readyProviderHealth.preview !== "Friday provider ready.") {
  throw new Error("Friday provider health did not handle a valid streamed response.");
}

const failedProviderHealth = await checkFridayProviderHealth({
  fetcher: async () => {
    throw new Error("provider offline");
  },
  modelKey: "groq-llama-3-1-8b-instant",
});

if (failedProviderHealth.status !== "error" || failedProviderHealth.message !== "provider offline") {
  throw new Error("Friday provider health did not return a controlled fetch failure.");
}

const readySyncHealth = await checkFridaySyncHealth({
  fetcher: async (_input, init) => {
    if (init?.method !== "GET") {
      throw new Error("Friday sync health used the wrong HTTP method.");
    }

    return Response.json({
      checkedAt: timestamp,
      latencyMs: 2,
      message: "Friday account database is reachable.",
      ready: true,
      requirements: {
        authUrlConfigured: true,
        databaseConfigured: true,
        tokenConfigured: true,
      },
      status: "ready",
    });
  },
});

if (!readySyncHealth.ready || readySyncHealth.status !== "ready") {
  throw new Error("Friday sync health did not handle a valid ready response.");
}

const failedSyncHealth = await checkFridaySyncHealth({
  fetcher: async () => {
    throw new Error("sync offline");
  },
});

if (failedSyncHealth.status !== "error" || failedSyncHealth.message !== "sync offline") {
  throw new Error("Friday sync health did not return a controlled fetch failure.");
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

const blockedLocalUrl = normalizeWebInspectionUrl("http://localhost:8735");
if (blockedLocalUrl.ok) {
  throw new Error("Friday web inspection allowed a local URL.");
}

const normalizedWebUrl = normalizeWebInspectionUrl("https://example.com/path");
if (!normalizedWebUrl.ok || normalizedWebUrl.url !== "https://example.com/path") {
  throw new Error("Friday web inspection did not normalize a public URL.");
}

const sampleHtml =
  "<html><head><title>Friday &amp; Research</title><style>.x{}</style></head><body><script>bad()</script><main>Useful cited source text.</main></body></html>";
if (extractHtmlTitle(sampleHtml) !== "Friday & Research") {
  throw new Error("Friday web inspection did not extract the HTML title.");
}

if (!extractReadableHtmlText(sampleHtml).includes("Useful cited source text.")) {
  throw new Error("Friday web inspection did not extract readable source text.");
}

const inspectedSource = await inspectWebSource("https://example.com/friday", {
  fetcher: async (_input, init) => {
    if (init?.method !== "POST") {
      throw new Error("Friday web inspection used the wrong HTTP method.");
    }

    return Response.json({
      ok: true,
      excerpt: "Useful inspected text.",
      fetchedAt: timestamp,
      title: "Example Friday",
      url: "https://example.com/friday",
    });
  },
});

if (!inspectedSource.ok || inspectedSource.title !== "Example Friday") {
  throw new Error("Friday web inspection did not handle a valid route response.");
}

const failedInspection = await inspectWebSource("https://example.com/friday", {
  fetcher: async () => {
    throw new Error("inspection offline");
  },
});

if (failedInspection.ok || failedInspection.message !== "inspection offline") {
  throw new Error("Friday web inspection did not return a controlled fetch failure.");
}

const sampleSearchResults = parseDuckDuckGoLiteResults(`
  <html>
    <body>
      <a rel="nofollow" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Ffriday">Friday research result</a>
      <td class="result-snippet">Useful result &amp; snippet for the research topic.</td>
    </body>
  </html>
`);

if (
  sampleSearchResults[0]?.url !== "https://example.com/friday" ||
  !sampleSearchResults[0]?.snippet.includes("Useful result & snippet")
) {
  throw new Error("Friday web search parser did not extract candidate source results.");
}

const searchedSources = await searchWebSources("friday local research", {
  fetcher: async (_input, init) => {
    if (init?.method !== "POST") {
      throw new Error("Friday web search used the wrong HTTP method.");
    }

    return Response.json({
      ok: true,
      query: "friday local research",
      results: sampleSearchResults,
      searchedAt: timestamp,
    });
  },
});

if (!searchedSources.ok || searchedSources.results[0]?.title !== "Friday research result") {
  throw new Error("Friday web search did not handle a valid route response.");
}

const failedSearch = await searchWebSources("friday local research", {
  fetcher: async () => {
    throw new Error("search offline");
  },
});

if (failedSearch.ok || failedSearch.message !== "search offline") {
  throw new Error("Friday web search did not return a controlled fetch failure.");
}

const backup = buildFridayWorkspaceBackup(
  (key) =>
    key === "friday.projects.v1"
      ? JSON.stringify([{ id: "project_test", name: "Friday OS" }])
      : key === "friday.connectors.v1"
        ? JSON.stringify({ localFiles: true, webSearch: false, aiGateway: true })
        : null,
  "2026-05-14T00:00:00.000Z",
);
const parsedBackup = parseFridayWorkspaceBackup(serializeFridayWorkspaceBackup(backup));

if (!parsedBackup.ok || getFridayWorkspaceBackupEntries(parsedBackup.backup).length !== 2) {
  throw new Error("Friday workspace backup did not round-trip known local keys.");
}

const rejectedBackup = parseFridayWorkspaceBackup(
  JSON.stringify({ app: "Other", version: 1, exportedAt: timestamp, keys: {} }),
);

if (rejectedBackup.ok) {
  throw new Error("Friday workspace backup accepted a non-Friday payload.");
}

const syncPayload = validateFridayWorkspaceSyncPayload(backup);
if (!syncPayload.ok || !syncPayload.raw.includes("Friday")) {
  throw new Error("Friday workspace sync did not accept a valid backup snapshot.");
}

const storedSnapshot = parseStoredFridayWorkspaceSnapshot(syncPayload.raw);
if (!storedSnapshot.ok || JSON.stringify(storedSnapshot.payload) !== syncPayload.raw) {
  throw new Error("Friday workspace sync did not parse a stored backup snapshot.");
}

const corruptedStoredSnapshot = parseStoredFridayWorkspaceSnapshot("{bad json");
if (corruptedStoredSnapshot.ok) {
  throw new Error("Friday workspace sync accepted a corrupted stored snapshot.");
}

const rejectedSyncPayload = validateFridayWorkspaceSyncPayload({ app: "Other" });
if (rejectedSyncPayload.ok) {
  throw new Error("Friday workspace sync accepted an invalid backup snapshot.");
}

const savedAuthEnv = {
  betterAuthUrl: process.env.BETTER_AUTH_URL,
  tursoAuthToken: process.env.TURSO_AUTH_TOKEN,
  tursoDatabaseUrl: process.env.TURSO_DATABASE_URL,
};
const restoreEnv = (key: "BETTER_AUTH_URL" | "TURSO_AUTH_TOKEN" | "TURSO_DATABASE_URL", value: string | undefined) => {
  if (value === undefined) {
    delete process.env[key];
    return;
  }

  process.env[key] = value;
};

delete process.env.BETTER_AUTH_URL;
delete process.env.TURSO_AUTH_TOKEN;
delete process.env.TURSO_DATABASE_URL;

const localOnlySyncSession = await requireFridayWorkspaceSyncSession(
  new Request("http://localhost/api/friday/sync/workspace"),
);

restoreEnv("BETTER_AUTH_URL", savedAuthEnv.betterAuthUrl);
restoreEnv("TURSO_AUTH_TOKEN", savedAuthEnv.tursoAuthToken);
restoreEnv("TURSO_DATABASE_URL", savedAuthEnv.tursoDatabaseUrl);

if (localOnlySyncSession.ok || localOnlySyncSession.response.status !== 503) {
  throw new Error("Friday workspace sync did not stay controlled in local-only mode.");
}

const cloudSyncStorage = createTestStorage({
  "friday.projects.v1": JSON.stringify([{ id: "project_test", name: "Friday OS" }]),
});

const unavailableStoragePush = await pushFridayWorkspaceSnapshot({
  storage: null,
});

if (unavailableStoragePush.ok || !unavailableStoragePush.message.includes("storage is unavailable")) {
  throw new Error("Friday workspace push assumed browser storage outside the app shell.");
}

const pushedWorkspace = await pushFridayWorkspaceSnapshot({
  fetcher: async (_input, init) => {
    if (init?.method !== "PUT") {
      throw new Error("Friday workspace push used the wrong HTTP method.");
    }

    return Response.json({ ok: true, keyCount: 1, updatedAt: timestamp });
  },
  storage: cloudSyncStorage,
});

if (!pushedWorkspace.ok || pushedWorkspace.keyCount !== 1) {
  throw new Error("Friday workspace push did not report a successful upload.");
}

const failedWorkspacePush = await pushFridayWorkspaceSnapshot({
  fetcher: async () => {
    throw new Error("offline");
  },
  storage: cloudSyncStorage,
});

if (failedWorkspacePush.ok || failedWorkspacePush.message !== "offline") {
  throw new Error("Friday workspace push did not return a controlled network failure.");
}

const pulledWorkspace = await pullFridayWorkspaceSnapshot({
  fetcher: async (_input, init) => {
    if (init?.method !== "GET") {
      throw new Error("Friday workspace pull used the wrong HTTP method.");
    }

    return Response.json({
      ok: true,
      snapshot: {
        payload: backup,
        updatedAt: timestamp,
      },
    });
  },
});

if (!pulledWorkspace.ok || pulledWorkspace.keyCount !== 2 || !pulledWorkspace.payload) {
  throw new Error("Friday workspace pull did not return a valid synced backup.");
}

const failedWorkspacePull = await pullFridayWorkspaceSnapshot({
  fetcher: async () => {
    throw new Error("offline");
  },
});

if (failedWorkspacePull.ok || failedWorkspacePull.message !== "offline") {
  throw new Error("Friday workspace pull did not return a controlled network failure.");
}

const automationBase = new Date("2026-05-14T00:00:00.000Z");
if (nextScheduledAutomationRun("Hourly", automationBase) !== "2026-05-14T01:00:00.000Z") {
  throw new Error("Friday automation hourly schedule did not advance correctly.");
}

if (nextScheduledAutomationRun("Manual", automationBase) !== undefined) {
  throw new Error("Friday manual automation should not auto-schedule.");
}

if (!isAutomationDue("2026-05-13T23:59:00.000Z", automationBase.getTime())) {
  throw new Error("Friday automation due check missed an elapsed run.");
}

if (isAutomationDue("2026-05-14T00:01:00.000Z", automationBase.getTime())) {
  throw new Error("Friday automation due check ran a future task.");
}

const selectedDueAutomation = selectNextDueAutomation(
  [
    {
      cadence: "Daily",
      enabled: true,
      nextRunAt: "2026-05-13T23:50:00.000Z",
      title: "Later due",
    },
    {
      cadence: "Hourly",
      enabled: true,
      nextRunAt: "2026-05-13T23:40:00.000Z",
      title: "First due",
    },
    {
      cadence: "Manual",
      enabled: true,
      nextRunAt: "2026-05-13T23:30:00.000Z",
      title: "Manual skip",
    },
  ],
  automationBase.getTime(),
);

if (selectedDueAutomation?.title !== "First due") {
  throw new Error("Friday automation runner did not pick the earliest due scheduled task.");
}

const automationPromptText = createAutomationPrompt({
  title: "Follow up",
  cadence: "Daily",
  instruction: "Summarize open Friday work.",
});

if (!automationPromptText.includes("Instruction: Summarize open Friday work.")) {
  throw new Error("Friday automation prompt did not preserve explicit instructions.");
}

if (!createAutomationFallbackResult({ title: "Follow up" }).includes("Follow up")) {
  throw new Error("Friday automation fallback result did not include the automation title.");
}

const authConfigStatus = getFridayAuthConfigStatus();
if (
  typeof authConfigStatus.authUrlConfigured !== "boolean" ||
  typeof authConfigStatus.databaseConfigured !== "boolean" ||
  typeof authConfigStatus.tokenConfigured !== "boolean"
) {
  throw new Error("Friday auth config status did not stay import-safe and typed.");
}

console.log(`Friday local stream smoke passed with ${model.label}.`);
