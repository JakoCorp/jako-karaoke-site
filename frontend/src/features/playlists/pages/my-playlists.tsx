import { useState } from "react";
import { useNavigate } from "react-router";

import { useUserPlaylists } from "@/hooks/api/playlists";
import { useAuthStore } from "@/store/auth";

import { CreatePlaylistDialog } from "../components/create-playlist-dialog";
import { PlaylistGrid } from "../components/playlist-grid";

export function MyPlaylistsPage() {
  const user = useAuthStore((s) => s.user);
  const { data, isLoading } = useUserPlaylists(user?.id ?? null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const navigate = useNavigate();

  const userPlaylists = data?.filter((p) => p.kind === "user") ?? [];

  if (!user) {
    return <div className="playlist-empty">Sign in to see your playlists.</div>;
  }

  return (
    <div>
      <div className="playlist-page-header">
        <div className="playlist-page-header-row">
          <div className="playlist-page-title">My Playlists</div>
          <button type="button" className="btn btn-primary" onClick={() => setDialogOpen(true)}>
            New playlist
          </button>
        </div>
        {!isLoading && (
          <div className="playlist-page-sub">
            {userPlaylists.length.toLocaleString()}{" "}
            {userPlaylists.length === 1 ? "playlist" : "playlists"}
          </div>
        )}
      </div>
      {isLoading ? (
        <div className="playlist-empty">Loading…</div>
      ) : (
        <PlaylistGrid
          playlists={userPlaylists}
          showVisibility
          emptyMessage="You have no playlists."
        />
      )}
      <CreatePlaylistDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onCreated={(playlist) => {
          void navigate(`/playlist/${playlist.id}`);
        }}
      />
    </div>
  );
}
