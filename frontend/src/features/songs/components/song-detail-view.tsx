import type { SongResponse } from "@/api/songs";
import { FilteredPerformanceList } from "@/features/performances";

interface Props {
  song: SongResponse;
  lyricsContent: string | null;
}

export function SongDetailView({ song, lyricsContent }: Props) {
  const coverImage = song.images.find((i) => i.kind === "cover_art")?.public_url;
  const artistNames = song.artists.map((a) => a.name).join(", ");

  const tagsByKind = song.tags.reduce<Record<string, typeof song.tags>>((acc, tag) => {
    const list = acc[tag.kind] ?? [];
    list.push(tag);
    acc[tag.kind] = list;
    return acc;
  }, {});
  const tagKinds = Object.keys(tagsByKind);
  const hasTags = tagKinds.length > 0;

  return (
    <div>
      <div className="song-detail-hero">
        {coverImage ? (
          <img src={coverImage} alt="" className="song-detail-thumbnail" />
        ) : (
          <div className="song-detail-thumbnail-ph" />
        )}
        <div className="song-detail-info">
          <div className="song-detail-title">{song.title}</div>
          {artistNames && <div className="song-detail-artists">by {artistNames}</div>}
          <div className="song-detail-count">
            {song.performance_count.toLocaleString()} performance
            {song.performance_count !== 1 ? "s" : ""}
          </div>
        </div>
      </div>

      <FilteredPerformanceList filter={{ song_id: song.id }} />

      <div className="perf-detail-layout">
        <div className="perf-detail-main">
          {lyricsContent && (
            <div className="perf-detail-card">
              <div className="perf-detail-card-title">Lyrics</div>
              <div className="perf-detail-lyrics-body">{lyricsContent}</div>
            </div>
          )}
        </div>
        <div className="perf-detail-aside">
          {hasTags && (
            <div className="perf-detail-card">
              <div className="perf-detail-card-title">Tags</div>
              {tagKinds.map((kind) => (
                <div key={kind} className="perf-detail-tag-group">
                  <div className="perf-detail-tag-kind-label">{kind}</div>
                  <div className="perf-detail-tags">
                    {tagsByKind[kind]!.map((tag) => (
                      <span key={tag.id} className="perf-detail-tag">
                        {tag.name}
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
