import { msg } from "@lingui/core/macro";
import {
  useState,
  useEffect,
  useRef,
  useMemo,
  useCallback,
  type Dispatch,
  type SetStateAction,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, emit, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  checkMacAccessibilityPermission,
  checkMacInputMonitoringPermission,
} from "../../shared/lib/macosPermissions";
import { getPlatformCapabilities } from "../../platform/service";
import { getProviderPreset } from "../../shared/lib/llmProviders";
import { parseTextSizeMode } from "../../shared/lib/textSize";
import { useModelDownloadEvents } from "../../shared/hooks/useModelDownloadEvents";
import { logout, type User as AuthUser } from "../auth/api";
import {
  buildTranscriptionLanguageView,
  getActiveTranscriptionEngine,
  getCatalogTranscriptionEngines,
  getInstalledTranscriptionEngines,
  type TranscriptionEngineId,
} from "../../shared/lib/transcriptionLanguages";
import { useShortcutCapture } from "../../shared/hooks/useShortcutCapture";
import { i18n } from "../../i18n";
import { useAppInfo, useInputDevices, useSettings } from "./queries";
import { useModelCatalog, useTtsModelCatalog } from "./models-queries";
import type {
  TranscriptionMode,
  TextSizeMode,
  ThemeMode,
  StoredSettings,
  ModelStatus,
  DownloadEvent,
  LlmProvider,
  RecordingPrunePolicy,
  LocalDataStoragePolicy,
  AppLocaleSetting,
  TtsVoiceMode,
  WakeSpeakerProfile,
} from "../../types";

const TEXT_SIZE_MODE_STORAGE_KEY = "flow_text_size_mode";

type ActiveTab = "general" | "models" | "about" | "account" | "app" | "vibe";
type ShortcutCaptureMode = "smart" | "hold" | "toggle" | "command" | "paste-last" | "cancel";
type ShortcutCaptureTarget = `${ShortcutCaptureMode}:${number}`;

const MAX_SHORTCUTS_PER_ACTION = 4;
const DEFAULT_TTS_VOLUME = 0.1;

function normalizeTtsVolume(value: number | null | undefined) {
  return Number.isFinite(value) ? Math.min(1, Math.max(0, Number(value))) : DEFAULT_TTS_VOLUME;
}

function normalizeShortcutList(shortcuts: string[] | undefined, fallback: string) {
  const seen = new Set<string>();
  const values = [...(shortcuts ?? []), fallback]
    .map((shortcut) => shortcut.trim())
    .filter(Boolean)
    .filter((shortcut) => {
      const key = shortcut.toLowerCase();
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    })
    .slice(0, MAX_SHORTCUTS_PER_ACTION);

  return values.length > 0 ? values : [fallback];
}

function captureTarget(mode: ShortcutCaptureMode, index: number): ShortcutCaptureTarget {
  return `${mode}:${index}` as ShortcutCaptureTarget;
}

function parseCaptureTarget(target: ShortcutCaptureTarget | null) {
  if (!target) return null;
  const [mode, index] = target.split(":") as [ShortcutCaptureMode, string];
  return { mode, index: Number(index) };
}

interface UseSettingsFormOptions {
  isOpen: boolean;
  onClose: () => void;
  initialTab?: ActiveTab;
  currentUser: AuthUser | null;
  onUpdateUser: () => Promise<void>;
  transcriptionMode: TranscriptionMode;
}

