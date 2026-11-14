import type { ReactNode } from "react";
import { useRef } from "react";

import { useInfiniteSentinel } from "@/hooks/use-infinite-sentinel";

interface PickerResults<T> {
  data: { pages: { items: T[] }[] } | undefined;
  isLoading: boolean;
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  fetchNextPage: () => unknown;
}

/** Multi select that lists server search results and loads more as the dropdown scrolls. */
export function ItemPicker<T extends { id: string }>({
  label,
  selected,
  onChange,
  searchPlaceholder,
  getLabel,
  renderDropdownItem,
  search,
  onSearchChange,
  isDebouncing,
  results,
}: {
  label: string;
  selected: T[];
  onChange: (items: T[]) => void;
  searchPlaceholder: string;
  getLabel: (item: T) => string;
  renderDropdownItem?: (item: T) => ReactNode;
  search: string;
  onSearchChange: (search: string) => void;
  isDebouncing: boolean;
  results: PickerResults<T>;
}) {
  const scrollContainerRef = useRef<HTMLUListElement>(null);
  const sentinelRef = useInfiniteSentinel<HTMLLIElement>({
    hasNextPage: results.hasNextPage,
    isFetchingNextPage: results.isFetchingNextPage,
    fetchNextPage: results.fetchNextPage,
    scrollContainerRef,
  });

  const selectedIds = new Set(selected.map((item) => item.id));
  const options = (results.data?.pages.flatMap((page) => page.items) ?? []).filter(
    (item) => !selectedIds.has(item.id),
  );

  return (
    <div className="form-field">
      <span className="form-label">{label}</span>
      {selected.length > 0 && (
        <div className="admin-pills">
          {selected.map((item) => (
            <button
              key={item.id}
              type="button"
              className="admin-pill"
              onClick={() => {
                onChange(selected.filter((entry) => entry.id !== item.id));
              }}
            >
              {getLabel(item)} ×
            </button>
          ))}
        </div>
      )}
      <input
        type="search"
        className="form-input"
        placeholder={searchPlaceholder}
        value={search}
        onChange={(event) => {
          onSearchChange(event.target.value);
        }}
      />
      <ul ref={scrollContainerRef} className="admin-multiselect-list" hidden={search.trim() === ""}>
        {options.length === 0 && (
          <li className="admin-multiselect-empty">
            {results.isLoading || isDebouncing || results.hasNextPage ? "Searching…" : "No results"}
          </li>
        )}
        {options.map((item) => (
          <li key={item.id}>
            <button
              type="button"
              className="admin-picker-item"
              onClick={() => {
                onChange([...selected, item]);
                onSearchChange("");
              }}
            >
              {renderDropdownItem ? renderDropdownItem(item) : getLabel(item)}
            </button>
          </li>
        ))}
        <li ref={sentinelRef} />
        {results.isFetchingNextPage && <li className="admin-multiselect-empty">Loading more…</li>}
      </ul>
    </div>
  );
}
