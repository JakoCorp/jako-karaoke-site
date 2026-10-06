import { useEffect, useState } from "react";

import { usePlayerStore } from "@/store/player";

export function useSleepTimer(): { remaining: number | null } {
  const sleepTimerEndsAt = usePlayerStore((s) => s.sleepTimerEndsAt);
  const clearSleepTimer = usePlayerStore((s) => s.clearSleepTimer);
  const pause = usePlayerStore((s) => s.pause);

  const [remainingSeconds, setRemainingSeconds] = useState<number | null>(null);

  useEffect(() => {
    const id =
      sleepTimerEndsAt !== null
        ? setInterval(() => {
            const secs = Math.max(0, Math.ceil((sleepTimerEndsAt - Date.now()) / 1000));
            setRemainingSeconds(secs);
            if (secs === 0) {
              pause();
              clearSleepTimer();
            }
          }, 1000)
        : null;

    return () => {
      if (id !== null) clearInterval(id);
    };
  }, [sleepTimerEndsAt, pause, clearSleepTimer]);

  return { remaining: sleepTimerEndsAt !== null ? remainingSeconds : null };
}
