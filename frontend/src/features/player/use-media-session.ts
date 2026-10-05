import { useEffect, useRef } from "react";

import { selectCurrent, usePlayerStore } from "@/store/player";

interface MediaSessionParams {
  currentTime: number;
  duration: number;
  seek: (time: number) => void;
}

export function useMediaSession({ currentTime, duration, seek }: MediaSessionParams): void {
  const current = usePlayerStore(selectCurrent);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const currentThumbnailUrl = usePlayerStore((s) => s.currentThumbnailUrl);
  const resume = usePlayerStore((s) => s.resume);
  const pause = usePlayerStore((s) => s.pause);
  const next = usePlayerStore((s) => s.next);
  const prev = usePlayerStore((s) => s.prev);

  const currentTimeRef = useRef(currentTime);
  const durationRef = useRef(duration);

  useEffect(() => {
    currentTimeRef.current = currentTime;
    durationRef.current = duration;
  });

  useEffect(() => {
    if (!("mediaSession" in navigator)) return () => undefined;

    navigator.mediaSession.setActionHandler("play", () => {
      resume();
    });
    navigator.mediaSession.setActionHandler("pause", () => {
      pause();
    });
    navigator.mediaSession.setActionHandler("previoustrack", () => {
      prev();
    });
    navigator.mediaSession.setActionHandler("nexttrack", () => {
      next();
    });
    navigator.mediaSession.setActionHandler("seekto", (details) => {
      if (details.seekTime != null) seek(details.seekTime);
    });
    navigator.mediaSession.setActionHandler("seekbackward", (details) => {
      const skip = details.seekOffset ?? 10;
      seek(Math.max(0, currentTimeRef.current - skip));
    });
    navigator.mediaSession.setActionHandler("seekforward", (details) => {
      const skip = details.seekOffset ?? 10;
      seek(Math.min(durationRef.current || Infinity, currentTimeRef.current + skip));
    });

    return () => {
      navigator.mediaSession.setActionHandler("play", null);
      navigator.mediaSession.setActionHandler("pause", null);
      navigator.mediaSession.setActionHandler("previoustrack", null);
      navigator.mediaSession.setActionHandler("nexttrack", null);
      navigator.mediaSession.setActionHandler("seekto", null);
      navigator.mediaSession.setActionHandler("seekbackward", null);
      navigator.mediaSession.setActionHandler("seekforward", null);
      navigator.mediaSession.playbackState = "none";
      navigator.mediaSession.metadata = null;
    };
  }, [resume, pause, prev, next, seek]);

  useEffect(() => {
    if (!("mediaSession" in navigator)) return;
    if (!current) {
      navigator.mediaSession.metadata = null;
      return;
    }
    const title =
      current.title?.trim() || current.songs.map((s) => s.title).join(", ") || "No title";
    const artist = current.singers.map((s) => s.name).join(", ");
    const artwork = currentThumbnailUrl ? [{ src: currentThumbnailUrl }] : undefined;
    navigator.mediaSession.metadata = new MediaMetadata({ title, artist, artwork });
  }, [current, currentThumbnailUrl]);

  useEffect(() => {
    if (!("mediaSession" in navigator)) return;
    navigator.mediaSession.playbackState = isPlaying ? "playing" : "paused";
  }, [isPlaying]);

  useEffect(() => {
    if (!("mediaSession" in navigator)) return;
    if (duration <= 0) return;
    try {
      navigator.mediaSession.setPositionState({ duration, position: currentTime, playbackRate: 1 });
    } catch {
      // Guard against position > duration during rapid updates
    }
  }, [currentTime, duration]);
}
