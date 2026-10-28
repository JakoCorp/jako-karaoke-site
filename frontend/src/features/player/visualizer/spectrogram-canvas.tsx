import { Spectrogram } from "@jako/spectrogram";
import type { MeshDensity } from "@jako/spectrogram";
import { useEffect, useRef } from "react";

import { errorMessage } from "@/lib/error-message";
import { toast } from "@/lib/toast";
import { usePlayerStore } from "@/store/player";
import { useVisualizerStore } from "@/store/visualizer";

const SPECTROGRAM_ERROR_TOAST_ID = "spectrogram-error";
const MAX_DEVICE_PIXEL_RATIO = 1.5;
const WEAK_DEVICE_DENSITY: MeshDensity = { columns: 3 * 128, rows: 3 * 128 };

function pickMeshDensity(): MeshDensity | undefined {
  const isWeakDevice =
    navigator.hardwareConcurrency < 4 || window.matchMedia("(pointer: coarse)").matches;
  return isWeakDevice ? WEAK_DEVICE_DENSITY : undefined;
}

function createSpectrogram(
  canvas: HTMLCanvasElement,
  getAudioSource: () => AudioNode | null,
): Spectrogram | null {
  try {
    const source = getAudioSource();
    if (source === null) return null;
    return new Spectrogram(canvas, { source, meshDensity: pickMeshDensity() });
  } catch (error) {
    useVisualizerStore.getState().setSpectrogramEnabled(false);
    toast.error("Couldn't start the spectrogram", {
      id: SPECTROGRAM_ERROR_TOAST_ID,
      description: errorMessage(error, "Something went wrong."),
    });
    return null;
  }
}

interface SpectrogramCanvasProps {
  getAudioSource: () => AudioNode | null;
}

/** Draws the spectrogram of the playing audio. Turns the setting off when it cannot start. */
export function SpectrogramCanvas({ getAudioSource }: SpectrogramCanvasProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const spectrogramRef = useRef<Spectrogram | null>(null);
  const isPlaying = usePlayerStore((s) => s.isPlaying);

  useEffect(() => {
    const canvas = document.createElement("canvas");
    hostRef.current?.append(canvas);

    const spectrogram = createSpectrogram(canvas, getAudioSource);
    spectrogramRef.current = spectrogram;
    if (usePlayerStore.getState().isPlaying) spectrogram?.start();

    const observer = new ResizeObserver(([entry]) => {
      if (entry === undefined) return;
      const dpr = Math.min(window.devicePixelRatio, MAX_DEVICE_PIXEL_RATIO);
      spectrogram?.resize(entry.contentRect.width, entry.contentRect.height, dpr);
    });
    observer.observe(canvas);

    return () => {
      observer.disconnect();
      spectrogram?.dispose();
      spectrogramRef.current = null;
      canvas.remove();
    };
  }, [getAudioSource]);

  useEffect(() => {
    if (isPlaying) {
      spectrogramRef.current?.start();
    } else {
      spectrogramRef.current?.stop();
    }
  }, [isPlaying]);

  return <div ref={hostRef} className="player-fullscreen-visualizer-host" />;
}
