import { useQuery } from "@tanstack/react-query";

import { artistsApi } from "@/api/artists";
import type { SearchPaginationParams } from "@/api/types";

export const artistKeys = {
  all: () => ["artists"] as const,
  list: (params?: SearchPaginationParams) => ["artists", "list", params] as const,
  detail: (id: string) => ["artists", "detail", id] as const,
};

export function useArtists(params?: SearchPaginationParams, enabled = true) {
  return useQuery({
    queryKey: artistKeys.list(params),
    queryFn: async () => {
      const { data, error } = await artistsApi.list(params);
      if (error) throw error;
      return data;
    },
    enabled,
  });
}

export function useArtist(id: string, enabled = true) {
  return useQuery({
    queryKey: artistKeys.detail(id),
    queryFn: async () => {
      const { data, error } = await artistsApi.get(id);
      if (error) throw error;
      return data;
    },
    enabled,
  });
}
