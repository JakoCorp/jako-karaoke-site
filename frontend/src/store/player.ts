import { create } from "zustand";

/** A performance queued for playback. Properties are readonly, update via store actions. */
export interface QueuedPerformance {
  readonly id: string;
  readonly title: string | null;
  readonly singers: readonly { readonly id: string; readonly name: string }[];
  readonly duration: number | null;
  readonly audioUrl: string | null;
}

interface PlayerState {
  current: QueuedPerformance | null;
  isPlaying: boolean;
  volume: number;
  play: (performance: QueuedPerformance) => void;
  pause: () => void;
  resume: () => void;
  stop: () => void;
  setVolume: (volume: number) => void;
}

/** Global player store. Manages the currently playing performance and playback state. */
export const usePlayerStore = create<PlayerState>((set) => ({
  current: null,
  isPlaying: false,
  volume: 1,
  play: (performance) => {
    set({ current: performance, isPlaying: true });
  },
  pause: () => {
    set({ isPlaying: false });
  },
  resume: () => {
    set({ isPlaying: true });
  },
  stop: () => {
    set({ current: null, isPlaying: false });
  },
  setVolume: (volume) => {
    set({ volume });
  },
}));
