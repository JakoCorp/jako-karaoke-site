import {
  keepPreviousData,
  queryOptions,
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";

import { ensureOk, unwrap } from "@/api/client";
import { soundbitesApi } from "@/api/soundbites";
import type { SoundbiteListParams } from "@/api/soundbites";
import type { WithoutPaging } from "@/api/types";
import { usersApi } from "@/api/users";
import { errorMessage } from "@/lib/error-message";
import { toast } from "@/lib/toast";

export const soundbiteKeys = {
  all: () => ["soundbites"] as const,
  list: (params?: SoundbiteListParams) => ["soundbites", "list", params] as const,
  players: (id: string, page: number) => ["soundbites", "players", id, page] as const,
  favorites: (userId: string) => ["soundbites", "favorites", userId] as const,
  favoriteIds: (userId: string) => ["soundbites", "favorites", userId, "ids"] as const,
};

function favoriteIdsQueryOptions(userId: string | null) {
  return queryOptions({
    queryKey: soundbiteKeys.favoriteIds(userId ?? ""),
    queryFn: async () => unwrap(await usersApi.listFavoriteSoundbiteIds(userId!)),
    enabled: !!userId,
  });
}

const PLAYERS_PER_PAGE = 20;

export function useSoundbites(params?: SoundbiteListParams) {
  return useQuery({
    queryKey: soundbiteKeys.list(params),
    queryFn: async () => unwrap(await soundbitesApi.list(params)),
    placeholderData: keepPreviousData,
  });
}

export function useSoundbitePlayers(id: string, page: number, enabled = true) {
  return useQuery({
    queryKey: soundbiteKeys.players(id, page),
    queryFn: async () => unwrap(await soundbitesApi.listPlayers(id, page, PLAYERS_PER_PAGE)),
    enabled,
  });
}

/** Whether the user has favorited a soundbite. Rows share one cached list of the user's favorite IDs. */
export function useIsFavoriteSoundbite(userId: string | null, soundbiteId: string): boolean {
  const { data } = useQuery({
    ...favoriteIdsQueryOptions(userId),
    select: (ids) => ids.includes(soundbiteId),
  });
  return data ?? false;
}

/** Adds or removes a soundbite from a user's favorites, updating the cached IDs at once. */
export function useSetFavoriteSoundbite() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      userId,
      soundbiteId,
      favorite,
    }: {
      userId: string;
      soundbiteId: string;
      favorite: boolean;
    }) => {
      ensureOk(
        favorite
          ? await usersApi.addFavoriteSoundbite(userId, soundbiteId)
          : await usersApi.removeFavoriteSoundbite(userId, soundbiteId),
      );
    },
    onMutate: async ({ userId, soundbiteId, favorite }) => {
      const queryKey = soundbiteKeys.favoriteIds(userId);
      await queryClient.cancelQueries({ queryKey });
      queryClient.setQueryData<string[]>(queryKey, (ids) => {
        if (!ids) return ids;
        const others = ids.filter((id) => id !== soundbiteId);
        return favorite ? [...others, soundbiteId] : others;
      });
    },
    onError: (error) => {
      toast.error("Could not update favorite", {
        description: errorMessage(error, "Something went wrong."),
      });
    },
    onSettled: async (_data, _error, { userId }) => {
      await queryClient.invalidateQueries({ queryKey: soundbiteKeys.favorites(userId) });
    },
  });
}

/** Infinite scroll query for the admin soundbites list. Appends pages as the user scrolls. */
export function useInfiniteSoundbites(params?: WithoutPaging<SoundbiteListParams>) {
  const SOUNDBITES_PER_PAGE = 50;

  return useInfiniteQuery({
    queryKey: ["soundbites", "infinite", params],
    queryFn: async ({ pageParam }) =>
      unwrap(
        await soundbitesApi.list({ ...params, page: pageParam, per_page: SOUNDBITES_PER_PAGE }),
      ),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const fetched = allPages.length * SOUNDBITES_PER_PAGE;
      return fetched < lastPage.total ? allPages.length + 1 : undefined;
    },
  });
}
