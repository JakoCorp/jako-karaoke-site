import type { ArtistResponse } from "@/api/artists";
import { ClampedText } from "@/components/clamped-text";
import { FilteredPerformanceList } from "@/features/performances";
import { resolveAssetUrl } from "@/lib/asset-url";

interface Props {
  artist: ArtistResponse;
}

function pluralize(count: number, singular: string): string {
  return `${count.toLocaleString()} ${singular}${count !== 1 ? "s" : ""}`;
}

export function ArtistDetailView({ artist }: Props) {
  const avatarUrl = resolveAssetUrl(artist.images.find((image) => image.kind === "avatar"));
  const initial = artist.name.trimStart()[0]?.toUpperCase() ?? "?";

  return (
    <div>
      <div className="artist-detail-hero">
        {avatarUrl ? (
          <img src={avatarUrl} alt="" className="artist-detail-avatar avatar" />
        ) : (
          <div
            className="avatar-placeholder artist-detail-avatar avatar artist-detail-avatar-initial"
            aria-hidden="true"
          >
            {initial}
          </div>
        )}
        <div className="artist-detail-info">
          <div className="artist-detail-name" title={artist.name}>
            {artist.name}
          </div>
          <div className="artist-detail-count">
            {pluralize(artist.song_count, "song")} ·{" "}
            {pluralize(artist.performance_count, "performance")}
          </div>
          {artist.description && (
            <ClampedText key={artist.id} text={artist.description} className="mt-2" />
          )}
        </div>
      </div>

      {artist.performance_count > 0 && (
        <FilteredPerformanceList scope={{ field: "singer", id: artist.id }} />
      )}

      {artist.links.length > 0 && (
        <div className="artist-detail-links perf-detail-card">
          <div className="perf-detail-card-title">Links</div>
          <div className="artist-detail-link-list">
            {artist.links.map((link) => (
              <a
                key={link.id}
                href={link.url}
                target="_blank"
                rel="noreferrer noopener"
                className="artist-detail-link"
              >
                {link.label ?? link.kind}
              </a>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
