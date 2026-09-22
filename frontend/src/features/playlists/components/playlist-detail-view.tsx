import type { PlaylistEntry } from "@/api/playlists";

import { PlaylistDetailMenu } from "./playlist-menu";
import { PlaylistPerfTable } from "./playlist-perf-table";

interface Props {
  playlistId: string;
  title: string;
  description?: string | null;
  entries: PlaylistEntry[];
  isLoading?: boolean;
  onDelete?: () => void;
}

export function PlaylistDetailView({
  playlistId,
  title,
  description,
  entries,
  isLoading = false,
  onDelete,
}: Props) {
  return (
    <div>
      <div className="playlist-detail-header">
        <div className="playlist-detail-title-row">
          <div className="playlist-detail-title">{title}</div>
          {onDelete && <PlaylistDetailMenu playlistTitle={title} onDelete={onDelete} />}
        </div>
        {description && <div className="playlist-detail-sub">{description}</div>}
      </div>
      {isLoading ? (
        <div className="playlist-empty">Loading…</div>
      ) : entries.length === 0 ? (
        <div className="playlist-empty">No performances in this playlist.</div>
      ) : (
        <PlaylistPerfTable entries={entries} playlistId={playlistId} playlistName={title} />
      )}
    </div>
  );
}
