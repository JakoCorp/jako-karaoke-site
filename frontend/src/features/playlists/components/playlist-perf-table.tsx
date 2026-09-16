import type { PlaylistEntry } from "@/api/playlists";

import { PlaylistPerfRow } from "./playlist-perf-row";

interface Props {
  entries: PlaylistEntry[];
  playlistId: string;
  playlistName: string;
}

export function PlaylistPerfTable({ entries, playlistId, playlistName }: Props) {
  return (
    <div>
      <div className="playlist-perf-row-header">
        <div />
        <div className="perf-header-label">Title</div>
        <div className="perf-header-label--right perf-header-label">Plays</div>
        <div className="perf-header-label--right perf-header-label">Duration</div>
        <div className="perf-header-label--right perf-header-label">Date</div>
        <div className="perf-header-label--right perf-header-label">Added</div>
      </div>
      {entries.map((entry, index) => (
        <PlaylistPerfRow
          key={entry.id}
          entry={entry}
          entries={entries}
          index={index}
          playlistId={playlistId}
          playlistName={playlistName}
        />
      ))}
    </div>
  );
}
