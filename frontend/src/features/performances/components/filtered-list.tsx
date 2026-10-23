import { useRef, useState } from "react";

import type { PerformanceSortField } from "@/api/performances";
import type { SortDir } from "@/api/types";
import { FilterInput } from "@/components/filter-input/filter-input";
import { useFilterCatalog } from "@/hooks/api/filter-catalog";
import { useInfinitePerformances } from "@/hooks/api/performances";
import { useDebounced } from "@/hooks/use-debounced";
import { useInfiniteSentinel } from "@/hooks/use-infinite-sentinel";
import { PERFORMANCE_FILTER_FIELDS } from "@/lib/filter-fields";
import {
  chipExpressions,
  fieldExpression,
  plainSearchText,
  type FilterChip,
} from "@/lib/filter-query";

import { PerformanceRow } from "./row";
import { SortHeader } from "./sort-header";

interface FilteredPerformanceListProps {
  /** The song or singer every listed performance belongs to. Not offered as a filter. */
  scope: { field: "song" | "singer"; id: string };
}

export function FilteredPerformanceList({ scope }: FilteredPerformanceListProps) {
  const [inputValue, setInputValue] = useState("");
  const [chips, setChips] = useState<FilterChip[]>([]);
  const [sort, setSort] = useState<PerformanceSortField>("performance_date");
  const [sortDir, setSortDir] = useState<SortDir>("desc");
  const debouncedInput = useDebounced(inputValue);
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  const { data: catalog = [] } = useFilterCatalog("performances");
  const fieldKeys = PERFORMANCE_FILTER_FIELDS.map((field) => field.key);
  const q = plainSearchText(debouncedInput, fieldKeys).trim() || undefined;
  const filter = [fieldExpression(scope.field, false, [scope.id]), ...chipExpressions(chips)];
  const { data, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage } =
    useInfinitePerformances({ filter, q, sort, sort_dir: sortDir });

  const sentinelRef = useInfiniteSentinel({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
    scrollContainerRef,
  });

  const total = data?.pages[0]?.total ?? 0;
  const performances = data?.pages.flatMap((page) => page.items) ?? [];

  function handleSortChange(field: PerformanceSortField) {
    if (sort === field) {
      setSortDir((prev) => (prev === "desc" ? "asc" : "desc"));
    } else {
      setSort(field);
      setSortDir("desc");
    }
  }

  return (
    <>
      <div className="filtered-perf-search-row">
        <FilterInput
          fields={PERFORMANCE_FILTER_FIELDS}
          chips={chips}
          onChipsChange={setChips}
          text={inputValue}
          onTextChange={setInputValue}
          catalog={catalog}
          placeholder="Search performances…"
          ariaLabel="Search performances"
        />
      </div>
      <div className="filtered-perf-container">
        <div className="filtered-perf-meta">
          {isLoading
            ? "Loading…"
            : `${total.toLocaleString()} performance${total !== 1 ? "s" : ""}`}
        </div>
        <div className="perf-row-header">
          <div />
          <div className="perf-header-label">Title</div>
          <SortHeader
            label="Plays"
            field="play_count"
            sort={sort}
            sortDir={sortDir}
            onSort={handleSortChange}
          />
          <SortHeader
            label="Duration"
            field="duration"
            sort={sort}
            sortDir={sortDir}
            onSort={handleSortChange}
          />
          <SortHeader
            label="Date"
            field="performance_date"
            sort={sort}
            sortDir={sortDir}
            onSort={handleSortChange}
          />
          <div />
        </div>
        <div ref={scrollContainerRef} className="filtered-perf-scroll">
          {performances.map((perf, index) => (
            <PerformanceRow
              key={perf.id}
              performance={perf}
              performances={performances}
              index={index}
            />
          ))}
          {!isLoading && performances.length === 0 && (
            <div className="search-empty">No performances found.</div>
          )}
          <div ref={sentinelRef} />
        </div>
      </div>
    </>
  );
}
