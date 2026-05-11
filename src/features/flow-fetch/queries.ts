import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as flowFetchApi from "./api";

export const flowFetchKeys = {
  all: ["flow-fetch"] as const,
  list: (limit: number) => [...flowFetchKeys.all, "list", limit] as const,
};

export function useFlowFetchLinks(limit: number = 30, enabled: boolean = true) {
  return useQuery({
    queryKey: flowFetchKeys.list(limit),
    queryFn: () => flowFetchApi.listFlowFetchLinks(limit),
    enabled,
    gcTime: 60_000,
  });
}

export function useDeleteFlowFetchLink() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: flowFetchApi.deleteFlowFetchLink,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: flowFetchKeys.all });
    },
  });
}

export function useCopyFlowFetchLink() {
  return useMutation({
    mutationFn: flowFetchApi.copyFlowFetchLink,
  });
}
