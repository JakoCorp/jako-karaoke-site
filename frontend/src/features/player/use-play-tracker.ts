import { useState } from "react";

import { unwrap } from "@/api/client";
import { performancesApi } from "@/api/performances";
import { useAuthStore } from "@/store/auth";
import { selectCurrent, usePlayerStore } from "@/store/player";

import { PlayTracker } from "./play-tracker";

/** Creates the tracker that reports plays for the current track while the user is signed in and online. */
export function usePlayTracker(): PlayTracker {
  const [tracker] = useState(
    () =>
      new PlayTracker({
        getTargetId: () => {
          if (useAuthStore.getState().user === null || !navigator.onLine) return null;
          return selectCurrent(usePlayerStore.getState())?.id ?? null;
        },
        isAudible: () => {
          const { isMuted, volume } = usePlayerStore.getState();
          return !isMuted && volume > 0;
        },
        start: async (performanceId) => {
          const started = unwrap(await performancesApi.startPlay(performanceId));
          return { playId: started.play_id, requiredSeconds: started.required_seconds };
        },
        complete: async (performanceId, playId) => {
          unwrap(await performancesApi.completePlay(performanceId, playId));
        },
      }),
  );
  return tracker;
}
