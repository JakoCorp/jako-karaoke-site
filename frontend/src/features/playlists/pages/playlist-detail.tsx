import { useParams } from "react-router";

import { usePlaylist, usePlaylistEntries } from "@/hooks/api/playlists";

import { PlaylistDetailView } from "../components/playlist-detail-view";

export function PlaylistDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data: playlist, isLoading: playlistLoading } = usePlaylist(id!);
  const { data: entries, isLoading: entriesLoading } = usePlaylistEntries(id!);

  if (!playlistLoading && playlist === null) {
    return <div className="playlist-empty">Playlist not found.</div>;
  }

  return (
    <PlaylistDetailView
      playlistId={id!}
      title={playlist?.title ?? ""}
      description={playlist?.description}
      entries={entries ?? []}
      isLoading={playlistLoading || entriesLoading}
    />
  );
}
