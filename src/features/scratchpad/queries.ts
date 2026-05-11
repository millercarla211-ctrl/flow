import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as scratchpadApi from "./api";

export const scratchpadKeys = {
  all: ["scratchpad"] as const,
  list: (search: string) => [...scratchpadKeys.all, "list", search] as const,
};

export function useScratchpadEntries(searchQuery: string = "", enabled: boolean = true) {
  return useQuery({
    queryKey: scratchpadKeys.list(searchQuery),
    queryFn: () => scratchpadApi.listScratchpadEntries(searchQuery || null),
    enabled,
    gcTime: 60_000,
  });
}

export function useCreateScratchpadEntry() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ body, source }: { body: string; source?: string }) =>
      scratchpadApi.createScratchpadEntry(body, source),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: scratchpadKeys.all });
    },
  });
}

export function useUpdateScratchpadEntry() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: scratchpadApi.updateScratchpadEntry,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: scratchpadKeys.all });
    },
  });
}

export function useDeleteScratchpadEntry() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: scratchpadApi.deleteScratchpadEntry,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: scratchpadKeys.all });
    },
  });
}
