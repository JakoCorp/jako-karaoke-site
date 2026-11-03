import type { SortDir } from "@/api/types";

interface SortHeaderProps<Field extends string> {
  label: string;
  field: Field;
  sort: Field;
  sortDir: SortDir;
  onSort: (field: Field) => void;
}

export function SortHeader<Field extends string>({
  label,
  field,
  sort,
  sortDir,
  onSort,
}: SortHeaderProps<Field>) {
  const isActive = sort === field;
  return (
    <button
      className={
        isActive ? "perf-header-sort-btn perf-header-sort-btn--active" : "perf-header-sort-btn"
      }
      onClick={() => onSort(field)}
      aria-label={
        isActive
          ? `${label}, sorted ${sortDir === "asc" ? "ascending" : "descending"}, click to reverse`
          : `Sort by ${label}`
      }
    >
      {label}
      {isActive && <span aria-hidden="true">{sortDir === "asc" ? "↑" : "↓"}</span>}
    </button>
  );
}
