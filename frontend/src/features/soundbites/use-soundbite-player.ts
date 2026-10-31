import { useEffect, useRef, useState } from "react";

import { ClipPlayer } from "@/lib/clip-player";
import { toast } from "@/lib/toast";
import { usePlayerStore } from "@/store/player";

const PLAYBACK_ERROR_TOAST_ID = "soundbite-playback-error";

export interface SoundbitePlayer {
  /** ID of the soundbite that is playing, or `null` when none is. */
  activeId: string | null;
  /** Plays the soundbite, or stops it when it is the one playing. */
  toggle: (id: string, url: string) => void;
}

/**
 * Plays soundbites one at a time, separately from the performance queue.
 *
 * Starting a soundbite pauses the main player, and the soundbite stops when the main player
 * starts again or the component unmounts. It follows the global volume and mute.
 */
export function useSoundbitePlayer(): SoundbitePlayer {
  const playerRef = useRef<ClipPlayer | null>(null);
  const [activeId, setActiveId] = useState<string | null>(null);

  useEffect(() => {
    const player = new ClipPlayer({
      onEnded: () => {
        setActiveId(null);
      },
      onError: () => {
        setActiveId(null);
        toast.error("Couldn't play this soundbite", { id: PLAYBACK_ERROR_TOAST_ID });
      },
    });
    playerRef.current = player;

    const unsubscribe = usePlayerStore.subscribe((state, previous) => {
      if (state.isPlaying && !previous.isPlaying) {
        player.stop();
        setActiveId(null);
      }
      if (state.volume !== previous.volume || state.isMuted !== previous.isMuted) {
        player.setVolume(state.isMuted ? 0 : state.volume);
      }
    });

    return () => {
      unsubscribe();
      player.destroy();
      playerRef.current = null;
    };
  }, []);

  function toggle(id: string, url: string): void {
    const player = playerRef.current;
    if (player === null) return;

    if (activeId === id) {
      player.stop();
      setActiveId(null);
      return;
    }

    const { volume, isMuted, pause } = usePlayerStore.getState();
    pause();
    player.setVolume(isMuted ? 0 : volume);
    player.play(url);
    setActiveId(id);
  }

  return { activeId, toggle };
}
