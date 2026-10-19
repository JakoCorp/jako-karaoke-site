import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { ensureOk, unwrap } from "@/api/client";
import { type CreatePlaylistRequest, playlists } from "@/api/playlists";
import { errorMessage } from "@/lib/error-message";
import { toast } from "@/lib/toast";

const PLAYLISTS_PER_PAGE = 24;
const SHORT_TOAST_TIMEOUT_MS = 3000;

export const playlistKeys = {
  all: () => ["playlists"] as const,
  detail: (id: string) => ["playlists", "detail", id] as const,
  performances: (id: string) => ["playlists", "performances", id] as const,
  favorites: (userId: string) => ["playlists", "favorites", userId] as const,
};

export function usePlaylist(id: string) {
  return useQuery({
    queryKey: playlistKeys.detail(id),
    queryFn: async () => unwrap(await playlists.get(id)),
  });
}

/** Infinite scroll query for the public playlists list. */
export function useInfinitePublicPlaylists(q?: string) {
  return useInfiniteQuery({
    queryKey: ["playlists", "infinite", "public", { q }],
    queryFn: async ({ pageParam }) =>
      unwrap(await playlists.list({ page: pageParam, per_page: PLAYLISTS_PER_PAGE, q })),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const fetched = allPages.length * PLAYLISTS_PER_PAGE;
      return fetched < lastPage.total ? allPages.length + 1 : undefined;
    },
  });
}

/** Infinite scroll query for a user's playlist list. */
export function useInfiniteUserPlaylists(userId: string | null, q?: string) {
  return useInfiniteQuery({
    queryKey: ["playlists", "infinite", "user", userId, { q }],
    queryFn: async ({ pageParam }) =>
      unwrap(
        await playlists.listByUser(userId!, { page: pageParam, per_page: PLAYLISTS_PER_PAGE, q }),
      ),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const fetched = allPages.length * PLAYLISTS_PER_PAGE;
      return fetched < lastPage.total ? allPages.length + 1 : undefined;
    },
    enabled: !!userId,
  });
}

export function usePlaylistEntries(playlistId: string) {
  return useQuery({
    queryKey: playlistKeys.performances(playlistId),
    queryFn: async () => unwrap(await playlists.getPerformances(playlistId)),
  });
}

export function useUserFavorites(userId: string | null) {
  return useQuery({
    queryKey: playlistKeys.favorites(userId ?? ""),
    queryFn: async () => unwrap(await playlists.getFavorites(userId!)),
    enabled: !!userId,
  });
}

/** Returns the set of a user's playlist IDs that contain a given performance. */
export function usePlaylistsContaining(userId: string | null, performanceId: string) {
  return useQuery({
    queryKey: ["playlists", "containing", userId, performanceId],
    queryFn: async () => {
      const playlistIds = unwrap(await playlists.getPlaylistsContaining(userId!, performanceId));
      return new Set<string>(playlistIds.map(String));
    },
    enabled: !!userId && !!performanceId,
  });
}

/** Fetches all of a user's playlists (excluding favorites) for use in picker UI. */
export function useUserPlaylistsForPicker(userId: string | null) {
  return useQuery({
    queryKey: ["playlists", "picker", userId],
    queryFn: async () => {
      const { items } = unwrap(await playlists.listByUser(userId!, { per_page: 200 }));
      return [
        ...items.filter((p) => p.kind === "favorites"),
        ...items.filter((p) => p.kind !== "favorites"),
      ];
    },
    enabled: !!userId,
  });
}

/** Adds one or more performances to a playlist. */
export function useAddToPlaylist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      playlistId,
      performanceIds,
    }: {
      playlistId: string;
      playlistTitle: string;
      performanceIds: string[];
    }) => playlists.addPerformances(playlistId, performanceIds).then(ensureOk),
    onSuccess: (_, { playlistId, playlistTitle }) => {
      void queryClient.invalidateQueries({ queryKey: playlistKeys.performances(playlistId) });
      void queryClient.invalidateQueries({ queryKey: ["playlists", "containing"] });
      toast.success(`Added to ${playlistTitle}`, { timeout: SHORT_TOAST_TIMEOUT_MS });
    },
    onError: (error) => {
      toast.error("Could not add to playlist", {
        description: errorMessage(error, "Something went wrong."),
      });
    },
  });
}

/** Removes one performance from a playlist. */
export function useRemoveFromPlaylist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ playlistId, performanceId }: { playlistId: string; performanceId: string }) =>
      playlists.removePerformances(playlistId, [performanceId]).then(ensureOk),
    onSuccess: (_, { playlistId }) => {
      void queryClient.invalidateQueries({ queryKey: playlistKeys.performances(playlistId) });
      void queryClient.invalidateQueries({ queryKey: ["playlists", "containing"] });
      toast.success("Removed from playlist", { timeout: SHORT_TOAST_TIMEOUT_MS });
    },
    onError: (error) => {
      toast.error("Could not remove from playlist", {
        description: errorMessage(error, "Something went wrong."),
      });
    },
  });
}

/** Deletes a playlist. */
export function useDeletePlaylist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => playlists.delete(id).then(ensureOk),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["playlists", "infinite"] });
      void queryClient.invalidateQueries({ queryKey: ["playlists", "picker"] });
      toast.success("Playlist deleted");
    },
    onError: (error) => {
      toast.error("Could not delete playlist", {
        description: errorMessage(error, "Something went wrong."),
      });
    },
  });
}

/** Creates a playlist. Failures are left to the caller so a form can show them inline. */
export function useCreatePlaylist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreatePlaylistRequest) => playlists.create(body).then(unwrap),
    onSuccess: (playlist) => {
      void queryClient.invalidateQueries({ queryKey: ["playlists", "infinite", "public"] });
      void queryClient.invalidateQueries({ queryKey: ["playlists", "picker"] });
      if (playlist.created_by) {
        void queryClient.invalidateQueries({
          queryKey: ["playlists", "infinite", "user", playlist.created_by],
        });
      }
      toast.success("Playlist created", { description: playlist.title });
    },
  });
}
