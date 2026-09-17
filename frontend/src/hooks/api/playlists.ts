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

export function useCreatePlaylist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreatePlaylistRequest) => playlists.create(body),
    onSuccess: (result) => {
      if (result.data) {
        void queryClient.invalidateQueries({ queryKey: ["playlists", "infinite", "public"] });
        if (result.data.created_by) {
          void queryClient.invalidateQueries({
            queryKey: ["playlists", "infinite", "user", result.data.created_by],
          });
        }
      }
    },
  });
}
