import { MagnifyingGlassIcon } from "@phosphor-icons/react";
import { useEffect, useRef, useState } from "react";

import type { PerformanceSortDir, PerformanceSortField } from "@/api/performances";
import { useInfinitePerformances } from "@/hooks/api/performances";
import { useDebounced } from "@/hooks/use-debounced";

import { PerformanceRow } from "./performance-row";
import { SortHeader } from "./sort-header";

interface FilteredPerformanceListProps {
  filter: { song_id?: string };
}

export function FilteredPerformanceList({ filter }: FilteredPerformanceListProps) {
  const [inputValue, setInputValue] = useState("");
  const [sort, setSort] = useState<PerformanceSortField>("performance_date");
  const [sortDir, setSortDir] = useState<PerformanceSortDir>("desc");
  const debouncedInput = useDebounced(inputValue);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  const q = debouncedInput.trim() || undefined;
  const { data, isLoading, isFetchingNextPage, hasNextPage, fetchNextPage } =
    useInfinitePerformances({ ...filter, q, sort, sort_dir: sortDir });

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting && hasNextPage && !isFetchingNextPage) {
          void fetchNextPage();
        }
      },
      { root: scrollContainerRef.current, rootMargin: "200px" },
    );
    if (sentinelRef.current) observer.observe(sentinelRef.current);
    return () => {
      observer.disconnect();
    };
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

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
    <div className="filtered-perf-container">
      <div className="filtered-perf-search-row">
        <div className="search-input-wrapper">
          <MagnifyingGlassIcon size={14} className="search-input-icon" />
          <input
            type="search"
            className="filtered-perf-search-input search-input"
            placeholder="Search performances…"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            aria-label="Search performances"
          />
        </div>
      </div>
      <div className="filtered-perf-meta">
        {isLoading ? "Loading…" : `${total.toLocaleString()} performance${total !== 1 ? "s" : ""}`}
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
  );
}
