/** Common interface for audio playback. */
export interface AudioEngine {
  play(url: string): void;
  pause(): void;
  resume(): void;
  stop(): void;
  seek(time: number): void;
  setVolume(level: number): void;
  onEnded(callback: () => void): void;
  onError(callback: () => void): void;
  onTimeUpdate(callback: (currentTime: number, duration: number) => void): void;
  destroy(): void;
}

/** Audio playback. */
export class NativeAudioEngine implements AudioEngine {
  private readonly audio: HTMLAudioElement;
  private endedHandler: (() => void) | null = null;
  private errorHandler: (() => void) | null = null;
  private timeUpdateHandler: (() => void) | null = null;

  constructor() {
    this.audio = new Audio();
  }

  play(url: string): void {
    this.audio.src = url;
    this.audio.currentTime = 0;
    void this.audio.play();
  }

  pause(): void {
    this.audio.pause();
  }

  resume(): void {
    void this.audio.play();
  }

  stop(): void {
    this.audio.pause();
    this.audio.src = "";
  }

  seek(time: number): void {
    this.audio.currentTime = time;
  }

  setVolume(level: number): void {
    this.audio.volume = level;
  }

  onEnded(callback: () => void): void {
    if (this.endedHandler !== null) {
      this.audio.removeEventListener("ended", this.endedHandler);
    }
    this.endedHandler = callback;
    this.audio.addEventListener("ended", callback);
  }

  onError(callback: () => void): void {
    if (this.errorHandler !== null) {
      this.audio.removeEventListener("error", this.errorHandler);
    }
    this.errorHandler = callback;
    this.audio.addEventListener("error", callback);
  }

  onTimeUpdate(callback: (currentTime: number, duration: number) => void): void {
    if (this.timeUpdateHandler !== null) {
      this.audio.removeEventListener("timeupdate", this.timeUpdateHandler);
    }
    this.timeUpdateHandler = () => callback(this.audio.currentTime, this.audio.duration);
    this.audio.addEventListener("timeupdate", this.timeUpdateHandler);
  }

  destroy(): void {
    this.audio.pause();
    if (this.endedHandler !== null) {
      this.audio.removeEventListener("ended", this.endedHandler);
      this.endedHandler = null;
    }
    if (this.errorHandler !== null) {
      this.audio.removeEventListener("error", this.errorHandler);
      this.errorHandler = null;
    }
    if (this.timeUpdateHandler !== null) {
      this.audio.removeEventListener("timeupdate", this.timeUpdateHandler);
      this.timeUpdateHandler = null;
    }
    this.audio.src = "";
  }
}
