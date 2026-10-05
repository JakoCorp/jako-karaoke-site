/**
 * Adapted from Swarmtunes (AceandGaming/swarmtunes-client).
 * Source: https://github.com/AceandGaming/swarmtunes-client/blob/1bf02b38bbf4ae3e4bf2c34c98c09deeae5258a2/src/scripts/audio/youtube.ts
 */

import { extractYouTubeVideoId } from "@/lib/video-embed";

import type { AudioEngine } from "./audio-engine";

declare global {
  interface Window {
    YT: {
      Player: new (
        element: string | HTMLIFrameElement,
        options: {
          events?: {
            onReady?: (event: { target: YTPlayer }) => void;
            onStateChange?: (event: { data: number; target: YTPlayer }) => void;
            onError?: (event: { data: number }) => void;
          };
        },
      ) => YTPlayer;
      PlayerState: {
        ENDED: number;
        PLAYING: number;
        PAUSED: number;
      };
    };
    onYouTubeIframeAPIReady: (() => void) | undefined;
  }
}

interface YTPlayer {
  loadVideoById(videoId: string, startSeconds?: number): void;
  playVideo(): void;
  pauseVideo(): void;
  stopVideo(): void;
  seekTo(seconds: number, allowSeekAhead: boolean): void;
  setVolume(volume: number): void;
  getCurrentTime(): number;
  getDuration(): number;
  getPlayerState(): number;
  destroy(): void;
}

function injectIframeApi(): void {
  if (document.getElementById("yt-iframe-api")) return;
  const script = document.createElement("script");
  script.id = "yt-iframe-api";
  script.src = "https://www.youtube.com/iframe_api";
  document.head.appendChild(script);
}

/** Plays YouTube audio via the IFrame Player API. Views are credited because the real YT player runs in an iframe. */
export class YouTubeAudioEngine implements AudioEngine {
  private iframe: HTMLIFrameElement;
  private player: YTPlayer | null = null;
  private ready = false;
  private pendingVideoId: string | null = null;

  private endedCb: (() => void) | null = null;
  private errorCb: (() => void) | null = null;
  private timeCb: ((currentTime: number, duration: number) => void) | null = null;

  private pollId: ReturnType<typeof setInterval> | null = null;
  private currentVolume = 1;

  constructor() {
    injectIframeApi();

    const iframe = document.createElement("iframe");
    iframe.id = "yt-engine-player";
    iframe.allow = "autoplay; encrypted-media";
    iframe.setAttribute("playsinline", "1");
    iframe.sandbox.add("allow-scripts", "allow-same-origin");
    iframe.style.cssText = "position:absolute;width:1px;height:1px;top:-9999px;left:-9999px;"; // CSS size causes iframe to run at 480p
    iframe.src =
      "https://www.youtube-nocookie.com/embed/?enablejsapi=1&controls=0&playsinline=1&cc_load_policy=0";
    document.body.appendChild(iframe);
    this.iframe = iframe;

    const initPlayer = () => {
      this.player = new window.YT.Player(iframe.id, {
        events: {
          onReady: () => {
            this.ready = true;
            if (this.pendingVideoId !== null) {
              this.player!.setVolume(this.currentVolume * 100);
              this.player!.loadVideoById(this.pendingVideoId);
              this.pendingVideoId = null;
            }
          },
          onStateChange: (event) => {
            if (event.data === 0) {
              this.stopPoll();
              this.endedCb?.();
            } else if (event.data === 1) {
              this.startPoll();
            } else if (event.data === 2 || event.data === 5) {
              this.stopPoll();
            }
          },
          onError: () => {
            this.stopPoll();
            this.errorCb?.();
          },
        },
      });
    };

    if (window.YT?.Player) {
      initPlayer();
    } else {
      const existing = window.onYouTubeIframeAPIReady;
      window.onYouTubeIframeAPIReady = () => {
        existing?.();
        initPlayer();
      };
    }
  }

  private startPoll(): void {
    if (this.pollId !== null) return;
    this.pollId = setInterval(() => {
      if (!this.player || this.player.getPlayerState() !== 1) return;
      this.timeCb?.(this.player.getCurrentTime(), this.player.getDuration());
    }, 250);
  }

  private stopPoll(): void {
    if (this.pollId === null) return;
    clearInterval(this.pollId);
    this.pollId = null;
  }

  play(url: string): void {
    const videoId = extractYouTubeVideoId(url);
    if (!videoId) return;

    if (!this.ready || !this.player) {
      this.pendingVideoId = videoId;
      return;
    }

    this.player.setVolume(this.currentVolume * 100);
    this.player.loadVideoById(videoId);
  }

  pause(): void {
    if (!this.ready) return;
    this.player?.pauseVideo();
  }

  resume(): void {
    if (!this.ready) return;
    this.player?.playVideo();
  }

  stop(): void {
    this.stopPoll();
    this.pendingVideoId = null;
    if (!this.ready) return;
    this.player?.stopVideo();
  }

  seek(time: number): void {
    if (!this.ready) return;
    this.player?.seekTo(time, true);
  }

  setVolume(level: number): void {
    this.currentVolume = level;
    if (!this.ready) return;
    this.player?.setVolume(level * 100);
  }

  onEnded(callback: () => void): void {
    this.endedCb = callback;
  }

  onError(callback: () => void): void {
    this.errorCb = callback;
  }

  onTimeUpdate(callback: (currentTime: number, duration: number) => void): void {
    this.timeCb = callback;
  }

  destroy(): void {
    this.stopPoll();
    this.player?.destroy();
    this.iframe.remove();
    this.player = null;
    this.ready = false;
  }
}
