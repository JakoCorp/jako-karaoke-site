import { FileArrowDownIcon, HeartIcon, PlayIcon, StopIcon } from "@phosphor-icons/react";
import type { KeyboardEvent } from "react";

import type { SoundbiteResponse } from "@/api/soundbites";
import { RowContextMenu } from "@/components/row-context-menu";
import type { RowMenuItemDef } from "@/components/row-context-menu";
import { useSoundbiteDownloadAction } from "@/features/download";
import { resolveAssetUrl } from "@/lib/asset-url";
import { formatDate, formatDuration } from "@/lib/format";

import { useSoundbiteFavoriteAction } from "../use-soundbite-favorite";

interface Props {
  soundbite: SoundbiteResponse;
  isActive: boolean;
  onToggle: (id: string, url: string) => void;
}

// The global shortcuts take Space on the document, which would toggle the main player.
function keepSpaceKey(event: KeyboardEvent) {
  if (event.key === " ") event.stopPropagation();
}

/**
 * A single row in the soundbites list. Clicking it plays or stops the soundbite, the buttons on
 * its right and the right click menu act on the soundbite.
 */
export function SoundbiteRow({ soundbite, isActive, onToggle }: Props) {
  const favorite = useSoundbiteFavoriteAction(soundbite);
  const download = useSoundbiteDownloadAction(soundbite);
  const audioUrl = resolveAssetUrl(soundbite.audio);
  const artists = soundbite.artists.map((artist) => artist.name).join(" & ");
  const hasStreamPosition = soundbite.stream_number != null && soundbite.soundbite_number != null;
  const heartWeight = favorite.isFavorite ? "fill" : "regular";

  const items: RowMenuItemDef[] = [
    {
      icon: <HeartIcon size={14} weight={heartWeight} />,
      label: favorite.label,
      onClick: favorite.toggle,
    },
    {
      icon: <FileArrowDownIcon size={14} />,
      label: download.label,
      disabled: !download.isAvailable,
      onClick: download.download,
    },
  ];

  return (
    <RowContextMenu items={items} triggerClassName="soundbite-row" menuButton={false}>
      <button
        type="button"
        className="soundbite-row-main"
        disabled={audioUrl === undefined}
        onClick={() => {
          if (audioUrl !== undefined) onToggle(soundbite.id, audioUrl);
        }}
        onKeyDown={keepSpaceKey}
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
        </div>
        <div className="soundbite-row-artists" title={artists}>
          {artists}
        </div>
        <div className="perf-row-plays">{soundbite.play_count.toLocaleString()}</div>
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
      <div className="soundbite-row-actions">
        <button
          type="button"
          className="soundbite-row-action"
          aria-label={`Favorite ${soundbite.title}`}
          aria-pressed={favorite.isFavorite}
          data-tooltip={favorite.label}
          onClick={favorite.toggle}
          onKeyDown={keepSpaceKey}
        >
          <HeartIcon size={16} weight={heartWeight} />
        </button>
        <button
          type="button"
          className="soundbite-row-action"
          aria-label={`${download.label} ${soundbite.title}`}
          data-tooltip={download.label}
          disabled={!download.isAvailable}
          onClick={download.download}
          onKeyDown={keepSpaceKey}
        >
          <FileArrowDownIcon size={16} />
        </button>
      </div>
    </RowContextMenu>
  );
}
