import type { PerformanceSortField } from "@/api/performances";
import type { SortDir } from "@/api/types";

interface SortHeaderProps {
  label: string;
  field: PerformanceSortField;
  sort: PerformanceSortField;
  sortDir: SortDir;
  onSort: (field: PerformanceSortField) => void;
}

export function SortHeader({ label, field, sort, sortDir, onSort }: SortHeaderProps) {
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
