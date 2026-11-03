import { PlayIcon, StopIcon } from "@phosphor-icons/react";

import type { SoundbiteResponse } from "@/api/soundbites";
import { resolveAssetUrl } from "@/lib/asset-url";
import { formatDate, formatDuration } from "@/lib/format";

interface Props {
  soundbite: SoundbiteResponse;
  isActive: boolean;
  onToggle: (id: string, url: string) => void;
}

/** A single row in the soundbites list. Clicking it plays or stops the soundbite. */
export function SoundbiteRow({ soundbite, isActive, onToggle }: Props) {
  const audioUrl = resolveAssetUrl(soundbite.audio);
  const artists = soundbite.artists.map((artist) => artist.name).join(" & ");
  const hasStreamPosition = soundbite.stream_number != null && soundbite.soundbite_number != null;

  return (
    <button
      type="button"
      className="soundbite-row"
      disabled={audioUrl === undefined}
      onClick={() => {
        if (audioUrl !== undefined) onToggle(soundbite.id, audioUrl);
      }}
      onKeyDown={(event) => {
        // The global shortcuts take Space key on the document, which would toggle the main player.
        if (event.key === " ") event.stopPropagation();
      }}
    >
      <div
        className={
          isActive ? "perf-row-indicator perf-row-indicator--playing" : "perf-row-indicator"
        }
        aria-hidden="true"
      >
        {isActive ? <StopIcon size={14} weight="fill" /> : <PlayIcon size={14} weight="fill" />}
      </div>
      <div className="perf-row-info">
        <span className="perf-row-title">{soundbite.title}</span>
        <span className="perf-row-sub">{artists}</span>
      </div>
      <div className="perf-row-duration">
        {audioUrl === undefined
          ? "No audio"
          : soundbite.duration != null
            ? formatDuration(soundbite.duration)
            : null}
      </div>
      <div className="perf-row-date">
        <span>{formatDate(soundbite.soundbite_date)}</span>
        {hasStreamPosition && (
          <span className="perf-row-date-id">
            S{soundbite.stream_number} · #{soundbite.soundbite_number}
          </span>
        )}
      </div>
    </button>
  );
}
