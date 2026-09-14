import { create } from "zustand";

import type { PerformanceSummary } from "@/api/performances";

export type RepeatMode = "none" | "all" | "one";

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
  readonly shuffleEnabled: boolean;
  readonly repeatMode: RepeatMode;
  readonly playHistory: readonly number[];
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
  toggleShuffle: () => void;
  cycleRepeatMode: () => void;
}

/** Appends `index` to `history`, deduplicating and capping at 50 entries. */
function appendHistory(history: readonly number[], index: number): readonly number[] {
  const filtered = history.filter((i) => i !== index);
  const trimmed = filtered.length >= 50 ? filtered.slice(1) : filtered;
  return [...trimmed, index];
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
  shuffleEnabled: false,
  repeatMode: "none",
  playHistory: [],
  playQueue: (performances, startIndex, source) => {
    const { queue, queueIndex, playHistory } = get();
    if (performances === queue) {
      if (startIndex < 0 || startIndex >= performances.length) return;
      const history =
        queueIndex >= 0 && queueIndex !== startIndex
          ? appendHistory(playHistory, queueIndex)
          : playHistory;
      set({
        queueIndex: startIndex,
        queueSource: source,
        isPlaying: true,
        currentAudioUrl: null,
        currentThumbnailUrl: null,
        playHistory: history,
      });
      return;
    }
    set({
      queue: performances,
      queueIndex: startIndex,
      queueSource: source,
      isPlaying: true,
      currentAudioUrl: null,
      currentThumbnailUrl: null,
      playHistory: [],
    });
  },
  jumpTo: (index) => {
    const { queue, queueIndex, playHistory } = get();
    if (index < 0 || index >= queue.length) return;
    const history =
      queueIndex >= 0 && queueIndex !== index
        ? appendHistory(playHistory, queueIndex)
        : playHistory;
    set({
      queueIndex: index,
      isPlaying: true,
      currentAudioUrl: null,
      currentThumbnailUrl: null,
      playHistory: history,
    });
  },
  next: () => {
    const { queue, queueIndex, shuffleEnabled, repeatMode, playHistory } = get();
    if (queue.length === 0) return;

    if (shuffleEnabled && queue.length > 1) {
      const indices = queue.map((_, i) => i).filter((i) => i !== queueIndex);
      const randomIndex = indices[Math.floor(Math.random() * indices.length)]!;
      set({
        queueIndex: randomIndex,
        currentAudioUrl: null,
        currentThumbnailUrl: null,
        isPlaying: true,
        playHistory: appendHistory(playHistory, queueIndex),
      });
      return;
    }

    if (queueIndex < queue.length - 1) {
      set({
        queueIndex: queueIndex + 1,
        currentAudioUrl: null,
        currentThumbnailUrl: null,
        isPlaying: true,
        playHistory: appendHistory(playHistory, queueIndex),
      });
    } else if (repeatMode === "all") {
      set({
        queueIndex: 0,
        currentAudioUrl: null,
        currentThumbnailUrl: null,
        isPlaying: true,
        playHistory: appendHistory(playHistory, queueIndex),
      });
    }
  },
  prev: () => {
    const { playHistory } = get();
    if (playHistory.length > 0) {
      const prevIndex = playHistory[playHistory.length - 1]!;
      set({
        queueIndex: prevIndex,
        playHistory: playHistory.slice(0, -1),
        currentAudioUrl: null,
        currentThumbnailUrl: null,
        isPlaying: true,
      });
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
      playHistory: [],
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
  toggleShuffle: () => {
    set((s) => ({ shuffleEnabled: !s.shuffleEnabled }));
  },
  cycleRepeatMode: () => {
    set((s) => ({
      repeatMode: s.repeatMode === "none" ? "all" : s.repeatMode === "all" ? "one" : "none",
    }));
  },
}));

/** Returns the currently active performance, or null if the queue is empty. */
export const selectCurrent = (s: PlayerState): PerformanceSummary | null =>
  s.queueIndex >= 0 ? (s.queue[s.queueIndex] ?? null) : null;

/** True when the next action will advance playback. */
export const selectHasNext = (s: PlayerState): boolean => {
  if (s.queue.length === 0 || s.queueIndex < 0) return false;
  if (s.repeatMode === "all") return true;
  if (s.shuffleEnabled && s.queue.length > 1) return true;
  return s.queueIndex < s.queue.length - 1;
};

/** True when there is history to go back to or the current track can be rewound. */
export const selectHasPrev = (s: PlayerState): boolean => s.queueIndex >= 0;
