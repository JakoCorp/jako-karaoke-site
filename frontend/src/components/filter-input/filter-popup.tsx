import { Combobox } from "@base-ui/react/combobox";
import { CaretLeftIcon, CheckIcon, ProhibitIcon } from "@phosphor-icons/react";
import { useState } from "react";
import type { MouseEvent } from "react";

import { dateRangeValue, parseDateRange, type FilterValue } from "@/lib/filter-query";

import type { PopupGroup, PopupMode, PopupRow } from "./rows";
import type { FilterFieldDef } from "./types";

interface FilterPopupProps {
  anchor: React.RefObject<HTMLElement | null>;
  mode: PopupMode;
  groups: readonly PopupGroup[];
  /** Field whose values are listed, when a field prefix is typed. */
  field?: FilterFieldDef;
  exclude: boolean;
  onBack: () => void;
  onExcludeChange: (exclude: boolean) => void;
  onPickDate: (field: FilterFieldDef, value: FilterValue) => void;
}

/** Keeps focus in the search input when a control in the popup is pressed. */
function keepInputFocus(event: MouseEvent) {
  event.preventDefault();
}

export function FilterPopup({
  anchor,
  mode,
  groups,
  field,
  exclude,
  onBack,
  onExcludeChange,
  onPickDate,
}: FilterPopupProps) {
  const isEmpty = groups.every((group) => group.items.length === 0);

  return (
    <Combobox.Portal>
      <Combobox.Positioner
        anchor={anchor}
        side="bottom"
        align="start"
        sideOffset={6}
        className="filter-popup-positioner"
      >
        <Combobox.Popup className="filter-popup" initialFocus={false}>
          {(mode === "values" || mode === "date") && field && (
            <div className="filter-popup-header">
              <button
                type="button"
                className="filter-popup-back"
                onMouseDown={keepInputFocus}
                onClick={onBack}
              >
                <CaretLeftIcon size={14} aria-hidden="true" />
                {field.label}
              </button>
              {mode === "values" && (
                <label className="filter-popup-exclude">
                  <input
                    type="checkbox"
                    checked={exclude}
                    onMouseDown={keepInputFocus}
                    onChange={(event) => {
                      onExcludeChange(event.target.checked);
                    }}
                  />
                  <ProhibitIcon size={12} aria-hidden="true" />
                  Exclude
                </label>
              )}
            </div>
          )}

          <Combobox.List className="filter-popup-list">
            {(group: PopupGroup) => (
              <Combobox.Group key={group.value || "rows"} items={group.items}>
                {group.value && (
                  <Combobox.GroupLabel className="filter-popup-heading">
                    {group.value}
                  </Combobox.GroupLabel>
                )}
                <Combobox.Collection>
                  {(row: PopupRow) => (
                    <Combobox.Item
                      key={row.key}
                      value={row}
                      className="filter-popup-row"
                      disabled={row.kind === "value" && row.usedAs !== undefined}
                    >
                      <PopupRowContent row={row} />
                    </Combobox.Item>
                  )}
                </Combobox.Collection>
              </Combobox.Group>
            )}
          </Combobox.List>
          {isEmpty && <div className="filter-popup-empty">No matches.</div>}

          {mode === "date" && field && <CustomDateRange field={field} onPick={onPickDate} />}
        </Combobox.Popup>
      </Combobox.Positioner>
    </Combobox.Portal>
  );
}

function PopupRowContent({ row }: { row: PopupRow }) {
  if (row.kind === "field") {
    const FieldIcon = row.field.icon;
    return (
      <>
        <FieldIcon size={16} aria-hidden="true" />
        <span className="filter-popup-label">{row.field.label}</span>
        <span className="filter-popup-hint">{row.field.hint}</span>
      </>
    );
  }

  if (row.kind === "value") {
    return (
      <>
        {row.toggles && (
          <span
            className={
              row.checked ? "filter-popup-check filter-popup-check--on" : "filter-popup-check"
            }
            aria-hidden="true"
          >
            {row.checked && <CheckIcon size={12} weight="bold" />}
          </span>
        )}
        <span className="filter-popup-label">{row.option.name}</span>
        {row.usedAs && (
          <span className="filter-popup-hint">
            {row.usedAs === "included" ? "Included" : "Excluded"}
          </span>
        )}
      </>
    );
  }

  return <span className="filter-popup-label">{row.label}</span>;
}

function CustomDateRange({
  field,
  onPick,
}: {
  field: FilterFieldDef;
  onPick: (field: FilterFieldDef, value: FilterValue) => void;
}) {
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const range = parseDateRange(`${from}..${to}`);

  return (
    <div className="filter-popup-custom">
      <label className="filter-popup-date">
        From
        <input
          type="date"
          className="form-input"
          value={from}
          max={to || undefined}
          onChange={(event) => {
            setFrom(event.target.value);
          }}
        />
      </label>
      <label className="filter-popup-date">
        To
        <input
          type="date"
          className="form-input"
          value={to}
          min={from || undefined}
          onChange={(event) => {
            setTo(event.target.value);
          }}
        />
      </label>
      <button
        type="button"
        className="btn btn-secondary"
        disabled={!range}
        onClick={() => {
          if (range) onPick(field, dateRangeValue(range));
        }}
      >
        Apply
      </button>
    </div>
  );
}
