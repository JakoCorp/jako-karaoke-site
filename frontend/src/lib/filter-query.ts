/** A selected filter value. `id` is what the API receives, `name` is what the chip shows. */
export interface FilterValue {
  id: string;
  name: string;
}

/** One filter condition. Values are alternatives, separate chips must all hold. */
export interface FilterChip {
  field: string;
  exclude: boolean;
  values: FilterValue[];
}

/** A `[-]field:query` prefix typed into the filter input. */
export interface FieldPrefix {
  field: string;
  exclude: boolean;
  query: string;
}

/** An inclusive date range where either bound may be open. Dates are `YYYY-MM-DD`. */
export interface DateRange {
  from: string | null;
  to: string | null;
}

const PREFIX_PATTERN = /^(-?)([a-z_]+):(.*)$/;
const DATE_PATTERN = /^(\d{4})(?:-(\d{2}))?(?:-(\d{2}))?$/;

/** Returns the API expression for a field and its values, e.g. `-genre:id1,id2`. */
export function fieldExpression(field: string, exclude: boolean, ids: readonly string[]): string {
  return `${exclude ? "-" : ""}${field}:${ids.join(",")}`;
}

/** Returns the API expression for a chip. */
export function chipExpression(chip: FilterChip): string {
  return fieldExpression(
    chip.field,
    chip.exclude,
    chip.values.map((value) => value.id),
  );
}

/** Returns the API expressions for all chips, in order. */
export function chipExpressions(chips: readonly FilterChip[]): string[] {
  return chips.map(chipExpression);
}

/** Returns the text that starts a value search for `field`, e.g. `-genre:`. */
export function fieldPrefixText(field: string, exclude: boolean): string {
  return `${exclude ? "-" : ""}${field}:`;
}

/** Parses a `[-]field:query` prefix, or returns null when `text` is plain search text. */
export function parseFieldPrefix(text: string, fieldKeys: readonly string[]): FieldPrefix | null {
  const match = PREFIX_PATTERN.exec(text);
  if (!match) return null;
  const [, minus = "", field = "", rest = ""] = match;
  if (!fieldKeys.includes(field)) return null;
  const query = rest.trim().replace(/^"(.*?)"?$/, "$1");
  return { field, exclude: minus === "-", query };
}

/** Returns the part of `text` that is a plain text search, empty when it is a field prefix. */
export function plainSearchText(text: string, fieldKeys: readonly string[]): string {
  return parseFieldPrefix(text, fieldKeys) ? "" : text;
}

function lastDayOfMonth(year: number, month: number): number {
  return new Date(Date.UTC(year, month, 0)).getUTCDate();
}

function boundary(text: string, edge: "start" | "end"): string | null {
  const match = DATE_PATTERN.exec(text.trim());
  if (!match) return null;
  const year = Number(match[1]);
  const month = match[2] === undefined ? undefined : Number(match[2]);
  const day = match[3] === undefined ? undefined : Number(match[3]);
  if (month !== undefined && (month < 1 || month > 12)) return null;
  const resolvedMonth = month ?? (edge === "start" ? 1 : 12);
  const lastDay = lastDayOfMonth(year, resolvedMonth);
  if (day !== undefined && (day < 1 || day > lastDay)) return null;
  const resolvedDay = day ?? (edge === "start" ? 1 : lastDay);
  return `${String(year).padStart(4, "0")}-${String(resolvedMonth).padStart(2, "0")}-${String(resolvedDay).padStart(2, "0")}`;
}

/**
 * Parses typed date text into a range. Accepts `2024`, `2024-03`, `2024-03-15` and ranges like
 * `2024-01..2024-06` where either side may be empty. Returns null when nothing valid is typed.
 */
export function parseDateRange(text: string): DateRange | null {
  const trimmed = text.trim();
  if (!trimmed) return null;
  const separator = trimmed.indexOf("..");
  if (separator === -1) {
    const from = boundary(trimmed, "start");
    const to = boundary(trimmed, "end");
    return from && to ? { from, to } : null;
  }
  const fromText = trimmed.slice(0, separator);
  const toText = trimmed.slice(separator + 2);
  const from = fromText.trim() ? boundary(fromText, "start") : null;
  const to = toText.trim() ? boundary(toText, "end") : null;
  if ((fromText.trim() && !from) || (toText.trim() && !to)) return null;
  if (!from && !to) return null;
  return { from, to };
}

