import { useQuery } from "@tanstack/react-query";

import { performancesApi } from "@/api/performances";
import type { PerformanceListParams } from "@/api/performances";

export const performanceKeys = {
  all: () => ["performances"] as const,
  list: (params?: PerformanceListParams) => ["performances", "list", params] as const,
  detail: (id: string) => ["performances", "detail", id] as const,
};

export function usePerformances(params?: PerformanceListParams) {
  return useQuery({
    queryKey: performanceKeys.list(params),
    queryFn: async () => {
      const { data, error } = await performancesApi.list(params);
      if (error) throw error;
      return data;
    },
  });
}

export function usePerformance(id: string, enabled = true) {
  return useQuery({
    queryKey: performanceKeys.detail(id),
    queryFn: async () => {
      const { data, error } = await performancesApi.get(id);
      if (error) throw error;
      return data;
    },
    enabled,
  });
}
