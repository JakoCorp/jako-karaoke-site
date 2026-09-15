import { Link } from "react-router";

import type { PlaylistResponse } from "@/api/playlists";

interface Props {
  playlist: PlaylistResponse;
  showVisibility?: boolean;
}

export function PlaylistCard({ playlist, showVisibility = false }: Props) {
  const count = playlist.performance_count;
  return (
    <Link to={`/playlist/${playlist.id}`} className="playlist-card">
      <div className="playlist-card-title" title={playlist.title}>
        {playlist.title}
      </div>
      {playlist.description && <div className="playlist-card-sub">{playlist.description}</div>}
      <div className="playlist-card-meta">
        <span>
          {count.toLocaleString()} {count === 1 ? "track" : "tracks"}
        </span>
        {showVisibility && (
          <span className="playlist-card-badge">{playlist.is_public ? "Public" : "Private"}</span>
        )}
      </div>
    </Link>
  );
}
