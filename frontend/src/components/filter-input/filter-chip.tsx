import { XIcon } from "@phosphor-icons/react";

import type { FilterChip } from "@/lib/filter-query";

import type { FilterFieldDef } from "./types";

const VISIBLE_VALUES = 2;

interface FilterChipPillProps {
  chip: FilterChip;
  field: FilterFieldDef;
  onEdit: () => void;
  onRemove: () => void;
}

/** Summarizes chip values, e.g. `Rock, Pop +2`. */
function valueSummary(chip: FilterChip): string {
  const names = chip.values.slice(0, VISIBLE_VALUES).map((value) => value.name);
  const hidden = chip.values.length - names.length;
  return hidden > 0 ? `${names.join(", ")} +${String(hidden)}` : names.join(", ");
}

export function FilterChipPill({ chip, field, onEdit, onRemove }: FilterChipPillProps) {
  const FieldIcon = field.icon;
  const label = `${chip.exclude ? "Not " : ""}${field.label}`;

  return (
    <span className={chip.exclude ? "filter-chip filter-chip--exclude" : "filter-chip"}>
      <button
        type="button"
        className="filter-chip-body"
        onClick={onEdit}
        aria-label={`Edit filter ${label}: ${valueSummary(chip)}`}
      >
        <FieldIcon size={12} aria-hidden="true" />
        <span className="filter-chip-label">{label}</span>
        <span className="filter-chip-value">{valueSummary(chip)}</span>
      </button>
      <button
        type="button"
        className="filter-chip-remove"
        onClick={onRemove}
        aria-label={`Remove filter ${label}`}
      >
        <XIcon size={12} aria-hidden="true" />
      </button>
    </span>
  );
}
