import type { PlaylistResponse } from "@/api/playlists";

import { PlaylistCard } from "./card";

interface Props {
  playlists: PlaylistResponse[];
  showVisibility?: boolean;
  emptyMessage?: string;
  onDelete?: (id: string) => void;
}

export function PlaylistGrid({
  playlists,
  showVisibility = false,
  emptyMessage = "No playlists found.",
  onDelete,
}: Props) {
  if (playlists.length === 0) {
    return <div className="playlist-empty">{emptyMessage}</div>;
  }
  return (
    <div className="playlist-grid">
      {playlists.map((playlist) => (
        <PlaylistCard
          key={playlist.id}
          playlist={playlist}
          showVisibility={showVisibility}
          onDelete={onDelete ? () => onDelete(playlist.id) : undefined}
        />
      ))}
    </div>
  );
}