/** Returns the date range covering the whole of `year`. */
export function yearRange(year: number): DateRange {
  return { from: `${String(year)}-01-01`, to: `${String(year)}-12-31` };
}

/** Builds the filter value for a date range, labelled by year when it spans exactly one. */
export function dateRangeValue(range: DateRange): FilterValue {
  const id = `${range.from ?? ""}..${range.to ?? ""}`;
  const year = range.from?.slice(0, 4);
  const isWholeYear =
    year !== undefined && range.from === `${year}-01-01` && range.to === `${year}-12-31`;
  if (isWholeYear) return { id, name: year };
  return { id, name: `${range.from ?? "…"} – ${range.to ?? "…"}` };
}

/** Returns the chip holding the included or excluded values of `field`, if any. */
export function findChip(
  chips: readonly FilterChip[],
  field: string,
  exclude: boolean,
): FilterChip | undefined {
  return chips.find((chip) => chip.field === field && chip.exclude === exclude);
}

/** Returns whether `chip` holds the value with `valueId`. */
export function chipHasValue(chip: FilterChip | undefined, valueId: string): boolean {
  return chip?.values.some((value) => value.id === valueId) ?? false;
}

/**
 * Adds `value` to the chip for `field` and `exclude`, creating the chip when it does not exist.
 * A value already held by either chip of the field is ignored, so a value is never both included
 * and excluded.
 */
export function addChipValue(
  chips: readonly FilterChip[],
  field: string,
  exclude: boolean,
  value: FilterValue,
): FilterChip[] {
  const chip = findChip(chips, field, exclude);
  if (chipHasValue(chip, value.id) || chipHasValue(findChip(chips, field, !exclude), value.id)) {
    return [...chips];
  }
  if (!chip) return [...chips, { field, exclude, values: [value] }];
  return chips.map((existing) =>
    existing === chip ? { ...chip, values: [...chip.values, value] } : existing,
  );
}

/** Removes `value` from the chip for `field` and `exclude`, dropping the chip once it is empty. */
export function removeChipValue(
  chips: readonly FilterChip[],
  field: string,
  exclude: boolean,
  valueId: string,
): FilterChip[] {
  return chips.flatMap((chip) => {
    if (chip.field !== field || chip.exclude !== exclude) return [chip];
    const values = chip.values.filter((existing) => existing.id !== valueId);
    return values.length > 0 ? [{ ...chip, values }] : [];
  });
}

/** Removes `value` from the matching chip when it is there, otherwise adds it. */
export function toggleChipValue(
  chips: readonly FilterChip[],
  field: string,
  exclude: boolean,
  value: FilterValue,
): FilterChip[] {
  return chipHasValue(findChip(chips, field, exclude), value.id)
    ? removeChipValue(chips, field, exclude, value.id)
    : addChipValue(chips, field, exclude, value);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/**
 * Validates chips read from a URL, dropping anything malformed or using an unknown field so a
 * corrupted URL degrades to fewer filters.
 */
export function validateChips(raw: unknown, fieldKeys: readonly string[]): FilterChip[] {
  if (!Array.isArray(raw)) return [];
  const chips: FilterChip[] = [];
  for (const item of raw as unknown[]) {
    if (!isRecord(item)) continue;
    if (typeof item.field !== "string" || !fieldKeys.includes(item.field)) continue;
    if (!Array.isArray(item.values)) continue;
    const values: FilterValue[] = [];
    for (const value of item.values as unknown[]) {
      if (isRecord(value) && typeof value.id === "string" && typeof value.name === "string") {
        values.push({ id: value.id, name: value.name });
      }
    }
    if (values.length === 0) continue;
    chips.push({ field: item.field, exclude: item.exclude === true, values });
  }
  return chips;
}
