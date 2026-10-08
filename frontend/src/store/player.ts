import { create } from "zustand";

import type { PerformanceSummary } from "@/api/performances";

export type RepeatMode = "none" | "all" | "one";

/** The context from which the current queue was initiated. */
export type QueueSource =
  | { readonly type: "search" }
  | { readonly type: "playlist"; readonly id: string; readonly name: string }
  | { readonly type: "local" }
  | { readonly type: "single" };

interface PlayerState {
  /** Ordered list of performances in the current queue. */
  readonly queue: readonly PerformanceSummary[];
  /** Index of the currently active track, or -1 when the queue is idle. */
  readonly queueIndex: number;
  /** Context from which the current queue was initiated. */
  readonly queueSource: QueueSource | null;
  /** Whether the player is actively playing. */
  readonly isPlaying: boolean;
  /** Volume level from 0 to 1. */
  readonly volume: number;
  /** Whether audio output is muted. */
  readonly isMuted: boolean;
  /** Resolved audio URL for the current track, populated by the track resolver. */
  readonly currentAudioUrl: string | null;
  /** Resolved thumbnail URL for the current track, populated by the track resolver. */
  readonly currentThumbnailUrl: string | null;
  /** Whether shuffle mode is active. */
  readonly shuffleEnabled: boolean;
  /** Current repeat behavior. */
  readonly repeatMode: RepeatMode;
  /** Stack of previously played queue indices, consumed by prev(). */
  readonly playHistory: readonly number[];
  /** Asset ID of the audio track to prefer when the resolver runs. Cleared after use. */
  readonly preferredAudioAssetId: string | null;
  /** Epoch ms at which the sleep timer expires, or null when inactive. */
  readonly sleepTimerEndsAt: number | null;
  /** When true, playback pauses after the current track ends naturally. */
  readonly sleepTimerTrackEnd: boolean;
  /** Replaces the queue with the given performances and begins playback at startIndex. */
  playQueue: (
    performances: readonly PerformanceSummary[],
    startIndex: number,
    source: QueueSource,
  ) => void;
  /** Jumps to index, recording the current position in history. */
  jumpTo: (index: number) => void;
  /** Advances to the next track, respecting shuffle and repeat mode. */
  next: () => void;
  /** Returns to the most recently played track via history. */
  prev: () => void;
  /** Pauses playback. */
  pause: () => void;
  /** Resumes playback. */
  resume: () => void;
  /** Stops playback and clears the queue. */
  stop: () => void;
  /** Sets the volume level. */
  setVolume: (volume: number) => void;
  /** Toggles the muted state. */
  toggleMute: () => void;
  /** Sets the resolved audio URL for the current track. */
  setCurrentAudioUrl: (url: string | null) => void;
  /** Sets the resolved thumbnail URL for the current track. */
  setCurrentThumbnailUrl: (url: string | null) => void;
  /** Sets the preferred audio asset ID for the next resolver run. */
  setPreferredAudioAssetId: (assetId: string | null) => void;
  /** Toggles shuffle mode. */
  toggleShuffle: () => void;
  /** Cycles through repeat modes: none, all, one. */
  cycleRepeatMode: () => void;
  /** Starts a countdown sleep timer that pauses playback after the given milliseconds. */
  setSleepTimer: (ms: number) => void;
  /** Schedules playback to pause after the current track ends naturally. */
  setSleepTimerTrackEnd: () => void;
  /** Cancels any active sleep timer. */
  clearSleepTimer: () => void;
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
  isMuted: false,
  currentAudioUrl: null,
  currentThumbnailUrl: null,
  shuffleEnabled: false,
  repeatMode: "none",
  playHistory: [],
  preferredAudioAssetId: null,
  sleepTimerEndsAt: null,
  sleepTimerTrackEnd: false,
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
      preferredAudioAssetId: null,
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
        preferredAudioAssetId: null,
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
        preferredAudioAssetId: null,
        isPlaying: true,
        playHistory: appendHistory(playHistory, queueIndex),
      });
    } else if (repeatMode === "all") {
      set({
        queueIndex: 0,
        currentAudioUrl: null,
        currentThumbnailUrl: null,
        preferredAudioAssetId: null,
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
        preferredAudioAssetId: null,
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
      preferredAudioAssetId: null,
      playHistory: [],
    });
  },
  setVolume: (volume) => {
    set({ volume });
  },
  toggleMute: () => {
    set((s) => ({ isMuted: !s.isMuted }));
  },
  setCurrentAudioUrl: (url) => {
    set({ currentAudioUrl: url });
  },
  setCurrentThumbnailUrl: (url) => {
    set({ currentThumbnailUrl: url });
  },
  setPreferredAudioAssetId: (assetId) => {
    set({ preferredAudioAssetId: assetId });
  },
  toggleShuffle: () => {
    set((s) => ({ shuffleEnabled: !s.shuffleEnabled }));
  },
  cycleRepeatMode: () => {
    set((s) => ({
      repeatMode: s.repeatMode === "none" ? "all" : s.repeatMode === "all" ? "one" : "none",
    }));
  },
  setSleepTimer: (ms) => {
    set({ sleepTimerEndsAt: Date.now() + ms, sleepTimerTrackEnd: false });
  },
  setSleepTimerTrackEnd: () => {
    set({ sleepTimerTrackEnd: true, sleepTimerEndsAt: null });
  },
  clearSleepTimer: () => {
    set({ sleepTimerEndsAt: null, sleepTimerTrackEnd: false });
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
