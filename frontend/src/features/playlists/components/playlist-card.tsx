import { Link } from "react-router";

import type { PlaylistResponse } from "@/api/playlists";

import { PlaylistCardMenu } from "./playlist-menu";

interface Props {
  playlist: PlaylistResponse;
  showVisibility?: boolean;
  onDelete?: () => void;
}

export function PlaylistCard({ playlist, showVisibility = false, onDelete }: Props) {
  const count = playlist.performance_count;

  const cardContent = (
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

  if (onDelete) {
    return (
      <PlaylistCardMenu playlistTitle={playlist.title} onDelete={onDelete}>
        {cardContent}
      </PlaylistCardMenu>
    );
  }

  return cardContent;
}
