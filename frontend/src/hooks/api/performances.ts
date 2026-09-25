import { useInfiniteQuery, useQuery } from "@tanstack/react-query";

import { performancesApi } from "@/api/performances";
import type { PerformanceListParams } from "@/api/performances";

export const performanceKeys = {
  all: () => ["performances"] as const,
  list: (params?: PerformanceListParams) => ["performances", "list", params] as const,
  detail: (id: string) => ["performances", "detail", id] as const,
};

export function usePerformances(params?: PerformanceListParams, enabled = true) {
  return useQuery({
    queryKey: performanceKeys.list(params),
    queryFn: async () => {
      const { data, error } = await performancesApi.list(params);
      if (error) throw error;
      return data;
    },
    enabled,
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

export function usePerformanceLyrics(id: string, enabled = true) {
  return useQuery({
    queryKey: ["performances", "lyrics", id],
    queryFn: async () => {
      const { data, error, response } = await performancesApi.getLyrics(id);
      if (response.status === 404) return null;
      if (error) throw error;
      return data ?? null;
    },
    enabled,
  });
}

export function useInfinitePerformances(
  params?: Omit<PerformanceListParams, "page" | "per_page">,
  enabled = true,
) {
  const PERFORMANCES_PER_PAGE = 200;

  return useInfiniteQuery({
    queryKey: ["performances", "infinite", params],
    queryFn: async ({ pageParam }) => {
      const { data, error } = await performancesApi.list({
        ...params,
        page: pageParam,
        per_page: PERFORMANCES_PER_PAGE,
      });
      if (error) throw error;
      return data;
    },
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const fetched = allPages.length * PERFORMANCES_PER_PAGE;
      return fetched < lastPage.total ? allPages.length + 1 : undefined;
    },
    enabled,
  });
}
