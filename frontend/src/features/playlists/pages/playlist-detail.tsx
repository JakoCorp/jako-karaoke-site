import { useNavigate, useParams } from "react-router";

import { useDeletePlaylist, usePlaylist, usePlaylistEntries } from "@/hooks/api/playlists";
import { useAuthStore } from "@/store/auth";

import { PlaylistDetailView } from "../components/playlist-detail-view";

export function PlaylistDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const user = useAuthStore((s) => s.user);
  const { data: playlist, isLoading: playlistLoading } = usePlaylist(id!);
  const { data: entries, isLoading: entriesLoading } = usePlaylistEntries(id!);
  const deletePlaylist = useDeletePlaylist();

  if (!playlistLoading && playlist === null) {
    return <div className="playlist-empty">Playlist not found.</div>;
  }

  const isOwner = !!user && playlist?.created_by === user.id && playlist?.kind !== "favorites";

  return (
    <PlaylistDetailView
      playlistId={id!}
      title={playlist?.title ?? ""}
      description={playlist?.description}
      entries={entries ?? []}
      isLoading={playlistLoading || entriesLoading}
      onDelete={
        isOwner
          ? () =>
              deletePlaylist.mutate(id!, {
                onSuccess: () => void navigate("/my-playlists"),
              })
          : undefined
      }
    />
  );
}
