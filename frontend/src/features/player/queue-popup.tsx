import { Popover } from "@base-ui/react";
import { QueueIcon } from "@phosphor-icons/react";

import type { QueueSource } from "@/store/player";
import { usePlayerStore } from "@/store/player";

function sourceLabel(source: QueueSource): string {
  if (source.type === "playlist") return `playlist: ${source.name}`;
  if (source.type === "search") return "search results";
  return "";
}

/** Popover panel showing the current queue. Clicking an item jumps to it. */
export function QueuePopup() {
  const queue = usePlayerStore((s) => s.queue);
  const queueIndex = usePlayerStore((s) => s.queueIndex);
  const queueSource = usePlayerStore((s) => s.queueSource);
  const jumpTo = usePlayerStore((s) => s.jumpTo);

  const label = queueSource ? sourceLabel(queueSource) : "";

  return (
    <Popover.Root>
      <Popover.Trigger className="player-btn" aria-label="Queue">
        <QueueIcon size={20} />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Positioner side="top" align="end" sideOffset={8}>
          <Popover.Popup className="player-queue-popup">
            <div className="player-queue-header">
              {queue.length} track{queue.length !== 1 ? "s" : ""}
              {label && <> · from {label}</>}
            </div>
            <ul className="player-queue-list">
              {queue.map((perf, index) => {
                const title = perf.title ?? perf.songs[0]?.title ?? "Untitled";
                const singers = perf.singers.map((s) => s.name).join(" & ");
                const isActive = index === queueIndex;
                return (
                  <li key={perf.id}>
                    <Popover.Close
                      className={
                        isActive
                          ? "player-queue-item player-queue-item--active"
                          : "player-queue-item"
                      }
                      onClick={() => {
                        jumpTo(index);
                      }}
                    >
                      <div className="player-queue-item-title">{title}</div>
                      {singers && <div className="player-queue-item-sub">{singers}</div>}
                    </Popover.Close>
                  </li>
                );
              })}
            </ul>
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>
    </Popover.Root>
  );
}
