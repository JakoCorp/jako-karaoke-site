import { PlayIcon } from "@phosphor-icons/react";
import { Link } from "react-router";

import type { PerformanceResponse } from "@/api/performances";
import { formatDate, formatDuration, formatStreamTime } from "@/lib/format";
import { usePlayerStore } from "@/store/player";

import { PerformanceDetailMenu } from "./performance-menu";

interface Props {
  performance: PerformanceResponse;
  lyricsContent: string | null;
}

export function PerformanceDetailView({ performance, lyricsContent }: Props) {
  const playQueue = usePlayerStore((s) => s.playQueue);

  const coverImage = performance.songs[0]?.images.find((i) => i.kind === "cover_art")?.public_url;

  const title =
    performance.title?.trim() || performance.songs.map((s) => s.title).join(" / ") || "Untitled";
  const initial = title[0]?.toUpperCase() ?? "?";

  const singerNames = performance.singers.map((s) => s.name).join(", ");

  const uniqueOriginalArtists = [
    ...new Map(performance.songs.flatMap((s) => s.artists).map((a) => [a.id, a])).values(),
  ];

  const tagsByKind = performance.tags.reduce<Record<string, typeof performance.tags>>(
    (acc, tag) => {
      const list = acc[tag.kind] ?? [];
      list.push(tag);
      acc[tag.kind] = list;
      return acc;
    },
    {},
  );

  const tagKinds = Object.keys(tagsByKind);
  const hasTags = tagKinds.length > 0;
  const hasSongs = performance.songs.length > 0;

  return (
    <div>
      <div className="perf-detail-hero">
        {coverImage ? (
          <img src={coverImage} alt="" className="perf-detail-thumbnail" />
        ) : (
          <div className="perf-detail-thumbnail-ph" aria-hidden="true">
            {initial}
          </div>
        )}
        <div className="perf-detail-info">
          <div className="perf-detail-title">{title}</div>
          {singerNames && <div className="perf-detail-cover-by">Cover by {singerNames}</div>}
          {uniqueOriginalArtists.length > 0 && (
            <div className="perf-detail-original-by">
              Original by {uniqueOriginalArtists.map((a) => a.name).join(", ")}
            </div>
          )}
          <div className="perf-detail-meta">
            {formatDate(performance.performance_date)} · Stream {performance.stream_number} -
            Performance #{performance.performance_number}
          </div>
          {performance.stream_time != null && (
            <div className="perf-detail-meta">
              Starts at {formatStreamTime(performance.stream_time)} in stream
            </div>
          )}
          <div className="perf-detail-stats">
            {performance.duration != null && (
              <>
                <span>{formatDuration(performance.duration)}</span>
                <span className="perf-detail-stats-dot" />
              </>
            )}
            <span>{performance.play_count.toLocaleString()} plays</span>
          </div>
          <div className="perf-detail-actions">
            <button
              type="button"
              className="btn perf-detail-play-btn btn-primary"
              onClick={() => {
                playQueue([performance], 0, { type: "single" });
              }}
            >
              <PlayIcon size={16} weight="fill" />
              Play
            </button>
            <PerformanceDetailMenu performanceId={performance.id} />
          </div>
        </div>
      </div>

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

          {hasSongs && (
            <div className="perf-detail-card">
              <div className="perf-detail-card-title">
                {performance.songs.length === 1 ? "Song" : "Songs"}
              </div>
              {performance.songs.map((song) => (
                <div key={song.id} className="perf-detail-song-item">
                  <Link to={`/song/${song.id}`} className="perf-detail-song-link">
                    {song.title}
                  </Link>
                  {song.artists.length > 0 && (
                    <div className="perf-detail-song-artists">
                      {song.artists.map((a) => a.name).join(", ")}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
