import { PauseIcon, PlayIcon } from "@phosphor-icons/react";

import type { PerformanceSummary } from "@/api/performances";
import { formatDuration, formatRelativeDate } from "@/lib/format";
import { selectCurrent, usePlayerStore } from "@/store/player";

import { PerformanceRowMenu } from "./performance-menu";

interface Props {
  performance: PerformanceSummary;
  performances: readonly PerformanceSummary[];
  index: number;
}

/** A single row in the search results list. */
export function PerformanceRow({ performance, performances, index }: Props) {
  const playQueue = usePlayerStore((s) => s.playQueue);
  const pause = usePlayerStore((s) => s.pause);
  const resume = usePlayerStore((s) => s.resume);
  const current = usePlayerStore(selectCurrent);
  const isPlaying = usePlayerStore((s) => s.isPlaying);

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
    playQueue(performances, index, { type: "search" });
  }

  return (
    <PerformanceRowMenu performanceId={performance.id}>
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
          {showSongContext && <> · {performance.songs[0]?.title}</>}
        </span>
      </div>
      <div className="perf-row-plays">{performance.play_count.toLocaleString()}</div>
      <div className="perf-row-duration">
        {performance.duration !== null && performance.duration !== undefined
          ? formatDuration(performance.duration)
          : null}
      </div>
      <div className="perf-row-date">{formatRelativeDate(performance.performance_date)}</div>
    </PerformanceRowMenu>
  );
}
