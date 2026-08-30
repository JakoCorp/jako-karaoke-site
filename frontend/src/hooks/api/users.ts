import { useQuery } from "@tanstack/react-query";

import { users } from "@/api/users";

export const userKeys = {
  all: () => ["users"] as const,
  list: (q?: string) => ["users", "list", q] as const,
  capabilities: (id: string) => ["users", "capabilities", id] as const,
};

export function useUsers(q?: string) {
  return useQuery({
    queryKey: userKeys.list(q),
    queryFn: async () => {
      const { data, error } = await users.search(q);
      if (error) throw error;
      return data ?? [];
    },
  });
}

export function useUserCapabilities(userId: string | undefined) {
  return useQuery({
    queryKey: userKeys.capabilities(userId ?? ""),
    queryFn: async () => {
      const { data, error } = await users.listCapabilities(userId!);
      if (error) throw error;
      return data ?? [];
    },
    enabled: !!userId,
  });
}
