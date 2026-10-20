import { keepPreviousData, useInfiniteQuery, useQuery } from "@tanstack/react-query";

import { unwrap } from "@/api/client";
import { performancesApi } from "@/api/performances";
import type { PerformanceListParams } from "@/api/performances";

export const performanceKeys = {
  all: () => ["performances"] as const,
  list: (params?: PerformanceListParams) => ["performances", "list", params] as const,
  detail: (id: string) => ["performances", "detail", id] as const,
  players: (id: string, page: number) => ["performances", "players", id, page] as const,
};

const PLAYERS_PER_PAGE = 20;

export function usePerformances(params?: PerformanceListParams, enabled = true) {
  return useQuery({
    queryKey: performanceKeys.list(params),
    queryFn: async () => unwrap(await performancesApi.list(params)),
    enabled,
    placeholderData: keepPreviousData,
  });
}

export function usePerformance(id: string, enabled = true) {
  return useQuery({
    queryKey: performanceKeys.detail(id),
    queryFn: async () => unwrap(await performancesApi.get(id)),
    enabled,
  });
}

export function usePerformancePlayers(id: string, page: number, enabled = true) {
  return useQuery({
    queryKey: performanceKeys.players(id, page),
    queryFn: async () => unwrap(await performancesApi.listPlayers(id, page, PLAYERS_PER_PAGE)),
    enabled,
  });
}

export function usePerformanceLyrics(id: string, enabled = true) {
  return useQuery({
    queryKey: ["performances", "lyrics", id],
    queryFn: async () => {
      const result = await performancesApi.getLyrics(id);
      return result.response.status === 404 ? null : unwrap(result);
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
    queryFn: async ({ pageParam }) =>
      unwrap(
        await performancesApi.list({ ...params, page: pageParam, per_page: PERFORMANCES_PER_PAGE }),
      ),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const fetched = allPages.length * PERFORMANCES_PER_PAGE;
      return fetched < lastPage.total ? allPages.length + 1 : undefined;
    },
    enabled,
  });
}
