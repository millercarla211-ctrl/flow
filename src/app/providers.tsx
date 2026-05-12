import { I18nProvider } from "@lingui/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useEffect, type ReactNode } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { authKeys } from "../features/auth/queries";
import { activateLocale, i18n } from "../i18n";
import { settingsKeys, useSettings } from "../features/settings/queries";
import { transcriptionKeys } from "../features/transcriptions/queries";
import { scratchpadKeys } from "../features/scratchpad/queries";
import { snippetKeys } from "../features/snippets/queries";
import { flowFetchKeys } from "../features/flow-fetch/queries";
import { insightsKeys } from "../features/insights/queries";
import { updateKeys } from "../features/updates/queries";
import { isTauriRuntime } from "../platform/tauriRuntime";
import type { StoredSettings } from "../types";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function QuerySyncBridge() {
  const isSettingsWindow = getCurrentWindow().label === "settings";

  useEffect(() => {
    let cancelled = false;
    const unlisteners: UnlistenFn[] = [];

    const register = <TPayload,>(event: string, handler: (payload: TPayload) => void) => {
      listen<TPayload>(event, (eventPayload) => {
        if (!cancelled) {
          handler(eventPayload.payload);
        }
      }).then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlisteners.push(fn);
        }
      });
    };

    register<StoredSettings>("settings:changed", (settings) => {
      queryClient.setQueryData(settingsKeys.detail(), settings);
    });
    register("scratchpad:changed", () => {
      queryClient.invalidateQueries({ queryKey: scratchpadKeys.all });
    });
    register("scratchpad:entry-created", () => {
      queryClient.invalidateQueries({ queryKey: scratchpadKeys.all });
    });
    register("snippets:changed", () => {
      queryClient.invalidateQueries({ queryKey: snippetKeys.all });
    });
    register("flow-fetch:changed", () => {
      queryClient.invalidateQueries({ queryKey: flowFetchKeys.all });
    });
    register("flow-fetch:link-captured", () => {
      queryClient.invalidateQueries({ queryKey: flowFetchKeys.all });
    });
    register("transcription:complete", () => {
      queryClient.invalidateQueries({ queryKey: insightsKeys.all });
    });
    register("transcription:error", () => {
      queryClient.invalidateQueries({ queryKey: insightsKeys.all });
    });

    if (isSettingsWindow) {
      register("auth:changed", () => {
        queryClient.invalidateQueries({ queryKey: authKeys.user() });
      });
      register("update:available", () => {
        queryClient.invalidateQueries({ queryKey: updateKeys.status() });
      });
      register("update:cleared", () => {
        queryClient.invalidateQueries({ queryKey: updateKeys.status() });
      });
      register("transcription:complete", () => {
        queryClient.invalidateQueries({ queryKey: transcriptionKeys.all });
        queryClient.invalidateQueries({ queryKey: insightsKeys.all });
      });
      register("transcription:error", () => {
        queryClient.invalidateQueries({ queryKey: transcriptionKeys.all });
        queryClient.invalidateQueries({ queryKey: insightsKeys.all });
      });
      register("audio:input-devices-changed", () => {
        queryClient.invalidateQueries({ queryKey: settingsKeys.devices() });
      });
    }

    return () => {
      cancelled = true;
      unlisteners.forEach((fn) => fn());
    };
  }, [isSettingsWindow]);

  return null;
}

function LocaleSyncBridge() {
  const { data: settings } = useSettings(undefined, true);

  useEffect(() => {
    activateLocale(settings?.app_locale);
  }, [settings?.app_locale]);

  return null;
}

type TtsCompletePayload = {
  path: string;
  auto_play: boolean;
  volume?: number;
  audio_data_url?: string | null;
};

function TtsPlaybackBridge() {
  const windowLabel = getCurrentWindow().label;

  useEffect(() => {
    if (windowLabel !== "main") {
      return;
    }

    let cancelled = false;
    let currentAudio: HTMLAudioElement | null = null;
    let unlisten: UnlistenFn | null = null;

    listen<TtsCompletePayload>("tts:complete", (event) => {
      if (cancelled || !event.payload.auto_play || !event.payload.path) return;

      currentAudio?.pause();
      currentAudio = null;
      currentAudio = new Audio(event.payload.audio_data_url ?? convertFileSrc(event.payload.path));
      currentAudio.volume = Math.max(0, Math.min(1, event.payload.volume ?? 0.1));
      currentAudio.play().catch((err) => {
        console.error("Failed to play generated TTS audio:", err);
      });
    }).then((fn) => {
      if (cancelled) fn();
      else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
      currentAudio?.pause();
      currentAudio = null;
    };
  }, [windowLabel]);

  return null;
}

export function AppProviders({ children }: { children: ReactNode }) {
  const tauriRuntime = isTauriRuntime();

  return (
    <I18nProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        {tauriRuntime && <LocaleSyncBridge />}
        {tauriRuntime && <QuerySyncBridge />}
        {tauriRuntime && <TtsPlaybackBridge />}
        {children}
      </QueryClientProvider>
    </I18nProvider>
  );
}
