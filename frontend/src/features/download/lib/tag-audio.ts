import MP3Tag from "mp3tag.js";
import type { MP3TagWriteOptions } from "mp3tag.js";

/** Metadata embedded into a downloaded audio file. Absent fields are not written. */
export interface AudioTags {
  title: string;
  artist?: string;
  /** Performance date as `YYYY-MM-DD`. */
  date?: string;
  cover?: { mimeType: string; data: Uint8Array };
  lyrics?: string;
}

const FRONT_COVER_PICTURE_TYPE = 3;

const WRITE_OPTIONS: Readonly<Record<string, MP3TagWriteOptions>> = {
  mp3: { id3v2: { version: 3 } },
  m4a: { id3v2: { version: 3, padding: 0 } },
  aac: { id3v2: { version: 3, unsynch: true } },
};

/** Returns whether tags can be written into files with the given extension. */
export function isTaggable(extension: string): boolean {
  return extension in WRITE_OPTIONS;
}

/**
 * Writes ID3v2 tags into an audio file and returns the tagged bytes.
 *
 * Throws when the file cannot be parsed or written, so callers can fall back to the
 * untagged original.
 */
export function tagAudio(
  buffer: ArrayBuffer,
  extension: string,
  tags: AudioTags,
): Uint8Array<ArrayBuffer> {
  const mp3tag = new MP3Tag(buffer.slice(0));
  mp3tag.read();
  if (mp3tag.error !== "") throw new Error(mp3tag.error);

  mp3tag.tags.v2 ??= {};
  const frames = mp3tag.tags.v2;
  frames.TIT2 = tags.title;
  if (tags.artist) frames.TPE1 = tags.artist;

  if (tags.date) {
    const [year, month, day] = tags.date.split("-");
    frames.TYER = year;
    frames.TDAT = `${day}${month}`;
  }

  if (tags.cover) {
    frames.APIC = [
      {
        format: tags.cover.mimeType,
        type: FRONT_COVER_PICTURE_TYPE,
        description: "",
        data: Array.from(tags.cover.data),
      },
    ];
  }

  if (tags.lyrics) {
    frames.USLT = [{ language: "eng", descriptor: "", text: tags.lyrics }];
  }

  const tagged = mp3tag.save(WRITE_OPTIONS[extension]);
  if (mp3tag.error !== "") throw new Error(mp3tag.error);
  return Uint8Array.from(new Uint8Array(tagged));
}
