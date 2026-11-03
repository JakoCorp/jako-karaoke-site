import { useEffect, useRef, useState } from "react";

import { unwrap } from "@/api/client";
import { soundbitesApi } from "@/api/soundbites";
import { ClipPlayer } from "@/lib/clip-player";
import { PlayTracker } from "@/lib/play-tracker";
import { toast } from "@/lib/toast";
import { useAuthStore } from "@/store/auth";
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
 * starts again or the component unmounts. It follows the global volume and mute, and reports a
 * play to the server once enough of the soundbite was listened to.
 */
export function useSoundbitePlayer(): SoundbitePlayer {
  const playerRef = useRef<ClipPlayer | null>(null);
  const trackerRef = useRef<PlayTracker | null>(null);
  const activeIdRef = useRef<string | null>(null);
  const [activeId, setActiveId] = useState<string | null>(null);

  useEffect(() => {
    const tracker = new PlayTracker({
      getTargetId: () => {
        if (useAuthStore.getState().user === null || !navigator.onLine) return null;
        return activeIdRef.current;
      },
      isAudible: () => {
        const { isMuted, volume } = usePlayerStore.getState();
        return !isMuted && volume > 0;
      },
      start: async (soundbiteId) => {
        const started = unwrap(await soundbitesApi.startPlay(soundbiteId));
        return { playId: started.play_id, requiredSeconds: started.required_seconds };
      },
      complete: async (soundbiteId, playId) => {
        unwrap(await soundbitesApi.completePlay(soundbiteId, playId));
      },
    });
    trackerRef.current = tracker;

    function clearActive() {
      tracker.reset();
      activeIdRef.current = null;
      setActiveId(null);
    }

    const player = new ClipPlayer({
      onTimeUpdate: (currentTime) => {
        tracker.tick(currentTime);
      },
      onEnded: clearActive,
      onError: () => {
        clearActive();
        toast.error("Couldn't play this soundbite", { id: PLAYBACK_ERROR_TOAST_ID });
      },
    });
    playerRef.current = player;

    const unsubscribe = usePlayerStore.subscribe((state, previous) => {
      if (state.isPlaying && !previous.isPlaying) {
        player.stop();
        clearActive();
      }
      if (state.volume !== previous.volume || state.isMuted !== previous.isMuted) {
        player.setVolume(state.isMuted ? 0 : state.volume);
      }
    });

    return () => {
      unsubscribe();
      player.destroy();
      playerRef.current = null;
      trackerRef.current = null;
    };
  }, []);

  function toggle(id: string, url: string): void {
    const player = playerRef.current;
    const tracker = trackerRef.current;
    if (player === null || tracker === null) return;

    tracker.reset();
    if (activeId === id) {
      player.stop();
      activeIdRef.current = null;
      setActiveId(null);
      return;
    }

    const { volume, isMuted, pause } = usePlayerStore.getState();
    pause();
    player.setVolume(isMuted ? 0 : volume);
    activeIdRef.current = id;
    player.play(url);
    setActiveId(id);
  }

  return { activeId, toggle };
}
