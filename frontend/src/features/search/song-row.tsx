import type { SongSummary } from "@/api/songs";

interface Props {
  song: SongSummary;
}

/** A single row in the song search results list. */
export function SongRow({ song }: Props) {
  const artists = song.artists.map((a) => a.name).join(" & ");

  return (
    <div className="song-row">
      <div className="song-row-info">
        <span className="song-row-title">{song.title}</span>
        {artists && <span className="song-row-sub">{artists}</span>}
      </div>
      <div className="song-row-count">{song.performance_count.toLocaleString()}</div>
    </div>
  );
}
