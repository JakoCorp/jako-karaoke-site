import {
  BookOpenIcon,
  CalendarBlankIcon,
  GuitarIcon,
  SlidersIcon,
  TagIcon,
  TranslateIcon,
  VinylRecordIcon,
} from "@phosphor-icons/react";

import type { FilterField } from "@/api/filters";
import type { FilterFieldDef } from "@/components/filter-input/types";

type KnownFieldDef = FilterFieldDef & { key: FilterField };

/** Filter fields available when listing performances. */
export const PERFORMANCE_FILTER_FIELDS: readonly KnownFieldDef[] = [
  { key: "genre", label: "Genre", hint: "Rock, pop…", icon: VinylRecordIcon, kind: "values" },
  { key: "source", label: "Source", hint: "Anime, game…", icon: BookOpenIcon, kind: "values" },
  {
    key: "language",
    label: "Language",
    hint: "English, Japanese…",
    icon: TranslateIcon,
    kind: "values",
  },
  {
    key: "instrument",
    label: "Instrument",
    hint: "Ukulele, piano…",
    icon: GuitarIcon,
    kind: "values",
  },
  { key: "modifier", label: "Modifier", hint: "Slow version…", icon: SlidersIcon, kind: "values" },
  { key: "misc", label: "Misc", hint: "Other tags", icon: TagIcon, kind: "values" },
  { key: "date", label: "Date", hint: "A year or a range", icon: CalendarBlankIcon, kind: "date" },
];

const SONG_FIELD_KEYS: readonly FilterField[] = ["genre", "source", "language", "misc"];

/** Filter fields available when listing songs. */
export const SONG_FILTER_FIELDS: readonly KnownFieldDef[] = PERFORMANCE_FILTER_FIELDS.filter(
  (field) => SONG_FIELD_KEYS.includes(field.key),
);
