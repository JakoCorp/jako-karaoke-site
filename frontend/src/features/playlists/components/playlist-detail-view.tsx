import type { PlaylistEntry } from "@/api/playlists";

import { PlaylistPerfTable } from "./playlist-perf-table";

interface Props {
  playlistId: string;
  title: string;
  description?: string | null;
  entries: PlaylistEntry[];
  isLoading?: boolean;
}

export function PlaylistDetailView({
  playlistId,
  title,
  description,
  entries,
  isLoading = false,
}: Props) {
  return (
    <div>
      <div className="playlist-detail-header">
        <div className="playlist-detail-title">{title}</div>
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
