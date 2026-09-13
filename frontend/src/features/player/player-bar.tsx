import {
  ShuffleIcon,
  MusicNotesPlusIcon,
  PauseIcon,
  PlayIcon,
  RepeatIcon,
  SkipBackIcon,
  SkipForwardIcon,
  SpeakerHighIcon,
  InfoIcon,
  ClockCountdownIcon,
  CornersOutIcon,
} from "@phosphor-icons/react";

import { selectCurrent, selectHasNext, selectHasPrev, usePlayerStore } from "@/store/player";

import { QueuePopup } from "./queue-popup";

export function MusicPlayer() {
  const current = usePlayerStore(selectCurrent);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const pause = usePlayerStore((s) => s.pause);
  const resume = usePlayerStore((s) => s.resume);
  const next = usePlayerStore((s) => s.next);
  const prev = usePlayerStore((s) => s.prev);
  const hasNext = usePlayerStore(selectHasNext);
  const hasPrev = usePlayerStore(selectHasPrev);
  const volume = usePlayerStore((s) => s.volume);
  const setVolume = usePlayerStore((s) => s.setVolume);

  if (!current) return null;

  const title = current.title ?? "No title";
  const artists = current.singers.map((s) => s.name).join(", ") || "Unknown artist";

  return (
    <div id="music-player" className="player-bar">
      <div id="song-info" className="flex min-w-0 items-center gap-4 pl-2">
        <div className="player-thumbnail" />
        <div className="min-w-0">
          <p className="player-title">{title}</p>
          <p className="player-artist">{artists}</p>
        </div>
        <button type="button" className="player-btn-sm" aria-label="Add to favorite">
          <MusicNotesPlusIcon size={20} />
        </button>
      </div>

      <div className="player-col">
        <div id="playback-control" className="flex items-center gap-4">
          <button type="button" className="player-btn hidden md:flex" aria-label="Shuffle">
            <ShuffleIcon size={18} />
          </button>

          <button
            type="button"
            className="player-btn"
            aria-label="Previous"
            disabled={!hasPrev}
            onClick={() => {
              prev();
            }}
          >
            <SkipBackIcon size={18} weight="fill" />
          </button>

          <button
            type="button"
            className="player-btn-play"
            aria-label={isPlaying ? "Pause" : "Play"}
            onClick={isPlaying ? pause : resume}
          >
            {isPlaying ? (
              <PauseIcon size={22} weight="fill" />
            ) : (
              <PlayIcon size={22} weight="fill" />
            )}
          </button>

          <button
            type="button"
            className="player-btn"
            aria-label="Next"
            disabled={!hasNext}
            onClick={() => {
              next();
            }}
          >
            <SkipForwardIcon size={18} weight="fill" />
          </button>

          <button type="button" className="player-btn hidden md:flex" aria-label="Repeat">
            <RepeatIcon size={18} />
          </button>
        </div>

        <div id="progress-control" className="hidden w-full max-w-xl items-center gap-3 lg:flex">
          <span className="w-10 text-right player-time">0:00</span>

          <input type="range" min="0" max="100" defaultValue="0" className="player-range" />
          <span className="w-10 player-time">0:00</span>
        </div>
      </div>

      <div id="button-controls" className="hidden items-center justify-end gap-1 lg:flex">
        <button type="button" className="player-btn" aria-label="Sleep timer">
          <ClockCountdownIcon size={20} />
        </button>

        <button type="button" className="player-btn" aria-label="Song info">
          <InfoIcon size={20} />
        </button>

        <QueuePopup />

        <div id="volume-control" className="player-volume-control">
          <button type="button" className="player-btn hidden lg:flex" aria-label="Volume">
            <SpeakerHighIcon size={20} />
          </button>

          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={volume}
            onChange={(e) => setVolume(e.target.valueAsNumber)}
            className="player-volume-range"
          />
        </div>

        <button type="button" className="player-btn" aria-label="Fullscreen">
          <CornersOutIcon size={20} />
        </button>
      </div>
    </div>
  );
}
