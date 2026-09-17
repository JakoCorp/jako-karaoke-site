import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { type CreatePlaylistRequest, playlists } from "@/api/playlists";

export const playlistKeys = {
  all: () => ["playlists"] as const,
  list: () => ["playlists", "list"] as const,
  listByUser: (userId: string) => ["playlists", "list", "user", userId] as const,
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

export function usePublicPlaylists() {
  return useQuery({
    queryKey: playlistKeys.list(),
    queryFn: async () => {
      const { data, error } = await playlists.list();
      if (error) throw error;
      return data ?? [];
    },
  });
}

export function useUserPlaylists(userId: string | null) {
  return useQuery({
    queryKey: playlistKeys.listByUser(userId ?? ""),
    queryFn: async () => {
      const { data, error } = await playlists.listByUser(userId!);
      if (error) throw error;
      return data ?? [];
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
        void queryClient.invalidateQueries({ queryKey: playlistKeys.list() });
        if (result.data.created_by) {
          void queryClient.invalidateQueries({
            queryKey: playlistKeys.listByUser(result.data.created_by),
          });
        }
      }
    },
  });
}
