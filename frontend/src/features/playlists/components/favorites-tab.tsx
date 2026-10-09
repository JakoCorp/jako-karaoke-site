import { useUserFavorites } from "@/hooks/api/playlists";

import { PlaylistPerfTable } from "./perf-table";

interface Props {
  userId: string;
}

export function FavoritesPlaylistTab({ userId }: Props) {
  const { data: entries, isLoading } = useUserFavorites(userId);

  if (isLoading) return <div className="page-empty">Loading…</div>;
  if (!entries?.length) return <div className="page-empty">No performances in your favorites.</div>;

  return (
    <PlaylistPerfTable
      entries={entries}
      playlistId="favorites"
      playlistName="Favorites"
      isOwner={false}
    />
  );
}
