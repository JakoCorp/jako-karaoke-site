import type { CatalogGroup, CatalogValue } from "@/api/filters";
import {
  chipHasValue,
  dateRangeValue,
  findChip,
  parseDateRange,
  yearRange,
  type FieldPrefix,
  type FilterChip,
  type FilterValue,
} from "@/lib/filter-query";

import type { FilterFieldDef } from "./types";

/** What the popup lists, decided by what is typed in the input. */
export type PopupMode = "fields" | "suggest" | "values" | "date";

/** How a value is already used by another chip of the same field. */
export type UsedAs = "included" | "excluded";

export type PopupRow =
  | { kind: "field"; key: string; field: FilterFieldDef }
  | {
      kind: "value";
      key: string;
      field: FilterFieldDef;
      option: CatalogValue;
      /** Shows a checkbox that is ticked when the value is part of the chip being built. */
      checked: boolean;
      toggles: boolean;
      /** Set when another chip already holds the value, which makes the row unavailable. */
      usedAs?: UsedAs;
    }
  | { kind: "date"; key: string; field: FilterFieldDef; label: string; value: FilterValue };

interface RowInput {
  mode: PopupMode;
  fields: readonly FilterFieldDef[];
  text: string;
  prefix: FieldPrefix | null;
  catalog: readonly CatalogGroup[];
  chips: readonly FilterChip[];
  today: Date;
}

/** A labelled section of popup rows. The label is empty for an unlabelled section. */
export interface PopupGroup {
  value: string;
  items: PopupRow[];
}

const RECENT_DAYS = 30;
const SUGGESTIONS_PER_FIELD = 5;

function isoDate(date: Date): string {
  return date.toISOString().slice(0, 10);
}

function matches(option: CatalogValue, query: string): boolean {
  return option.name.toLowerCase().includes(query);
}

function yearRows(field: FilterFieldDef, options: readonly CatalogValue[]): PopupRow[] {
  const rows: PopupRow[] = [];
  for (const option of options) {
    const year = Number(option.id);
    if (!Number.isInteger(year)) continue;
    rows.push({
      kind: "date",
      key: `year:${option.id}`,
      field,
      label: option.name,
      value: dateRangeValue(yearRange(year)),
    });
  }
  return rows;
}

function dateRows(
  field: FilterFieldDef,
  input: RowInput,
  group: CatalogGroup | undefined,
): PopupRow[] {
  const rows: PopupRow[] = [];
  const query = input.prefix?.query ?? "";
  const typed = parseDateRange(query);
  if (typed) {
    const value = dateRangeValue(typed);
    rows.push({ kind: "date", key: `typed:${value.id}`, field, label: `Use ${value.name}`, value });
  }

  if (!query) {
    const recentStart = new Date(input.today);
    recentStart.setUTCDate(recentStart.getUTCDate() - RECENT_DAYS);
    const recent = { from: isoDate(recentStart), to: isoDate(input.today) };
    const label = `Last ${String(RECENT_DAYS)} days`;
    rows.push({
      kind: "date",
      key: "recent",
      field,
      label,
      value: { id: `${recent.from}..${recent.to}`, name: label },
    });
  }

  const lowered = query.toLowerCase();
  rows.push(
    ...yearRows(
      field,
      (group?.values ?? []).filter((option) => matches(option, lowered)),
    ),
  );
  return rows;
}

function valueRow(
  field: FilterFieldDef,
  option: CatalogValue,
  state: { checked: boolean; toggles: boolean; usedAs?: UsedAs },
): PopupRow {
  return { kind: "value", key: `${field.key}:${option.id}`, field, option, ...state };
}

function usedAs(chips: readonly FilterChip[], field: string, optionId: string): UsedAs | undefined {
  if (chipHasValue(findChip(chips, field, false), optionId)) return "included";
  if (chipHasValue(findChip(chips, field, true), optionId)) return "excluded";
  return undefined;
}

function valueRows(
  field: FilterFieldDef,
  input: RowInput,
  group: CatalogGroup | undefined,
): PopupRow[] {
  const query = input.prefix?.query.toLowerCase() ?? "";
  const exclude = input.prefix?.exclude ?? false;
  const same = findChip(input.chips, field.key, exclude);
  const opposite = findChip(input.chips, field.key, !exclude);

  const selected = (same?.values ?? []).filter((value) => matches(value, query));
  const selectedIds = new Set(selected.map((value) => value.id));
  const rows = selected.map((value) => valueRow(field, value, { checked: true, toggles: true }));
  for (const option of group?.values ?? []) {
    if (selectedIds.has(option.id) || !matches(option, query)) continue;
    const used = chipHasValue(opposite, option.id)
      ? exclude
        ? "included"
        : "excluded"
      : undefined;
    rows.push(valueRow(field, option, { checked: false, toggles: true, usedAs: used }));
  }
  return rows;
}

function suggestionGroups(input: RowInput): PopupGroup[] {
  const query = input.text.trim().toLowerCase();
  const sections: PopupGroup[] = [];
  for (const group of input.catalog) {
    const field = input.fields.find((candidate) => candidate.key === group.field);
    if (!field) continue;
    const found = group.values
      .filter((option) => matches(option, query))
      .slice(0, SUGGESTIONS_PER_FIELD);
    if (found.length === 0) continue;
    const items =
      field.kind === "date"
        ? yearRows(field, found)
        : found.map((option) =>
            valueRow(field, option, {
              checked: false,
              toggles: false,
              usedAs: usedAs(input.chips, field.key, option.id),
            }),
          );
    sections.push({ value: field.label, items });
  }
  return sections;
}

/** Builds the sections the popup shows for the current input. */
export function buildGroups(input: RowInput): PopupGroup[] {
  if (input.mode === "fields") {
    const items: PopupRow[] = input.fields.map((field) => ({
      kind: "field",
      key: field.key,
      field,
    }));
    return [{ value: "", items }];
  }

  if (input.mode === "suggest") return suggestionGroups(input);

  const field = input.fields.find((candidate) => candidate.key === input.prefix?.field);
  if (!field) return [];
  const group = input.catalog.find((candidate) => candidate.field === field.key);
  const items =
    input.mode === "date" ? dateRows(field, input, group) : valueRows(field, input, group);
  return [{ value: "", items }];
}
