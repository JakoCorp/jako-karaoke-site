import { useQuery } from "@tanstack/react-query";

import { capabilitiesApi } from "@/api/capabilities";

export const capabilityKeys = {
  all: () => ["capabilities"] as const,
};

export function useCapabilities() {
  return useQuery({
    queryKey: capabilityKeys.all(),
    queryFn: async () => {
      const { data, error } = await capabilitiesApi.list();
      if (error) throw error;
      return data ?? [];
    },
  });
}
