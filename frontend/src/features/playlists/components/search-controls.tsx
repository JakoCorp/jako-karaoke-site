import { MagnifyingGlassIcon } from "@phosphor-icons/react";

import { PLAYLIST_SORT_OPTIONS, type PlaylistSortOption } from "../use-playlist-search";

interface PlaylistSearchControlsProps {
  inputValue: string;
  onInputChange: (value: string) => void;
  sortOption: PlaylistSortOption;
  onSortChange: (value: PlaylistSortOption) => void;
}

export function PlaylistSearchControls({
  inputValue,
  onInputChange,
  sortOption,
  onSortChange,
}: PlaylistSearchControlsProps) {
  return (
    <div className="playlist-controls">
      <div className="playlist-search-wrapper">
        <MagnifyingGlassIcon size={16} className="playlist-search-icon" />
        <input
          type="search"
          className="form-input search-input"
          placeholder="Search playlists…"
          value={inputValue}
          onChange={(e) => onInputChange(e.target.value)}
          aria-label="Search playlists"
        />
      </div>
      <select
        className="playlist-sort-select"
        value={sortOption}
        onChange={(e) => {
          const option = PLAYLIST_SORT_OPTIONS.find((opt) => opt.value === e.target.value);
          if (option) onSortChange(option.value);
        }}
        aria-label="Sort playlists"
      >
        {PLAYLIST_SORT_OPTIONS.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </div>
  );
}
