import { useParams } from "react-router";

import { useArtist } from "@/hooks/api/artists";

import { ArtistDetailView } from "../components/detail-view";

export function ArtistDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data: artist, isLoading } = useArtist(id!);

  if (isLoading) return <div className="perf-detail-empty">Loading…</div>;
  if (!artist) return <div className="perf-detail-empty">Artist not found.</div>;

  return <ArtistDetailView artist={artist} />;
}
