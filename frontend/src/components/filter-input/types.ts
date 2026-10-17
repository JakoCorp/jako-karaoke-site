import type { Icon } from "@phosphor-icons/react";

/** How a field's values are chosen. `date` fields hold one range, others hold a set of ids. */
export type FilterFieldKind = "values" | "date";

/** Describes one filter field offered by a filter input. */
export interface FilterFieldDef {
  /** Key used in filter expressions, e.g. `genre`. */
  key: string;
  label: string;
  /** Short example shown next to the label in the field list. */
  hint: string;
  icon: Icon;
  kind: FilterFieldKind;
}
