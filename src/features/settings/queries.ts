import { useQuery } from "@tanstack/react-query";
import * as settingsApi from "./api";
import type { StoredSettings } from "../../types";

export const settingsKeys = {
  all: ["settings"] as const,
  detail: () => [...settingsKeys.all, "detail"] as const,
  appInfo: () => ["appInfo"] as const,
  devices: () => ["inputDevices"] as const,
  autoPaste: () => ["autoPasteStatus"] as const,
};

export function useSettings<TSelect = StoredSettings>(
  select?: (data: StoredSettings) => TSelect,
  enabled: boolean = true,
) {
  return useQuery({
    queryKey: settingsKeys.detail(),
    queryFn: settingsApi.getSettings,
    select,
    enabled,
  });
}

export function useAppInfo(enabled: boolean = true) {
  return useQuery({
    queryKey: settingsKeys.appInfo(),
    queryFn: settingsApi.getAppInfo,
    enabled,
    staleTime: Infinity,
  });
}

export function useInputDevices(enabled: boolean = true) {
  return useQuery({
    queryKey: settingsKeys.devices(),
    queryFn: settingsApi.listInputDevices,
    enabled,
    refetchOnMount: "always",
  });
}

export function useAutoPasteStatus(enabled: boolean = true) {
  return useQuery({
    queryKey: settingsKeys.autoPaste(),
    queryFn: settingsApi.getAutoPasteStatus,
    enabled,
    refetchInterval: 5000,
    refetchOnWindowFocus: true,
  });
}
