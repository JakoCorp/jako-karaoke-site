import { useEffect, useRef } from "react";

import { usePlayerStore } from "@/store/player";

export function useKeyboardShortcuts(): void {
  const isPlayingRef = useRef(false);

  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const pause = usePlayerStore((s) => s.pause);
  const resume = usePlayerStore((s) => s.resume);
  const toggleMute = usePlayerStore((s) => s.toggleMute);
  const toggleShuffle = usePlayerStore((s) => s.toggleShuffle);
  const cycleRepeatMode = usePlayerStore((s) => s.cycleRepeatMode);

  useEffect(() => {
    isPlayingRef.current = isPlaying;
  }, [isPlaying]);

  useEffect(() => {
    function handler(event: KeyboardEvent) {
      if (!(event.target instanceof HTMLElement)) return;
      const target = event.target;
      if (
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target.isContentEditable ||
        event.metaKey ||
        event.ctrlKey
      )
        return;

      if (event.key === " ") {
        if (event.repeat) return;
        event.preventDefault();
        if (isPlayingRef.current) {
          pause();
        } else {
          resume();
        }
      } else if (event.key === "m") {
        toggleMute();
      } else if (event.key === "s") {
        toggleShuffle();
      } else if (event.key === "r") {
        cycleRepeatMode();
      }
    }

    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [pause, resume, toggleMute, toggleShuffle, cycleRepeatMode]);
}
