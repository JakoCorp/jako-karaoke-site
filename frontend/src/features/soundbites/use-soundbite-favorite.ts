import type { SoundbiteResponse } from "@/api/soundbites";
import { useIsFavoriteSoundbite, useSetFavoriteSoundbite } from "@/hooks/api/soundbites";
import { toast } from "@/lib/toast";
import { useAuthStore } from "@/store/auth";

interface SoundbiteFavoriteAction {
  isFavorite: boolean;
  label: string;
  toggle: () => void;
}

/** Returns the favorite state of a soundbite for the signed in user and the action to flip it. */
export function useSoundbiteFavoriteAction(soundbite: SoundbiteResponse): SoundbiteFavoriteAction {
  const userId = useAuthStore((state) => state.user?.id ?? null);
  const isFavorite = useIsFavoriteSoundbite(userId, soundbite.id);
  const setFavorite = useSetFavoriteSoundbite();

  return {
    isFavorite,
    label: isFavorite ? "Remove from favorites" : "Add to favorites",
    toggle: () => {
      if (userId === null) {
        toast.info("Sign in to favorite soundbites");
        return;
      }
      setFavorite.mutate({ userId, soundbiteId: soundbite.id, favorite: !isFavorite });
    },
  };
}
