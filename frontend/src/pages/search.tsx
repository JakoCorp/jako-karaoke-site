import { MagnifyingGlassIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useSearchParams } from "react-router";

import { performances } from "@/api/performances";
import { PerformanceRow } from "@/components/search/performance-row";

function usePage(searchParams: URLSearchParams) {
  const raw = searchParams.get("page");
  const parsed = raw ? parseInt(raw, 10) : 1;
  return Number.isFinite(parsed) && parsed >= 1 ? parsed : 1;
}

function useSort(searchParams: URLSearchParams): "performance_date" | "play_count" {
  const raw = searchParams.get("sort");
  return raw === "play_count" ? "play_count" : "performance_date";
}

function useSortDir(searchParams: URLSearchParams): "asc" | "desc" {
  const raw = searchParams.get("sort_dir");
  return raw === "asc" ? "asc" : "desc";
}

function usePerPage(searchParams: URLSearchParams): number {
  const raw = searchParams.get("per_page");
  const parsed = raw ? parseInt(raw, 10) : 20;
  return parsed === 50 || parsed === 100 ? parsed : 20;
}

export function SearchPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const q = searchParams.get("q") ?? "";
  const page = usePage(searchParams);
  const sort = useSort(searchParams);
  const sortDir = useSortDir(searchParams);
  const perPage = usePerPage(searchParams);

  const [inputValue, setInputValue] = useState(q);

  useEffect(() => {
    setInputValue(searchParams.get("q") ?? "");
  }, [searchParams]);

  useEffect(() => {
    const timer = setTimeout(() => {
      setSearchParams(
        (prev) => {
          const next = new URLSearchParams(prev);
          if (inputValue.trim()) {
            next.set("q", inputValue.trim());
          } else {
            next.delete("q");
          }
          next.delete("page");
          return next;
        },
        { replace: true },
      );
    }, 300); // 300ms debounce delay
    return () => {
      clearTimeout(timer);
    };
  }, [inputValue, setSearchParams]);

  const { data, isLoading } = useQuery({
    queryKey: ["performances", { q: q.trim() || undefined, page, sort, sortDir, perPage }],
    queryFn: async () => {
      const { data: result, error } = await performances.list({
        q: q.trim() || undefined,
        page,
        per_page: perPage,
        sort,
        sort_dir: sortDir,
      });
      if (error) throw error;
      return result;
    },
  });

  function setPage(next: number) {
    setSearchParams((prev) => {
      const params = new URLSearchParams(prev);
      params.set("page", String(next));
      return params;
    });
  }

  function setPerPage(next: number) {
    setSearchParams((prev) => {
      const params = new URLSearchParams(prev);
      params.set("per_page", String(next));
      params.delete("page");
      return params;
    });
  }

  function handleSortChange(field: "performance_date" | "play_count") {
    setSearchParams((prev) => {
      const params = new URLSearchParams(prev);
      const currentSort = params.get("sort") ?? "performance_date";
      const currentDir = params.get("sort_dir") ?? "desc";
      if (currentSort === field) {
        params.set("sort_dir", currentDir === "desc" ? "asc" : "desc");
      } else {
        params.set("sort", field);
        params.delete("sort_dir");
      }
      params.delete("page");
      return params;
    });
  }

  const total = data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / perPage));

  return (
    <div>
      <div className="search-header">
        <div className="search-input-wrapper">
          <MagnifyingGlassIcon size={16} className="search-input-icon" />
          <input
            type="search"
            className="form-input search-input"
            placeholder="Search by title, song, or artist…"
            value={inputValue}
            onChange={(e) => {
              setInputValue(e.target.value);
            }}
            aria-label="Search performances"
          />
        </div>
      </div>

      <div className="search-meta">
        <span>
          {isLoading ? (
            "Loading…"
          ) : (
            <>
              {total.toLocaleString()} result{total !== 1 ? "s" : ""}
              {q.trim() && <> for &ldquo;{q.trim()}&rdquo;</>}
            </>
          )}
        </span>
        <label className="search-per-page-label">
          Show{" "}
          <select
            className="search-per-page-select"
            value={perPage}
            onChange={(e) => {
              const val = parseInt(e.target.value, 10);
              if (val === 20 || val === 50 || val === 100) setPerPage(val);
            }}
            aria-label="Results per page"
          >
            <option value={20}>20</option>
            <option value={50}>50</option>
            <option value={100}>100</option>
          </select>
        </label>
      </div>

      <div className="search-results">
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
          <div className="perf-header-label perf-header-label--right">Duration</div>
          <SortHeader
            label="Date"
            field="performance_date"
            sort={sort}
            sortDir={sortDir}
            onSort={handleSortChange}
          />
        </div>
        {data?.items.map((perf) => (
          <PerformanceRow key={perf.id} performance={perf} />
        ))}
        {!isLoading && data?.items.length === 0 && (
          <div className="search-empty">No performances found.</div>
        )}
      </div>

      {totalPages > 1 && (
        <div className="search-pagination">
          <button
            className="page-btn"
            disabled={page <= 1}
            onClick={() => {
              setPage(page - 1);
            }}
            aria-label="Previous page"
          >
            ‹
          </button>
          {Array.from({ length: Math.min(totalPages, 7) }, (_, i) => {
            const pageNum = getPageWindow(page, totalPages)[i];
            if (pageNum === undefined) return null;
            return (
              <button
                key={pageNum}
                className={`page-btn${pageNum === page ? " page-btn--active" : ""}`}
                onClick={() => {
                  setPage(pageNum);
                }}
                aria-current={pageNum === page ? "page" : undefined}
              >
                {pageNum}
              </button>
            );
          })}
          <button
            className="page-btn"
            disabled={page >= totalPages}
            onClick={() => {
              setPage(page + 1);
            }}
            aria-label="Next page"
          >
            ›
          </button>
        </div>
      )}
    </div>
  );
}

interface SortHeaderProps {
  label: string;
  field: "performance_date" | "play_count";
  sort: "performance_date" | "play_count";
  sortDir: "asc" | "desc";
  onSort: (field: "performance_date" | "play_count") => void;
}

function SortHeader({ label, field, sort, sortDir, onSort }: SortHeaderProps) {
  const isActive = sort === field;
  return (
    <button
      className={`perf-header-sort-btn${isActive ? " perf-header-sort-btn--active" : ""}`}
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

/** Returns up to 7 page numbers centered around the current page. */
function getPageWindow(current: number, total: number): number[] {
  const half = 3;
  let start = Math.max(1, current - half);
  const end = Math.min(total, start + 6);
  start = Math.max(1, end - 6);
  return Array.from({ length: end - start + 1 }, (_, i) => start + i);
}
