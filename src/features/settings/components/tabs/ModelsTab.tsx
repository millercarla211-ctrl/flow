import { msg } from "@lingui/core/macro";
import { useLingui } from "@lingui/react/macro";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AnimatePresence, motion, type Variants } from "framer-motion";
import {
  AlertCircle,
  Check,
  ChevronRight,
  Download,
  Mic,
  Settings2,
  Square,
  Trash2,
  Volume2,
} from "lucide-react";
import DotMatrix from "../../../../shared/ui/DotMatrix";
import { i18n } from "../../../../i18n";
import LanguageModelPanel from "../LanguageModelPanel";
import type {
  DownloadEvent,
  LlmProvider,
  ModelInfo,
  ModelStatus,
  TtsVoiceMode,
} from "../../../../types";

type ModelCategory = "speech" | "system";

type EngineGroup = {
  id: string;
  label: string;
  description: string;
  recommended?: boolean;
  models: ModelInfo[];
};

const engineDescription = (engineId: string, engineLabel: string) => {
  if (engineId === "whisper") {
    return i18n._(
      msg({
        id: "settings.models.engine.whisper.description",
        message: "OpenAI's speech recognition with custom vocabulary support.",
      }),
    );
  }
  if (engineId === "nvidia") {
    return i18n._(
      msg({
        id: "settings.models.engine.nvidia.description",
        message:
          "NVIDIA local speech models, including Parakeet for transcription and Nemotron for live streaming.",
      }),
    );
  }
  return i18n._(
    msg({
      id: "settings.models.engine.generic.description",
      message: `${engineLabel} transcription engine.`,
    }),
  );
};

const getSizeColorVar = (sizeMb: number): string => {
  if (sizeMb < 200) return "var(--color-size-small)";
  if (sizeMb < 1000) return "var(--color-size-medium)";
  return "var(--color-size-large)";
};

const enginePriority = (engineId: string): number => {
  if (engineId === "whisper") return 0;
  if (engineId === "nvidia") return 1;
  return 2;
};

type ModelsTabProps = {
  variants: Variants;
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
  modelCatalog: ModelInfo[];
  modelStatus: Record<string, ModelStatus>;
  ttsModelCatalog: ModelInfo[];
  ttsModelStatus: Record<string, ModelStatus>;
  downloadState: Record<string, DownloadEvent>;
  localModel: string;
  setLocalModel: (value: string) => void;
  ttsEnabled: boolean;
  setTtsEnabled: (value: boolean) => void;
  ttsAutoAfterStt: boolean;
  setTtsAutoAfterStt: (value: boolean) => void;
  ttsAutoPlay: boolean;
  setTtsAutoPlay: (value: boolean) => void;
  ttsVolume: number;
  setTtsVolume: (value: number) => void;
  ttsModel: string;
  setTtsModel: (value: string) => void;
  ttsVoiceMode: TtsVoiceMode;
  setTtsVoiceMode: (value: TtsVoiceMode) => void;
  ttsSpeaker: string;
  setTtsSpeaker: (value: string) => void;
  ttsInstruction: string;
  setTtsInstruction: (value: string) => void;
  handleDownload: (modelKey: string) => void;
  handleDelete: (modelKey: string) => void;
  handleTtsDownload: (modelKey: string) => void;
  handleTtsDelete: (modelKey: string) => void;
  handleCancelDownload: (modelKey: string) => void;
  formatBytes: (bytes: number) => string;
};

