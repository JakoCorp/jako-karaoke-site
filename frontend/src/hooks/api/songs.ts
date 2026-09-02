import { useQuery } from "@tanstack/react-query";

import { songsApi } from "@/api/songs";
import type { SearchPaginationParams } from "@/api/types";

export const songKeys = {
  all: () => ["songs"] as const,
  list: (params?: SearchPaginationParams) => ["songs", "list", params] as const,
  detail: (id: string) => ["songs", "detail", id] as const,
};

export function useSongs(params?: SearchPaginationParams, enabled = true) {
  return useQuery({
    queryKey: songKeys.list(params),
    queryFn: async () => {
      const { data, error } = await songsApi.list(params);
      if (error) throw error;
      return data;
    },
    enabled,
  });
}

export function useSong(id: string) {
  return useQuery({
    queryKey: songKeys.detail(id),
    queryFn: async () => {
      const { data, error } = await songsApi.get(id);
      if (error) throw error;
      return data;
    },
  });
}
