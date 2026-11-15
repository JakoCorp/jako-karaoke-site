import type { ArtistInfo } from "@/api/artists";
import type { SongSummary } from "@/api/songs";
import { useInfiniteArtists } from "@/hooks/api/artists";
import { useInfiniteSongs } from "@/hooks/api/songs";

import { ItemPicker } from "./item-picker";
import { usePickerSearch } from "./use-picker-search";

interface EntityPickerProps<T> {
  label: string;
  selected: T[];
  onChange: (items: T[]) => void;
}

export function SongPicker({ label, selected, onChange }: EntityPickerProps<SongSummary>) {
  const { query, ...searchState } = usePickerSearch();
  const results = useInfiniteSongs({ q: query }, query !== undefined);

  return (
    <ItemPicker<SongSummary>
      label={label}
      selected={selected}
      onChange={onChange}
      searchPlaceholder={`Search ${label.toLowerCase()}…`}
      getLabel={(song) => song.title}
      renderDropdownItem={(song) => (
        <>
          {song.title}
          {song.artists.length > 0 && (
            <span className="admin-item-sub">
              {" "}
              — {song.artists.map((artist) => artist.name).join(", ")}
            </span>
          )}
        </>
      )}
      results={results}
      {...searchState}
    />
  );
}

export function ArtistPicker({ label, selected, onChange }: EntityPickerProps<ArtistInfo>) {
  const { query, ...searchState } = usePickerSearch();
  const results = useInfiniteArtists({ q: query }, query !== undefined);

  return (
    <ItemPicker<ArtistInfo>
      label={label}
      selected={selected}
      onChange={onChange}
      searchPlaceholder={`Search ${label.toLowerCase()}…`}
      getLabel={(artist) => artist.name}
      results={results}
      {...searchState}
    />
  );
}
