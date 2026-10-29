import { create } from "zustand";
import { persist } from "zustand/middleware";

interface VisualizerState {
  /** Whether the fullscreen player shows the spectrogram. */
  readonly spectrogramEnabled: boolean;
  /** Turns the spectrogram on or off. */
  setSpectrogramEnabled: (enabled: boolean) => void;
}

export const useVisualizerStore = create<VisualizerState>()(
  persist(
    (set) => ({
      spectrogramEnabled: false,
      setSpectrogramEnabled: (enabled) => {
        set({ spectrogramEnabled: enabled });
      },
    }),
    { name: "visualizer-settings" },
  ),
);
