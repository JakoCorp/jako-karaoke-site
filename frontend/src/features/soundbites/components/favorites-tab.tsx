import { SoundbiteBrowser } from "./soundbite-browser";

interface Props {
  userId: string;
}

export function FavoriteSoundbitesTab({ userId }: Props) {
  return <SoundbiteBrowser favoritesOf={userId} />;
}
