import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { type CreatePlaylistRequest, playlists } from "@/api/playlists";

const PLAYLISTS_PER_PAGE = 24;

export const playlistKeys = {
  all: () => ["playlists"] as const,
  detail: (id: string) => ["playlists", "detail", id] as const,
  performances: (id: string) => ["playlists", "performances", id] as const,
  favorites: (userId: string) => ["playlists", "favorites", userId] as const,
};

export function usePlaylist(id: string) {
  return useQuery({
    queryKey: playlistKeys.detail(id),
    queryFn: async () => {
      const { data, error } = await playlists.get(id);
      if (error) throw error;
      return data ?? null;
    },
  });
}

/** Infinite scroll query for the public playlists list. */
export function useInfinitePublicPlaylists(q?: string) {
  return useInfiniteQuery({
    queryKey: ["playlists", "infinite", "public", { q }],
    queryFn: async ({ pageParam }) => {
      const { data, error } = await playlists.list({
        page: pageParam,
        per_page: PLAYLISTS_PER_PAGE,
        q,
      });
      if (error) throw error;
      return data;
    },
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
    queryFn: async ({ pageParam }) => {
      const { data, error } = await playlists.listByUser(userId!, {
        page: pageParam,
        per_page: PLAYLISTS_PER_PAGE,
        q,
      });
      if (error) throw error;
      return data;
    },
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
    queryFn: async () => {
      const { data, error } = await playlists.getPerformances(playlistId);
      if (error) throw error;
      return data ?? [];
    },
  });
}

export function useUserFavorites(userId: string | null) {
  return useQuery({
    queryKey: playlistKeys.favorites(userId ?? ""),
    queryFn: async () => {
      const { data, error } = await playlists.getFavorites(userId!);
      if (error) throw error;
      return data ?? [];
    },
    enabled: !!userId,
  });
}

/** Returns the set of a user's playlist IDs that contain a given performance. */
export function usePlaylistsContaining(userId: string | null, performanceId: string) {
  return useQuery({
    queryKey: ["playlists", "containing", userId, performanceId],
    queryFn: async () => {
      const { data, error } = await playlists.getPlaylistsContaining(userId!, performanceId);
      if (error) throw error;
      return new Set<string>((data ?? []).map(String));
    },
    enabled: !!userId && !!performanceId,
  });
}

/** Fetches all of a user's playlists (excluding favorites) for use in picker UI. */
export function useUserPlaylistsForPicker(userId: string | null) {
  return useQuery({
    queryKey: ["playlists", "picker", userId],
    queryFn: async () => {
      const { data, error } = await playlists.listByUser(userId!, { per_page: 200 });
      if (error) throw error;
      const items = data?.items ?? [];
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
      performanceIds: string[];
    }) => playlists.addPerformances(playlistId, performanceIds),
    onSuccess: (_, { playlistId }) => {
      void queryClient.invalidateQueries({ queryKey: playlistKeys.performances(playlistId) });
      void queryClient.invalidateQueries({ queryKey: ["playlists", "containing"] });
    },
  });
}

export function useCreatePlaylist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreatePlaylistRequest) => playlists.create(body),
    onSuccess: (result) => {
      if (result.data) {
        void queryClient.invalidateQueries({ queryKey: ["playlists", "infinite", "public"] });
        void queryClient.invalidateQueries({ queryKey: ["playlists", "picker"] });
        if (result.data.created_by) {
          void queryClient.invalidateQueries({
            queryKey: ["playlists", "infinite", "user", result.data.created_by],
          });
        }
      }
    },
  });
}
