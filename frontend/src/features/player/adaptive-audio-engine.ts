import { extractYouTubeVideoId } from "@/lib/video-embed";

import { NativeAudioEngine } from "./audio-engine";
import type { AudioEngine } from "./audio-engine";
import { YouTubeAudioEngine } from "./youtube-audio-engine";

/**
 * Transparent mux engine that delegates to NativeAudioEngine or YouTubeAudioEngine
 * based on the URL passed to play(). Swaps the active engine on demand and
 * re-applies stored callbacks and volume to the new engine after a swap.
 */
export class AdaptiveAudioEngine implements AudioEngine {
  private native: NativeAudioEngine = new NativeAudioEngine();
  private youtube: YouTubeAudioEngine | null = null;
  private active: AudioEngine = this.native;

  private endedCb: (() => void) | null = null;
  private errorCb: (() => void) | null = null;
  private timeCb: ((currentTime: number, duration: number) => void) | null = null;
  private prevCb: (() => void) | null = null;
  private nextCb: (() => void) | null = null;
  private currentVolume = 1;

  private activateEngine(engine: AudioEngine): void {
    this.active = engine;
    engine.setVolume(this.currentVolume);
    if (this.endedCb) engine.onEnded(this.endedCb);
    if (this.errorCb) engine.onError(this.errorCb);
    if (this.timeCb) engine.onTimeUpdate(this.timeCb);
    if (this.prevCb) engine.onPrev(this.prevCb);
    if (this.nextCb) engine.onNext(this.nextCb);
  }

  play(url: string): void {
    const isYouTube = extractYouTubeVideoId(url) !== null;

    if (isYouTube) {
      if (!this.youtube) {
        this.youtube = new YouTubeAudioEngine();
      }
      if (this.active !== this.youtube) {
        this.native.stop();
        this.activateEngine(this.youtube);
      }
    } else {
      if (this.active !== this.native) {
        this.youtube?.stop();
        this.activateEngine(this.native);
      }
    }

    this.active.play(url);
  }

  pause(): void {
    this.active.pause();
  }

  resume(): void {
    this.active.resume();
  }

  stop(): void {
    this.active.stop();
  }

  seek(time: number): void {
    this.active.seek(time);
  }

  setVolume(level: number): void {
    this.currentVolume = level;
    this.active.setVolume(level);
  }

  onEnded(callback: () => void): void {
    this.endedCb = callback;
    this.active.onEnded(callback);
  }

  onError(callback: () => void): void {
    this.errorCb = callback;
    this.active.onError(callback);
  }

  onTimeUpdate(callback: (currentTime: number, duration: number) => void): void {
    this.timeCb = callback;
    this.active.onTimeUpdate(callback);
  }

  onPrev(callback: () => void): void {
    this.prevCb = callback;
    this.active.onPrev(callback);
  }

  onNext(callback: () => void): void {
    this.nextCb = callback;
    this.active.onNext(callback);
  }

  destroy(): void {
    this.native.destroy();
    this.youtube?.destroy();
  }
}