const ModelsTab = ({
  variants,
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
  modelCatalog,
  modelStatus,
  ttsModelCatalog,
  ttsModelStatus,
  downloadState,
  localModel,
  setLocalModel,
  ttsEnabled,
  setTtsEnabled,
  ttsAutoAfterStt,
  setTtsAutoAfterStt,
  ttsAutoPlay,
  setTtsAutoPlay,
  ttsVolume,
  setTtsVolume,
  ttsModel,
  setTtsModel,
  ttsVoiceMode,
  setTtsVoiceMode,
  ttsSpeaker,
  setTtsSpeaker,
  ttsInstruction,
  setTtsInstruction,
  handleDownload,
  handleDelete,
  handleTtsDownload,
  handleTtsDelete,
  handleCancelDownload,
  formatBytes,
}: ModelsTabProps) => {
  const { t } = useLingui();
  const [activeCategory, setActiveCategory] = useState<ModelCategory>("speech");
  const [expandedEngine, setExpandedEngine] = useState<string | null>(null);

  const groupedMap = new Map<string, ModelInfo[]>();
  for (const model of modelCatalog) {
    const existing = groupedMap.get(model.engine_id);
    if (existing) {
      existing.push(model);
    } else {
      groupedMap.set(model.engine_id, [model]);
    }
  }

  const groupedModels: EngineGroup[] = Array.from(groupedMap.entries())
    .map(([id, models]) => {
      const label = models[0]?.engine ?? id;
      const recommended = models.some((model) =>
        model.tags.some((tag) => tag.toLowerCase() === "recommended"),
      );
      return {
        id,
        label,
        description: engineDescription(id, label),
        recommended,
        models,
      };
    })
    .sort((a, b) => {
      const priorityDelta = enginePriority(a.id) - enginePriority(b.id);
      if (priorityDelta !== 0) return priorityDelta;
      return a.label.localeCompare(b.label);
    });

  const toggleEngine = (engineId: string) => {
    setExpandedEngine((prev) => (prev === engineId ? null : engineId));
  };

  const categories: { id: ModelCategory; label: string; icon: typeof Mic }[] = [
    {
      id: "speech",
      label: t({ id: "settings.models.category.speech", message: "Speech" }),
      icon: Mic,
    },
    {
      id: "system",
      label: t({ id: "settings.models.category.system", message: "System" }),
      icon: Settings2,
    },
  ];

  return (
    <motion.div
      key="models"
      variants={variants}
      initial="hidden"
      animate="visible"
      exit="exit"
      className="space-y-5"
    >
      <header>
        <h1 className="ui-text-title-lg font-medium ui-color-primary">
          {t({
            id: "settings.models.title",
            message: "Models",
          })}
        </h1>
        <div className="mt-1 flex items-center justify-between gap-3">
          <p className="ui-text-body-sm ui-color-muted">
            {t({
              id: "settings.models.description",
              message: "Manage local speech engines, voice output, and optional remote providers.",
            })}
          </p>
          <div className="flex gap-0.5 p-0.5 rounded-md bg-[var(--color-bg-primary)] border border-border-primary shrink-0">
            {categories.map((cat) => {
              const isActive = activeCategory === cat.id;
              const Icon = cat.icon;
              return (
                <button
                  key={cat.id}
                  onClick={() => setActiveCategory(cat.id)}
                  className={`relative flex items-center justify-center gap-1.5 px-2.5 py-1 rounded-[5px] ui-text-meta transition-colors ${
                    isActive ? "ui-color-primary" : "ui-color-muted hover:text-content-primary"
                  }`}
                >
                  {isActive && (
                    <motion.div
                      layoutId="models-category-pill"
                      className="absolute inset-0 rounded-[5px] bg-surface-surface border border-border-secondary shadow-[var(--shadow-sm)]"
                      transition={{ type: "spring", stiffness: 500, damping: 35 }}
                    />
                  )}
                  <span className="relative flex items-center gap-1.5">
                    <Icon size={12} aria-hidden="true" />
                    {cat.label}
                  </span>
                </button>
              );
            })}
          </div>
        </div>
      </header>

      <AnimatePresence mode="wait" initial={false}>
        {activeCategory === "speech" && (
          <motion.div
            key="speech"
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.15 }}
            className="space-y-5"
          >
            <LanguageModelPanel
              llmEnabled={llmEnabled}
              setLlmEnabled={setLlmEnabled}
              llmProvider={llmProvider}
              setLlmProvider={setLlmProvider}
              llmEndpoint={llmEndpoint}
              setLlmEndpoint={setLlmEndpoint}
              llmApiKey={llmApiKey}
              setLlmApiKey={setLlmApiKey}
              llmModel={llmModel}
              setLlmModel={setLlmModel}
              availableModels={availableModels}
              fetchAvailableModels={fetchAvailableModels}
            />

            <VoiceOutputPanel
              ttsModels={ttsModelCatalog}
              ttsModelStatus={ttsModelStatus}
              downloadState={downloadState}
              selectedModel={ttsModel}
              setSelectedModel={setTtsModel}
              enabled={ttsEnabled}
              setEnabled={setTtsEnabled}
              autoAfterStt={ttsAutoAfterStt}
              setAutoAfterStt={setTtsAutoAfterStt}
              autoPlay={ttsAutoPlay}
              setAutoPlay={setTtsAutoPlay}
              volume={ttsVolume}
              setVolume={setTtsVolume}
              voiceMode={ttsVoiceMode}
              setVoiceMode={setTtsVoiceMode}
              speaker={ttsSpeaker}
              setSpeaker={setTtsSpeaker}
              instruction={ttsInstruction}
              setInstruction={setTtsInstruction}
              onDownload={handleTtsDownload}
              onDelete={handleTtsDelete}
              onCancel={handleCancelDownload}
              formatBytes={formatBytes}
            />

            <div>
              <h3 className="ui-text-section-label-sm ui-color-disabled mb-3">
                {t({
                  id: "settings.models.transcription_engines",
                  message: "Transcription Engines",
                })}
              </h3>
              <div className="rounded-xl border border-border-primary bg-surface-surface overflow-hidden divide-y divide-border-primary shadow-[var(--shadow-sm)]">
                {groupedModels.map((group, groupIndex) => {
                  const isExpanded = expandedEngine === group.id;
                  const installedCount = group.models.filter(
                    (m) => modelStatus[m.key]?.installed,
                  ).length;
                  const hasActiveModel = group.models.some(
                    (m) => localModel === m.key && modelStatus[m.key]?.installed,
                  );
                  const activeModel = group.models.find(
                    (m) => localModel === m.key && modelStatus[m.key]?.installed,
                  );

                  return (
                    <div key={group.id || `model-group-${groupIndex}`}>
                      <button
                        onClick={() => toggleEngine(group.id)}
                        className="w-full px-4 py-3 flex items-center gap-3 hover:bg-surface-elevated/50 transition-colors"
                        aria-expanded={isExpanded}
                      >
                        <motion.div
                          animate={{ rotate: isExpanded ? 90 : 0 }}
                          transition={{ duration: 0.15 }}
                          className="text-content-disabled"
                        >
                          <ChevronRight size={14} aria-hidden="true" />
                        </motion.div>
                        <div className="flex-1 text-left">
                          <div className="flex items-center gap-2">
                            <span className="ui-text-body-strong ui-color-primary">
                              {group.label}
                            </span>
                            {group.recommended && (
                              <span className="ui-text-meta ui-color-local">
                                {t({
                                  id: "settings.models.recommended",
                                  message: "Recommended",
                                })}
                              </span>
                            )}
                            {hasActiveModel && activeModel && (
                              <span className="ui-text-meta ui-color-muted">
                                {activeModel.label}
                              </span>
                            )}
                          </div>
                          <p className="ui-text-label ui-color-disabled">{group.description}</p>
                        </div>
                        <div className="flex items-center gap-2">
                          {hasActiveModel && (
                            <span className="flex items-center gap-1 ui-text-meta ui-color-local">
                              <Check size={12} aria-hidden="true" />
                              {t({
                                id: "settings.models.active",
                                message: "Active",
                              })}
                            </span>
                          )}
                          {!hasActiveModel && installedCount > 0 && (
                            <span className="ui-text-meta ui-color-disabled">
                              {t({
                                id: "settings.models.installed_count",
                                message: `${installedCount} installed`,
                              })}
                            </span>
                          )}
                        </div>
                      </button>

                      <AnimatePresence initial={false}>
                        {isExpanded && (
                          <motion.div
                            initial={{ height: 0 }}
                            animate={{ height: "auto" }}
                            exit={{ height: 0 }}
                            transition={{ duration: 0.2 }}
                            className="overflow-hidden bg-surface-elevated/30"
                          >
                            <div className="px-4 py-2 space-y-1">
                              {group.models.map((model, modelIndex) => (
                                <ModelRow
                                  key={model.key || `group-model-${groupIndex}-${modelIndex}`}
                                  model={model}
                                  modelStatus={modelStatus[model.key]}
                                  downloadState={downloadState[model.key]}
                                  isActive={
                                    localModel === model.key && modelStatus[model.key]?.installed
                                  }
                                  onUse={() => setLocalModel(model.key)}
                                  onDownload={() => handleDownload(model.key)}
                                  onDelete={() => handleDelete(model.key)}
                                  onCancel={() => handleCancelDownload(model.key)}
                                  formatBytes={formatBytes}
                                />
                              ))}
                            </div>
                          </motion.div>
                        )}
                      </AnimatePresence>
                    </div>
                  );
                })}
              </div>
            </div>
          </motion.div>
        )}

        {activeCategory === "system" && (
          <motion.div
            key="system"
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.15 }}
            className="space-y-5"
          >
            <div className="rounded-xl border border-border-primary bg-surface-surface overflow-hidden shadow-[var(--shadow-sm)]">
              <div className="px-5 py-8 flex flex-col items-center text-center">
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-surface-elevated border border-border-primary mb-3">
                  <Settings2 size={18} className="ui-color-disabled" aria-hidden="true" />
                </div>
                <p className="ui-text-body-strong ui-color-primary">
                  {t({
                    id: "settings.models.system.empty_title",
                    message: "No system models yet",
                  })}
                </p>
                <p className="mt-1 ui-text-body-sm ui-color-disabled max-w-[280px]">
                  {t({
                    id: "settings.models.system.empty_description",
                    message:
                      "Background processing models like speaker diarization will appear here as they become available.",
                  })}
                </p>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
};

