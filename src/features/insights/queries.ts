import { useQuery } from "@tanstack/react-query";
import * as insightsApi from "./api";

export const insightsKeys = {
  all: ["insights"] as const,
  summary: (days: number) => [...insightsKeys.all, "summary", days] as const,
};

export function useInsights(days: number = 30, enabled: boolean = true) {
  return useQuery({
    queryKey: insightsKeys.summary(days),
    queryFn: () => insightsApi.getInsights(days),
    enabled,
    gcTime: 60_000,
  });
}
