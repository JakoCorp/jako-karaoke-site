import { MagnifyingGlassIcon } from "@phosphor-icons/react";
import { useEffect, useState } from "react";
import { useSearchParams } from "react-router";

import type { PerformanceSortDir, PerformanceSortField } from "@/api/performances";
import type { SongSortDir } from "@/api/songs";
import { usePerformances } from "@/hooks/api/performances";
import { useSongs } from "@/hooks/api/songs";
import { useDebounced } from "@/hooks/use-debounced";
import { createStateCodec } from "@/lib/url-state";

import { PerformanceRow } from "./performance-row";
import { SongRow } from "./song-row";

type SearchMode = "performances" | "songs";

/**
 * All active search and filter parameters, stored as a single ?query= URL param.
 * Extend this interface as new filter dimensions (tags, artists, date range) are added.
 */
interface SearchState {
  q?: string;
  mode?: "songs";
  sort?: PerformanceSortField;
  perf_sort_dir?: PerformanceSortDir;
  song_sort_dir?: SongSortDir;
  page?: number;
  per_page?: number;
}

const searchStateCodec = createStateCodec<SearchState>({
  compact: (state) => {
    const compact: SearchState = {};
    if (state.q) compact.q = state.q;
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
  const mode: SearchMode = searchState.mode ?? "performances";
  const sort = searchState.sort ?? "performance_date";
  const perfSortDir: PerformanceSortDir = searchState.perf_sort_dir ?? "desc";
  const songSortDir: SongSortDir = searchState.song_sort_dir ?? "desc";
  const page = searchState.page ?? 1;
  const perPage = searchState.per_page ?? 20;

  const [inputValue, setInputValue] = useState(q);
  const [prevQ, setPrevQ] = useState(q);
  const debouncedInput = useDebounced(inputValue);

  if (q !== prevQ) {
    setPrevQ(q);
    setInputValue(q);
  }

  useEffect(() => {
    setSearchParams(
      (prev) => {
        const current = searchStateCodec.decode(prev.get("query") ?? "");
        const next = searchStateCodec.toParam({
          ...current,
          q: debouncedInput.trim() || undefined,
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
  }, [debouncedInput, setSearchParams]);

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

  const { data: perfData, isLoading: perfLoading } = usePerformances(
    { q: q || undefined, page, per_page: perPage, sort, sort_dir: perfSortDir },
    mode === "performances",
  );

  const { data: songData, isLoading: songLoading } = useSongs(
    { q: q || undefined, page, per_page: perPage, sort_dir: songSortDir },
    mode === "songs",
  );

  const isLoading = mode === "performances" ? perfLoading : songLoading;

  function handleModeChange(next: SearchMode) {
    updateSearch({
      mode: next === "songs" ? "songs" : undefined,
      sort: undefined,
      perf_sort_dir: undefined,
      page: undefined,
    });
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
        <div className="search-input-wrapper">
          <MagnifyingGlassIcon size={16} className="search-input-icon" />
          <input
            type="search"
            className="form-input search-input"
            placeholder={
              mode === "songs" ? "Search by title or artist…" : "Search by title, song, or artist…"
            }
            value={inputValue}
            onChange={(event) => {
              setInputValue(event.target.value);
            }}
            aria-label={mode === "songs" ? "Search songs" : "Search performances"}
          />
        </div>
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
            </div>
            {perfData?.items.map((perf) => (
              <PerformanceRow key={perf.id} performance={perf} />
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

interface SortHeaderProps {
  label: string;
  field: PerformanceSortField;
  sort: PerformanceSortField;
  sortDir: PerformanceSortDir;
  onSort: (field: PerformanceSortField) => void;
}

function SortHeader({ label, field, sort, sortDir, onSort }: SortHeaderProps) {
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

/** Returns up to 7 page numbers centered around the current page. */
function getPageWindow(current: number, total: number): number[] {
  const half = 3;
  let start = Math.max(1, current - half);
  const end = Math.min(total, start + 6);
  start = Math.max(1, end - 6);
  return Array.from({ length: end - start + 1 }, (_, i) => start + i);
}