type VoiceOutputPanelProps = {
  ttsModels: ModelInfo[];
  ttsModelStatus: Record<string, ModelStatus>;
  downloadState: Record<string, DownloadEvent>;
  selectedModel: string;
  setSelectedModel: (value: string) => void;
  enabled: boolean;
  setEnabled: (value: boolean) => void;
  autoAfterStt: boolean;
  setAutoAfterStt: (value: boolean) => void;
  autoPlay: boolean;
  setAutoPlay: (value: boolean) => void;
  volume: number;
  setVolume: (value: number) => void;
  voiceMode: TtsVoiceMode;
  setVoiceMode: (value: TtsVoiceMode) => void;
  speaker: string;
  setSpeaker: (value: string) => void;
  instruction: string;
  setInstruction: (value: string) => void;
  onDownload: (modelKey: string) => void;
  onDelete: (modelKey: string) => void;
  onCancel: (modelKey: string) => void;
  formatBytes: (bytes: number) => string;
};

type TtsTestStatus =
  | { state: "idle" }
  | { state: "running"; startedAt: number }
  | { state: "ready"; message: string; elapsedMs: number; path: string }
  | { state: "error"; message: string };

type TtsCompletePayload = {
  path: string;
  transcript: string;
  model: string;
  elapsed_ms: number;
  auto_play: boolean;
  audio_data_url?: string | null;
};

