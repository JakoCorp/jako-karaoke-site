import { SpectrogramRenderer } from "./renderer";
import type { MeshDensity } from "./renderer";

/** Mesh resolution used when none is given, sized for desktop GPUs. */
const DEFAULT_DENSITY: MeshDensity = { columns: 7 * 128, rows: 6 * 128 };
/** FFT window size, which sets the frequency resolution of every row. */
const DEFAULT_FFT_SIZE = 4096;
/** Rows written per second, which sets the scroll speed. */
const DEFAULT_ROWS_PER_SECOND = 240;
/** Decibel range that the analyser maps onto byte values, anything quieter than the minimum is silent. */
const MIN_DECIBELS = -80;
const MAX_DECIBELS = -32;
/** Longest gap that is converted to history rows, so returning to a hidden tab does not flood the history. */
const MAX_FRAME_SECONDS = 0.1;

/** Options for {@link Spectrogram}. */
export interface SpectrogramOptions {
  /** Audio node that is analysed. It is only read from, its own connections are untouched. */
  source: AudioNode;
  /** Mesh resolution, lower values render faster on weak GPUs. Defaults to 896 columns by 768 rows. */
  meshDensity?: MeshDensity;
  /** Size of the FFT window, a power of two from 32 to 32768. Defaults to 4096. */
  fftSize?: number;
  /**
   * Time resolution of the history, this sets how fast the spectrogram scrolls.
   * Defaults to 240, which shows about 8.5 seconds of audio at once.
   */
  rowsPerSecond?: number;
}

/**
 * A scrolling 3D spectrogram of an audio node drawn on a canvas with WebGL2.
 *
 * Call {@link Spectrogram.start} to begin drawing and {@link Spectrogram.dispose}
 * once the instance is no longer needed to release the GPU resources.
 * The drawing buffer does not follow the canvas size, call {@link Spectrogram.resize}
 * whenever the canvas changes size.
 *
 * @example
 * const spectrogram = new Spectrogram(canvas, { source: mediaElementSource });
 * spectrogram.resize(canvas.clientWidth, canvas.clientHeight, devicePixelRatio);
 * spectrogram.start();
 */
export class Spectrogram {
  private readonly source: AudioNode;
  private readonly analyser: AnalyserNode;
  private readonly renderer: SpectrogramRenderer;
  private readonly rowsPerSecond: number;
  private readonly frequencyData: Uint8Array<ArrayBuffer>;
  private frameId: number | null = null;
  private lastTimestamp = 0;
  /** Fraction of a row carried over to the next frame. */
  private pendingRows = 0;
  private disposed = false;

  /** Throws when WebGL2 is unavailable or the shaders cannot be built. */
  constructor(canvas: HTMLCanvasElement, options: SpectrogramOptions) {
    const density = options.meshDensity ?? DEFAULT_DENSITY;

    this.source = options.source;
    this.rowsPerSecond = options.rowsPerSecond ?? DEFAULT_ROWS_PER_SECOND;

    this.analyser = options.source.context.createAnalyser();
    this.analyser.fftSize = options.fftSize ?? DEFAULT_FFT_SIZE;
    this.analyser.smoothingTimeConstant = 0;
    this.analyser.minDecibels = MIN_DECIBELS;
    this.analyser.maxDecibels = MAX_DECIBELS;
    this.frequencyData = new Uint8Array(this.analyser.frequencyBinCount);

    this.renderer = new SpectrogramRenderer(
      canvas,
      {
        columns: Math.max(2, Math.floor(density.columns)),
        rows: Math.max(2, Math.floor(density.rows)),
      },
      this.analyser.frequencyBinCount,
    );
    this.source.connect(this.analyser);
  }

  /** Starts drawing, new audio scrolls in from the right. */
  start(): void {
    if (this.disposed || this.frameId !== null) return;
    this.lastTimestamp = performance.now();
    this.frameId = requestAnimationFrame(this.tick);
  }

  /** Pauses drawing and keeps the current image. */
  stop(): void {
    if (this.frameId === null) return;
    cancelAnimationFrame(this.frameId);
    this.frameId = null;
  }

  /** Resizes the canvas, `width` and `height` are in CSS pixels. */
  resize(width: number, height: number, devicePixelRatio: number): void {
    if (this.disposed) return;
    this.renderer.resize(
      Math.round(width * devicePixelRatio),
      Math.round(height * devicePixelRatio),
    );
    this.renderer.render();
  }

  /**
   * Stops drawing, disconnects from the audio node and frees the GPU resources.
   *
   * The canvas loses its WebGL context and cannot be used for another instance.
   */
  dispose(): void {
    if (this.disposed) return;
    this.stop();
    this.disposed = true;
    this.source.disconnect(this.analyser);
    this.renderer.dispose();
  }

  /** Animation frame callback that turns the elapsed time into history rows and draws them. */
  private readonly tick = (timestamp: number): void => {
    const elapsedSeconds = Math.min((timestamp - this.lastTimestamp) / 1000, MAX_FRAME_SECONDS);
    this.lastTimestamp = timestamp;
    this.frameId = requestAnimationFrame(this.tick);

    this.pendingRows += elapsedSeconds * this.rowsPerSecond;
    const rowCount = Math.floor(this.pendingRows);
    if (rowCount === 0) return;
    this.pendingRows -= rowCount;

    this.analyser.getByteFrequencyData(this.frequencyData);
    this.renderer.pushRows(this.frequencyData, rowCount);
    this.renderer.render();
  };
}
