import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router";

import type { PerformanceSortField } from "@/api/performances";
import type { SortDir } from "@/api/types";
import { FilterInput } from "@/components/filter-input/filter-input";
import { PerformanceRow, SortHeader } from "@/features/performances";
import { useFilterCatalog } from "@/hooks/api/filter-catalog";
import { usePerformances } from "@/hooks/api/performances";
import { useSongs } from "@/hooks/api/songs";
import { useDebounced } from "@/hooks/use-debounced";
import { PERFORMANCE_FILTER_FIELDS, SONG_FILTER_FIELDS } from "@/lib/filter-fields";
import {
  chipExpressions,
  plainSearchText,
  validateChips,
  type FilterChip,
} from "@/lib/filter-query";
import { createStateCodec } from "@/lib/url-state";

import { SongRow } from "./song-row";

type SearchMode = "performances" | "songs";

function filterFieldsFor(mode: SearchMode) {
  return mode === "songs" ? SONG_FILTER_FIELDS : PERFORMANCE_FILTER_FIELDS;
}

function filterFieldKeys(mode: SearchMode): string[] {
  return filterFieldsFor(mode).map((field) => field.key);
}

/** All active search and filter parameters, stored as a single ?query= URL param. */
interface SearchState {
  q?: string;
  filters?: FilterChip[];
  mode?: "songs";
  sort?: PerformanceSortField;
  perf_sort_dir?: SortDir;
  song_sort_dir?: SortDir;
  page?: number;
  per_page?: number;
}

const searchStateCodec = createStateCodec<SearchState>({
  compact: (state) => {
    const compact: SearchState = {};
    if (state.q) compact.q = state.q;
    if (state.filters && state.filters.length > 0) compact.filters = state.filters;
    if (state.mode === "songs") compact.mode = state.mode;
    if (state.sort && state.sort !== "performance_date") compact.sort = state.sort;
    if (state.perf_sort_dir && state.perf_sort_dir !== "desc")
      compact.perf_sort_dir = state.perf_sort_dir;
    if (state.song_sort_dir && state.song_sort_dir !== "desc")
      compact.song_sort_dir = state.song_sort_dir;
    if (state.page && state.page > 1) compact.page = state.page;
    if (state.per_page && state.per_page !== 20) compact.per_page = state.per_page;
    return compact;
  },
  validate: (parsed) => {
    const state: SearchState = {};
    if (typeof parsed.q === "string" && parsed.q) state.q = parsed.q;
    if (parsed.mode === "songs") state.mode = parsed.mode;
    const filters = validateChips(parsed.filters, filterFieldKeys(state.mode ?? "performances"));
    if (filters.length > 0) state.filters = filters;
    if (
      parsed.sort === "performance_date" ||
      parsed.sort === "play_count" ||
      parsed.sort === "duration"
    )
      state.sort = parsed.sort;
    if (parsed.perf_sort_dir === "asc" || parsed.perf_sort_dir === "desc")
      state.perf_sort_dir = parsed.perf_sort_dir;
    if (parsed.song_sort_dir === "asc" || parsed.song_sort_dir === "desc")
      state.song_sort_dir = parsed.song_sort_dir;
    if (typeof parsed.page === "number" && parsed.page >= 1) state.page = Math.floor(parsed.page);
    if (parsed.per_page === 50 || parsed.per_page === 100) state.per_page = parsed.per_page;
    return state;
  },
});

