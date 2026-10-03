import { Dialog } from "@base-ui/react";
import { ArrowSquareOutIcon, FilmStripIcon, PlayIcon, XIcon } from "@phosphor-icons/react";
import { useState } from "react";
import { Link } from "react-router";

import type { PerformanceResponse } from "@/api/performances";
import { formatDate, formatDuration, formatStreamTime } from "@/lib/format";
import { getVideoEmbedInfo } from "@/lib/video-embed";
import { selectCurrent, usePlayerStore } from "@/store/player";

import { PerformanceDetailMenu } from "./menu";

interface Props {
  performance: PerformanceResponse;
  lyricsContent: string | null;
}

function mediaLabel(asset: {
  title?: string | null;
  storage_url?: string | null;
  external_url?: string | null;
}): string {
  if (asset.title) return asset.title;
  if (asset.storage_url) return asset.storage_url.split("/").pop() ?? "";
  return asset.external_url ?? "";
}

export function PerformanceDetailView({ performance, lyricsContent }: Props) {
  const playQueue = usePlayerStore((s) => s.playQueue);
  const setCurrentAudioUrl = usePlayerStore((s) => s.setCurrentAudioUrl);
  const setPreferredAudioAssetId = usePlayerStore((s) => s.setPreferredAudioAssetId);
  const resumePlayer = usePlayerStore((s) => s.resume);
  const current = usePlayerStore(selectCurrent);
  const currentAudioUrl = usePlayerStore((s) => s.currentAudioUrl);
  const isPlaying = usePlayerStore((s) => s.isPlaying);
  const [popupVideoId, setPopupVideoId] = useState<string | null>(null);

  const coverImg = performance.songs[0]?.images.find((i) => i.kind === "cover_art");
  const coverImage = coverImg?.storage_url ?? coverImg?.external_url ?? undefined;

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
  const hasVideos = performance.video.length > 0;
  const hasAudio = performance.audio.length > 0;

  const popupVideo = performance.video.find((v) => v.asset_id === popupVideoId) ?? null;
  const popupEmbedInfo = popupVideo?.external_url
    ? getVideoEmbedInfo(popupVideo.external_url)
    : null;

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
          {hasVideos && (
            <div className="perf-detail-card">
              <div className="perf-detail-card-title">Videos</div>
              <div className="perf-detail-media-list">
                {performance.video.map((video) => {
                  const label = mediaLabel(video);
                  const embedInfo = video.external_url
                    ? getVideoEmbedInfo(video.external_url)
                    : null;
                  const canEmbed = !!video.storage_url || !!embedInfo?.embedUrl;

                  return (
                    <div key={video.asset_id} className="perf-detail-video-card">
                      <button
                        type="button"
                        className="perf-detail-video-header"
                        onClick={() => {
                          if (canEmbed) {
                            setPopupVideoId(video.asset_id);
                          } else if (video.external_url) {
                            window.open(video.external_url, "_blank", "noreferrer");
                          }
                        }}
                      >
                        <div className="perf-detail-video-thumb">
                          {embedInfo?.thumbnailUrl ? (
                            <img src={embedInfo.thumbnailUrl} alt="" />
                          ) : (
                            <FilmStripIcon size={22} />
                          )}
                        </div>
                        <div className="min-w-0 flex-1">
                          <div className="perf-detail-video-title">{label}</div>
                          <div className="mt-1 flex items-center gap-1.5">
                            <span className="perf-detail-kind-badge">{video.kind}</span>
                            {embedInfo?.platform && (
                              <span className="perf-detail-video-platform">
                                {embedInfo.platform}
                              </span>
                            )}
                          </div>
                        </div>
                        {!canEmbed && video.external_url && (
                          <ArrowSquareOutIcon size={16} className="shrink-0 text-fg-muted" />
                        )}
                      </button>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {hasAudio && (
            <div className="perf-detail-card">
              <div className="perf-detail-card-title">Audio</div>
              <div className="perf-detail-media-list">
                {performance.audio.map((audio) => {
                  const label = mediaLabel(audio);
                  const isInternal = !!audio.storage_url;
                  const isActive =
                    isInternal &&
                    current?.id === performance.id &&
                    currentAudioUrl === audio.storage_url &&
                    isPlaying;

                  return (
                    <button
                      key={audio.asset_id}
                      type="button"
                      className={
                        isInternal
                          ? "perf-detail-audio-row"
                          : "perf-detail-audio-row perf-detail-audio-row--external"
                      }
                      disabled={!isInternal}
                      onClick={() => {
                        if (!isInternal) return;
                        if (current?.id === performance.id) {
                          setCurrentAudioUrl(audio.storage_url!);
                          resumePlayer();
                        } else {
                          setPreferredAudioAssetId(audio.asset_id);
                          playQueue([performance], 0, { type: "single" });
                          setCurrentAudioUrl(audio.storage_url!);
                        }
                      }}
                    >
                      <PlayIcon
                        size={14}
                        weight="fill"
                        className={isActive ? "text-playing" : "text-fg-muted"}
                      />
                      <span className="perf-detail-audio-label">{label}</span>
                      <span className="perf-detail-kind-badge">{audio.kind}</span>
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          <Dialog.Root
            open={popupVideoId !== null}
            onOpenChange={(open) => {
              if (!open) setPopupVideoId(null);
            }}
          >
            <Dialog.Portal>
              <Dialog.Backdrop className="dialog-backdrop" />
              <Dialog.Popup className="perf-video-popup">
                <div className="perf-video-popup-embed">
                  {popupEmbedInfo?.autoplayEmbedUrl ? (
                    <>
                      <iframe
                        src={popupEmbedInfo.autoplayEmbedUrl}
                        title={popupVideo ? mediaLabel(popupVideo) : ""}
                        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                        allowFullScreen
                        // oxlint-disable-next-line react/iframe-missing-sandbox
                        sandbox="allow-scripts allow-same-origin allow-presentation allow-popups allow-forms"
                      />
                    </>
                  ) : popupVideo?.storage_url ? (
                    <video src={popupVideo.storage_url} controls autoPlay>
                      <track kind="captions" />
                    </video>
                  ) : null}
                </div>
                <div className="perf-video-popup-footer">
                  <div className="min-w-0 flex-1">
                    {popupVideo && (
                      <div className="perf-detail-video-title">{mediaLabel(popupVideo)}</div>
                    )}
                    <div className="mt-1 flex items-center gap-1.5">
                      {popupVideo && (
                        <span className="perf-detail-kind-badge">{popupVideo.kind}</span>
                      )}
                      {popupEmbedInfo?.platform && (
                        <span className="perf-detail-video-platform">
                          {popupEmbedInfo.platform}
                        </span>
                      )}
                    </div>
                  </div>
                  <Dialog.Close className="perf-detail-action-btn" aria-label="Close">
                    <XIcon size={16} />
                  </Dialog.Close>
                </div>
              </Dialog.Popup>
            </Dialog.Portal>
          </Dialog.Root>

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