const VoiceOutputPanel = ({
  ttsModels,
  ttsModelStatus,
  downloadState,
  selectedModel,
  setSelectedModel,
  enabled,
  setEnabled,
  autoAfterStt,
  setAutoAfterStt,
  autoPlay,
  setAutoPlay,
  volume,
  setVolume,
  voiceMode,
  setVoiceMode,
  speaker,
  setSpeaker,
  instruction,
  setInstruction,
  onDownload,
  onDelete,
  onCancel,
  formatBytes,
}: VoiceOutputPanelProps) => {
  const { t } = useLingui();
  const [testText, setTestText] = useState("Hello, this is Friday speaking with Kokoro TTS.");
  const [testStatus, setTestStatus] = useState<TtsTestStatus>({ state: "idle" });
  const currentModel = ttsModels.find((model) => model.key === selectedModel);
  const currentInstalled = Boolean(selectedModel && ttsModelStatus[selectedModel]?.installed);
  const supportsSourceAudio = Boolean(
    currentModel?.capabilities.some((capability) => capability === "voice_clone"),
  );
  const supportsPreset = Boolean(
    currentModel?.capabilities.some((capability) => capability === "custom_voice"),
  );
  const installedCustomVoiceModel = ttsModels.find(
    (model) => model.capabilities.includes("custom_voice") && ttsModelStatus[model.key]?.installed,
  );

  const selectModel = (model: ModelInfo) => {
    setSelectedModel(model.key);
    if (voiceMode === "source_audio" && !model.capabilities.includes("voice_clone")) {
      setVoiceMode("preset");
    }
    if (voiceMode === "preset" && !model.capabilities.includes("custom_voice")) {
      setVoiceMode("source_audio");
    }
  };

  const runTtsTest = async () => {
    const text = testText.trim();
    if (!text) {
      setTestStatus({ state: "error", message: "Enter text to test TTS." });
      return;
    }

    setTestStatus({ state: "running", startedAt: Date.now() });
    try {
      const testModel =
        voiceMode === "source_audio" && installedCustomVoiceModel
          ? installedCustomVoiceModel.key
          : selectedModel;
      const testVoiceMode =
        voiceMode === "source_audio" && installedCustomVoiceModel ? "preset" : voiceMode;
      const result = await invoke<TtsCompletePayload>("synthesize_tts", {
        args: {
          text,
          referenceAudioPath: null,
          referenceText: null,
          model: testModel,
          voiceMode: testVoiceMode,
          speaker,
          instruction,
          autoPlay,
          volume,
        },
      });
      setTestStatus({
        state: "ready",
        message: `Generated and ${result.auto_play ? "played" : "saved"} in ${(result.elapsed_ms / 1000).toFixed(1)}s.`,
        elapsedMs: result.elapsed_ms,
        path: result.path,
      });
    } catch (error) {
      setTestStatus({ state: "error", message: String(error) });
    }
  };

  return (
    <div className="rounded-xl border border-border-primary bg-surface-surface overflow-hidden shadow-[var(--shadow-sm)]">
      <div className="px-4 py-3 border-b border-border-primary flex items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="flex h-7 w-7 items-center justify-center rounded-full bg-surface-elevated border border-border-primary">
              <Volume2 size={14} className="ui-color-muted" aria-hidden="true" />
            </span>
            <div>
              <h3 className="ui-text-body-strong ui-color-primary">
                {t({ id: "settings.models.tts.title", message: "Voice Output" })}
              </h3>
              <p className="ui-text-label ui-color-disabled">
                {t({
                  id: "settings.models.tts.description",
                  message: "Optionally turn each transcript back into speech with local TTS.",
                })}
              </p>
            </div>
          </div>
        </div>
        <button
          onClick={() => setEnabled(!enabled)}
          className={`relative h-6 w-11 rounded-full border transition-colors ${
            enabled ? "border-local-30 bg-local/20" : "border-border-primary bg-surface-elevated"
          }`}
          aria-pressed={enabled}
          aria-label={t({ id: "settings.models.tts.toggle", message: "Toggle voice output" })}
        >
          <motion.span
            animate={{ x: enabled ? 20 : 2 }}
            transition={{ type: "spring", stiffness: 500, damping: 32 }}
            className="absolute top-1 h-4 w-4 rounded-full bg-content-primary"
          />
        </button>
      </div>

      <div className="p-4 space-y-4">
        <div className="grid grid-cols-2 gap-2">
          <label className="flex items-center justify-between gap-3 rounded-lg border border-border-primary bg-surface-elevated/40 px-3 py-2">
            <span className="ui-text-body-sm ui-color-primary">
              {t({ id: "settings.models.tts.after_stt", message: "Auto after STT" })}
            </span>
            <input
              type="checkbox"
              checked={autoAfterStt}
              onChange={(event) => setAutoAfterStt(event.target.checked)}
              className="accent-[var(--color-accent)]"
            />
          </label>
          <label className="flex items-center justify-between gap-3 rounded-lg border border-border-primary bg-surface-elevated/40 px-3 py-2">
            <span className="ui-text-body-sm ui-color-primary">
              {t({ id: "settings.models.tts.auto_play", message: "Auto play" })}
            </span>
            <input
              type="checkbox"
              checked={autoPlay}
              onChange={(event) => setAutoPlay(event.target.checked)}
              className="accent-[var(--color-accent)]"
            />
          </label>
        </div>

        <label className="flex items-center justify-between gap-4 rounded-lg border border-border-primary bg-surface-elevated/40 px-3 py-2">
          <span>
            <span className="block ui-text-body-sm ui-color-primary">
              {t({ id: "settings.models.tts.volume", message: "Playback volume" })}
            </span>
            <span className="block ui-text-label ui-color-disabled">
              {t({
                id: "settings.models.tts.volume_hint",
                message: "Default is quiet so generated speech does not overpower your system.",
              })}
            </span>
          </span>
          <span className="flex min-w-[190px] items-center gap-2">
            <input
              type="range"
              min={0}
              max={100}
              step={1}
              value={Math.round(Math.min(1, Math.max(0, volume)) * 100)}
              onChange={(event) => setVolume(Number(event.target.value) / 100)}
              className="h-1 w-32 accent-[var(--color-accent)]"
            />
            <span className="w-9 text-right ui-text-label-strong ui-color-primary">
              {Math.round(Math.min(1, Math.max(0, volume)) * 100)}%
            </span>
          </span>
        </label>

        <div className="grid grid-cols-2 gap-2 rounded-lg border border-border-primary bg-surface-elevated/40 p-1">
          {[
            {
              id: "source_audio" as TtsVoiceMode,
              label: t({ id: "settings.models.tts.mode.source_audio", message: "Clone input" }),
              disabled: currentModel ? !supportsSourceAudio : false,
            },
            {
              id: "preset" as TtsVoiceMode,
              label: t({ id: "settings.models.tts.mode.preset", message: "Preset voice" }),
              disabled: currentModel ? !supportsPreset : false,
            },
          ].map((option) => {
            const active = voiceMode === option.id;
            return (
              <button
                key={option.id}
                disabled={option.disabled}
                onClick={() => setVoiceMode(option.id)}
                className={`relative rounded-md px-3 py-1.5 ui-text-button-sm transition-colors ${
                  active ? "ui-color-primary" : "ui-color-muted hover:text-content-primary"
                } ${option.disabled ? "opacity-40 cursor-not-allowed" : ""}`}
              >
                {active && (
                  <motion.span
                    layoutId="tts-voice-mode-pill"
                    className="absolute inset-0 rounded-md bg-surface-surface border border-border-secondary"
                    transition={{ type: "spring", stiffness: 500, damping: 36 }}
                  />
                )}
                <span className="relative">{option.label}</span>
              </button>
            );
          })}
        </div>

        {voiceMode === "preset" && (
          <div className="grid grid-cols-2 gap-2">
            <input
              value={speaker}
              onChange={(event) => setSpeaker(event.target.value)}
              placeholder={t({
                id: "settings.models.tts.speaker.placeholder",
                message: "Speaker, or blank for first supported",
              })}
              className="h-8 rounded-md border border-border-primary bg-bg-primary px-3 ui-text-body-sm ui-color-primary outline-none focus:border-border-secondary"
            />
            <input
              value={instruction}
              onChange={(event) => setInstruction(event.target.value)}
              placeholder={t({
                id: "settings.models.tts.instruct.placeholder",
                message: "Optional tone instruction",
              })}
              className="h-8 rounded-md border border-border-primary bg-bg-primary px-3 ui-text-body-sm ui-color-primary outline-none focus:border-border-secondary"
            />
          </div>
        )}

        {!currentInstalled && enabled && (
          <div className="rounded-lg border border-border-primary bg-surface-elevated/50 px-3 py-2">
            <p className="ui-text-label ui-color-muted">
              {t({
                id: "settings.models.tts.install_hint",
                message:
                  "Download a voice model first. Friday will use it locally for generated speech.",
              })}
            </p>
          </div>
        )}

        <div className="rounded-lg border border-border-primary bg-surface-elevated/40 p-3">
          <div className="flex items-center justify-between gap-3">
            <div>
              <p className="ui-text-body-sm-strong ui-color-primary">
                {t({ id: "settings.models.tts.test.title", message: "Test TTS" })}
              </p>
              <p className="ui-text-label ui-color-disabled">
                {voiceMode === "source_audio"
                  ? t({
                      id: "settings.models.tts.test.source_audio_hint",
                      message:
                        "Text-only tests need Preset voice. Source-audio clone runs after an STT recording.",
                    })
                  : t({
                      id: "settings.models.tts.test.preset_hint",
                      message: "Generates a short WAV and auto-plays it if Auto play is on.",
                    })}
              </p>
            </div>
            <button
              type="button"
              onClick={runTtsTest}
              disabled={!currentInstalled || testStatus.state === "running"}
              className="h-8 rounded-md border border-border-primary bg-bg-primary px-3 ui-text-button-sm ui-color-primary disabled:opacity-45 disabled:cursor-not-allowed"
            >
              {testStatus.state === "running"
                ? t({ id: "settings.models.tts.test.running", message: "Generating..." })
                : t({ id: "settings.models.tts.test.button", message: "Play Test" })}
            </button>
          </div>
          <textarea
            value={testText}
            onChange={(event) => setTestText(event.target.value)}
            rows={2}
            className="mt-3 w-full resize-none rounded-md border border-border-primary bg-bg-primary px-3 py-2 ui-text-body-sm ui-color-primary outline-none focus:border-border-secondary"
          />
          {testStatus.state !== "idle" && (
            <p
              className={`mt-2 ui-text-label ${
                testStatus.state === "error" ? "ui-color-error" : "ui-color-muted"
              }`}
            >
              {testStatus.state === "running"
                ? "Loading the local voice engine and generating audio. Kokoro usually finishes in a few seconds after startup."
                : testStatus.message}
            </p>
          )}
          {testStatus.state === "ready" && (
            <p className="mt-1 ui-text-micro ui-color-disabled truncate">{testStatus.path}</p>
          )}
        </div>

        <div className="space-y-1">
          {ttsModels.map((model) => (
            <ModelRow
              key={model.key}
              model={model}
              modelStatus={ttsModelStatus[model.key]}
              downloadState={downloadState[model.key]}
              isActive={selectedModel === model.key && ttsModelStatus[model.key]?.installed}
              onUse={() => selectModel(model)}
              onDownload={() => onDownload(model.key)}
              onDelete={() => onDelete(model.key)}
              onCancel={() => onCancel(model.key)}
              formatBytes={formatBytes}
            />
          ))}
        </div>
      </div>
    </div>
  );
};

