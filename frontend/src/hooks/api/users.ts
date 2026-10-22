import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { authApi, toAuthUser } from "@/api/auth";
import { ensureOk, unwrap } from "@/api/client";
import { usersApi } from "@/api/users";
import { errorMessage } from "@/lib/error-message";
import { toast } from "@/lib/toast";
import { useAuthStore } from "@/store/auth";

export const userKeys = {
  all: () => ["users"] as const,
  list: (q?: string) => ["users", "infinite", q] as const,
  capabilities: (id: string) => ["users", "capabilities", id] as const,
  plays: (id: string, page: number) => ["users", "plays", id, page] as const,
  allPlays: (id: string) => ["users", "plays", id] as const,
};

const PLAYS_PER_PAGE = 20;
const USERS_PER_PAGE = 24;

/** Infinite scroll query for the users list. Appends pages as the user scrolls. */
export function useInfiniteUsers(q?: string) {
  return useInfiniteQuery({
    queryKey: userKeys.list(q),
    queryFn: async ({ pageParam }) =>
      unwrap(await usersApi.search({ q, page: pageParam, per_page: USERS_PER_PAGE })),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const fetched = allPages.length * USERS_PER_PAGE;
      return fetched < lastPage.total ? allPages.length + 1 : undefined;
    },
  });
}

export function useUserCapabilities(userId: string | undefined) {
  return useQuery({
    queryKey: userKeys.capabilities(userId ?? ""),
    queryFn: async () => unwrap(await usersApi.listCapabilities(userId!)),
    enabled: !!userId,
  });
}

export function useUserPlays(userId: string | undefined, page: number) {
  return useQuery({
    queryKey: userKeys.plays(userId ?? "", page),
    queryFn: async () => unwrap(await usersApi.listPlays(userId!, page, PLAYS_PER_PAGE)),
    enabled: !!userId,
  });
}

/** Removes a user's counted plays on one performance, or all of them when `performanceId` is null. */
export function useRemoveUserPlays() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      userId,
      performanceId,
    }: {
      userId: string;
      performanceId: string | null;
    }) => {
      const result =
        performanceId === null
          ? await usersApi.deletePlays(userId)
          : await usersApi.deletePerformancePlays(userId, performanceId);
      ensureOk(result);
    },
    onSuccess: async (_data, { userId }) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: userKeys.allPlays(userId) }),
        queryClient.invalidateQueries({ queryKey: ["performances"] }),
      ]);
      toast.success("Plays removed");
    },
    onError: (error) => {
      toast.error("Could not remove plays", {
        description: errorMessage(error, "Something went wrong."),
      });
    },
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
