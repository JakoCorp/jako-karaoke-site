import { Dialog } from "@base-ui/react";
import {
  PauseIcon,
  PlayIcon,
  RepeatIcon,
  RepeatOnceIcon,
  ShuffleIcon,
  SkipBackIcon,
  SkipForwardIcon,
  SpeakerHighIcon,
  SpeakerLowIcon,
  SpeakerSlashIcon,
  XIcon,
} from "@phosphor-icons/react";
import { useState } from "react";

import { usePerformance } from "@/hooks/api/performances";
import { resolveAssetUrl } from "@/lib/asset-url";
import { formatDuration } from "@/lib/format";
import { selectCurrent, selectHasNext, selectHasPrev, usePlayerStore } from "@/store/player";

interface FullscreenPlayerProps {
  open: boolean;
  onClose: () => void;
  currentTime: number;
  duration: number;
  seek: (time: number) => void;
}

export function FullscreenPlayer({
  open,
  onClose,
  currentTime,
  duration,
  seek,
}: FullscreenPlayerProps) {
  const [isSeeking, setIsSeeking] = useState(false);
  const [seekValue, setSeekValue] = useState(0);

  const current = usePlayerStore(selectCurrent);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const pause = usePlayerStore((s) => s.pause);
  const resume = usePlayerStore((s) => s.resume);
  const next = usePlayerStore((s) => s.next);
  const prev = usePlayerStore((s) => s.prev);
  const hasNext = usePlayerStore(selectHasNext);
  const hasPrev = usePlayerStore(selectHasPrev);
  const volume = usePlayerStore((s) => s.volume);
  const isMuted = usePlayerStore((s) => s.isMuted);
  const setVolume = usePlayerStore((s) => s.setVolume);
  const toggleMute = usePlayerStore((s) => s.toggleMute);
  const shuffleEnabled = usePlayerStore((s) => s.shuffleEnabled);
  const toggleShuffle = usePlayerStore((s) => s.toggleShuffle);
  const repeatMode = usePlayerStore((s) => s.repeatMode);
  const cycleRepeatMode = usePlayerStore((s) => s.cycleRepeatMode);

  const { data: detail } = usePerformance(current?.id ?? "", !!current && open);

  if (!current) return null;

  const title = current.title?.trim() || current.songs.map((s) => s.title).join(", ") || "No title";
  const artists = current.singers.map((s) => s.name).join(", ") || "Unknown artist";
  const initial = title[0]?.toUpperCase() ?? "?";

  const artImg =
    detail?.songs[0]?.images.find((img) => img.kind === "full_art") ??
    detail?.songs[0]?.images.find((img) => img.kind === "cover_art");
  const artUrl = resolveAssetUrl(artImg) ?? null;

  return (
    <Dialog.Root
      open={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) onClose();
      }}
    >
      <Dialog.Portal>
        <Dialog.Popup className="player-fullscreen">
          <header className="player-fullscreen-header">
            <div>
              <p className="player-fullscreen-title">{title}</p>
              <p className="player-fullscreen-artist">{artists}</p>
            </div>
            <button
              type="button"
              className="player-fullscreen-close player-btn"
              aria-label="Close fullscreen"
              onClick={onClose}
            >
              <XIcon size={24} />
            </button>
          </header>

          <div className="player-fullscreen-art-wrap">
            {artUrl ? (
              <img src={artUrl} alt="" className="player-fullscreen-art" />
            ) : (
              <div className="player-fullscreen-art-ph" aria-hidden="true">
                {initial}
              </div>
            )}
          </div>

          <div className="player-fullscreen-controls">
            <div className="player-fullscreen-progress">
              <span className="w-10 text-right player-time">
                {formatDuration(isSeeking ? seekValue : currentTime)}
              </span>
              <input
                type="range"
                min="0"
                max={duration > 0 ? duration : 1}
                step="0.5"
                value={isSeeking ? seekValue : currentTime}
                onChange={(e) => {
                  setSeekValue(e.target.valueAsNumber);
                }}
                onPointerDown={() => {
                  setIsSeeking(true);
                  setSeekValue(currentTime);
                }}
                onPointerUp={(e) => {
                  seek(e.currentTarget.valueAsNumber);
                  setIsSeeking(false);
                }}
                className="player-range"
              />
              <span className="w-10 player-time">{formatDuration(duration)}</span>
            </div>

            <div className="player-fullscreen-bottom">
              <div className="player-fullscreen-btns">
                <button
                  type="button"
                  className={shuffleEnabled ? "player-btn player-btn--active" : "player-btn"}
                  aria-label="Shuffle"
                  onClick={toggleShuffle}
                >
                  <ShuffleIcon size={20} />
                </button>

                <button
                  type="button"
                  className="player-btn"
                  aria-label="Previous"
                  disabled={!hasPrev}
                  onClick={() => {
                    if (currentTime > 5) {
                      seek(0);
                    } else {
                      prev();
                    }
                  }}
                >
                  <SkipBackIcon size={22} weight="fill" />
                </button>

                <button
                  type="button"
                  className="player-btn-play"
                  aria-label={isPlaying ? "Pause" : "Play"}
                  onClick={isPlaying ? pause : resume}
                >
                  {isPlaying ? (
                    <PauseIcon size={28} weight="fill" />
                  ) : (
                    <PlayIcon size={28} weight="fill" />
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
                  <SkipForwardIcon size={22} weight="fill" />
                </button>

                <button
                  type="button"
                  className={repeatMode !== "none" ? "player-btn player-btn--active" : "player-btn"}
                  aria-label="Repeat"
                  onClick={cycleRepeatMode}
                >
                  {repeatMode === "one" ? <RepeatOnceIcon size={20} /> : <RepeatIcon size={20} />}
                </button>
              </div>

              <div className="player-fullscreen-volume">
                <button
                  type="button"
                  className="player-btn"
                  aria-label={isMuted ? "Unmute" : "Mute"}
                  onClick={toggleMute}
                >
                  {isMuted || volume === 0 ? (
                    <SpeakerSlashIcon size={20} />
                  ) : volume < 0.5 ? (
                    <SpeakerLowIcon size={20} />
                  ) : (
                    <SpeakerHighIcon size={20} />
                  )}
                </button>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.01"
                  value={volume}
                  onChange={(e) => setVolume(e.target.valueAsNumber)}
                  className="player-fullscreen-volume-range"
                />
              </div>
            </div>
          </div>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
