import { useQuery } from "@tanstack/react-query";

import { unwrap } from "@/api/client";
import { tagsApi } from "@/api/tags";

export const tagKeys = {
  all: () => ["tags"] as const,
};

export function useTags(enabled = true) {
  return useQuery({
    queryKey: tagKeys.all(),
    queryFn: async () => unwrap(await tagsApi.list()),
    enabled,
  });
}
