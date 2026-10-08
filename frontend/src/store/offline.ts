import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { PerformanceSummary } from "@/api/performances";

/** Metadata for one performance saved for offline playback. */
export interface OfflineEntry {
  /** Summary used to render the row and build the playback queue. */
  readonly performance: PerformanceSummary;
  /** Asset the saved audio was downloaded from. */
  readonly assetId: string;
  /** SHA-256 hex digest of the saved audio, compared against the server copy. */
  readonly hash: string;
  /** Size of the saved audio in bytes. */
  readonly sizeBytes: number;
  /** ISO timestamp of when the audio was saved. */
  readonly savedAt: string;
}

interface OfflineState {
  /** Saved entries keyed by performance id. */
  readonly entries: Readonly<Record<string, OfflineEntry>>;
  /** Adds or replaces the entry for a performance. */
  add: (entry: OfflineEntry) => void;
  /** Removes the entry for a performance. */
  remove: (performanceId: string) => void;
  /** Removes every entry. */
  clear: () => void;
}

export const useOfflineStore = create<OfflineState>()(
  persist(
    (set) => ({
      entries: {},
      add: (entry) => {
        set((state) => ({ entries: { ...state.entries, [entry.performance.id]: entry } }));
      },
      remove: (performanceId) => {
        set((state) => {
          const { [performanceId]: _removed, ...rest } = state.entries;
          return { entries: rest };
        });
      },
      clear: () => {
        set({ entries: {} });
      },
    }),
    { name: "offline-entries" },
  ),
);

/** Returns saved entries, newest first. */
export function selectEntryList(state: OfflineState): OfflineEntry[] {
  return Object.values(state.entries).toSorted((a, b) => b.savedAt.localeCompare(a.savedAt));
}

/** Returns the combined size in bytes of all saved audio. */
export function selectTotalBytes(state: OfflineState): number {
  return Object.values(state.entries).reduce((total, entry) => total + entry.sizeBytes, 0);
}
