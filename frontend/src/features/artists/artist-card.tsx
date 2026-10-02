import type { ArtistSummary } from "@/api/artists";

interface ArtistCardProps {
  artist: ArtistSummary;
}

export function ArtistCard({ artist }: ArtistCardProps) {
  const avatar = artist.images.find((img) => img.kind === "avatar");
  const initial = artist.name.trimStart()[0]?.toUpperCase() ?? "?";

  return (
    <div className="artist-card">
      {avatar ? (
        <img
          className="artist-avatar"
          src={avatar.storage_url ?? avatar.external_url ?? undefined}
          alt={artist.name}
        />
      ) : (
        <div className="artist-avatar-placeholder" aria-hidden="true">
          {initial}
        </div>
      )}
      <div className="artist-card-name" title={artist.name}>
        {artist.name}
      </div>
      <div className="artist-card-count">
        {artist.song_count} {artist.song_count === 1 ? "song" : "songs"}
      </div>
      {artist.description && <p className="artist-card-desc">{artist.description}</p>}
    </div>
  );
}
