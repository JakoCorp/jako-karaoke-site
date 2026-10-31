import { keepPreviousData, useInfiniteQuery, useQuery } from "@tanstack/react-query";

import { unwrap } from "@/api/client";
import { soundbitesApi } from "@/api/soundbites";
import type { SoundbiteListParams } from "@/api/soundbites";
import type { WithoutPaging } from "@/api/types";

export const soundbiteKeys = {
  all: () => ["soundbites"] as const,
  list: (params?: SoundbiteListParams) => ["soundbites", "list", params] as const,
  players: (id: string, page: number) => ["soundbites", "players", id, page] as const,
};

const PLAYERS_PER_PAGE = 20;

export function useSoundbites(params?: SoundbiteListParams) {
  return useQuery({
    queryKey: soundbiteKeys.list(params),
    queryFn: async () => unwrap(await soundbitesApi.list(params)),
    placeholderData: keepPreviousData,
  });
}

export function useSoundbitePlayers(id: string, page: number, enabled = true) {
  return useQuery({
    queryKey: soundbiteKeys.players(id, page),
    queryFn: async () => unwrap(await soundbitesApi.listPlayers(id, page, PLAYERS_PER_PAGE)),
    enabled,
  });
}

/** Infinite scroll query for the admin soundbites list. Appends pages as the user scrolls. */
export function useInfiniteSoundbites(params?: WithoutPaging<SoundbiteListParams>) {
  const SOUNDBITES_PER_PAGE = 50;

  return useInfiniteQuery({
    queryKey: ["soundbites", "infinite", params],
    queryFn: async ({ pageParam }) =>
      unwrap(
        await soundbitesApi.list({ ...params, page: pageParam, per_page: SOUNDBITES_PER_PAGE }),
      ),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const fetched = allPages.length * SOUNDBITES_PER_PAGE;
      return fetched < lastPage.total ? allPages.length + 1 : undefined;
    },
  });
}
