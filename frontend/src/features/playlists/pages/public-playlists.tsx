import { usePublicPlaylists } from "@/hooks/api/playlists";

import { PlaylistGrid } from "../components/playlist-grid";

export function PublicPlaylistsPage() {
  const { data, isLoading } = usePublicPlaylists();

  const publicPlaylists = data?.filter((p) => p.kind !== "favorites") ?? [];

  return (
    <div>
      <div className="playlist-page-header">
        <div className="playlist-page-title">Public Playlists</div>
        {!isLoading && (
          <div className="playlist-page-sub">
            {publicPlaylists.length.toLocaleString()}{" "}
            {publicPlaylists.length === 1 ? "playlist" : "playlists"}
          </div>
        )}
      </div>
      {isLoading ? (
        <div className="playlist-empty">Loading…</div>
      ) : (
        <PlaylistGrid playlists={publicPlaylists} emptyMessage="No public playlists yet." />
      )}
    </div>
  );
}
