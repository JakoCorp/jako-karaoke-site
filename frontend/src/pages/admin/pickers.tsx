import type { ReactNode } from "react";
import { useState } from "react";

import type { TagResponse } from "@/api/tags";

export type TagAssignment<K extends string> =
  | { type: "existing"; tagId: string; kind: K }
  | { type: "pending"; pendingName: string; kind: K };

export function ItemPicker<T extends { id: string }>({
  label,
  allItems,
  selectedIds,
  onToggle,
  searchPlaceholder,
  getLabel,
  renderDropdownItem,
}: {
  label: string;
  allItems: T[];
  selectedIds: string[];
  onToggle: (id: string) => void;
  searchPlaceholder: string;
  getLabel: (item: T) => string;
  renderDropdownItem?: (item: T) => ReactNode;
}) {
  const [search, setSearch] = useState("");

  const filtered = allItems.filter(
    (item) =>
      !selectedIds.includes(item.id) && getLabel(item).toLowerCase().includes(search.toLowerCase()),
  );

  const selected = allItems.filter((item) => selectedIds.includes(item.id));

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
                onToggle(item.id);
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
          setSearch(event.target.value);
        }}
      />
      {search.trim() !== "" && (
        <ul className="admin-multiselect-list">
          {filtered.length === 0 && <li className="admin-multiselect-empty">No results</li>}
          {filtered.map((item) => (
            <li key={item.id}>
              <button
                type="button"
                className="admin-picker-item"
                onClick={() => {
                  onToggle(item.id);
                  setSearch("");
                }}
              >
                {renderDropdownItem ? renderDropdownItem(item) : getLabel(item)}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function TagPicker<K extends string>({
  allTags,
  selected,
  kinds,
  onAddExisting,
  onAddPending,
  onRemove,
  onKindChange,
}: {
  allTags: TagResponse[];
  selected: TagAssignment<K>[];
  kinds: readonly K[];
  onAddExisting: (tagId: string) => void;
  onAddPending: (name: string) => void;
  onRemove: (key: string) => void;
  onKindChange: (key: string, kind: K) => void;
}) {
  const [search, setSearch] = useState("");

  const existingIds = selected
    .filter((a): a is Extract<TagAssignment<K>, { type: "existing" }> => a.type === "existing")
    .map((a) => a.tagId);

  const pendingNames = selected
    .filter((a): a is Extract<TagAssignment<K>, { type: "pending" }> => a.type === "pending")
    .map((a) => a.pendingName.toLowerCase());

  const filtered = allTags.filter(
    (tag) => !existingIds.includes(tag.id) && tag.name.toLowerCase().includes(search.toLowerCase()),
  );

  const searchTrimmed = search.trim();
  const canCreate =
    searchTrimmed !== "" &&
    !allTags.some((tag) => tag.name.toLowerCase() === searchTrimmed.toLowerCase()) &&
    !pendingNames.includes(searchTrimmed.toLowerCase());

  const selectedWithInfo = selected.map((assignment) => {
    if (assignment.type === "existing") {
      return {
        key: assignment.tagId,
        displayName: allTags.find((tag) => tag.id === assignment.tagId)?.name ?? assignment.tagId,
        kind: assignment.kind,
        isPending: false as const,
      };
    }
    return {
      key: assignment.pendingName,
      displayName: assignment.pendingName,
      kind: assignment.kind,
      isPending: true as const,
    };
  });

  return (
    <div className="form-field">
      <span className="form-label">Tags</span>
      {selectedWithInfo.length > 0 && (
        <div className="admin-pills">
          {selectedWithInfo.map(({ key, kind, displayName, isPending }) => (
            <div key={key} className="admin-tag-selection">
              <button
                type="button"
                className={`admin-pill${isPending ? " admin-pill--pending" : ""}`}
                onClick={() => {
                  onRemove(key);
                }}
              >
                {displayName}
                {isPending && <span className="admin-pill-new">new</span>}
                {" ×"}
              </button>
              <select
                className="admin-kind-select"
                value={kind}
                onChange={(event) => {
                  const nextKind = kinds.find((k) => k === event.target.value);
                  if (nextKind !== undefined) onKindChange(key, nextKind);
                }}
              >
                {kinds.map((kindOption) => (
                  <option key={kindOption} value={kindOption}>
                    {kindOption}
                  </option>
                ))}
              </select>
            </div>
          ))}
        </div>
      )}
      <input
        type="search"
        className="form-input"
        placeholder="Search tags…"
        value={search}
        onChange={(event) => {
          setSearch(event.target.value);
        }}
      />
      {search.trim() !== "" && (
        <ul className="admin-multiselect-list">
          {filtered.length === 0 && !canCreate && (
            <li className="admin-multiselect-empty">No results</li>
          )}
          {filtered.map((tag) => (
            <li key={tag.id}>
              <button
                type="button"
                className="admin-picker-item"
                onClick={() => {
                  onAddExisting(tag.id);
                  setSearch("");
                }}
              >
                {tag.name}
              </button>
            </li>
          ))}
          {canCreate && (
            <li>
              <button
                type="button"
                className="admin-picker-item admin-multiselect-create"
                onClick={() => {
                  onAddPending(searchTrimmed);
                  setSearch("");
                }}
              >
                Create "{searchTrimmed}"
              </button>
            </li>
          )}
        </ul>
      )}
    </div>
  );
}
