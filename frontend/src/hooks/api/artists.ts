import { useInfiniteQuery, useQuery } from "@tanstack/react-query";

import { artistsApi } from "@/api/artists";
import type { ArtistListParams } from "@/api/artists";
import { unwrap } from "@/api/client";
import type { WithoutPaging } from "@/api/types";

export const artistKeys = {
  all: () => ["artists"] as const,
  list: (params?: ArtistListParams) => ["artists", "list", params] as const,
  detail: (id: string) => ["artists", "detail", id] as const,
};

export function useArtists(params?: ArtistListParams, enabled = true) {
  return useQuery({
    queryKey: artistKeys.list(params),
    queryFn: async () => unwrap(await artistsApi.list(params)),
    enabled,
  });
}

/** Infinite scroll query for the artists list. Appends pages as the user scrolls. */
export function useInfiniteArtists(params?: WithoutPaging<ArtistListParams>) {
  const ARTISTS_PER_PAGE = 24;

  return useInfiniteQuery({
    queryKey: ["artists", "infinite", params],
    queryFn: async ({ pageParam }) =>
      unwrap(await artistsApi.list({ ...params, page: pageParam, per_page: ARTISTS_PER_PAGE })),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const fetched = allPages.length * ARTISTS_PER_PAGE;
      return fetched < lastPage.total ? allPages.length + 1 : undefined;
    },
  });
}

export function useArtist(id: string, enabled = true) {
  return useQuery({
    queryKey: artistKeys.detail(id),
    queryFn: async () => unwrap(await artistsApi.get(id)),
    enabled,
  });
}
