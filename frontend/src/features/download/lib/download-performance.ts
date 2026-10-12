import type { PerformanceResponse } from "@/api/performances";
import { performancesApi } from "@/api/performances";
import type { DownloadSettings } from "@/api/settings";
import { fetchVerifiedAudio, findDownloadableAudio } from "@/lib/verified-audio";

import { performanceFilenameValues, renderFilename } from "./filename-template";
import type { FilenameValues } from "./filename-template";
import { isTaggable, tagAudio } from "./tag-audio";
import type { AudioTags } from "./tag-audio";

const EMBEDDABLE_IMAGE_TYPES = new Set(["image/jpeg", "image/png"]);
const OBJECT_URL_LIFETIME_MS = 60_000;

function extensionOf(url: string): string {
  const name = new URL(url).pathname.split("/").pop() ?? "";
  const dot = name.lastIndexOf(".");
  return dot === -1 ? "" : name.slice(dot + 1).toLowerCase();
}

async function fetchCover(detail: PerformanceResponse): Promise<AudioTags["cover"]> {
  const images = detail.songs[0]?.images ?? [];
  const image =
    images.find((candidate) => candidate.kind === "cover_art" && candidate.storage_url) ??
    images.find((candidate) => candidate.kind === "full_art" && candidate.storage_url);
  if (!image?.storage_url) return undefined;

  try {
    const response = await fetch(image.storage_url);
    if (!response.ok) return undefined;
    const blob = await response.blob();
    if (!EMBEDDABLE_IMAGE_TYPES.has(blob.type)) return undefined;
    return { mimeType: blob.type, data: new Uint8Array(await blob.arrayBuffer()) };
  } catch {
    return undefined;
  }
}

async function fetchLyrics(performanceId: string): Promise<string | undefined> {
  const { data, response } = await performancesApi.getLyrics(performanceId);
  if (response.status === 404) return undefined;
  return data?.content;
}

const ARTIST_SEPARATOR = " - ";

function artistTag(values: FilenameValues, settings: DownloadSettings): string | undefined {
  const parts = [
    settings.include_singers ? values.singer : "",
    settings.include_original_artists ? values.original_artist : "",
  ].filter((part) => part !== "");
  return parts.length > 0 ? parts.join(ARTIST_SEPARATOR) : undefined;
}

function saveBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  window.setTimeout(() => {
    URL.revokeObjectURL(url);
  }, OBJECT_URL_LIFETIME_MS);
}

/**
 * Downloads the primary audio of a performance to disk using the given preferences.
 *
 * The audio is verified against the server hash before tags are written. Formats that
 * cannot be tagged, or a tagging failure, still download the original file under the
 * templated filename. Throws when there is no downloadable audio or the download fails.
 */
export async function downloadPerformance(
  performanceId: string,
  settings: DownloadSettings,
): Promise<void> {
  const { data: detail, error } = await performancesApi.get(performanceId);
  if (error || !detail) throw new Error("Failed to load performance.");

  const audio = findDownloadableAudio(detail);
  if (!audio) throw new Error("This performance has no downloadable audio.");

  const extension = extensionOf(audio.storage_url);
  const original = await fetchVerifiedAudio(audio);
  let bytes: BlobPart = original;

  if (isTaggable(extension)) {
    const values = performanceFilenameValues(detail);
    const [cover, lyrics] = await Promise.all([
      settings.include_cover_art ? fetchCover(detail) : undefined,
      settings.include_lyrics ? fetchLyrics(performanceId) : undefined,
    ]);

    try {
      bytes = tagAudio(original, extension, {
        title: values.title,
        artist: artistTag(values, settings),
        date: settings.include_date ? values.date : undefined,
        cover,
        lyrics,
      });
    } catch {
      bytes = original;
    }
  }

  const filename = renderFilename(
    settings.filename_template,
    performanceFilenameValues(detail),
    extension,
  );
  saveBlob(new Blob([bytes]), filename);
}
