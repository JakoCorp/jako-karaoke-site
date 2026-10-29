import { WaveformIcon } from "@phosphor-icons/react";

import { useVisualizerStore } from "@/store/visualizer";

interface VisualizerTogglesProps {
  spectrogramAvailable: boolean;
}

/** Buttons that switch the fullscreen visualizers on and off. */
export function VisualizerToggles({ spectrogramAvailable }: VisualizerTogglesProps) {
  const spectrogramEnabled = useVisualizerStore((s) => s.spectrogramEnabled);
  const setSpectrogramEnabled = useVisualizerStore((s) => s.setSpectrogramEnabled);

  return (
    <div className="player-fullscreen-toggles">
      <button
        type="button"
        className={spectrogramEnabled ? "player-btn player-btn--active" : "player-btn"}
        aria-label="Spectrogram"
        aria-pressed={spectrogramEnabled}
        title={
          spectrogramAvailable
            ? "Spectrogram Visualizer (by kcpr)"
            : "Spectrogram Visualizer isn't available for this track"
        }
        disabled={!spectrogramAvailable}
        onClick={() => {
          setSpectrogramEnabled(!spectrogramEnabled);
        }}
      >
        <WaveformIcon size={22} />
      </button>
    </div>
  );
}
