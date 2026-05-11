import { useMutation, useQuery } from "@tanstack/react-query";
import * as transformsApi from "./api";

export const transformKeys = {
  all: ["transforms"] as const,
  presets: () => [...transformKeys.all, "presets"] as const,
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
  return useMutation({
    mutationFn: transformsApi.transformText,
  });
}
