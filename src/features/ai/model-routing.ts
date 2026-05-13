export type FridayModelProvider = "local" | "gateway";

export type FridayModelRole =
  | "instant-helper"
  | "tool-router"
  | "daily-assistant"
  | "deep-backup"
  | "cloud-frontier";

export type FridayModelOption = {
  key: string;
  label: string;
  provider: FridayModelProvider;
  role: FridayModelRole;
  speedLabel: string;
  privacyLabel: string;
  description: string;
  gatewayModel?: string;
  disabledReason?: string;
};

export const DEFAULT_FRIDAY_MODEL_KEY = "qwen35-4b-revised-q4km";

export const FRIDAY_LOCAL_MODELS: FridayModelOption[] = [
  {
    key: "qwen3-0.6b",
    label: "Qwen3 0.6B",
    provider: "local",
    role: "instant-helper",
    speedLabel: "Fastest",
    privacyLabel: "Local",
    description: "Instant cleanup, labels, short rewrites, and low-latency helper tasks.",
  },
  {
    key: "xlam2-3b-fc-r-q4km",
    label: "xLAM 2 3B FC",
    provider: "local",
    role: "tool-router",
    speedLabel: "Fast",
    privacyLabel: "Local research",
    description: "Structured tool routing and JSON function-call decisions.",
  },
  {
    key: DEFAULT_FRIDAY_MODEL_KEY,
    label: "Qwen3.5 4B Revised",
    provider: "local",
    role: "daily-assistant",
    speedLabel: "Balanced",
    privacyLabel: "Local",
    description: "Default Friday assistant model for coding, UI work, writing, and everyday help.",
  },
  {
    key: "qwen35-9b-q4km",
    label: "Qwen3.5 9B",
    provider: "local",
    role: "deep-backup",
    speedLabel: "Slow",
    privacyLabel: "Local",
    description: "A slower backup for harder reasoning when latency is acceptable.",
  },
];

export const FRIDAY_GATEWAY_MODELS: FridayModelOption[] = [
  {
    key: "gateway-openai-gpt-5-4",
    label: "OpenAI GPT-5.4",
    provider: "gateway",
    role: "cloud-frontier",
    speedLabel: "Provider",
    privacyLabel: "Cloud optional",
    description: "Frontier assistant mode through Vercel AI Gateway when cloud mode is enabled.",
    gatewayModel: "openai/gpt-5.4",
    disabledReason: "Cloud providers are disabled while Friday is in local-only mode.",
  },
  {
    key: "gateway-anthropic-claude-sonnet-4-6",
    label: "Claude Sonnet 4.6",
    provider: "gateway",
    role: "cloud-frontier",
    speedLabel: "Provider",
    privacyLabel: "Cloud optional",
    description: "Frontier coding and artifact mode through Vercel AI Gateway when configured.",
    gatewayModel: "anthropic/claude-sonnet-4.6",
    disabledReason: "Cloud providers are disabled while Friday is in local-only mode.",
  },
];

export const FRIDAY_MODEL_OPTIONS = [...FRIDAY_LOCAL_MODELS, ...FRIDAY_GATEWAY_MODELS];

export function resolveFridayModel(modelKey?: string | null): FridayModelOption {
  return (
    FRIDAY_MODEL_OPTIONS.find((model) => model.key === modelKey) ??
    FRIDAY_LOCAL_MODELS.find((model) => model.key === DEFAULT_FRIDAY_MODEL_KEY) ??
    FRIDAY_LOCAL_MODELS[0]
  );
}

export function isGatewayModelAvailable(model: FridayModelOption): boolean {
  if (model.provider !== "gateway") return true;
  return process.env.NEXT_PUBLIC_FRIDAY_ENABLE_CLOUD_AI === "true";
}
