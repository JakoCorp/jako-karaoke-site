import type { PlaylistEntry } from "@/api/playlists";

import { PlaylistDetailMenu } from "./menu";
import { PlaylistPerfTable } from "./perf-table";

interface Props {
  playlistId: string;
  title: string;
  description?: string | null;
  entries: PlaylistEntry[];
  isLoading?: boolean;
  isOwner?: boolean;
  onDelete?: () => void;
}

export function PlaylistDetailView({
  playlistId,
  title,
  description,
  entries,
  isLoading = false,
  isOwner = false,
  onDelete,
}: Props) {
  return (
    <div>
      <div className="page-header">
        <div className="playlist-detail-title-row">
          <div className="page-title">{title}</div>
          {onDelete && <PlaylistDetailMenu playlistTitle={title} onDelete={onDelete} />}
        </div>
        {description && <div className="playlist-detail-sub">{description}</div>}
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
