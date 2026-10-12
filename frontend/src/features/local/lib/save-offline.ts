import type { PerformanceResponse, PerformanceSummary } from "@/api/performances";
import { performancesApi } from "@/api/performances";
import { fetchVerifiedAudio, findDownloadableAudio } from "@/lib/verified-audio";
import { useOfflineStore } from "@/store/offline";

import { clearAudioBlobs, deleteAudioBlob, putAudioBlob } from "./offline-db";

function toSummary(detail: PerformanceResponse): PerformanceSummary {
  return {
    id: detail.id,
    title: detail.title,
    duration: detail.duration,
    performance_date: detail.performance_date,
    stream_number: detail.stream_number,
    performance_number: detail.performance_number,
    play_count: detail.play_count,
    singers: detail.singers,
    songs: detail.songs.map((song) => ({ id: song.id, title: song.title })),
  };
}

/**
 * Downloads the primary audio of a performance and stores it for offline playback.
 *
 * Throws when the performance has no downloadable audio, the download fails, or the
 * downloaded bytes do not match the server hash.
 */
export async function saveOffline(performanceId: string): Promise<void> {
  const { data: detail, error } = await performancesApi.get(performanceId);
  if (error || !detail) throw new Error("Failed to load performance.");

  const audio = findDownloadableAudio(detail);
  if (!audio) throw new Error("This performance has no downloadable audio.");

  const blob = new Blob([await fetchVerifiedAudio(audio)]);

  await putAudioBlob(performanceId, blob);
  useOfflineStore.getState().add({
    performance: toSummary(detail),
    assetId: audio.asset_id,
    hash: audio.hash,
    sizeBytes: blob.size,
    savedAt: new Date().toISOString(),
  });
}

/** Removes the saved audio and index entry for a performance. */
export async function removeOffline(performanceId: string): Promise<void> {
  await deleteAudioBlob(performanceId);
  useOfflineStore.getState().remove(performanceId);
}

/** Removes every saved audio file and index entry. */
export async function clearOffline(): Promise<void> {
  await clearAudioBlobs();
  useOfflineStore.getState().clear();
}
