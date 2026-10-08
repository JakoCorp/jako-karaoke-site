import type { PerformanceResponse, PerformanceSummary } from "@/api/performances";
import { performancesApi } from "@/api/performances";
import { useOfflineStore } from "@/store/offline";

import { clearAudioBlobs, deleteAudioBlob, putAudioBlob } from "./offline-db";

/** Returns the primary hosted audio asset of a performance, if one can be downloaded. */
export function findDownloadableAudio(detail: PerformanceResponse) {
  return detail.audio.find((audio) => audio.kind === "primary" && audio.storage_url && audio.hash);
}

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

async function sha256Hex(blob: Blob): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", await blob.arrayBuffer());
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("");
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
  if (!audio?.storage_url || !audio.hash) {
    throw new Error("This performance has no downloadable audio.");
  }

  const response = await fetch(audio.storage_url);
  if (!response.ok) throw new Error("Failed to download audio.");
  const blob = await response.blob();

  if ((await sha256Hex(blob)) !== audio.hash) {
    throw new Error("Downloaded audio did not match the server copy.");
  }

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
