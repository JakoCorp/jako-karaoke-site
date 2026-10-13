import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { authApi, toAuthUser } from "@/api/auth";
import { unwrap } from "@/api/client";
import { usersApi } from "@/api/users";
import { toast } from "@/lib/toast";
import { useAuthStore } from "@/store/auth";

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

/**
 * Changes another user's username as an admin and refreshes the cached user lists.
 *
 * When the signed in user renames themselves, the auth store is refreshed as well.
 */
export function useUpdateUsername() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ userId, username }: { userId: string; username: string }) =>
      unwrap(await usersApi.updateUsername(userId, username)),
    onSuccess: async (user) => {
      await queryClient.invalidateQueries({ queryKey: userKeys.all() });
      toast.success("Username changed");

      const { user: signedInUser, setUser } = useAuthStore.getState();
      if (signedInUser?.id === user.id) {
        const { data } = await authApi.me();
        if (data) setUser(toAuthUser(data));
      }
    },
  });
}
