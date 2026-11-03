import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { authApi, toAuthUser } from "@/api/auth";
import { ensureOk, unwrap } from "@/api/client";
import { usersApi } from "@/api/users";
import { soundbiteKeys } from "@/hooks/api/soundbites";
import { errorMessage } from "@/lib/error-message";
import { toast } from "@/lib/toast";
import { useAuthStore } from "@/store/auth";

export const userKeys = {
  all: () => ["users"] as const,
  list: (q?: string) => ["users", "infinite", q] as const,
  capabilities: (id: string) => ["users", "capabilities", id] as const,
  performancePlays: (id: string, page: number) =>
    ["users", "plays", "performances", id, page] as const,
  allPerformancePlays: (id: string) => ["users", "plays", "performances", id] as const,
  soundbitePlays: (id: string, page: number) => ["users", "plays", "soundbites", id, page] as const,
  allSoundbitePlays: (id: string) => ["users", "plays", "soundbites", id] as const,
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

export function useUserPerformancePlays(userId: string | undefined, page: number, enabled = true) {
  return useQuery({
    queryKey: userKeys.performancePlays(userId ?? "", page),
    queryFn: async () => unwrap(await usersApi.listPerformancePlays(userId!, page, PLAYS_PER_PAGE)),
    enabled: !!userId && enabled,
  });
}

export function useUserSoundbitePlays(userId: string | undefined, page: number, enabled = true) {
  return useQuery({
    queryKey: userKeys.soundbitePlays(userId ?? "", page),
    queryFn: async () => unwrap(await usersApi.listSoundbitePlays(userId!, page, PLAYS_PER_PAGE)),
    enabled: !!userId && enabled,
  });
}

/** Removes a user's counted plays on one performance, or all of them when `performanceId` is null. */
export function useRemoveUserPerformancePlays() {
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
          ? await usersApi.deleteAllPerformancePlays(userId)
          : await usersApi.deletePlaysOnPerformance(userId, performanceId);
      ensureOk(result);
    },
    onSuccess: async (_data, { userId }) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: userKeys.allPerformancePlays(userId) }),
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

/** Removes a user's counted plays on one soundbite, or all of them when `soundbiteId` is null. */
export function useRemoveUserSoundbitePlays() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ userId, soundbiteId }: { userId: string; soundbiteId: string | null }) => {
      const result =
        soundbiteId === null
          ? await usersApi.deleteAllSoundbitePlays(userId)
          : await usersApi.deletePlaysOnSoundbite(userId, soundbiteId);
      ensureOk(result);
    },
    onSuccess: async (_data, { userId }) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: userKeys.allSoundbitePlays(userId) }),
        queryClient.invalidateQueries({ queryKey: soundbiteKeys.all() }),
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
