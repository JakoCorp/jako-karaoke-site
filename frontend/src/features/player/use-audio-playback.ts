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
  const stop = usePlayerStore((s) => s.stop);

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
      if (hasNext) {
        next();
      } else {
        stop();
      }
    });

    engine.onError(() => {
      if (hasNext) {
        next();
      } else {
        stop();
      }
    });
  }, [hasNext, next, stop]);

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
