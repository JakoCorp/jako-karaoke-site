import { useRef, useState } from "react";

import { type SoundbiteResponse } from "@/api/soundbites";
import { useInfiniteSoundbites } from "@/hooks/api/soundbites";
import { useDebounced } from "@/hooks/use-debounced";
import { useInfiniteSentinel } from "@/hooks/use-infinite-sentinel";
import { formatDate } from "@/lib/format";

import { SoundbiteDetailPanel } from "./detail";

function listSubtitle(soundbite: SoundbiteResponse): string {
  const parts = [formatDate(soundbite.soundbite_date)];
  if (soundbite.stream_number != null && soundbite.soundbite_number != null) {
    parts.push(`S${String(soundbite.stream_number)} #${String(soundbite.soundbite_number)}`);
  }
  if (!soundbite.audio) parts.push("No audio");
  return parts.join(" · ");
}

export function SoundbitesAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [selectedSoundbite, setSelectedSoundbite] = useState<SoundbiteResponse | null>(null);
  const [creating, setCreating] = useState(false);

  const debouncedQuery = useDebounced(searchInput.trim());

  const scrollContainerRef = useRef<HTMLUListElement>(null);

  const {
    data: soundbitePages,
    isLoading: soundbitesLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteSoundbites({ q: debouncedQuery || undefined });
  const sentinelRef = useInfiniteSentinel<HTMLLIElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
    scrollContainerRef,
  });

  const soundbites = soundbitePages?.pages.flatMap((page) => page.items) ?? [];

  return (
    <div className="admin-layout">
      <div className="admin-panel">
        <div className="admin-panel-header">
          <input
            type="search"
            className="form-input flex-1"
            placeholder="Search soundbites…"
            value={searchInput}
            onChange={(event) => {
              setSearchInput(event.target.value);
            }}
            aria-label="Search soundbites"
          />
          <button
            className="btn btn-primary"
            onClick={() => {
              setSelectedSoundbite(null);
              setCreating(true);
            }}
          >
            New
          </button>
        </div>
        <ul ref={scrollContainerRef} className="admin-user-list">
          {soundbitesLoading && <li className="admin-empty">Loading…</li>}
          {!soundbitesLoading && soundbites.length === 0 && (
            <li className="admin-empty">No soundbites found.</li>
          )}
          {soundbites.map((soundbite) => (
            <li key={soundbite.id}>
              <button
                className={
                  selectedSoundbite?.id === soundbite.id && !creating
                    ? "admin-user-item admin-user-item--active"
                    : "admin-user-item"
                }
                onClick={() => {
                  setCreating(false);
                  setSelectedSoundbite(soundbite);
                }}
              >
                <div className="admin-item-title">{soundbite.title}</div>
                <div className="admin-item-sub">{listSubtitle(soundbite)}</div>
                {soundbite.artists.length > 0 && (
                  <div className="admin-item-sub">
                    {soundbite.artists.map((artist) => artist.name).join(", ")}
                  </div>
                )}
              </button>
            </li>
          ))}
          <li ref={sentinelRef} />
          {isFetchingNextPage && <li className="admin-empty">Loading more…</li>}
        </ul>
      </div>

      <div className="admin-panel">
        {creating ? (
          <SoundbiteDetailPanel
            key="new"
            soundbite={null}
            onClose={() => setCreating(false)}
            onUpdated={setSelectedSoundbite}
          />
        ) : selectedSoundbite ? (
          <SoundbiteDetailPanel
            key={selectedSoundbite.id}
            soundbite={selectedSoundbite}
            onClose={() => setSelectedSoundbite(null)}
            onUpdated={setSelectedSoundbite}
          />
        ) : (
          <p className="admin-empty">Select a soundbite to view details.</p>
        )}
      </div>
    </div>
  );
}
