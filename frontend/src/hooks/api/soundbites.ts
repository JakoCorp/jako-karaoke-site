import { keepPreviousData, useQuery } from "@tanstack/react-query";

import { unwrap } from "@/api/client";
import { soundbitesApi } from "@/api/soundbites";
import type { SoundbiteListParams } from "@/api/soundbites";

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
