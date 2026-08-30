import { useQuery } from "@tanstack/react-query";

import { tags as tagsApi } from "@/api/tags";

export const tagKeys = {
  all: () => ["tags"] as const,
};

export function useTags(enabled = true) {
  return useQuery({
    queryKey: tagKeys.all(),
    queryFn: async () => {
      const { data, error } = await tagsApi.list();
      if (error) throw error;
      return data ?? [];
    },
    enabled,
  });
}
