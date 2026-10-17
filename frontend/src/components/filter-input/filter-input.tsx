import { Combobox } from "@base-ui/react/combobox";
import { MagnifyingGlassIcon, XIcon } from "@phosphor-icons/react";
import { useRef, useState } from "react";
import type { KeyboardEvent as ReactKeyboardEvent } from "react";

import type { CatalogGroup, CatalogValue } from "@/api/filters";
import {
  fieldPrefixText,
  parseFieldPrefix,
  toggleChipValue,
  type FilterChip,
  type FilterValue,
} from "@/lib/filter-query";

import { FilterChipPill } from "./filter-chip";
import { FilterPopup } from "./filter-popup";
import { buildGroups, type PopupMode, type PopupRow } from "./rows";
import type { FilterFieldDef } from "./types";

interface FilterInputProps {
  fields: readonly FilterFieldDef[];
  chips: readonly FilterChip[];
  onChipsChange: (chips: FilterChip[]) => void;
  /** Raw input text. A `field:` prefix is a value search, anything else is a plain text search. */
  text: string;
  onTextChange: (text: string) => void;
  /** Every value that can be picked, searched locally. Empty while still loading. */
  catalog: readonly CatalogGroup[];
  placeholder: string;
  ariaLabel: string;
}

/** Keeps only what a chip stores of a catalog value, so API additions never reach the URL. */
function toFilterValue(option: CatalogValue): FilterValue {
  return { id: option.id, name: option.name };
}

/**
 * Search box that also holds filters as chips. Focusing it lists the filterable fields, picking a
 * field lists its values, and typing text suggests matching values across fields. Typing
 * `field:text` jumps straight to that field's values.
 */
export function FilterInput({
  fields,
  chips,
  onChipsChange,
  text,
  onTextChange,
  catalog,
  placeholder,
  ariaLabel,
}: FilterInputProps) {
  const boxRef = useRef<HTMLLabelElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const highlightedRef = useRef<PopupRow | undefined>(undefined);
  const [open, setOpen] = useState(false);
  const [today] = useState(() => new Date());

  const fieldKeys = fields.map((field) => field.key);
  const prefix = parseFieldPrefix(text, fieldKeys);
  const prefixField = prefix ? fields.find((field) => field.key === prefix.field) : undefined;
  let mode: PopupMode = "suggest";
  if (text === "") mode = "fields";
  else if (prefixField) mode = prefixField.kind === "date" ? "date" : "values";

  const popupGroups = buildGroups({ mode, fields, text, prefix, catalog, chips, today });
  const hasRows = popupGroups.some((group) => group.items.length > 0);

  function removeChip(index: number) {
    onChipsChange(chips.filter((_, i) => i !== index));
    if (prefix) onTextChange("");
  }

  function close() {
    setOpen(false);
    if (prefix) onTextChange("");
  }

  function handleTextChange(next: string) {
    setOpen(true);
    onTextChange(next);
  }

  function pickDate(field: FilterFieldDef, value: FilterValue) {
    const others = chips.filter((chip) => chip.field !== field.key);
    onChipsChange([...others, { field: field.key, exclude: false, values: [value] }]);
    setOpen(false);
    onTextChange("");
  }

  function activate(row: PopupRow, viaKeyboard: boolean) {
    switch (row.kind) {
      case "field":
        onTextChange(fieldPrefixText(row.field.key, false));
        return;
      case "value": {
        if (viaKeyboard && row.checked) {
          close();
          return;
        }
        const exclude = row.toggles ? (prefix?.exclude ?? false) : false;
        onChipsChange(toggleChipValue(chips, row.field.key, exclude, toFilterValue(row.option)));
        onTextChange(row.toggles ? fieldPrefixText(row.field.key, exclude) : "");
        return;
      }
      case "date":
        pickDate(row.field, row.value);
        return;
    }
  }

  function changeExclude(exclude: boolean) {
    if (!prefix) return;
    onTextChange(`${fieldPrefixText(prefix.field, exclude)}${text.slice(text.indexOf(":") + 1)}`);
  }

  function editChip(index: number) {
    const chip = chips[index];
    if (!chip) return;
    setOpen(true);
    onTextChange(fieldPrefixText(chip.field, chip.exclude));
    inputRef.current?.focus();
  }

  function clearAll() {
    onChipsChange([]);
    onTextChange("");
    inputRef.current?.focus();
  }

  function handleKeyDown(event: ReactKeyboardEvent<HTMLInputElement>) {
    if (event.key === "Enter" && mode === "suggest" && !highlightedRef.current) {
      setOpen(false);
      return;
    }
    if (event.key !== "Backspace") return;
    if (text === "" && chips.length > 0) {
      event.preventDefault();
      removeChip(chips.length - 1);
    } else if (prefix && text === fieldPrefixText(prefix.field, prefix.exclude)) {
      event.preventDefault();
      onTextChange("");
    }
  }

  function handleOpenChange(next: boolean, details: Combobox.Root.ChangeEventDetails) {
    if (next) {
      setOpen(true);
      return;
    }
    if (details.reason === "item-press") return;
    if (details.reason === "focus-out") {
      const related = details.event instanceof FocusEvent ? details.event.relatedTarget : null;
      if (related instanceof Element && related.closest(".filter-popup")) return;
    }
    close();
  }

  function handleInputValueChange(next: string, details: Combobox.Root.ChangeEventDetails) {
    if (details.reason === "input-change") handleTextChange(next);
  }

  const hasContent = chips.length > 0 || text !== "";

  return (
    <Combobox.Root<PopupRow>
      items={popupGroups}
      filter={null}
      value={null}
      onValueChange={(row, details) => {
        if (row) activate(row, details.event instanceof KeyboardEvent);
      }}
      onItemHighlighted={(row) => {
        highlightedRef.current = row;
      }}
      inputValue={text}
      onInputValueChange={handleInputValueChange}
      open={open && (mode !== "suggest" || hasRows)}
      onOpenChange={handleOpenChange}
      autoHighlight={mode !== "suggest"}
      autoComplete="none"
      itemToStringValue={(row) => row.key}
    >
      <div className="filter-input">
        <label ref={boxRef} className="filter-input-box">
          <MagnifyingGlassIcon size={16} className="filter-input-icon" aria-hidden="true" />
          {chips.map((chip, index) => {
            const field = fields.find((candidate) => candidate.key === chip.field);
            if (!field) return null;
            return (
              <FilterChipPill
                key={`${chip.field}:${String(index)}`}
                chip={chip}
                field={field}
                onEdit={() => {
                  editChip(index);
                }}
                onRemove={() => {
                  removeChip(index);
                }}
              />
            );
          })}
          <Combobox.Input
            ref={inputRef}
            className="filter-input-field"
            aria-label={ariaLabel}
            spellCheck={false}
            placeholder={chips.length === 0 ? placeholder : undefined}
            onFocus={() => {
              setOpen(true);
            }}
            onKeyDown={handleKeyDown}
          />
          {hasContent && (
            <button
              type="button"
              className="filter-input-clear"
              onClick={clearAll}
              aria-label="Clear search and filters"
            >
              <XIcon size={14} aria-hidden="true" />
            </button>
          )}
        </label>
      </div>
      <FilterPopup
        anchor={boxRef}
        mode={mode}
        groups={popupGroups}
        field={prefixField}
        exclude={prefix?.exclude ?? false}
        onBack={() => {
          onTextChange("");
          inputRef.current?.focus();
        }}
        onExcludeChange={changeExclude}
        onPickDate={pickDate}
      />
    </Combobox.Root>
  );
}
