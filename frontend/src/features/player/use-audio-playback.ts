import { useCallback, useEffect, useRef, useState } from "react";

import { selectHasNext, usePlayerStore } from "@/store/player";

import { NativeAudioEngine } from "./audio-engine";
import type { AudioEngine } from "./audio-engine";

export interface PlaybackProgress {
  currentTime: number;
  duration: number;
  seek: (time: number) => void;
}

/** Drives audio playback in sync with the player store. */
export function useAudioPlayback(): PlaybackProgress {
  const engineRef = useRef<AudioEngine | null>(null);
  const prevAudioUrlRef = useRef<string | null>(null);

  if (engineRef.current === null) {
    engineRef.current = new NativeAudioEngine();
  }

  const currentAudioUrl = usePlayerStore((s) => s.currentAudioUrl);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const volume = usePlayerStore((s) => s.volume);
  const hasNext = usePlayerStore(selectHasNext);
  const next = usePlayerStore((s) => s.next);
  const pause = usePlayerStore((s) => s.pause);
  const repeatMode = usePlayerStore((s) => s.repeatMode);

  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);

  useEffect(() => {
    const engine = engineRef.current;
    if (engine === null) return;

    engine.onTimeUpdate((time, dur) => {
      setCurrentTime(time);
      setDuration(Number.isFinite(dur) ? dur : 0);
    });
  }, []);

  useEffect(() => {
    const engine = engineRef.current;
    if (engine === null) return;

    engine.onEnded(() => {
      if (repeatMode === "one") {
        engine.seek(0);
        engine.resume();
      } else if (hasNext) {
        next();
      } else {
        pause();
      }
    });

    engine.onError(() => {
      if (repeatMode === "one" || !hasNext) {
        pause();
      } else {
        next();
      }
    });
  }, [hasNext, next, pause, repeatMode]);

  useEffect(() => {
    const engine = engineRef.current;
    if (engine === null) return;

    if (currentAudioUrl === null) {
      prevAudioUrlRef.current = null;
      engine.stop();
      return;
    }

    if (currentAudioUrl !== prevAudioUrlRef.current) {
      prevAudioUrlRef.current = currentAudioUrl;
      if (isPlaying) {
        engine.play(currentAudioUrl);
      }
    } else if (isPlaying) {
      engine.resume();
    } else {
      engine.pause();
    }
  }, [currentAudioUrl, isPlaying]);

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | undefined;

    if (isPlaying && currentAudioUrl === null) {
      timer = setTimeout(() => {
        if (repeatMode === "one" || !hasNext) {
          pause();
        } else {
          next();
        }
      }, 3000); // Delay moving to next song by 3 sec if on empty
    }

    return () => clearTimeout(timer);
  }, [isPlaying, currentAudioUrl, hasNext, next, pause, repeatMode]);

  useEffect(() => {
    const engine = engineRef.current;
    if (engine === null) return;
    engine.setVolume(volume);
  }, [volume]);

  useEffect(() => {
    return () => {
      engineRef.current?.destroy();
    };
  }, []);

  const seek = useCallback((time: number) => {
    engineRef.current?.seek(time);
    setCurrentTime(time);
  }, []);

  return {
    currentTime: currentAudioUrl !== null ? currentTime : 0,
    duration: currentAudioUrl !== null ? duration : 0,
    seek,
  };
}
