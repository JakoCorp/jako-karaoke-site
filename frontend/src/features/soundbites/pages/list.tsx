import { MagnifyingGlassIcon } from "@phosphor-icons/react";

import { Pagination } from "@/components/pagination";
import { SortHeader } from "@/features/performances";
import { useSoundbites } from "@/hooks/api/soundbites";

import { SoundbiteRow } from "../components/row";
import { useSoundbitePlayer } from "../use-soundbite-player";
import { PER_PAGE_OPTIONS, useSoundbiteSearch } from "../use-soundbite-search";

export function SoundbitesPage() {
  const search = useSoundbiteSearch();
  const player = useSoundbitePlayer();

  const { data, isLoading } = useSoundbites(search.params);

  const total = data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / search.perPage));

  return (
    <div>
      <div className="search-header">
        <div className="soundbite-search-wrapper">
          <MagnifyingGlassIcon size={16} className="soundbite-search-icon" />
          <input
            type="search"
            className="form-input search-input"
            placeholder="Search by title or artist…"
            value={search.inputValue}
            onChange={(event) => {
              search.setInputValue(event.target.value);
            }}
            aria-label="Search soundbites"
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
              {search.q && <> for &ldquo;{search.q}&rdquo;</>}
            </>
          )}
        </span>
        <label className="search-per-page-label">
          Show{" "}
          <select
            className="search-per-page-select"
            value={search.perPage}
            onChange={(event) => {
              const value = parseInt(event.target.value, 10);
              const option = PER_PAGE_OPTIONS.find((candidate) => candidate === value);
              if (option !== undefined) search.setPerPage(option);
            }}
            aria-label="Results per page"
          >
            {PER_PAGE_OPTIONS.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="search-results">
        <div className="soundbite-row-header">
          <div />
          <div className="perf-header-label">Title</div>
          <SortHeader
            label="Plays"
            field="play_count"
            sort={search.sort}
            sortDir={search.sortDir}
            onSort={search.setSort}
          />
          <SortHeader
            label="Duration"
            field="duration"
            sort={search.sort}
            sortDir={search.sortDir}
            onSort={search.setSort}
          />
          <SortHeader
            label="Date"
            field="soundbite_date"
            sort={search.sort}
            sortDir={search.sortDir}
            onSort={search.setSort}
          />
        </div>
        {data?.items.map((soundbite) => (
          <SoundbiteRow
            key={soundbite.id}
            soundbite={soundbite}
            isActive={player.activeId === soundbite.id}
            onToggle={player.toggle}
          />
        ))}
        {!isLoading && data?.items.length === 0 && (
          <div className="search-empty">No soundbites found.</div>
        )}
      </div>

      <Pagination page={search.page} totalPages={totalPages} onPageChange={search.setPage} />
    </div>
  );
}
