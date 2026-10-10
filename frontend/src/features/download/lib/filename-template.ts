import type { PerformanceResponse } from "@/api/performances";

/** Tokens a user can place in a filename template, written as `{token}`. */
export const FILENAME_TOKENS = [
  { token: "title", description: "Performance title, or the song titles when it has none" },
  { token: "singer", description: "Singers of the performance" },
  { token: "original_artist", description: "Original artists of the songs" },
  { token: "song", description: "Song titles" },
  { token: "date", description: "Performance date as YYYY-MM-DD" },
  { token: "stream_number", description: "Stream number within the day" },
  { token: "performance_number", description: "Position within the stream" },
] as const;

export type FilenameToken = (typeof FILENAME_TOKENS)[number]["token"];
export type FilenameValues = Record<FilenameToken, string>;

const FALLBACK_FILENAME = "performance";
const MAX_FILENAME_LENGTH = 180;
const DISALLOWED_FILENAME_CHARS = /[^\p{L}\p{M}\p{N} _.,()[\]'&+!#@~=-]/gu;

function isFilenameToken(name: string): name is FilenameToken {
  return FILENAME_TOKENS.some((entry) => entry.token === name);
}

function unique(values: readonly string[]): string[] {
  return [...new Set(values)];
}

/** Resolves every filename token for a performance. */
export function performanceFilenameValues(detail: PerformanceResponse): FilenameValues {
  const songTitles = detail.songs.map((song) => song.title).join(", ");
  const originalArtists = unique(
    detail.songs.flatMap((song) => song.artists.map((artist) => artist.name)),
  ).join(", ");

  return {
    title: detail.title ?? songTitles,
    singer: detail.singers.map((singer) => singer.name).join(", "),
    original_artist: originalArtists,
    song: songTitles,
    date: detail.performance_date,
    stream_number: String(detail.stream_number),
    performance_number: String(detail.performance_number),
  };
}

/**
 * Renders a filename template into a safe filename.
 *
 * Only letters, numbers, spaces and a small set of punctuation survive, every other character
 * becomes an underscore. Unknown tokens are therefore kept as plain text. An empty result falls
 * back to a generic name and the extension is appended when given.
 */
export function renderFilename(
  template: string,
  values: FilenameValues,
  extension?: string,
): string {
  const rendered = template.replace(/\{(\w+)\}/g, (match, name: string) =>
    isFilenameToken(name) ? values[name] : match,
  );

  const name = rendered
    .replace(DISALLOWED_FILENAME_CHARS, "_")
    .replace(/ {2,}/g, " ")
    .trim()
    .slice(0, MAX_FILENAME_LENGTH)
    .replace(/^[. ]+|[. ]+$/g, "");

  const base = name === "" ? FALLBACK_FILENAME : name;
  return extension ? `${base}.${extension}` : base;
}
