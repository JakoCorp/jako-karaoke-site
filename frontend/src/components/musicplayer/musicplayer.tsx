import {
  ShuffleIcon,
  HeartIcon,
  PlayIcon,
  RepeatIcon,
  SkipBackIcon,
  SkipForwardIcon,
  SpeakerHighIcon,
  QueueIcon,
  ClockCountdownIcon,
  InfoIcon,
} from "@phosphor-icons/react";

export function MusicPlayer() {
  return (
    <div className="flex h-20 items-center gap-8 border-t border-white/10 bg-neutral-800 px-6 text-white">
      {/* Songs hard coded */}
      <div className="flex min-w-0 items-center gap-3">
        {/* albumb cover */}
        <div className="h-14 w-14 shrink-0 rounded bg-white/10" />
        <div className="min-w-0">
          <p className="truncate text-sm font-semibold">Song Title</p>
          <p className="truncate text-xs text-white/60">Artist</p>
        </div>
      </div>

      {/* icon action thingy */}
      <div className="flex shrink-0 items-center gap-2">
        <button className="ml-2 flex h-9 w-9 items-center justify-center text-white/70 hover:text-black">
          <HeartIcon size={18} weight="regular" />
        </button>

        <button className="ml-1 flex h-9 w-9 items-center justify-center text-white/70 hover:text-black">
          <ClockCountdownIcon size={18} weight="fill" alt="Sleep Timer" />
        </button>

        <button className="ml-1 flex h-9 w-9 items-center justify-center text-white/70 hover:text-black">
          <InfoIcon size={18} weight="fill" alt="Song Info" />
        </button>

        <button className="ml-1 flex h-9 w-9 items-center justify-center text-white/70 hover:text-black">
          <QueueIcon size={18} weight="regular" />
        </button>
      </div>

      {/* volume thingy hard coded */}
      <div className="flex shrink-0 items-center gap-2">
        <button>
          <SpeakerHighIcon size={18} />
        </button>
        <input
          type="range"
          min="0"
          max="100"
          defaultValue="70"
          className="w-24 accent-neutral-200"
        />
      </div>

      {/* Play Plause controls hard coded */}
      <div className="flex shrink-0 items-center gap-3">
        <button className="flex h-9 w-9 items-center justify-center text-white/70 hover:text-black">
          <ShuffleIcon size={18} />
        </button>

        <button className="flex h-9 w-9 items-center justify-center rounded-full bg-neutral-800 hover:bg-neutral-900">
          <SkipBackIcon size={18} weight="fill" />
        </button>

        <button className="flex h-12 w-12 items-center justify-center rounded-full bg-neutral-900 hover:bg-neutral-700">
          <PlayIcon size={22} weight="fill" />
        </button>

        <button className="flex h-9 w-9 items-center justify-center rounded-full bg-neutral-800 hover:bg-neutral-900">
          <SkipForwardIcon size={18} weight="fill" />
        </button>

        <button className="flex h-9 w-9 items-center justify-center text-white/70 hover:text-black">
          <RepeatIcon size={18} />
        </button>
      </div>

      {/* progress thingy hard coded*/}
      <div className="flex min-w-0 flex-1 items-center gap-3">
        <span className="w-10 text-right text-xs text-white/40">0:00</span>

        <input
          type="range"
          min="0"
          max="100"
          defaultValue="0"
          className="min-w-0 flex-1 accent-neutral-200"
        />

        <span className="w-10 text-xs text-white/40">6:70</span>
      </div>
    </div>
  );
}