export function SearchPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const searchState = searchStateCodec.decode(searchParams.get("query") ?? "");

  const q = searchState.q ?? "";
  const filters = searchState.filters ?? [];
  const mode: SearchMode = searchState.mode ?? "performances";
  const filterFields = filterFieldsFor(mode);
  const filterParams = filters.length > 0 ? chipExpressions(filters) : undefined;
  const sort = searchState.sort ?? "performance_date";
  const perfSortDir: SortDir = searchState.perf_sort_dir ?? "desc";
  const songSortDir: SortDir = searchState.song_sort_dir ?? "desc";
  const page = searchState.page ?? 1;
  const perPage = searchState.per_page ?? 20;

  const [inputValue, setInputValue] = useState(q);
  const debouncedInput = useDebounced(inputValue);
  const lastSyncedQ = useRef(q);

  useEffect(() => {
    if (q === lastSyncedQ.current) return;
    lastSyncedQ.current = q;
    setInputValue(q);
  }, [q, lastSyncedQ]);

  const debouncedQ = plainSearchText(debouncedInput, filterFieldKeys(mode)).trim();

  useEffect(() => {
    if (debouncedQ === lastSyncedQ.current) return;
    lastSyncedQ.current = debouncedQ;
    setSearchParams(
      (prev) => {
        const current = searchStateCodec.decode(prev.get("query") ?? "");
        const next = searchStateCodec.toParam({
          ...current,
          q: debouncedQ || undefined,
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
  }, [debouncedQ, setSearchParams, lastSyncedQ]);

  function updateSearch(partial: Partial<SearchState>) {
    setSearchParams((prev) => {
      const current = searchStateCodec.decode(prev.get("query") ?? "");
      const next = searchStateCodec.toParam({ ...current, ...partial });
      const params = new URLSearchParams(prev);
      if (next) {
        params.set("query", next);
      } else {
        params.delete("query");
      }
      return params;
    });
  }

  const { data: catalog = [] } = useFilterCatalog(mode);

  const { data: perfData, isLoading: perfLoading } = usePerformances(
    {
      q: q || undefined,
      filter: filterParams,
      page,
      per_page: perPage,
      sort,
      sort_dir: perfSortDir,
    },
    mode === "performances",
  );

  const { data: songData, isLoading: songLoading } = useSongs(
    { q: q || undefined, filter: filterParams, page, per_page: perPage, sort_dir: songSortDir },
    mode === "songs",
  );

  const isLoading = mode === "performances" ? perfLoading : songLoading;

  function handleModeChange(next: SearchMode) {
    const nextKeys = filterFieldKeys(next);
    updateSearch({
      mode: next === "songs" ? "songs" : undefined,
      filters: filters.filter((chip) => nextKeys.includes(chip.field)),
      sort: undefined,
      perf_sort_dir: undefined,
      page: undefined,
    });
  }

  function handleFiltersChange(next: FilterChip[]) {
    updateSearch({ filters: next, page: undefined });
  }

  function handleSortChange(field: PerformanceSortField) {
    if (sort === field) {
      updateSearch({ perf_sort_dir: perfSortDir === "desc" ? "asc" : "desc", page: undefined });
    } else {
      updateSearch({ sort: field, perf_sort_dir: undefined, page: undefined });
    }
  }

  function handleSongSortDirChange() {
    updateSearch({ song_sort_dir: songSortDir === "desc" ? "asc" : "desc", page: undefined });
  }

  const total = (mode === "performances" ? perfData?.total : songData?.total) ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / perPage));

  return (
    <div>
      <div className="search-header">
        <FilterInput
          fields={filterFields}
          chips={filters}
          onChipsChange={handleFiltersChange}
          text={inputValue}
          onTextChange={setInputValue}
          catalog={catalog}
          placeholder={
            mode === "songs" ? "Search by title or artist…" : "Search by title, song, or artist…"
          }
          ariaLabel={mode === "songs" ? "Search songs" : "Search performances"}
        />
        <div className="search-mode-toggle">
          <button
            className={
              mode === "performances"
                ? "search-mode-btn search-mode-btn--active"
                : "search-mode-btn"
            }
            onClick={() => handleModeChange("performances")}
            aria-pressed={mode === "performances"}
          >
            Performances
          </button>
          <button
            className={
              mode === "songs" ? "search-mode-btn search-mode-btn--active" : "search-mode-btn"
            }
            onClick={() => handleModeChange("songs")}
            aria-pressed={mode === "songs"}
          >
            Songs
          </button>
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
            onChange={(event) => {
              const val = parseInt(event.target.value, 10);
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
        {mode === "performances" ? (
          <>
            <div className="perf-row-header">
              <div />
              <div className="perf-header-label">Title</div>
              <SortHeader
                label="Plays"
                field="play_count"
                sort={sort}
                sortDir={perfSortDir}
                onSort={handleSortChange}
              />
              <SortHeader
                label="Duration"
                field="duration"
                sort={sort}
                sortDir={perfSortDir}
                onSort={handleSortChange}
              />
              <SortHeader
                label="Date"
                field="performance_date"
                sort={sort}
                sortDir={perfSortDir}
                onSort={handleSortChange}
              />
              <div />
            </div>
            {perfData?.items.map((perf, index) => (
              <PerformanceRow
                key={perf.id}
                performance={perf}
                performances={perfData.items}
                index={index}
              />
            ))}
            {!perfLoading && perfData?.items.length === 0 && (
              <div className="search-empty">No performances found.</div>
            )}
          </>
        ) : (
          <>
            <div className="song-row-header">
              <div className="perf-header-label">Title</div>
              <button
                className={"perf-header-sort-btn perf-header-sort-btn--active"}
                onClick={handleSongSortDirChange}
                aria-label={`Performances, sorted ${songSortDir === "asc" ? "ascending" : "descending"}, click to reverse`}
              >
                Performances
                <span aria-hidden="true">{songSortDir === "asc" ? "↑" : "↓"}</span>
              </button>
            </div>
            {songData?.items.map((song) => (
              <SongRow key={song.id} song={song} />
            ))}
            {!songLoading && songData?.items.length === 0 && (
              <div className="search-empty">No songs found.</div>
            )}
          </>
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
                className={pageNum === page ? "page-btn page-btn--active" : "page-btn"}
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

/** Returns up to 7 page numbers centered around the current page. */
function getPageWindow(current: number, total: number): number[] {
  const half = 3;
  let start = Math.max(1, current - half);
  const end = Math.min(total, start + 6);
  start = Math.max(1, end - 6);
  return Array.from({ length: end - start + 1 }, (_, i) => start + i);
}
