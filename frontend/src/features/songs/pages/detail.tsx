import { useParams } from "react-router";

import { useSong, useSongLyrics } from "@/hooks/api/songs";

import { SongDetailView } from "../components/detail-view";

export function SongDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data: song, isLoading } = useSong(id!);
  const { data: lyricsContent } = useSongLyrics(id!, !!song);

  if (isLoading) return <div className="perf-detail-empty">Loading…</div>;
  if (!song) return <div className="perf-detail-empty">Song not found.</div>;

  return <SongDetailView song={song} lyricsContent={lyricsContent?.content ?? null} />;
}
