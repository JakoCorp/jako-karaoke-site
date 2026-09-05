import { Dialog } from "@base-ui/react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { tagsApi } from "@/api/tags";
import { tagKeys, useTags } from "@/hooks/api/tags";

export function TagsAdminTab() {
  const [search, setSearch] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const [newTagName, setNewTagName] = useState("");
  const [createError, setCreateError] = useState<string | null>(null);
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);

  const queryClient = useQueryClient();

  const { data: allTags, isLoading } = useTags();

  const createMutation = useMutation({
    mutationFn: async (name: string) => {
      const { error } = await tagsApi.create({ name });
      if (error) throw error;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: tagKeys.all() });
      setCreateOpen(false);
      setNewTagName("");
      setCreateError(null);
    },
    onError: () => {
      setCreateError("Failed to create tag.");
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      const { error } = await tagsApi.delete(id);
      if (error) throw error;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: tagKeys.all() });
      setConfirmDeleteId(null);
    },
  });

  const filtered = (allTags ?? []).filter((tag) =>
    tag.name.toLowerCase().includes(search.toLowerCase()),
  );

  function handleCreateOpenChange(open: boolean) {
    if (!open) {
      setNewTagName("");
      setCreateError(null);
    }
    setCreateOpen(open);
  }

  return (
    <div className="admin-panel admin-panel--single">
      <div className="admin-panel-header">
        <input
          type="search"
          className="form-input flex-1"
          placeholder="Search tags…"
          value={search}
          onChange={(event) => {
            setSearch(event.target.value);
          }}
          aria-label="Search tags"
        />
        <button
          className="btn btn-primary"
          onClick={() => {
            setCreateOpen(true);
          }}
        >
          New tag
        </button>
      </div>

      {isLoading && <p className="admin-empty">Loading…</p>}
      {!isLoading && filtered.length === 0 && <p className="admin-empty">No tags found.</p>}

      <ul className="admin-tag-list">
        {filtered.map((tag) => (
          <li key={tag.id} className="admin-tag-item">
            <span className="admin-tag-name">{tag.name}</span>
            {confirmDeleteId === tag.id ? (
              <div className="admin-tag-confirm">
                <span className="admin-empty">Delete?</span>
                <button
                  className="btn btn-primary"
                  onClick={() => {
                    deleteMutation.mutate(tag.id);
                  }}
                  disabled={deleteMutation.isPending}
                >
                  Yes
                </button>
                <button
                  className="btn btn-secondary"
                  onClick={() => {
                    setConfirmDeleteId(null);
                  }}
                >
                  No
                </button>
              </div>
            ) : (
              <button
                className="btn btn-secondary"
                onClick={() => {
                  setConfirmDeleteId(tag.id);
                }}
              >
                Delete
              </button>
            )}
          </li>
        ))}
      </ul>

      <Dialog.Root open={createOpen} onOpenChange={handleCreateOpenChange}>
        <Dialog.Portal>
          <Dialog.Backdrop className="dialog-backdrop" />
          <Dialog.Popup className="dialog-popup">
            <Dialog.Title className="admin-dialog-title">New tag</Dialog.Title>
            <form
              className="admin-dialog-form"
              onSubmit={(event) => {
                event.preventDefault();
                createMutation.mutate(newTagName.trim());
              }}
            >
              <div className="form-field">
                <label className="form-label" htmlFor="tag-name">
                  Name
                </label>
                <input
                  id="tag-name"
                  className="form-input"
                  type="text"
                  value={newTagName}
                  onChange={(event) => {
                    setNewTagName(event.target.value);
                  }}
                  required
                />
              </div>
              {createError !== null && <p className="form-error">{createError}</p>}
              <div className="admin-dialog-actions">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => {
                    handleCreateOpenChange(false);
                  }}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="btn btn-primary"
                  disabled={createMutation.isPending || newTagName.trim() === ""}
                >
                  {createMutation.isPending ? "Creating…" : "Create"}
                </button>
              </div>
            </form>
          </Dialog.Popup>
        </Dialog.Portal>
      </Dialog.Root>
    </div>
  );
}
