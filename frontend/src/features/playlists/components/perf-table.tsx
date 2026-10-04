import type { PlaylistEntry } from "@/api/playlists";

import { PlaylistPerfRow } from "./perf-row";

interface Props {
  entries: PlaylistEntry[];
  playlistId: string;
  playlistName: string;
  isOwner: boolean;
}

export function PlaylistPerfTable({ entries, playlistId, playlistName, isOwner }: Props) {
  return (
    <div>
      <div className="playlist-perf-row-header">
        <div />
        <div className="perf-header-label">Title</div>
        <div className="perf-header-label--right perf-header-label">Plays</div>
        <div className="perf-header-label--right perf-header-label">Duration</div>
        <div className="perf-header-label--right perf-header-label">Date</div>
        <div className="perf-header-label--right perf-header-label">Added</div>
        <div />
      </div>
      {entries.map((entry, index) => (
        <PlaylistPerfRow
          key={entry.id}
          entry={entry}
          entries={entries}
          index={index}
          playlistId={playlistId}
          playlistName={playlistName}
          isOwner={isOwner}
        />
      ))}
    </div>
  );
}
