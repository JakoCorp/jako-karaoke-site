import { create } from "zustand";

import type { PerformanceSummary } from "@/api/performances";

/** The context from which the current queue was initiated. */
export type QueueSource =
  | { readonly type: "search" }
  | { readonly type: "playlist"; readonly id: string; readonly name: string }
  | { readonly type: "single" };

interface PlayerState {
  readonly queue: readonly PerformanceSummary[];
  readonly queueIndex: number;
  readonly queueSource: QueueSource | null;
  readonly isPlaying: boolean;
  readonly volume: number;
  readonly currentAudioUrl: string | null;
  readonly currentThumbnailUrl: string | null;
  playQueue: (
    performances: readonly PerformanceSummary[],
    startIndex: number,
    source: QueueSource,
  ) => void;
  jumpTo: (index: number) => void;
  next: () => void;
  prev: () => void;
  pause: () => void;
  resume: () => void;
  stop: () => void;
  setVolume: (volume: number) => void;
  setCurrentAudioUrl: (url: string | null) => void;
  setCurrentThumbnailUrl: (url: string | null) => void;
}

/** Global player store. Manages the queue, playback position, and playback state. */
export const usePlayerStore = create<PlayerState>((set, get) => ({
  queue: [],
  queueIndex: -1,
  queueSource: null,
  isPlaying: false,
  volume: 1,
  currentAudioUrl: null,
  currentThumbnailUrl: null,
  playQueue: (performances, startIndex, source) => {
    set({
      queue: performances,
      queueIndex: startIndex,
      queueSource: source,
      isPlaying: true,
      currentAudioUrl: null,
      currentThumbnailUrl: null,
    });
  },
  jumpTo: (index) => {
    const { queue } = get();
    if (index >= 0 && index < queue.length) {
      set({ queueIndex: index, isPlaying: true, currentAudioUrl: null, currentThumbnailUrl: null });
    }
  },
  next: () => {
    const { queueIndex, queue } = get();
    if (queueIndex < queue.length - 1) {
      set({ queueIndex: queueIndex + 1, currentAudioUrl: null, currentThumbnailUrl: null });
    }
  },
  prev: () => {
    const { queueIndex } = get();
    if (queueIndex > 0) {
      set({ queueIndex: queueIndex - 1, currentAudioUrl: null, currentThumbnailUrl: null });
    }
  },
  pause: () => {
    set({ isPlaying: false });
  },
  resume: () => {
    set({ isPlaying: true });
  },
  stop: () => {
    set({
      queue: [],
      queueIndex: -1,
      queueSource: null,
      isPlaying: false,
      currentAudioUrl: null,
      currentThumbnailUrl: null,
    });
  },
  setVolume: (volume) => {
    set({ volume });
  },
  setCurrentAudioUrl: (url) => {
    set({ currentAudioUrl: url });
  },
  setCurrentThumbnailUrl: (url) => {
    set({ currentThumbnailUrl: url });
  },
}));

/** Returns the currently active performance, or null if the queue is empty. */
export const selectCurrent = (s: PlayerState): PerformanceSummary | null =>
  s.queueIndex >= 0 ? (s.queue[s.queueIndex] ?? null) : null;

/** True when there is a next track in the queue. */
export const selectHasNext = (s: PlayerState): boolean =>
  s.queueIndex >= 0 && s.queueIndex < s.queue.length - 1;

/** True when there is a previous track in the queue. */
export const selectHasPrev = (s: PlayerState): boolean => s.queueIndex > 0;