type ModelRowProps = {
  model: ModelInfo;
  modelStatus?: ModelStatus;
  downloadState?: DownloadEvent;
  isActive: boolean;
  onUse: () => void;
  onDownload: () => void;
  onDelete: () => void;
  onCancel: () => void;
  formatBytes: (bytes: number) => string;
};

const ModelRow = ({
  model,
  modelStatus: status,
  downloadState: progress,
  isActive,
  onUse,
  onDownload,
  onDelete,
  onCancel,
  formatBytes,
}: ModelRowProps) => {
  const { t } = useLingui();
  const installed = status?.installed;
  const isDownloading = progress?.status === "downloading";
  const isCancelled = progress?.status === "cancelled";
  const showError = progress?.status === "error";
  const percent = progress?.percent ?? (installed ? 100 : 0);
  const isRecommended = model.tags.some((t) => t.toLowerCase() === "recommended");
  const visibleTags = model.tags.filter((tag) => tag.toLowerCase() !== "recommended");

  return (
    <div className="group rounded-lg px-3 py-2.5 transition-colors hover:bg-surface-elevated/50">
      <div className="flex items-center gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="ui-text-body-sm-strong ui-color-primary">{model.label}</span>
            {isRecommended && (
              <span className="ui-text-meta ui-color-local">
                {t({
                  id: "settings.models.recommended",
                  message: "Recommended",
                })}
              </span>
            )}
            {isActive && (
              <span className="flex items-center gap-1 ui-text-meta ui-color-local">
                <Check size={10} aria-hidden="true" />
                {t({
                  id: "settings.models.active",
                  message: "Active",
                })}
              </span>
            )}
          </div>
          <div className="flex items-center gap-1.5 mt-0.5">
            <span
              className="ui-text-meta whitespace-nowrap tabular-nums"
              style={{ color: getSizeColorVar(model.size_mb) }}
            >
              {formatBytes(model.size_mb * 1024 * 1024)}
            </span>
            {visibleTags.length > 0 && (
              <>
                <span className="ui-text-meta ui-color-disabled shrink-0">·</span>
                <span className="ui-text-meta ui-color-muted truncate">
                  {visibleTags.join(", ")}
                </span>
              </>
            )}
          </div>
        </div>

        {(isDownloading || showError || isCancelled) && (
          <div className="flex flex-col items-end justify-center mr-2 min-w-[160px]">
            <ModelProgress percent={percent} status={progress?.status ?? "idle"} />
            <div className="mt-1 flex h-3 w-full items-center justify-end">
              {isDownloading && (
                <p className="ui-text-micro ui-color-disabled tabular-nums truncate max-w-[150px] text-right">
                  {progress?.percent?.toFixed(0)}% ·{" "}
                  {(progress as Extract<DownloadEvent, { status: "downloading" }>).file}
                </p>
              )}
              {showError && (
                <p className="ui-text-micro ui-color-error flex items-center justify-end gap-1 w-full">
                  <AlertCircle size={9} className="shrink-0" />
                  <span className="truncate">
                    {(progress as Extract<DownloadEvent, { status: "error" }>).message}
                  </span>
                </p>
              )}
              {isCancelled && (
                <p className="ui-text-micro ui-color-disabled text-right w-full">
                  {t({
                    id: "settings.models.cancelled",
                    message: "Cancelled",
                  })}
                </p>
              )}
            </div>
          </div>
        )}

        <div className="flex items-center gap-2 shrink-0">
          {installed && !isActive && (
            <button
              onClick={onUse}
              className="px-2.5 py-1 rounded-md border border-border-primary bg-surface-surface ui-text-button-sm ui-color-secondary hover:border-local-30 hover:bg-local-5 hover:text-local transition-colors"
            >
              {t({
                id: "settings.models.use",
                message: "Use",
              })}
            </button>
          )}
          {isDownloading ? (
            <button
              onClick={onCancel}
              className="flex h-6 w-6 items-center justify-center rounded-md text-error hover:bg-error/10 transition-colors"
              title={t({
                id: "settings.models.cancel",
                message: "Cancel",
              })}
              aria-label={t({
                id: "settings.models.cancel_download",
                message: "Cancel download",
              })}
            >
              <Square size={10} fill="currentColor" aria-hidden="true" />
            </button>
          ) : installed ? (
            <button
              onClick={onDelete}
              className="flex h-6 w-6 items-center justify-center rounded-md text-content-disabled hover:text-error hover:bg-error/10 transition-colors"
              title={t({
                id: "settings.models.delete",
                message: "Delete",
              })}
              aria-label={t({
                id: "settings.models.delete_model",
                message: "Delete model",
              })}
            >
              <Trash2 size={12} aria-hidden="true" />
            </button>
          ) : (
            <button
              onClick={onDownload}
              disabled={isCancelled}
              className={`flex h-6 w-6 items-center justify-center rounded-md transition-colors ${
                isCancelled
                  ? "text-content-disabled cursor-default"
                  : "text-content-muted hover:text-content-primary hover:bg-surface-elevated"
              }`}
              title={t({
                id: "settings.models.download",
                message: "Download",
              })}
              aria-label={t({
                id: "settings.models.download_model",
                message: "Download model",
              })}
            >
              <Download size={12} aria-hidden="true" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

type ModelProgressProps = {
  percent: number;
  status: string;
};

const ModelProgress = ({ percent, status }: ModelProgressProps) => {
  const cols = 40;
  const rows = 2;
  const totalDots = cols * rows;
  const activeCount = Math.round((percent / 100) * totalDots);

  const activeDots = Array.from({ length: Math.min(activeCount, totalDots) }, (_, i) => i);

  const color =
    status === "error"
      ? "var(--color-error)"
      : status === "complete"
        ? "var(--color-success)"
        : "var(--color-accent)";

  return (
    <DotMatrix
      rows={rows}
      cols={cols}
      activeDots={activeDots}
      dotSize={2}
      gap={2}
      color={color}
      className={status === "downloading" ? "opacity-80" : "opacity-60"}
      morphOnActive={true}
      activeScale={1.0}
    />
  );
};

export default ModelsTab;
