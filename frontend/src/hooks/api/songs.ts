import { keepPreviousData, useInfiniteQuery, useQuery } from "@tanstack/react-query";

import { unwrap } from "@/api/client";
import { songsApi } from "@/api/songs";
import type { SongListParams } from "@/api/songs";
import type { WithoutPaging } from "@/api/types";

export const songKeys = {
  all: () => ["songs"] as const,
  list: (params?: SongListParams) => ["songs", "list", params] as const,
  detail: (id: string) => ["songs", "detail", id] as const,
};

export function useSongs(params?: SongListParams, enabled = true) {
  return useQuery({
    queryKey: songKeys.list(params),
    queryFn: async () => unwrap(await songsApi.list(params)),
    enabled,
    placeholderData: keepPreviousData,
  });
}

/** Infinite scroll query for the songs list. Appends pages as the user scrolls. */
export function useInfiniteSongs(params?: WithoutPaging<SongListParams>) {
  const SONGS_PER_PAGE = 24;

  return useInfiniteQuery({
    queryKey: ["songs", "infinite", params],
    queryFn: async ({ pageParam }) =>
      unwrap(await songsApi.list({ ...params, page: pageParam, per_page: SONGS_PER_PAGE })),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const fetched = allPages.length * SONGS_PER_PAGE;
      return fetched < lastPage.total ? allPages.length + 1 : undefined;
    },
  });
}

export function useSong(id: string, enabled = true) {
  return useQuery({
    queryKey: songKeys.detail(id),
    queryFn: async () => unwrap(await songsApi.get(id)),
    enabled,
  });
}

export function useSongLyrics(id: string, enabled = true) {
  return useQuery({
    queryKey: ["songs", "lyrics", id],
    queryFn: async () => {
      const result = await songsApi.getLyrics(id);
      return result.response.status === 404 ? null : unwrap(result);
    },
    enabled,
  });
}
