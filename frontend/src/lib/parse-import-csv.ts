import { parseStreamTime } from "./format";

type PerformanceTagKind = "instrument" | "modifier" | "misc";

const PERFORMANCE_TAG_KINDS: ReadonlyArray<PerformanceTagKind> = ["instrument", "modifier", "misc"];

function isPerformanceTagKind(value: string): value is PerformanceTagKind {
  return (PERFORMANCE_TAG_KINDS as ReadonlyArray<string>).includes(value);
}

const EXPECTED_HEADERS = [
  "performance_date",
  "stream_number",
  "performance_number",
  "stream_time",
  "duration",
  "title",
  "songs",
  "singers",
  "tags",
] as const;

export interface ParsedPerformanceImportRow {
  performance_date: string;
  stream_number: number;
  performance_number: number;
  stream_time: number | null;
  duration: number | null;
  title: string | null;
  song_titles: string[];
  singer_names: string[];
  tags: Array<{ name: string; kind: PerformanceTagKind }>;
}

type ParseSuccess = { ok: true; rows: ParsedPerformanceImportRow[] };
type ParseFailure = { ok: false; errors: string[] };
export type ParseImportCsvResult = ParseSuccess | ParseFailure;

function parseCsvLine(line: string): string[] {
  const fields: string[] = [];
  let pos = 0;
  while (pos < line.length) {
    if (line[pos] === '"') {
      let value = "";
      pos++;
      while (pos < line.length) {
        if (line[pos] === '"') {
          if (line[pos + 1] === '"') {
            value += '"';
            pos += 2;
          } else {
            pos++;
            break;
          }
        } else {
          value += line[pos];
          pos++;
        }
      }
      fields.push(value);
      if (line[pos] === ",") pos++;
    } else {
      const end = line.indexOf(",", pos);
      if (end === -1) {
        fields.push(line.slice(pos));
        break;
      }
      fields.push(line.slice(pos, end));
      pos = end + 1;
    }
  }
  if (line.endsWith(",")) fields.push("");
  return fields;
}

function splitPipe(value: string): string[] {
  return value
    .split("|")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

function isValidDate(s: string): boolean {
  return /^\d{4}-\d{2}-\d{2}$/.test(s) && !isNaN(Date.parse(s + "T00:00:00"));
}

export function parseImportCsv(text: string): ParseImportCsvResult {
  const lines = text
    .replace(/\r\n/g, "\n")
    .replace(/\r/g, "\n")
    .split("\n")
    .filter((l) => l.trim().length > 0);

  if (lines.length === 0) return { ok: false, errors: ["CSV is empty."] };

  const headers = parseCsvLine(lines[0]!).map((h) => h.trim().toLowerCase());
  for (let i = 0; i < EXPECTED_HEADERS.length; i++) {
    if (headers[i] !== EXPECTED_HEADERS[i]) {
      return {
        ok: false,
        errors: [
          `Invalid header: expected "${EXPECTED_HEADERS[i]}" at column ${String(i + 1)}, got "${headers[i] ?? "(missing)"}"`,
        ],
      };
    }
  }

  const errors: string[] = [];
  const rows: ParsedPerformanceImportRow[] = [];
  const seenTriplets = new Set<string>();

  for (let rowIndex = 1; rowIndex < lines.length; rowIndex++) {
    const rowNum = rowIndex + 1;
    const fields = parseCsvLine(lines[rowIndex]!);
    const get = (i: number): string => (fields[i] ?? "").trim();

    const dateStr = get(0);
    const streamNumStr = get(1);
    const perfNumStr = get(2);
    const streamTimeStr = get(3);
    const durationStr = get(4);
    const titleStr = get(5);
    const songsStr = get(6);
    const singersStr = get(7);
    const tagsStr = get(8);

    const rowErrors: string[] = [];

    if (!isValidDate(dateStr)) {
      rowErrors.push(
        `Row ${String(rowNum)}: invalid performance_date "${dateStr}" — expected YYYY-MM-DD.`,
      );
    }

    const streamNumber = parseInt(streamNumStr, 10);
    if (isNaN(streamNumber) || streamNumber < 1) {
      rowErrors.push(`Row ${String(rowNum)}: stream_number must be an integer ≥ 1.`);
    }

    const perfNumber = parseInt(perfNumStr, 10);
    if (isNaN(perfNumber) || perfNumber < 1) {
      rowErrors.push(`Row ${String(rowNum)}: performance_number must be an integer ≥ 1.`);
    }

    let streamTimeSecs: number | null = null;
    if (streamTimeStr !== "") {
      streamTimeSecs = parseStreamTime(streamTimeStr);
      if (streamTimeSecs === null) {
        rowErrors.push(`Row ${String(rowNum)}: invalid stream_time "${streamTimeStr}".`);
      }
    }

    let durationSecs: number | null = null;
    if (durationStr !== "") {
      durationSecs = parseStreamTime(durationStr);
      if (durationSecs === null) {
        rowErrors.push(`Row ${String(rowNum)}: invalid duration "${durationStr}".`);
      }
    }

    const songTitles = splitPipe(songsStr);
    const singerNames = splitPipe(singersStr);

    const tags: Array<{ name: string; kind: PerformanceTagKind }> = [];
    if (tagsStr !== "") {
      for (const entry of tagsStr.split("|")) {
        const trimmed = entry.trim();
        if (!trimmed) continue;
        const colonIdx = trimmed.lastIndexOf(":");
        if (colonIdx === -1) {
          rowErrors.push(`Row ${String(rowNum)}: tag "${trimmed}" must be in "name:kind" format.`);
          continue;
        }
        const tagName = trimmed.slice(0, colonIdx).trim();
        const tagKind = trimmed.slice(colonIdx + 1).trim();
        if (!isPerformanceTagKind(tagKind)) {
          rowErrors.push(
            `Row ${String(rowNum)}: invalid tag kind "${tagKind}" — valid values: instrument, modifier, misc.`,
          );
          continue;
        }
        if (!tagName) {
          rowErrors.push(`Row ${String(rowNum)}: tag name cannot be empty.`);
          continue;
        }
        tags.push({ name: tagName, kind: tagKind });
      }
    }

    if (rowErrors.length > 0) {
      errors.push(...rowErrors);
      continue;
    }

    const tripletKey = `${dateStr}|${String(streamNumber)}|${String(perfNumber)}`;
    if (seenTriplets.has(tripletKey)) {
      errors.push(
        `Row ${String(rowNum)}: duplicate (${dateStr}, stream ${String(streamNumber)}, perf ${String(perfNumber)}) — already appears earlier in this file.`,
      );
    } else {
      seenTriplets.add(tripletKey);
      rows.push({
        performance_date: dateStr,
        stream_number: streamNumber,
        performance_number: perfNumber,
        stream_time: streamTimeSecs,
        duration: durationSecs,
        title: titleStr || null,
        song_titles: songTitles,
        singer_names: singerNames,
        tags,
      });
    }
  }

  if (errors.length > 0) return { ok: false, errors };
  if (rows.length === 0) return { ok: false, errors: ["CSV has no data rows."] };
  return { ok: true, rows };
}
