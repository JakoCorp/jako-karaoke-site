import { useQuery } from "@tanstack/react-query";

import { capabilitiesApi } from "@/api/capabilities";
import { unwrap } from "@/api/client";

export const capabilityKeys = {
  all: () => ["capabilities"] as const,
};

export function useCapabilities() {
  return useQuery({
    queryKey: capabilityKeys.all(),
    queryFn: async () => unwrap(await capabilitiesApi.list()),
  });
}
