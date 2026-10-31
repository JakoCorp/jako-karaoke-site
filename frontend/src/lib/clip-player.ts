/** What the player reports back to its owner. */
export interface ClipPlayerEvents {
  /** Called with the playback position while the clip plays, and with its duration when it ends. */
  onTimeUpdate: (currentTime: number) => void;
  /** Called when the clip plays to its end. */
  onEnded: () => void;
  /** Called when the clip cannot be loaded or played. */
  onError: () => void;
}

/**
 * Plays one short audio clip at a time.
 *
 * It is separate from the performance audio engine, so a clip never touches the queue, play
 * tracking or the media session.
 */
export class ClipPlayer {
  private readonly audio = new Audio();
  private readonly events: ClipPlayerEvents;

  constructor(events: ClipPlayerEvents) {
    this.events = events;
    this.audio.addEventListener("timeupdate", this.handleTimeUpdate);
    this.audio.addEventListener("ended", this.handleEnded);
    this.audio.addEventListener("error", events.onError);
  }

  /** Starts the clip from the beginning, replacing the clip that is playing. */
  play(url: string): void {
    this.audio.src = url;
    void this.audio.play().catch((error: unknown) => {
      // The error event reports load and decode failures, and a replaced clip is not a failure.
      if (error instanceof DOMException && error.name === "NotAllowedError") {
        this.events.onError();
      }
    });
  }

  /** Stops the clip and rewinds it. */
  stop(): void {
    this.audio.pause();
    this.audio.currentTime = 0;
  }

  /** Sets the output level from 0 to 1. */
  setVolume(level: number): void {
    this.audio.volume = level;
  }

  /** Stops playback and releases the audio resource. */
  destroy(): void {
    this.audio.pause();
    this.audio.removeEventListener("timeupdate", this.handleTimeUpdate);
    this.audio.removeEventListener("ended", this.handleEnded);
    this.audio.removeEventListener("error", this.events.onError);
    this.audio.removeAttribute("src");
    this.audio.load();
  }

  private readonly handleTimeUpdate = (): void => {
    this.events.onTimeUpdate(this.audio.currentTime);
  };

  /** Reports the full duration first, because no position update arrives at the very end. */
  private readonly handleEnded = (): void => {
    if (Number.isFinite(this.audio.duration)) this.events.onTimeUpdate(this.audio.duration);
    this.events.onEnded();
  };
}
