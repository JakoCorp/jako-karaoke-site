import { useParams } from "react-router";

import { usePerformance, usePerformanceLyrics } from "@/hooks/api/performances";

import { PerformanceDetailView } from "../components/detail-view";

export function PerformanceDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data: performance, isLoading: perfLoading } = usePerformance(id!);
  const { data: lyrics, isLoading: lyricsLoading } = usePerformanceLyrics(id!);

  if (perfLoading) {
    return <div className="perf-detail-empty">Loading…</div>;
  }

  if (!performance) {
    return <div className="perf-detail-empty">Performance not found.</div>;
  }

  return (
    <PerformanceDetailView
      performance={performance}
      lyricsContent={!lyricsLoading ? (lyrics?.content ?? null) : null}
    />
  );
}
