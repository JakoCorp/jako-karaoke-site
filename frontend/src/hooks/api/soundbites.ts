import { keepPreviousData, useInfiniteQuery, useQuery } from "@tanstack/react-query";

import { unwrap } from "@/api/client";
import { soundbitesApi } from "@/api/soundbites";
import type { SoundbiteListParams } from "@/api/soundbites";
import type { WithoutPaging } from "@/api/types";

export const soundbiteKeys = {
  all: () => ["soundbites"] as const,
  list: (params?: SoundbiteListParams) => ["soundbites", "list", params] as const,
};

export function useSoundbites(params?: SoundbiteListParams) {
  return useQuery({
    queryKey: soundbiteKeys.list(params),
    queryFn: async () => unwrap(await soundbitesApi.list(params)),
    placeholderData: keepPreviousData,
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
