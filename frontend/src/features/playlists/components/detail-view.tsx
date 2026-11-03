import type { PlaylistEntry, PlaylistResponse } from "@/api/playlists";

import { PlaylistDetailMenu } from "./menu";
import { PlaylistPerfTable } from "./perf-table";

interface Props {
  playlistId: string;
  playlist?: PlaylistResponse;
  entries: PlaylistEntry[];
  isLoading?: boolean;
  isOwner?: boolean;
  onDelete?: () => void;
}

export function PlaylistDetailView({
  playlistId,
  playlist,
  entries,
  isLoading = false,
  isOwner = false,
  onDelete,
}: Props) {
  const title = playlist?.title ?? "";

  return (
    <div>
      <div className="page-header">
        <div className="playlist-detail-title-row">
          <div className="page-title">{title}</div>
          {playlist && <PlaylistDetailMenu playlist={playlist} onDelete={onDelete} />}
        </div>
        {playlist?.description && <div className="playlist-detail-sub">{playlist.description}</div>}
      </div>
      {isLoading ? (
        <div className="page-empty">Loading…</div>
      ) : entries.length === 0 ? (
        <div className="page-empty">No performances in this playlist.</div>
      ) : (
        <PlaylistPerfTable
          entries={entries}
          playlistId={playlistId}
          playlistName={title}
          isOwner={isOwner}
        />
      )}
    </div>
  );
}
