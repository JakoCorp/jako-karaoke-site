import { useQuery } from "@tanstack/react-query";

import { usersApi } from "@/api/users";

export const userKeys = {
  all: () => ["users"] as const,
  list: (q?: string) => ["users", "list", q] as const,
  capabilities: (id: string) => ["users", "capabilities", id] as const,
};

export function useUsers(q?: string) {
  return useQuery({
    queryKey: userKeys.list(q),
    queryFn: async () => {
      const { data, error } = await usersApi.search(q);
      if (error) throw error;
      return data ?? [];
    },
  });
}

export function useUserCapabilities(userId: string | undefined) {
  return useQuery({
    queryKey: userKeys.capabilities(userId ?? ""),
    queryFn: async () => {
      const { data, error } = await usersApi.listCapabilities(userId!);
      if (error) throw error;
      return data ?? [];
    },
    enabled: !!userId,
  });
}
