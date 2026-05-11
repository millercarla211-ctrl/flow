import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as snippetsApi from "./api";

export const snippetKeys = {
  all: ["snippets"] as const,
  list: () => [...snippetKeys.all, "list"] as const,
};

export function useSnippets(enabled: boolean = true) {
  return useQuery({
    queryKey: snippetKeys.list(),
    queryFn: snippetsApi.listSnippets,
    enabled,
    gcTime: 60_000,
  });
}

export function useCreateSnippet() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: snippetsApi.createSnippet,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: snippetKeys.all });
    },
  });
}

export function useUpdateSnippet() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: snippetsApi.updateSnippet,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: snippetKeys.all });
    },
  });
}

export function useDeleteSnippet() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: snippetsApi.deleteSnippet,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: snippetKeys.all });
    },
  });
}
