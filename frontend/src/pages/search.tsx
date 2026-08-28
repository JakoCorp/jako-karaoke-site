import { MagnifyingGlassIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useSearchParams } from "react-router";

import { performances } from "@/api/performances";
import { PerformanceRow } from "@/components/search/performance-row";

/**
 * All active search and filter parameters, stored as a single ?query= URL param.
 * Extend this interface as new filter dimensions (tags, artists, date range) are added.
 */
interface SearchState {
  q?: string;
  sort?: "performance_date" | "play_count";
  sort_dir?: "asc" | "desc";
  page?: number;
  per_page?: number;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/**
 * Serializes state to URL-safe base64 (RFC 4648 §5), substituting + and / so the
 * result can appear in a query string without percent-encoding.
 */
function encodeSearchState(state: SearchState): string {
  return btoa(JSON.stringify(state)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

/**
 * Deserializes and validates a ?query= value. Fields are checked individually so a
 * corrupted or manually edited URL degrades gracefully to defaults rather than throwing.
 */
function decodeSearchState(encoded: string): SearchState {
  if (!encoded) return {};
  try {
    const padded = encoded.replace(/-/g, "+").replace(/_/g, "/");
    const remainder = padded.length % 4;
    const normalized = remainder > 0 ? padded + "=".repeat(4 - remainder) : padded;
    const parsed: unknown = JSON.parse(atob(normalized));
    if (!isRecord(parsed)) return {};
    const state: SearchState = {};
    if (typeof parsed.q === "string" && parsed.q) state.q = parsed.q;
    if (parsed.sort === "performance_date" || parsed.sort === "play_count")
      state.sort = parsed.sort;
    if (parsed.sort_dir === "asc" || parsed.sort_dir === "desc") state.sort_dir = parsed.sort_dir;
    if (typeof parsed.page === "number" && parsed.page >= 1) state.page = Math.floor(parsed.page);
    if (parsed.per_page === 50 || parsed.per_page === 100) state.per_page = parsed.per_page;
    return state;
  } catch {
    return {};
  }
}

/** Omits fields that match their defaults so the encoded blob stays short. */
function compactState(state: SearchState): SearchState {
  const compact: SearchState = {};
  if (state.q) compact.q = state.q;
  if (state.sort && state.sort !== "performance_date") compact.sort = state.sort;
  if (state.sort_dir && state.sort_dir !== "desc") compact.sort_dir = state.sort_dir;
  if (state.page && state.page > 1) compact.page = state.page;
  if (state.per_page && state.per_page !== 20) compact.per_page = state.per_page;
  return compact;
}

function stateToParam(state: SearchState): string | null {
  const compact = compactState(state);
  return Object.keys(compact).length > 0 ? encodeSearchState(compact) : null;
}

export function SearchPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const searchState = decodeSearchState(searchParams.get("query") ?? "");

  const q = searchState.q ?? "";
  const sort = searchState.sort ?? "performance_date";
  const sortDir = searchState.sort_dir ?? "desc";
  const page = searchState.page ?? 1;
  const perPage = searchState.per_page ?? 20;

  const [inputValue, setInputValue] = useState(q);

  useEffect(() => {
    setInputValue(decodeSearchState(searchParams.get("query") ?? "").q ?? "");
  }, [searchParams]);

  useEffect(() => {
    const timer = setTimeout(() => {
      setSearchParams(
        (prev) => {
          const current = decodeSearchState(prev.get("query") ?? "");
          const next = stateToParam({
            ...current,
            q: inputValue.trim() || undefined,
            page: undefined,
          });
          const params = new URLSearchParams(prev);
          if (next) {
            params.set("query", next);
          } else {
            params.delete("query");
          }
          return params;
        },
        { replace: true },
      );
    }, 300); // 300ms debounce delay
    return () => {
      clearTimeout(timer);
    };
  }, [inputValue, setSearchParams]);

  function updateSearch(partial: Partial<SearchState>) {
    setSearchParams((prev) => {
      const current = decodeSearchState(prev.get("query") ?? "");
      const next = stateToParam({ ...current, ...partial });
      const params = new URLSearchParams(prev);
      if (next) {
        params.set("query", next);
      } else {
        params.delete("query");
      }
      return params;
    });
  }

  const { data, isLoading } = useQuery({
    queryKey: ["performances", { q, page, sort, sortDir, perPage }],
    queryFn: async () => {
      const { data: result, error } = await performances.list({
        q: q || undefined,
        page,
        per_page: perPage,
        sort,
        sort_dir: sortDir,
      });
      if (error) throw error;
      return result;
    },
  });

  function handleSortChange(field: "performance_date" | "play_count") {
    if (sort === field) {
      updateSearch({ sort_dir: sortDir === "desc" ? "asc" : "desc", page: undefined });
    } else {
      updateSearch({ sort: field, sort_dir: undefined, page: undefined });
    }
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
              {q && <> for &ldquo;{q}&rdquo;</>}
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
              if (val === 20 || val === 50 || val === 100) {
                updateSearch({ per_page: val, page: undefined });
              }
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
              updateSearch({ page: page - 1 });
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
                  updateSearch({ page: pageNum });
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
              updateSearch({ page: page + 1 });
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
