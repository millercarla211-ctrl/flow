import { useLingui } from "@lingui/react/macro";
import { useEffect, useMemo } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { Key, Server } from "lucide-react";
import ToggleSwitch from "../../../shared/ui/ToggleSwitch";
import {
  CLOUD_PROVIDERS,
  getProviderPreset,
  LOCAL_PROVIDERS,
} from "../../../shared/lib/llmProviders";
import type { LlmProvider } from "../../../types";
import { Dropdown } from "../../../shared/ui/Dropdown";

type LanguageModelPanelProps = {
  llmEnabled: boolean;
  setLlmEnabled: (value: boolean) => void;
  llmProvider: LlmProvider;
  setLlmProvider: (value: LlmProvider) => void;
  llmEndpoint: string;
  setLlmEndpoint: (value: string) => void;
  llmApiKey: string;
  setLlmApiKey: (value: string) => void;
  llmModel: string;
  setLlmModel: (value: string) => void;
  availableModels: string[];
  fetchAvailableModels: () => void;
};

const LanguageModelPanel = ({
  llmEnabled,
  setLlmEnabled,
  llmProvider,
  setLlmProvider,
  llmEndpoint,
  setLlmEndpoint,
  llmApiKey,
  setLlmApiKey,
  llmModel,
  setLlmModel,
  availableModels,
  fetchAvailableModels,
}: LanguageModelPanelProps) => {
  const { t } = useLingui();
  const providerPreset = getProviderPreset(llmProvider);
  const hasSelectedProvider = Boolean(providerPreset);
  const isFlowLocalProvider = llmProvider === "local";
  const uniqueModels = useMemo(
    () => Array.from(new Set(availableModels.map((model) => model.trim()).filter(Boolean))),
    [availableModels],
  );

  useEffect(() => {
    if (llmEnabled && isFlowLocalProvider) {
      fetchAvailableModels();
    }
  }, [fetchAvailableModels, isFlowLocalProvider, llmEnabled]);

  const localModelDetails: Record<
    string,
    { label: string; description: string; badges: string[] }
  > = {
    "qwen3-0.6b": {
      label: "Qwen3 0.6B",
      description: "Instant helper: dictation cleanup, short rewrites, labels.",
      badges: ["Instant"],
    },
    "xlam2-3b-fc-r-q4km": {
      label: "xLAM 2 3B FC",
      description: "Tool router: JSON and function-call decisions.",
      badges: ["Tools"],
    },
    "qwen35-4b-revised-q4km": {
      label: "Qwen3.5 4B Revised",
      description: "Daily smart model for coding, UI edits, and useful answers.",
      badges: ["Daily"],
    },
    "qwen35-9b-q4km": {
      label: "Qwen3.5 9B",
      description: "Slow backup when the 4B model is not enough.",
      badges: ["Backup"],
    },
    "smollm2-135m-instruct-q4km": {
      label: "SmolLM2 135M",
      description: "Tiny fallback only when the Qwen helper is unavailable.",
      badges: ["Fallback"],
    },
  };

  const localPolicy = [
    {
      id: "qwen3-0.6b",
      role: "Instant helper",
      detail: "cleanup, short rewrites",
    },
    {
      id: "xlam2-3b-fc-r-q4km",
      role: "Tool router",
      detail: "JSON/function calls",
    },
    {
      id: "qwen35-4b-revised-q4km",
      role: "Daily brain",
      detail: "coding and smart chat",
    },
    {
      id: "qwen35-9b-q4km",
      role: "Slow backup",
      detail: "harder answers",
    },
  ];

  const modelOption = (model: string) => {
    const detail = localModelDetails[model];
    return {
      value: model,
      label: detail?.label ? `${detail.label} (${model})` : model,
      description: detail?.description,
      badges: detail?.badges.map((label) => ({ label, highlighted: label === "Daily" })),
    };
  };

  return (
    <div className="rounded-xl border border-border-primary bg-surface-surface shadow-[0_3px_0_-1px_rgba(0,0,0,0.5),inset_0_1px_0_0_rgba(255,255,255,0.06)]">
      <div className="p-4">
        <div className="flex items-center justify-between mb-3">
          <div>
            <h3 className="ui-text-body-strong ui-color-primary">
              {t({
                id: "settings.language_model.title",
                message: "Text Enhancement Provider",
              })}
            </h3>
            <p className="ui-text-label ui-color-disabled">
              {t({
                id: "settings.language_model.description",
                message: "Shared by cleanup, command mode, and personalization.",
              })}
            </p>
          </div>
          <ToggleSwitch
            enabled={llmEnabled}
            onToggle={() => setLlmEnabled(!llmEnabled)}
            ariaLabel={t({
              id: "settings.language_model.toggle",
              message: "Toggle smart cleanup",
            })}
            size="md"
          />
        </div>

        <AnimatePresence initial={false}>
          {llmEnabled && (
            <motion.div
              initial={{ height: 0, opacity: 0, overflow: "hidden" }}
              animate={{
                height: "auto",
                opacity: 1,
                transitionEnd: { overflow: "visible" },
              }}
              exit={{ height: 0, opacity: 0, overflow: "hidden" }}
              transition={{ type: "spring", stiffness: 400, damping: 35 }}
            >
              <div className="pt-3 border-t border-border-primary space-y-3">
                <div className="space-y-1.5">
                  <label className="ui-text-label-strong ui-color-muted ml-1">
                    {t({
                      id: "settings.language_model.provider",
                      message: "Provider",
                    })}
                  </label>
                  <Dropdown
                    value={llmProvider}
                    onChange={(val) => {
                      setLlmProvider(val);
                      const preset = getProviderPreset(val);
                      if (preset) {
                        setLlmEndpoint(preset.endpoint);
                        setLlmModel(preset.defaultModel);
                      }
                    }}
                    options={[
                      {
                        value: "custom" as LlmProvider,
                        label: t({
                          id: "settings.language_model.provider.custom",
                          message: "Custom",
                        }),
                      },
                      {
                        value: "_local_header" as LlmProvider,
                        label: t({
                          id: "settings.language_model.provider.local",
                          message: "Local",
                        }),
                        isHeader: true,
                      },
                      ...LOCAL_PROVIDERS.filter((p) => p.id !== "custom").map((p) => ({
                        value: p.id,
                        label: p.label,
                      })),
                      {
                        value: "_cloud_header" as LlmProvider,
                        label: t({
                          id: "settings.language_model.provider.cloud",
                          message: "Cloud (API Key)",
                        }),
                        isHeader: true,
                      },
                      ...CLOUD_PROVIDERS.map((p) => ({
                        value: p.id,
                        label: p.label,
                      })),
                    ]}
                    placeholder={t({
                      id: "settings.language_model.provider.select",
                      message: "Select provider...",
                    })}
                    searchable
                    searchPlaceholder={t({
                      id: "settings.language_model.provider.search",
                      message: "Search providers...",
                    })}
                  />
                </div>

                {hasSelectedProvider && (
                  <>
                    {isFlowLocalProvider && (
                      <div className="rounded-lg border border-border-primary bg-[var(--surface-interactive)] p-3">
                        <div className="ui-text-label-strong ui-color-primary">
                          Friday local routing policy
                        </div>
                        <div className="mt-1 ui-text-meta ui-color-muted">
                          The selected model stays as your fallback. Friday routes common tasks to
                          the fastest installed local model for that job.
                        </div>
                        <div className="mt-3 grid grid-cols-2 gap-2">
                          {localPolicy.map((item) => {
                            const installed = uniqueModels.includes(item.id);
                            return (
                              <div
                                key={item.id}
                                className="rounded-md border border-border-primary bg-surface-surface px-2.5 py-2"
                              >
                                <div className="flex items-center justify-between gap-2">
                                  <span className="truncate ui-text-meta-strong ui-color-primary">
                                    {item.role}
                                  </span>
                                  <span
                                    className={`h-1.5 w-1.5 rounded-full ${
                                      installed
                                        ? "bg-[var(--foreground)]"
                                        : "bg-[var(--color-text-disabled)]"
                                    }`}
                                  />
                                </div>
                                <div className="mt-0.5 truncate ui-text-micro ui-color-muted">
                                  {localModelDetails[item.id]?.label ?? item.id}
                                </div>
                                <div className="truncate ui-text-micro ui-color-disabled">
                                  {item.detail}
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      </div>
                    )}

                    {!isFlowLocalProvider && (
                      <>
                        <div className="space-y-1.5">
                          <label className="ui-text-label-strong ui-color-muted ml-1">
                            {t({
                              id: "settings.language_model.endpoint",
                              message: "Endpoint",
                            })}{" "}
                            {llmProvider !== "custom" && (
                              <span className="text-content-disabled">
                                {t({
                                  id: "settings.language_model.endpoint.autofilled",
                                  message: "(auto-filled)",
                                })}
                              </span>
                            )}
                          </label>
                          <div className="flex items-center gap-2 rounded-lg border border-border-primary bg-surface-surface px-3 py-1.5 min-h-[40px] focus-within:border-border-hover transition-colors">
                            <Server
                              size={12}
                              className="text-content-muted shrink-0"
                              aria-hidden="true"
                            />
                            <input
                              type="text"
                              value={llmEndpoint}
                              onChange={(e) => setLlmEndpoint(e.target.value)}
                              placeholder={
                                providerPreset?.endpoint ??
                                t({
                                  id: "settings.language_model.endpoint.placeholder",
                                  message: "https://your-llm-endpoint.com",
                                })
                              }
                              aria-label={t({
                                id: "settings.language_model.endpoint.aria",
                                message: "LLM Endpoint URL",
                              })}
                              className="w-full bg-transparent ui-text-input ui-color-primary placeholder-content-disabled outline-hidden"
                            />
                          </div>
                        </div>

                        <div className="space-y-1.5">
                          <label className="ui-text-label-strong ui-color-muted ml-1">
                            {t({
                              id: "settings.language_model.api_key",
                              message: "API Key",
                            })}{" "}
                            {!providerPreset?.apiKeyRequired && (
                              <span className="text-content-disabled">
                                {t({
                                  id: "settings.language_model.api_key.optional_hint",
                                  message: "(if required)",
                                })}
                              </span>
                            )}
                          </label>
                          <div className="flex items-center gap-2 rounded-lg border border-border-primary bg-surface-surface px-3 py-1.5 min-h-[40px] focus-within:border-border-hover transition-colors">
                            <Key
                              size={12}
                              className="text-content-muted shrink-0"
                              aria-hidden="true"
                            />
                            <input
                              type="password"
                              value={llmApiKey}
                              onChange={(e) => setLlmApiKey(e.target.value)}
                              placeholder={
                                providerPreset?.apiKeyRequired
                                  ? t({
                                      id: "settings.language_model.api_key.required",
                                      message: "Required",
                                    })
                                  : t({
                                      id: "settings.language_model.api_key.optional",
                                      message: "Optional",
                                    })
                              }
                              aria-label={t({
                                id: "settings.language_model.api_key.aria",
                                message: "LLM API Key",
                              })}
                              className="w-full bg-transparent ui-text-input ui-color-primary placeholder-content-disabled outline-hidden"
                            />
                          </div>
                        </div>
                      </>
                    )}

                    <div className="relative z-0">
                      <Dropdown
                        value={llmModel}
                        onChange={(val) => setLlmModel(val)}
                        onOpen={hasSelectedProvider ? fetchAvailableModels : undefined}
                        options={[
                          ...uniqueModels.map(modelOption),
                          ...(llmModel && !uniqueModels.includes(llmModel)
                            ? [modelOption(llmModel)]
                            : []),
                        ]}
                        placeholder={t({
                          id: "settings.language_model.model.placeholder",
                          message: `Model (default: ${providerPreset?.defaultModel || "none"})`,
                        })}
                        searchable
                        searchPlaceholder={t({
                          id: "settings.language_model.model.search",
                          message: "Search available models...",
                        })}
                      />
                    </div>
                  </>
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
};

export default LanguageModelPanel;
