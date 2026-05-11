import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as transformsApi from "./api";

export const transformKeys = {
  all: ["transforms"] as const,
  presets: () => [...transformKeys.all, "presets"] as const,
  history: () => [...transformKeys.all, "history"] as const,
};

export function useTransformPresets(enabled: boolean = true) {
  return useQuery({
    queryKey: transformKeys.presets(),
    queryFn: transformsApi.getTransformPresets,
    enabled,
    gcTime: Number.POSITIVE_INFINITY,
    staleTime: Number.POSITIVE_INFINITY,
  });
}

export function useTransformText() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: transformsApi.transformText,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: transformKeys.history() });
    },
  });
}

export function useTransformSource() {
  return useMutation({
    mutationFn: transformsApi.getTransformSource,
  });
}

export function useTransformHistory(enabled: boolean = true, limit: number = 20) {
  return useQuery({
    queryKey: [...transformKeys.history(), limit],
    queryFn: () => transformsApi.listTransformHistory(limit),
    enabled,
    gcTime: 60_000,
  });
}

export function useDeleteTransformHistoryEntry() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: transformsApi.deleteTransformHistoryEntry,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: transformKeys.history() });
    },
  });
}
