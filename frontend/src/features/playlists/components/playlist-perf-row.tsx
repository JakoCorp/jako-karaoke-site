import { PauseIcon, PlayIcon } from "@phosphor-icons/react";

import type { PlaylistEntry } from "@/api/playlists";
import { formatDuration, formatRelativeDate } from "@/lib/format";
import { selectCurrent, usePlayerStore } from "@/store/player";

interface Props {
  entry: PlaylistEntry;
  entries: PlaylistEntry[];
  index: number;
  playlistId: string;
  playlistName: string;
}

export function PlaylistPerfRow({ entry, entries, index, playlistId, playlistName }: Props) {
  const playQueue = usePlayerStore((s) => s.playQueue);
  const pause = usePlayerStore((s) => s.pause);
  const resume = usePlayerStore((s) => s.resume);
  const current = usePlayerStore(selectCurrent);
  const isPlaying = usePlayerStore((s) => s.isPlaying);

  const isCurrent = current?.id === entry.id;
  const isActive = isCurrent && isPlaying;

  const primaryTitle = entry.title ?? entry.songs[0]?.title ?? "Untitled";
  const showSongContext =
    entry.title !== null && entry.title !== undefined && entry.songs.length > 0;
  const singers = entry.singers.map((s) => s.name).join(" & ");

  function handlePlay() {
    if (isCurrent) {
      if (isPlaying) {
        pause();
      } else {
        resume();
      }
      return;
    }
    playQueue(entries, index, { type: "playlist", id: playlistId, name: playlistName });
  }

  return (
    <div className="playlist-perf-row">
      <button
        className={isCurrent ? "perf-row-play perf-row-play--active" : "perf-row-play"}
        onClick={handlePlay}
        aria-label={isActive ? "Pause" : "Play"}
      >
        {isActive ? <PauseIcon size={14} weight="fill" /> : <PlayIcon size={14} weight="fill" />}
      </button>
      <div className="perf-row-info">
        <span className="perf-row-title">{primaryTitle}</span>
        <span className="perf-row-sub">
          {singers}
          {showSongContext && <> · {entry.songs[0]?.title}</>}
        </span>
      </div>
      <div className="perf-row-plays">{entry.play_count.toLocaleString()}</div>
      <div className="perf-row-duration">
        {entry.duration !== null && entry.duration !== undefined
          ? formatDuration(entry.duration)
          : null}
      </div>
      <div className="perf-row-date">{formatRelativeDate(entry.performance_date)}</div>
      <div className="playlist-perf-added">{formatRelativeDate(entry.added_at)}</div>
    </div>
  );
}
