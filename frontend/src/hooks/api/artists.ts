import { useQuery } from "@tanstack/react-query";

import { artists as artistsApi } from "@/api/artists";
import type { PaginationParams } from "@/api/types";

export const artistKeys = {
  all: () => ["artists"] as const,
  list: (params?: PaginationParams) => ["artists", "list", params] as const,
  detail: (id: string) => ["artists", "detail", id] as const,
};

export function useArtists(params?: PaginationParams, enabled = true) {
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
