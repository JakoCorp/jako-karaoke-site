import { PauseIcon, PlayIcon } from "@phosphor-icons/react";

import type { PerformanceSummary } from "@/api/performances";
import { formatDuration, formatRelativeDate } from "@/lib/format";
import { usePlayerStore } from "@/store/player";

interface Props {
  performance: PerformanceSummary;
}

/** A single row in the search results list. */
export function PerformanceRow({ performance }: Props) {
  const play = usePlayerStore((s) => s.play);
  const pause = usePlayerStore((s) => s.pause);
  const resume = usePlayerStore((s) => s.resume);
  const current = usePlayerStore((s) => s.current);
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
    play({
      id: performance.id,
      title: primaryTitle,
      singers: performance.singers,
      duration: performance.duration ?? null,
      audioUrl: null,
    });
  }

  return (
    <div className="perf-row">
      <button
        className={`perf-row-play${isCurrent ? " perf-row-play--active" : ""}`}
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
    </div>
  );
}
