import { PauseIcon, PlayIcon } from "@phosphor-icons/react";

import { formatBytes, formatDuration, formatRelativeDate } from "@/lib/format";
import type { OfflineEntry } from "@/store/offline";
import { selectCurrent, usePlayerStore } from "@/store/player";

import { SavedRowMenu } from "./saved-row-menu";

interface Props {
  entry: OfflineEntry;
  queue: OfflineEntry[];
  index: number;
}

export function SavedRow({ entry, queue, index }: Props) {
  const playQueue = usePlayerStore((s) => s.playQueue);
  const pause = usePlayerStore((s) => s.pause);
  const resume = usePlayerStore((s) => s.resume);
  const current = usePlayerStore(selectCurrent);
  const isPlaying = usePlayerStore((s) => s.isPlaying);

  const performance = entry.performance;
  const isCurrent = current?.id === performance.id;
  const isActive = isCurrent && isPlaying;

  const primaryTitle = performance.title ?? performance.songs[0]?.title ?? "Untitled";
  const showSongContext =
    performance.title !== null && performance.title !== undefined && performance.songs.length > 0;
  const singers = performance.singers.map((s) => s.name).join(" & ");

  function handlePlay() {
    if (isCurrent) {
      if (isPlaying) {
        pause();
      } else {
        resume();
      }
      return;
    }
    playQueue(
      queue.map((saved) => saved.performance),
      index,
      { type: "local" },
    );
  }

  const indicatorClass = isCurrent
    ? isActive
      ? "perf-row-indicator perf-row-indicator--playing"
      : "perf-row-indicator perf-row-indicator--active"
    : "perf-row-indicator";

  return (
    <SavedRowMenu performanceId={performance.id} onPlay={handlePlay}>
      <div className={indicatorClass} aria-hidden="true">
        {isActive ? <PauseIcon size={14} weight="fill" /> : <PlayIcon size={14} weight="fill" />}
      </div>
      <div className="perf-row-info">
        <span className="perf-row-title">{primaryTitle}</span>
        <span className="perf-row-sub">
          {singers}
          {showSongContext && <> · {performance.songs[0]?.title}</>}
        </span>
      </div>
      <div className="perf-row-duration">
        {performance.duration !== null && performance.duration !== undefined
          ? formatDuration(performance.duration)
          : null}
      </div>
      <div className="perf-row-plays">{formatBytes(entry.sizeBytes)}</div>
      <div className="playlist-perf-added">{formatRelativeDate(entry.savedAt)}</div>
    </SavedRowMenu>
  );
}
