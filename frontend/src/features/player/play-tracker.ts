/** Largest playback position advance, in seconds, that still counts as listening. Larger jumps are seeks. */
const MAX_LISTENED_STEP = 10;

interface PlaySession {
  readonly performanceId: string;
  state: "idle" | "starting" | "listening" | "untracked" | "completed";
  lastTime: number | null;
  listened: number;
  playId: string | null;
  requiredSeconds: number;
}

/** What the tracker needs from its surroundings. */
export interface PlayTrackerDeps {
  /** Returns the performance whose plays can be recorded right now, or `null` when none can. */
  getTargetId: () => string | null;
  /** Returns whether the user can currently hear the playback, muted playback earns no credit. */
  isAudible: () => boolean;
  /** Opens a play session on the server. */
  start: (performanceId: string) => Promise<{ playId: string; requiredSeconds: number }>;
  /** Completes a play session on the server. */
  complete: (performanceId: string, playId: string) => Promise<void>;
}

/**
 * Measures how long a track is actually listened to and reports a play once the server's
 * required listening time is reached.
 *
 * Only steady, audible playback progress counts, so seeking ahead or listening while muted
 * never earns credit. A session that fails to start is not retried or queued, which means
 * nothing is recorded while offline.
 */
export class PlayTracker {
  private readonly deps: PlayTrackerDeps;
  private session: PlaySession | null = null;

  constructor(deps: PlayTrackerDeps) {
    this.deps = deps;
  }

  /** Feeds a playback position update. */
  tick(currentTime: number): void {
    const performanceId = this.deps.getTargetId();
    if (performanceId === null) return;

    if (this.session?.performanceId !== performanceId) {
      this.session = {
        performanceId,
        state: "idle",
        lastTime: null,
        listened: 0,
        playId: null,
        requiredSeconds: Infinity,
      };
    }
    const session = this.session;

    const step = session.lastTime === null ? 0 : currentTime - session.lastTime;
    session.lastTime = currentTime;
    if (!this.deps.isAudible()) return;
    if (step > 0 && step <= MAX_LISTENED_STEP) session.listened += step;

    if (session.state === "idle") void this.open(session);
    this.completeWhenListened(session);
  }

  /** Forgets the current session so the next playback is tracked as a new listen. */
  reset(): void {
    this.session = null;
  }

  private async open(session: PlaySession): Promise<void> {
    session.state = "starting";
    try {
      const { playId, requiredSeconds } = await this.deps.start(session.performanceId);
      session.playId = playId;
      session.requiredSeconds = requiredSeconds;
      session.state = "listening";
    } catch {
      session.state = "untracked";
      return;
    }
    this.completeWhenListened(session);
  }

  private completeWhenListened(session: PlaySession): void {
    if (session.state !== "listening" || session.playId === null) return;
    if (session.listened < session.requiredSeconds) return;

    session.state = "completed";
    void this.deps.complete(session.performanceId, session.playId).catch(() => {
      // A failed completion is not retried, the play is simply not recorded.
    });
  }
}
