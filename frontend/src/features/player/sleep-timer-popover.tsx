import { Popover } from "@base-ui/react";
import { ClockCountdownIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { formatDuration } from "@/lib/format";
import { usePlayerStore } from "@/store/player";

interface Props {
  remaining: number | null;
}

const PRESETS = [
  { label: "5 min", ms: 5 * 60 * 1000 },
  { label: "15 min", ms: 15 * 60 * 1000 },
  { label: "30 min", ms: 30 * 60 * 1000 },
  { label: "60 min", ms: 60 * 60 * 1000 },
] as const;

export function SleepTimerPopover({ remaining }: Props) {
  const sleepTimerEndsAt = usePlayerStore((s) => s.sleepTimerEndsAt);
  const sleepTimerTrackEnd = usePlayerStore((s) => s.sleepTimerTrackEnd);
  const setSleepTimer = usePlayerStore((s) => s.setSleepTimer);
  const setSleepTimerTrackEnd = usePlayerStore((s) => s.setSleepTimerTrackEnd);
  const clearSleepTimer = usePlayerStore((s) => s.clearSleepTimer);
  const [customMinutes, setCustomMinutes] = useState("");

  const isActive = sleepTimerEndsAt !== null || sleepTimerTrackEnd;

  function handleCustomSet() {
    const minutes = parseInt(customMinutes, 10);
    if (minutes > 0) {
      setSleepTimer(minutes * 60 * 1000);
      setCustomMinutes("");
    }
  }

  return (
    <Popover.Root>
      <Popover.Trigger
        className={isActive ? "player-btn-timer" : "player-btn"}
        aria-label="Sleep timer"
      >
        <ClockCountdownIcon size={20} />
        {sleepTimerTrackEnd && <span className="text-xs">EoT</span>}
        {sleepTimerEndsAt !== null && remaining !== null && (
          <span className="text-xs tabular-nums">{formatDuration(remaining)}</span>
        )}
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Positioner side="top" align="end" sideOffset={8}>
          <Popover.Popup className="player-sleep-popup">
            {isActive && (
              <Popover.Close className="btn w-full btn-secondary" onClick={clearSleepTimer}>
                Cancel timer
              </Popover.Close>
            )}
            <div className="player-sleep-presets">
              {PRESETS.map(({ label, ms }) => (
                <Popover.Close
                  key={label}
                  className="btn btn-secondary"
                  onClick={() => setSleepTimer(ms)}
                >
                  {label}
                </Popover.Close>
              ))}
              <Popover.Close className="btn btn-secondary" onClick={setSleepTimerTrackEnd}>
                End of track
              </Popover.Close>
            </div>
            <div className="player-sleep-custom">
              <input
                type="number"
                min="1"
                className="player-sleep-input"
                placeholder="min"
                value={customMinutes}
                onChange={(e) => setCustomMinutes(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleCustomSet();
                }}
              />
              <Popover.Close className="btn btn-secondary" onClick={handleCustomSet}>
                Set
              </Popover.Close>
            </div>
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>
    </Popover.Root>
  );
}