export function useSettingsForm({
  isOpen,
  onClose,
  initialTab = "general",
  currentUser,
  onUpdateUser,
  transcriptionMode: initialTranscriptionMode,
}: UseSettingsFormOptions) {
  const [smartShortcut, setSmartShortcut] = useState("Control+Space");
  const [smartShortcuts, setSmartShortcuts] = useState(["Control+Space", "Win+Alt+1"]);
  const [smartEnabled, setSmartEnabled] = useState(true);
  const [holdShortcut, setHoldShortcut] = useState("Control+Shift+Space");
  const [holdShortcuts, setHoldShortcuts] = useState(["Control+Shift+Space"]);
  const [holdEnabled, setHoldEnabled] = useState(false);
  const [toggleShortcut, setToggleShortcut] = useState("Control+Alt+Space");
  const [toggleShortcuts, setToggleShortcuts] = useState(["Control+Alt+Space"]);
  const [toggleEnabled, setToggleEnabled] = useState(false);
  const [commandShortcut, setCommandShortcut] = useState("Control+Alt+E");
  const [commandShortcuts, setCommandShortcuts] = useState(["Control+Alt+E"]);
  const [commandEnabled, setCommandEnabled] = useState(false);
  const [pasteLastTranscriptShortcut, setPasteLastTranscriptShortcut] = useState("Shift+Alt+Z");
  const [pasteLastTranscriptShortcuts, setPasteLastTranscriptShortcuts] = useState(["Shift+Alt+Z"]);
  const [pasteLastTranscriptEnabled, setPasteLastTranscriptEnabled] = useState(true);
  const [cancelShortcut, setCancelShortcut] = useState("Control+Alt+Escape");
  const [cancelShortcuts, setCancelShortcuts] = useState(["Control+Alt+Escape"]);
  const [cancelEnabled, setCancelEnabled] = useState(false);
  const [wakeListeningEnabled, setWakeListeningEnabled] = useState(false);
  const [wakePhrases, setWakePhrases] = useState(["hello"]);
  const [wakeSpeakerVerificationEnabled, setWakeSpeakerVerificationEnabled] = useState(false);
  const [wakeSpeakerProfile, setWakeSpeakerProfile] = useState<WakeSpeakerProfile | null>(null);
  const [transcriptionMode, setTranscriptionModeRaw] =
    useState<TranscriptionMode>(initialTranscriptionMode);
  const [localModel, setLocalModel] = useState("");
  const [ttsEnabled, setTtsEnabled] = useState(true);
  const [ttsAutoAfterStt, setTtsAutoAfterStt] = useState(true);
  const [ttsAutoPlay, setTtsAutoPlay] = useState(true);
  const [ttsVolume, setTtsVolume] = useState(DEFAULT_TTS_VOLUME);
  const [ttsModel, setTtsModel] = useState("kokoro_82m");
  const [ttsVoiceMode, setTtsVoiceMode] = useState<TtsVoiceMode>("preset");
  const [ttsSpeaker, setTtsSpeaker] = useState("");
  const [ttsInstruction, setTtsInstruction] = useState("");
  const [microphoneDevice, setMicrophoneDevice] = useState<string | null>(null);
  const [language, setLanguage] = useState("en");
  const [appLocale, setAppLocale] = useState<AppLocaleSetting>("system");
  const [modelStatus, setModelStatus] = useState<Record<string, ModelStatus>>({});
  const [ttsModelStatus, setTtsModelStatus] = useState<Record<string, ModelStatus>>({});
  const [downloadState, setDownloadState] = useState<Record<string, DownloadEvent>>({});
  const [error, setError] = useState<string | null>(null);
  const [errorCopied, setErrorCopied] = useState(false);
  const [captureActive, setCaptureActive] = useState<ShortcutCaptureTarget | null>(null);
  const [capturePreview, setCapturePreview] = useState<string>("");
  const captureActiveRef = useRef<ShortcutCaptureTarget | null>(null);
  const [activeTab, setActiveTab] = useState<ActiveTab>("general");
  const [llmEnabled, setLlmEnabledRaw] = useState(false);
  const [cleanupEnabled, setCleanupEnabled] = useState(false);
  const [llmProvider, setLlmProviderRaw] = useState<LlmProvider>("none");
  const [llmEndpoint, setLlmEndpointRaw] = useState("");
  const [llmApiKey, setLlmApiKeyRaw] = useState("");
  const [llmModel, setLlmModel] = useState("");
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [editModeEnabled, setEditModeEnabled] = useState(false);
  const [autoTransformEnabled, setAutoTransformEnabled] = useState(false);
  const [autoTransformPresetId, setAutoTransformPresetId] = useState("polish");
  const [vibeCodingEnabled, setVibeCodingEnabled] = useState(true);
  const [vibeCodingVariableRecognition, setVibeCodingVariableRecognition] = useState(true);
  const [vibeCodingFileTagging, setVibeCodingFileTagging] = useState(true);
  const [vibeCodingIncludeWindowContext, setVibeCodingIncludeWindowContext] = useState(true);
  const [mediaControlEnabled, setMediaControlEnabled] = useState(false);
  const [autoUpdateEnabled, setAutoUpdateEnabled] = useState(false);
  const [autoLaunchEnabled, setAutoLaunchEnabled] = useState(false);
  const [recordingPrunePolicy, setRecordingPrunePolicy] = useState<RecordingPrunePolicy>("never");
  const [localDataStoragePolicy, setLocalDataStoragePolicy] =
    useState<LocalDataStoragePolicy>("store");
  const [contextAwarenessEnabled, setContextAwarenessEnabled] = useState(true);
  const [analyticsEnabled, setAnalyticsEnabled] = useState(true);
  const [textSizeMode, setTextSizeModeRaw] = useState<TextSizeMode>(() =>
    parseTextSizeMode(localStorage.getItem(TEXT_SIZE_MODE_STORAGE_KEY)),
  );
  const [themeMode, setThemeModeRaw] = useState<ThemeMode>("system");
  const [authLoading, setAuthLoading] = useState(false);
  const [showFAQModal, setShowFAQModal] = useState(false);
  const [micPermission, setMicPermission] = useState<boolean | null>(null);
  const [accessibilityPermission, setAccessibilityPermission] = useState<boolean | null>(null);
  const [inputMonitoringPermission, setInputMonitoringPermission] = useState<boolean | null>(null);
  const [whatsNewOpen, setWhatsNewOpen] = useState(false);
  const didHydrateRef = useRef(false);
  const settingsQuery = useSettings(undefined, isOpen);
  const appInfoQuery = useAppInfo(isOpen);
  const inputDevicesQuery = useInputDevices(isOpen);
  const modelCatalogQuery = useModelCatalog(isOpen);
  const ttsModelCatalogQuery = useTtsModelCatalog(isOpen);
  const inputDevices = inputDevicesQuery.data ?? [];
  const modelCatalog = modelCatalogQuery.data ?? [];
  const ttsModelCatalog = ttsModelCatalogQuery.data ?? [];
  const appInfo = appInfoQuery.data ?? null;
  const platformCapabilities = useMemo(() => getPlatformCapabilities(), []);
  const loading =
    isOpen &&
    (settingsQuery.isLoading ||
      modelCatalogQuery.isLoading ||
      ttsModelCatalogQuery.isLoading ||
      inputDevicesQuery.isLoading ||
      appInfoQuery.isLoading);

  const [cloudSyncEnabled, setCloudSyncEnabledRaw] = useState(() => {
    const stored = localStorage.getItem("flow_cloud_sync_enabled");
    return stored !== null ? stored === "true" : false;
  });

  const setLlmEnabled = useCallback((value: boolean) => {
    setLlmEnabledRaw(value);
    if (!value) {
      setCleanupEnabled(false);
      setEditModeEnabled(false);
      setAutoTransformEnabled(false);
      setCommandEnabled(false);
      setError(null);
    }
  }, []);

  const setTranscriptionMode = useCallback(
    (mode: TranscriptionMode) => {
      setTranscriptionModeRaw(mode);
      if (mode === "cloud" && activeTab === "models") {
        setActiveTab("general");
      }
    },
    [activeTab],
  );

  const setCloudSyncEnabled = useCallback((value: boolean) => {
    setCloudSyncEnabledRaw(value);
    localStorage.setItem("flow_cloud_sync_enabled", String(value));
    emit("auth:changed").catch(() => {});
  }, []);

  const setTextSizeMode = useCallback((mode: TextSizeMode) => {
    setTextSizeModeRaw(mode);
    localStorage.setItem(TEXT_SIZE_MODE_STORAGE_KEY, mode);
    emit("ui:text_size_changed", { mode }).catch(() => {});
  }, []);

  const setThemeMode = useCallback((mode: ThemeMode) => {
    setThemeModeRaw(mode);
    emit("ui:theme_changed", { mode }).catch(() => {});
  }, []);

  const setLlmProvider = useCallback((value: LlmProvider) => {
    setLlmProviderRaw(value);
    setAvailableModels([]);
  }, []);
  const setLlmEndpoint = useCallback((value: string) => {
    setLlmEndpointRaw(value);
    setAvailableModels([]);
  }, []);
  const setLlmApiKey = useCallback((value: string) => {
    setLlmApiKeyRaw(value);
    setAvailableModels([]);
  }, []);

  const hydrateFromSettings = useCallback((s: StoredSettings) => {
    const nextSmartShortcuts = normalizeShortcutList(s.smart_shortcuts, s.smart_shortcut);
    const nextHoldShortcuts = normalizeShortcutList(s.hold_shortcuts, s.hold_shortcut);
    const nextToggleShortcuts = normalizeShortcutList(s.toggle_shortcuts, s.toggle_shortcut);
    const nextCommandShortcuts = normalizeShortcutList(
      s.command_shortcuts,
      s.command_shortcut ?? "Control+Alt+E",
    );
    const nextPasteLastTranscriptShortcuts = normalizeShortcutList(
      s.paste_last_transcript_shortcuts,
      s.paste_last_transcript_shortcut ?? "Shift+Alt+Z",
    );
    const nextCancelShortcuts = normalizeShortcutList(
      s.cancel_shortcuts,
      s.cancel_shortcut ?? "Control+Alt+Escape",
    );

    setSmartShortcuts(nextSmartShortcuts);
    setSmartShortcut(nextSmartShortcuts[0]);
    setSmartEnabled(s.smart_enabled);
    setHoldShortcuts(nextHoldShortcuts);
    setHoldShortcut(nextHoldShortcuts[0]);
    setHoldEnabled(s.hold_enabled);
    setToggleShortcuts(nextToggleShortcuts);
    setToggleShortcut(nextToggleShortcuts[0]);
    setToggleEnabled(s.toggle_enabled);
    setCommandShortcuts(nextCommandShortcuts);
    setCommandShortcut(nextCommandShortcuts[0]);
    setCommandEnabled(s.command_enabled ?? false);
    setPasteLastTranscriptShortcuts(nextPasteLastTranscriptShortcuts);
    setPasteLastTranscriptShortcut(nextPasteLastTranscriptShortcuts[0]);
    setPasteLastTranscriptEnabled(s.paste_last_transcript_enabled ?? true);
    setCancelShortcuts(nextCancelShortcuts);
    setCancelShortcut(nextCancelShortcuts[0]);
    setCancelEnabled(s.cancel_enabled ?? false);
    setWakeListeningEnabled(s.wake_listening_enabled ?? false);
    setWakePhrases(s.wake_phrases?.length ? s.wake_phrases : ["hello"]);
    setWakeSpeakerVerificationEnabled(s.wake_speaker_verification_enabled ?? false);
    setWakeSpeakerProfile(s.wake_speaker_profile ?? null);
    setTranscriptionModeRaw(s.transcription_mode);
    setLocalModel(s.local_model);
    setTtsEnabled(s.tts_enabled ?? true);
    setTtsAutoAfterStt(s.tts_auto_after_stt ?? true);
    setTtsAutoPlay(s.tts_auto_play ?? true);
    setTtsVolume(normalizeTtsVolume(s.tts_volume));
    setTtsModel(s.tts_model ?? "kokoro_82m");
    setTtsVoiceMode(s.tts_voice_mode ?? "preset");
    setTtsSpeaker(s.tts_speaker ?? "");
    setTtsInstruction(s.tts_instruction ?? "");
    setMicrophoneDevice(s.microphone_device);
    setLanguage(s.language);
    setAppLocale(s.app_locale ?? "system");

    setLlmEnabledRaw(s.llm_enabled ?? false);
    setCleanupEnabled(false);
    setLlmProviderRaw(s.llm_provider ?? "none");
    setLlmEndpointRaw(s.llm_endpoint ?? "");
    setLlmApiKeyRaw(s.llm_api_key ?? "");
    setLlmModel(s.llm_model ?? "");
    setEditModeEnabled(s.edit_mode_enabled ?? false);
    setAutoTransformEnabled(false);
    setAutoTransformPresetId(s.auto_transform_preset_id ?? "polish");
    setVibeCodingEnabled(s.vibe_coding_enabled ?? true);
    setVibeCodingVariableRecognition(s.vibe_coding_variable_recognition ?? true);
    setVibeCodingFileTagging(s.vibe_coding_file_tagging ?? true);
    setVibeCodingIncludeWindowContext(s.vibe_coding_include_window_context ?? true);
    setMediaControlEnabled(s.media_control_enabled ?? false);
    setAutoUpdateEnabled(s.auto_update_enabled ?? false);
    setAutoLaunchEnabled(s.auto_launch_enabled ?? false);
    setRecordingPrunePolicy(s.recording_prune_policy ?? "never");
    setLocalDataStoragePolicy(s.local_data_storage_policy ?? "store");
    setContextAwarenessEnabled(s.context_awareness_enabled ?? true);
    setAnalyticsEnabled(s.analytics_enabled ?? true);
    setThemeModeRaw(s.theme_mode ?? "system");
  }, []);

  const isSubscriber = currentUser?.labels?.includes("cloud") ?? false;

  const activeTranscriptionEngine = useMemo(
    () => getActiveTranscriptionEngine(modelCatalog, localModel),
    [modelCatalog, localModel],
  );
  const installedTranscriptionEngines = useMemo(
    () => getInstalledTranscriptionEngines(modelCatalog, modelStatus),
    [modelCatalog, modelStatus],
  );
  const catalogTranscriptionEngines = useMemo(
    () => getCatalogTranscriptionEngines(modelCatalog),
    [modelCatalog],
  );
  const visibleTranscriptionEngines: TranscriptionEngineId[] = useMemo(() => {
    if (installedTranscriptionEngines.length > 0) return installedTranscriptionEngines;
    if (activeTranscriptionEngine) return [activeTranscriptionEngine];
    if (catalogTranscriptionEngines.length > 0) return [catalogTranscriptionEngines[0]];
    return [];
  }, [installedTranscriptionEngines, activeTranscriptionEngine, catalogTranscriptionEngines]);
  const showLanguageSupportBadges = installedTranscriptionEngines.length > 1;
  const autoTranscriptionLanguageLabel = i18n._(
    msg({
      id: "transcription.language.auto",
      message: "Auto",
    }),
  );
  const languageView = useMemo(
    () =>
      buildTranscriptionLanguageView(
        modelCatalog,
        activeTranscriptionEngine,
        visibleTranscriptionEngines,
        autoTranscriptionLanguageLabel,
      ),
    [
      modelCatalog,
      activeTranscriptionEngine,
      visibleTranscriptionEngines,
      autoTranscriptionLanguageLabel,
    ],
  );
  const displayedLanguage = language;
  const displayedLanguageOptions = languageView.options;

  const llmProviderPreset = useMemo(() => getProviderPreset(llmProvider), [llmProvider]);
  const llmConfigReady = Boolean(
    llmProviderPreset &&
    (llmProvider !== "custom" || llmEndpoint.trim()) &&
    (!llmProviderPreset.apiKeyRequired || llmApiKey.trim()) &&
    llmModel.trim(),
  );
  const aiFeaturesReady = llmEnabled && llmConfigReady;

  const finalizeCapture = useCallback(() => {
    invoke("set_shortcut_capture_active", { active: false }).catch(() => {});
    captureActiveRef.current = null;
    setCaptureActive(null);
  }, []);

  const updateShortcutList = useCallback(
    (mode: ShortcutCaptureMode, index: number, shortcut: string) => {
      const update = (
        setList: Dispatch<SetStateAction<string[]>>,
        setPrimary: (value: string) => void,
      ) => {
        setList((current) => {
          const next = [...current];
          next[index] = shortcut;
          const normalized = normalizeShortcutList(next, shortcut);
          setPrimary(normalized[0]);
          return normalized;
        });
      };

      if (mode === "smart") update(setSmartShortcuts, setSmartShortcut);
      if (mode === "hold") update(setHoldShortcuts, setHoldShortcut);
      if (mode === "toggle") update(setToggleShortcuts, setToggleShortcut);
      if (mode === "command") update(setCommandShortcuts, setCommandShortcut);
      if (mode === "paste-last") {
        update(setPasteLastTranscriptShortcuts, setPasteLastTranscriptShortcut);
      }
      if (mode === "cancel") update(setCancelShortcuts, setCancelShortcut);
    },
    [],
  );

  const { resetCaptureState } = useShortcutCapture({
    active: captureActive !== null,
    onCancel: finalizeCapture,
    onPreviewChange: setCapturePreview,
    onShortcutCaptured: (combo) => {
      const target = parseCaptureTarget(captureActive);
      if (target) {
        updateShortcutList(target.mode, target.index, combo);
      }
      setError(null);
    },
    onError: setError,
    onCaptureInput: () => setError(null),
  });

  // Guard: non-subscribers can't have cloud sync
  useEffect(() => {
    if (currentUser && !isSubscriber && cloudSyncEnabled) {
      setCloudSyncEnabled(false);
    }
  }, [currentUser, isSubscriber, cloudSyncEnabled, setCloudSyncEnabled]);

  useEffect(() => {
    if (aiFeaturesReady) return;
    setCleanupEnabled(false);
    setEditModeEnabled(false);
    setAutoTransformEnabled(false);
    setCommandEnabled(false);
  }, [aiFeaturesReady]);

  useEffect(() => {
    if (isOpen && initialTab) {
      setActiveTab(initialTab);
    }
  }, [isOpen, initialTab]);

  useEffect(() => {
    if (!isOpen) {
      didHydrateRef.current = false;
      if (captureActive) {
        finalizeCapture();
        resetCaptureState();
      }
    }
  }, [captureActive, finalizeCapture, isOpen, resetCaptureState]);

  useEffect(() => {
    return () => {
      invoke("set_shortcut_capture_active", { active: false }).catch(() => {});
    };
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("open_whats_new", () => {
      setWhatsNewOpen(true);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  const refreshPermissionState = useCallback(async () => {
    const [nativeMic, acc, inputMonitoring] = await Promise.allSettled([
      platformCapabilities.requiresNativeMicrophonePermission
        ? invoke<boolean>("check_microphone_permission")
        : Promise.resolve<boolean | null>(null),
      platformCapabilities.requiresAccessibilityPermission
        ? checkMacAccessibilityPermission()
        : Promise.resolve<boolean | null>(null),
      platformCapabilities.requiresInputMonitoringPermission
        ? checkMacInputMonitoringPermission()
        : Promise.resolve<boolean | null>(null),
    ]);

    setMicPermission(nativeMic.status === "fulfilled" ? nativeMic.value : false);
    setAccessibilityPermission(acc.status === "fulfilled" ? acc.value : false);
    setInputMonitoringPermission(
      inputMonitoring.status === "fulfilled" ? inputMonitoring.value : false,
    );
  }, [platformCapabilities]);

  useEffect(() => {
    if (activeTab !== "app" || !isOpen) return;

    let cancelled = false;
    let unlistenFocus: UnlistenFn | null = null;

    const refreshPermissions = () => {
      if (!cancelled) {
        void refreshPermissionState();
      }
    };

    refreshPermissions();

    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        refreshPermissions();
      }
    };

    window.addEventListener("focus", refreshPermissions);
    document.addEventListener("visibilitychange", handleVisibilityChange);

    getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (focused) {
          refreshPermissions();
        }
      })
      .then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlistenFocus = fn;
        }
      })
      .catch(() => {});

    return () => {
      cancelled = true;
      window.removeEventListener("focus", refreshPermissions);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      unlistenFocus?.();
    };
  }, [activeTab, isOpen, refreshPermissionState]);

  const handleRequestMicrophonePermission = useCallback(async () => {
    try {
      await invoke("request_microphone_permission");
    } catch {
      // Fall through to the settings fallback below.
    }

    try {
      const granted = await invoke<boolean>("check_microphone_permission");
      setMicPermission(granted);
      if (!granted) {
        await invoke("open_microphone_settings");
      }
    } catch {
      setMicPermission(false);
      try {
        await invoke("open_microphone_settings");
      } catch {
        // ignore
      }
    } finally {
      void refreshPermissionState();
    }
  }, [refreshPermissionState]);

  useEffect(() => {
    if (!isOpen) return;

    if (settingsQuery.error) {
      console.error("Failed to load settings:", settingsQuery.error);
      setError("Failed to load settings");
      return;
    }

    if (!settingsQuery.data || isSavingRef.current) return;

    hydrateFromSettings(settingsQuery.data);
  }, [hydrateFromSettings, isOpen, settingsQuery.data, settingsQuery.error]);

  const refreshModelStatus = useCallback((modelKey: string) => {
    invoke<ModelStatus>("check_model_status", { model: modelKey })
      .then((status) => {
        setModelStatus((prev) => ({ ...prev, [modelKey]: status }));
      })
      .catch((err) => {
        console.error(err);
        setModelStatus((prev) => ({
          ...prev,
          [modelKey]: {
            key: modelKey,
            installed: false,
            bytes_on_disk: 0,
            missing_files: [],
            directory: "",
          },
        }));
      });
  }, []);

  const refreshTtsModelStatus = useCallback((modelKey: string) => {
    invoke<ModelStatus>("check_tts_model_status", { model: modelKey })
      .then((status) => {
        setTtsModelStatus((prev) => ({ ...prev, [modelKey]: status }));
      })
      .catch((err) => {
        console.error(err);
        setTtsModelStatus((prev) => ({
          ...prev,
          [modelKey]: {
            key: modelKey,
            installed: false,
            bytes_on_disk: 0,
            missing_files: [],
            directory: "",
          },
        }));
      });
  }, []);

  useEffect(() => {
    if (!isOpen || modelCatalog.length === 0) return;

    for (const model of modelCatalog) {
      void refreshModelStatus(model.key);
    }

    setLocalModel((current) =>
      modelCatalog.some((model) => model.key === current) ? current : (modelCatalog[0]?.key ?? ""),
    );
  }, [isOpen, modelCatalog, refreshModelStatus]);

  useEffect(() => {
    if (!isOpen || ttsModelCatalog.length === 0) return;

    for (const model of ttsModelCatalog) {
      void refreshTtsModelStatus(model.key);
    }

    setTtsModel((current) =>
      ttsModelCatalog.some((model) => model.key === current)
        ? current
        : (ttsModelCatalog.find((model) => model.key === "kokoro_82m")?.key ??
          ttsModelCatalog.find((model) => model.key === "qwen3_tts_0_6b_custom_voice")?.key ??
          ttsModelCatalog[0]?.key ??
          "kokoro_82m"),
    );
  }, [isOpen, refreshTtsModelStatus, ttsModelCatalog]);

  useModelDownloadEvents({
    enabled: isOpen,
    onProgress: (payload) => {
      setDownloadState((prev) => ({
        ...prev,
        [payload.model]: {
          status: "downloading",
          percent: Math.min(100, payload.percent),
          downloaded: payload.downloaded,
          total: payload.total,
          file: payload.file,
        },
      }));
    },
    onComplete: ({ model }) => {
      setDownloadState((prev) => ({
        ...prev,
        [model]: {
          status: "complete",
          percent: 100,
          downloaded: prev[model]?.downloaded ?? 0,
          total: prev[model]?.total ?? 0,
        },
      }));
      if (ttsModelCatalog.some((entry) => entry.key === model)) {
        refreshTtsModelStatus(model);
      } else {
        refreshModelStatus(model);
      }
    },
    onError: ({ model, error }) => {
      if (error.toLowerCase().includes("cancelled")) return;
      setDownloadState((prev) => ({
        ...prev,
        [model]: {
          status: "error",
          message: error,
          percent: prev[model]?.percent ?? 0,
          downloaded: prev[model]?.downloaded ?? 0,
          total: prev[model]?.total ?? 0,
        },
      }));
    },
  });

  useEffect(() => {
    if (!isOpen) return;
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      if (e.defaultPrevented) return;
      if (captureActiveRef.current) {
        e.preventDefault();
        finalizeCapture();
        resetCaptureState();
        return;
      }
      onClose();
    };
    window.addEventListener("keydown", handleEscape);
    return () => window.removeEventListener("keydown", handleEscape);
  }, [finalizeCapture, isOpen, onClose, resetCaptureState]);

  const isSavingRef = useRef(false);

  useEffect(() => {
    if (loading) return;
    if (!didHydrateRef.current) {
      didHydrateRef.current = true;
      return;
    }

    const saveSettings = async () => {
      if (!localModel) return;

      isSavingRef.current = true;
      try {
        await invoke("update_settings", {
          args: {
            smartShortcut,
            smartShortcuts,
            smartEnabled,
            holdShortcut,
            holdShortcuts,
            holdEnabled,
            toggleShortcut,
            toggleShortcuts,
            toggleEnabled,
            commandShortcut,
            commandShortcuts,
            commandEnabled: aiFeaturesReady ? commandEnabled : false,
            pasteLastTranscriptShortcut,
            pasteLastTranscriptShortcuts,
            pasteLastTranscriptEnabled,
            cancelShortcut,
            cancelShortcuts,
            cancelEnabled,
            wakeListeningEnabled,
            wakePhrases,
            wakeSpeakerVerificationEnabled,
            transcriptionMode,
            localModel,
            ttsEnabled,
            ttsAutoAfterStt,
            ttsAutoPlay,
            ttsVolume,
            ttsModel,
            ttsVoiceMode,
            ttsSpeaker,
            ttsInstruction,
            microphoneDevice,
            language,
            appLocale,
            themeMode,

            llmEnabled: aiFeaturesReady,
            cleanupEnabled: false,
            llmProvider,
            llmEndpoint,
            llmApiKey,
            llmModel,
            editModeEnabled: aiFeaturesReady ? editModeEnabled : false,
            autoTransformEnabled: false,
            autoTransformPresetId: autoTransformPresetId || "polish",
            vibeCodingEnabled,
            vibeCodingVariableRecognition,
            vibeCodingFileTagging,
            vibeCodingIncludeWindowContext,
            mediaControlEnabled,
            autoUpdateEnabled,
            autoLaunchEnabled,
            recordingPrunePolicy,
            localDataStoragePolicy,
            contextAwarenessEnabled,
            analyticsEnabled,
          },
        });
        setError(null);
      } catch (err) {
        console.error(err);
        setError(String(err));
      } finally {
        isSavingRef.current = false;
      }
    };

    const timeoutId = setTimeout(() => {
      saveSettings();
    }, 500);

    return () => clearTimeout(timeoutId);
  }, [
    loading,
    smartShortcut,
    smartShortcuts,
    smartEnabled,
    holdShortcut,
    holdShortcuts,
    holdEnabled,
    toggleShortcut,
    toggleShortcuts,
    toggleEnabled,
    commandShortcut,
    commandShortcuts,
    commandEnabled,
    pasteLastTranscriptShortcut,
    pasteLastTranscriptShortcuts,
    pasteLastTranscriptEnabled,
    cancelShortcut,
    cancelShortcuts,
    cancelEnabled,
    wakeListeningEnabled,
    wakePhrases,
    wakeSpeakerVerificationEnabled,
    transcriptionMode,
    localModel,
    ttsEnabled,
    ttsAutoAfterStt,
    ttsAutoPlay,
    ttsVolume,
    ttsModel,
    ttsVoiceMode,
    ttsSpeaker,
    ttsInstruction,
    microphoneDevice,
    language,
    appLocale,
    themeMode,

    llmEnabled,
    cleanupEnabled,
    llmProvider,
    llmEndpoint,
    llmApiKey,
    llmModel,
    editModeEnabled,
    autoTransformEnabled,
    autoTransformPresetId,
    vibeCodingEnabled,
    vibeCodingVariableRecognition,
    vibeCodingFileTagging,
    vibeCodingIncludeWindowContext,
    mediaControlEnabled,
    autoUpdateEnabled,
    autoLaunchEnabled,
    recordingPrunePolicy,
    localDataStoragePolicy,
    contextAwarenessEnabled,
    analyticsEnabled,
    aiFeaturesReady,
  ]);

  const handleOpenDataDir = useCallback(async () => {
    if (!appInfo?.data_dir_path) return;
    try {
      await invoke("open_data_dir", { path: appInfo.data_dir_path });
    } catch (err) {
      console.error("Failed to open data directory:", err);
    }
  }, [appInfo?.data_dir_path]);

  const handleStartCapture = useCallback(
    (mode: ShortcutCaptureMode, index = 0) => {
      const target = captureTarget(mode, index);
      if (captureActive === target) {
        finalizeCapture();
        resetCaptureState();
        setError(null);
        return;
      }
      resetCaptureState();
      captureActiveRef.current = target;
      setCaptureActive(target);
      setError(null);
      invoke("set_shortcut_capture_active", { active: true }).catch((err) => {
        console.error("Failed to disable shortcuts for capture", err);
        captureActiveRef.current = null;
        setCaptureActive(null);
        resetCaptureState();
        setError(String(err));
      });
    },
    [captureActive, finalizeCapture, resetCaptureState],
  );

  const removeShortcut = useCallback(
    (mode: ShortcutCaptureMode, index: number) => {
      const update = (
        setList: Dispatch<SetStateAction<string[]>>,
        setPrimary: (value: string) => void,
        fallback: string,
      ) => {
        setList((current) => {
          if (current.length <= 1) return current;
          const next = current.filter((_, slot) => slot !== index);
          const normalized = normalizeShortcutList(next, fallback);
          setPrimary(normalized[0]);
          return normalized;
        });
      };

      if (mode === "smart") update(setSmartShortcuts, setSmartShortcut, smartShortcut);
      if (mode === "hold") update(setHoldShortcuts, setHoldShortcut, holdShortcut);
      if (mode === "toggle") update(setToggleShortcuts, setToggleShortcut, toggleShortcut);
      if (mode === "command") update(setCommandShortcuts, setCommandShortcut, commandShortcut);
      if (mode === "paste-last") {
        update(
          setPasteLastTranscriptShortcuts,
          setPasteLastTranscriptShortcut,
          pasteLastTranscriptShortcut,
        );
      }
      if (mode === "cancel") update(setCancelShortcuts, setCancelShortcut, cancelShortcut);
    },
    [
      cancelShortcut,
      commandShortcut,
      holdShortcut,
      pasteLastTranscriptShortcut,
      smartShortcut,
      toggleShortcut,
    ],
  );

  const addShortcutSlot = useCallback(
    (mode: ShortcutCaptureMode) => {
      const add = (
        current: string[],
        setList: Dispatch<SetStateAction<string[]>>,
        fallback: string,
      ) => {
        if (current.length >= MAX_SHORTCUTS_PER_ACTION) return;
        const next = [...current, fallback].slice(0, MAX_SHORTCUTS_PER_ACTION);
        setList(next);
        handleStartCapture(mode, next.length - 1);
      };

      if (mode === "smart") add(smartShortcuts, setSmartShortcuts, "Win+Alt+1");
      if (mode === "hold") add(holdShortcuts, setHoldShortcuts, holdShortcut);
      if (mode === "toggle") add(toggleShortcuts, setToggleShortcuts, toggleShortcut);
      if (mode === "command") add(commandShortcuts, setCommandShortcuts, commandShortcut);
      if (mode === "paste-last") {
        add(
          pasteLastTranscriptShortcuts,
          setPasteLastTranscriptShortcuts,
          pasteLastTranscriptShortcut,
        );
      }
      if (mode === "cancel") add(cancelShortcuts, setCancelShortcuts, cancelShortcut);
    },
    [
      cancelShortcut,
      cancelShortcuts,
      commandShortcut,
      commandShortcuts,
      handleStartCapture,
      holdShortcut,
      holdShortcuts,
      pasteLastTranscriptShortcut,
      pasteLastTranscriptShortcuts,
      smartShortcuts,
      toggleShortcut,
      toggleShortcuts,
    ],
  );

  const addMouseShortcut = useCallback(
    (mode: ShortcutCaptureMode, shortcut: string) => {
      const add = (
        current: string[],
        setList: Dispatch<SetStateAction<string[]>>,
        setPrimary: (value: string) => void,
        fallback: string,
      ) => {
        const normalized = normalizeShortcutList([...current, shortcut], fallback);
        setList(normalized);
        setPrimary(normalized[0]);
      };

      if (mode === "smart") add(smartShortcuts, setSmartShortcuts, setSmartShortcut, smartShortcut);
      if (mode === "hold") add(holdShortcuts, setHoldShortcuts, setHoldShortcut, holdShortcut);
      if (mode === "toggle") {
        add(toggleShortcuts, setToggleShortcuts, setToggleShortcut, toggleShortcut);
      }
      if (mode === "command") {
        add(commandShortcuts, setCommandShortcuts, setCommandShortcut, commandShortcut);
      }
      if (mode === "paste-last") {
        add(
          pasteLastTranscriptShortcuts,
          setPasteLastTranscriptShortcuts,
          setPasteLastTranscriptShortcut,
          pasteLastTranscriptShortcut,
        );
      }
      if (mode === "cancel")
        add(cancelShortcuts, setCancelShortcuts, setCancelShortcut, cancelShortcut);
    },
    [
      cancelShortcut,
      cancelShortcuts,
      commandShortcut,
      commandShortcuts,
      holdShortcut,
      holdShortcuts,
      pasteLastTranscriptShortcut,
      pasteLastTranscriptShortcuts,
      smartShortcut,
      smartShortcuts,
      toggleShortcut,
      toggleShortcuts,
    ],
  );

  const handleSignOut = useCallback(async () => {
    setAuthLoading(true);
    try {
      await logout();
      await onUpdateUser();
    } catch (err) {
      console.error("Sign out failed:", err);
    } finally {
      setAuthLoading(false);
    }
  }, [onUpdateUser]);

  const handleCancelAuth = useCallback(() => {
    setAuthLoading(false);
  }, []);

  const fetchAvailableModels = useCallback(async () => {
    try {
      const models = await invoke<string[]>("fetch_llm_models", {
        endpoint: llmEndpoint,
        provider: llmProvider,
        apiKey: llmApiKey,
      });
      setAvailableModels(models);
    } catch {
      setAvailableModels([]);
    }
  }, [llmEndpoint, llmProvider, llmApiKey]);

  const handleDownload = useCallback(
    async (modelKey: string) => {
      setDownloadState((prev) => ({
        ...prev,
        [modelKey]: {
          status: "downloading",
          percent: 0,
          downloaded: 0,
          total: 0,
          file: "starting",
        },
      }));
      try {
        await invoke("download_model", { model: modelKey });
        refreshModelStatus(modelKey);
      } catch (err) {
        const errorMsg = String(err);
        if (errorMsg.toLowerCase().includes("cancelled")) return;
        console.error(err);
        setDownloadState((prev) => ({
          ...prev,
          [modelKey]: {
            status: "error",
            message: String(err),
            percent: prev[modelKey]?.percent ?? 0,
            downloaded: prev[modelKey]?.downloaded ?? 0,
            total: prev[modelKey]?.total ?? 0,
          },
        }));
      }
    },
    [refreshModelStatus],
  );

  const handleTtsDownload = useCallback(
    async (modelKey: string) => {
      setDownloadState((prev) => ({
        ...prev,
        [modelKey]: {
          status: "downloading",
          percent: 0,
          downloaded: 0,
          total: 0,
          file: "starting",
        },
      }));
      try {
        await invoke("download_tts_model", { model: modelKey });
        refreshTtsModelStatus(modelKey);
      } catch (err) {
        const errorMsg = String(err);
        if (errorMsg.toLowerCase().includes("cancelled")) return;
        console.error(err);
        setDownloadState((prev) => ({
          ...prev,
          [modelKey]: {
            status: "error",
            message: String(err),
            percent: prev[modelKey]?.percent ?? 0,
            downloaded: prev[modelKey]?.downloaded ?? 0,
            total: prev[modelKey]?.total ?? 0,
          },
        }));
      }
    },
    [refreshTtsModelStatus],
  );

  const handleDelete = useCallback(
    async (modelKey: string) => {
      try {
        await invoke("delete_model", { model: modelKey });
        setDownloadState((prev) => ({
          ...prev,
          [modelKey]: { status: "idle", percent: 0, downloaded: 0, total: 0 },
        }));

        if (localModel === modelKey) {
          const otherInstalledModel = modelCatalog.find(
            (m) => m.key !== modelKey && modelStatus[m.key]?.installed,
          );
          if (otherInstalledModel) {
            setLocalModel(otherInstalledModel.key);
          }
        }

        refreshModelStatus(modelKey);
      } catch (err) {
        console.error(err);
        setDownloadState((prev) => ({
          ...prev,
          [modelKey]: {
            status: "error",
            message: String(err),
            percent: prev[modelKey]?.percent ?? 0,
            downloaded: prev[modelKey]?.downloaded ?? 0,
            total: prev[modelKey]?.total ?? 0,
          },
        }));
      }
    },
    [localModel, modelCatalog, modelStatus, refreshModelStatus],
  );

  const handleTtsDelete = useCallback(
    async (modelKey: string) => {
      try {
        await invoke("delete_tts_model", { model: modelKey });
        setDownloadState((prev) => ({
          ...prev,
          [modelKey]: { status: "idle", percent: 0, downloaded: 0, total: 0 },
        }));

        if (ttsModel === modelKey) {
          const otherInstalledModel = ttsModelCatalog.find(
            (m) => m.key !== modelKey && ttsModelStatus[m.key]?.installed,
          );
          if (otherInstalledModel) {
            setTtsModel(otherInstalledModel.key);
          }
        }

        refreshTtsModelStatus(modelKey);
      } catch (err) {
        console.error(err);
        setDownloadState((prev) => ({
          ...prev,
          [modelKey]: {
            status: "error",
            message: String(err),
            percent: prev[modelKey]?.percent ?? 0,
            downloaded: prev[modelKey]?.downloaded ?? 0,
            total: prev[modelKey]?.total ?? 0,
          },
        }));
      }
    },
    [refreshTtsModelStatus, ttsModel, ttsModelCatalog, ttsModelStatus],
  );

  const handleCancelDownload = useCallback(async (modelKey: string) => {
    try {
      await invoke("cancel_download", { model: modelKey });
      setDownloadState((prev) => ({
        ...prev,
        [modelKey]: {
          status: "cancelled",
          percent: 0,
          downloaded: 0,
          total: 0,
        },
      }));
      setTimeout(() => {
        setDownloadState((prev) => {
          if (prev[modelKey]?.status === "cancelled") {
            return {
              ...prev,
              [modelKey]: {
                status: "idle",
                percent: 0,
                downloaded: 0,
                total: 0,
              },
            };
          }
          return prev;
        });
      }, 1500);
    } catch (err) {
      console.error("Failed to cancel download:", err);
    }
  }, []);

  const formatBytes = useCallback((bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const decimals = i >= 3 ? 1 : 0;
    return parseFloat((bytes / Math.pow(k, i)).toFixed(decimals)) + " " + sizes[i];
  }, []);

  return {
    activeTab,
    setActiveTab,
    loading,
    error,
    errorCopied,
    setErrorCopied,

    smartShortcut,
    smartShortcuts,
    smartEnabled,
    setSmartEnabled,
    holdShortcut,
    holdShortcuts,
    holdEnabled,
    setHoldEnabled,
    toggleShortcut,
    toggleShortcuts,
    toggleEnabled,
    setToggleEnabled,
    commandShortcut,
    commandShortcuts,
    commandEnabled,
    setCommandEnabled,
    pasteLastTranscriptShortcut,
    pasteLastTranscriptShortcuts,
    pasteLastTranscriptEnabled,
    setPasteLastTranscriptEnabled,
    cancelShortcut,
    cancelShortcuts,
    cancelEnabled,
    setCancelEnabled,
    wakeListeningEnabled,
    setWakeListeningEnabled,
    wakePhrases,
    setWakePhrases,
    wakeSpeakerVerificationEnabled,
    setWakeSpeakerVerificationEnabled,
    wakeSpeakerProfile,
    setWakeSpeakerProfile,
    transcriptionMode,
    setTranscriptionMode,
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
    microphoneDevice,
    setMicrophoneDevice,
    language: displayedLanguage,
    setLanguage,
    appLocale,
    setAppLocale,
    languages: displayedLanguageOptions,
    languageBadgeColumns: languageView.badgeColumns,
    showLanguageSupportBadges,

    inputDevices,
    modelCatalog,
    modelStatus,
    ttsModelCatalog,
    ttsModelStatus,
    downloadState,
    appInfo,

    captureActive,
    capturePreview,
    handleStartCapture,
    removeShortcut,
    addShortcutSlot,
    addMouseShortcut,

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
    llmConfigReady,
    aiFeaturesReady,
    availableModels,
    fetchAvailableModels,
    cleanupEnabled,
    setCleanupEnabled,
    editModeEnabled,
    setEditModeEnabled,
    autoTransformEnabled,
    setAutoTransformEnabled,
    autoTransformPresetId,
    setAutoTransformPresetId,
    vibeCodingEnabled,
    setVibeCodingEnabled,
    vibeCodingVariableRecognition,
    setVibeCodingVariableRecognition,
    vibeCodingFileTagging,
    setVibeCodingFileTagging,
    vibeCodingIncludeWindowContext,
    setVibeCodingIncludeWindowContext,
    mediaControlEnabled,
    setMediaControlEnabled,
    autoUpdateEnabled,
    setAutoUpdateEnabled,
    autoLaunchEnabled,
    setAutoLaunchEnabled,
    recordingPrunePolicy,
    setRecordingPrunePolicy,
    localDataStoragePolicy,
    setLocalDataStoragePolicy,
    contextAwarenessEnabled,
    setContextAwarenessEnabled,
    analyticsEnabled,
    setAnalyticsEnabled,
    platformCapabilities,

    authLoading,
    currentUser,
    cloudSyncEnabled,
    setCloudSyncEnabled,
    handleSignOut,
    handleCancelAuth,

    micPermission,
    accessibilityPermission,
    inputMonitoringPermission,
    handleRequestMicrophonePermission,
    textSizeMode,
    setTextSizeMode,
    themeMode,
    setThemeMode,

    showFAQModal,
    setShowFAQModal,
    whatsNewOpen,
    setWhatsNewOpen,

    handleDownload,
    handleDelete,
    handleTtsDownload,
    handleTtsDelete,
    handleCancelDownload,
    handleOpenDataDir,
    formatBytes,

    onUpdateUser,
  };
}
