import { Dialog } from "@base-ui/react";
import { XIcon } from "@phosphor-icons/react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import {
  ARTIST_LINK_KINDS,
  artistsApi,
  type ArtistLinkKind,
  type ArtistSummary,
} from "@/api/artists";
import { artistKeys, useArtists } from "@/hooks/api/artists";
import { useDebounced } from "@/hooks/use-debounced";

import { ArtistDetailPanel } from "./artist-detail";

function isArtistLinkKind(value: string): value is ArtistLinkKind {
  return (ARTIST_LINK_KINDS as readonly string[]).includes(value);
}

type LinkDraft = { url: string; kind: ArtistLinkKind; label: string };

function CreateArtistDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [links, setLinks] = useState<LinkDraft[]>([]);
  const [error, setError] = useState<string | null>(null);

  const queryClient = useQueryClient();

  const createMutation = useMutation({
    mutationFn: async () => {
      const validLinks = links.filter((link) => link.url.trim() !== "");
      const { error: apiError } = await artistsApi.create({
        name: name.trim(),
        description: description.trim() !== "" ? description.trim() : null,
        images: [],
        links: validLinks.map((link) => ({
          url: link.url.trim(),
          kind: link.kind,
          label: link.label.trim() !== "" ? link.label.trim() : null,
        })),
      });
      if (apiError) throw apiError;
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: artistKeys.all() });
      onOpenChange(false);
    },
    onError: () => {
      setError("Failed to create artist.");
    },
  });

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen) {
      setName("");
      setDescription("");
      setLinks([]);
      setError(null);
    }
    onOpenChange(nextOpen);
  }

  function addLink() {
    setLinks((previous) => [...previous, { url: "", kind: "other", label: "" }]);
  }

  function removeLink(index: number) {
    setLinks((previous) => previous.filter((_, i) => i !== index));
  }

  function updateLink(index: number, patch: Partial<LinkDraft>) {
    setLinks((previous) => previous.map((link, i) => (i === index ? { ...link, ...patch } : link)));
  }

  return (
    <Dialog.Root open={open} onOpenChange={handleOpenChange}>
      <Dialog.Portal>
        <Dialog.Backdrop className="dialog-backdrop" />
        <Dialog.Popup className="dialog-popup dialog-popup--wide">
          <Dialog.Title className="admin-dialog-title">New artist</Dialog.Title>
          <form
            className="admin-dialog-form"
            onSubmit={(event) => {
              event.preventDefault();
              createMutation.mutate();
            }}
          >
            <div className="form-field">
              <label className="form-label" htmlFor="artist-name">
                Name
              </label>
              <input
                id="artist-name"
                className="form-input"
                type="text"
                value={name}
                onChange={(event) => {
                  setName(event.target.value);
                }}
                required
              />
            </div>
            <div className="form-field">
              <label className="form-label" htmlFor="artist-description">
                Description (optional)
              </label>
              <textarea
                id="artist-description"
                className="form-input admin-textarea"
                value={description}
                onChange={(event) => {
                  setDescription(event.target.value);
                }}
                rows={3}
              />
            </div>
            <div className="form-field">
              <span className="form-label">Links</span>
              <div className="admin-link-list">
                {links.map((link, index) => (
                  <div key={index} className="admin-link-card">
                    <div className="admin-link-card-header">
                      <select
                        className="admin-kind-select"
                        value={link.kind}
                        onChange={(event) => {
                          if (isArtistLinkKind(event.target.value)) {
                            updateLink(index, { kind: event.target.value });
                          }
                        }}
                      >
                        {ARTIST_LINK_KINDS.map((kind) => (
                          <option key={kind} value={kind}>
                            {kind}
                          </option>
                        ))}
                      </select>
                      <button
                        className="btn btn-secondary"
                        type="button"
                        onClick={() => {
                          removeLink(index);
                        }}
                      >
                        <XIcon />
                      </button>
                    </div>
                    <input
                      className="form-input"
                      type="url"
                      placeholder="URL"
                      value={link.url}
                      onChange={(event) => {
                        updateLink(index, { url: event.target.value });
                      }}
                    />
                    <input
                      className="form-input"
                      type="text"
                      placeholder="Label (optional)"
                      value={link.label}
                      onChange={(event) => {
                        updateLink(index, { label: event.target.value });
                      }}
                    />
                  </div>
                ))}
                <button className="btn btn-secondary" type="button" onClick={addLink}>
                  Add link
                </button>
              </div>
            </div>
            {error !== null && <p className="form-error">{error}</p>}
            <div className="admin-dialog-actions">
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => {
                  handleOpenChange(false);
                }}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="btn btn-primary"
                disabled={createMutation.isPending || name.trim() === ""}
              >
                {createMutation.isPending ? "Creating…" : "Create"}
              </button>
            </div>
          </form>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

export function ArtistsAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [selectedArtist, setSelectedArtist] = useState<ArtistSummary | null>(null);
  const [createOpen, setCreateOpen] = useState(false);

  const debouncedQuery = useDebounced(searchInput.trim());

  const { data: artistPage, isLoading: artistsLoading } = useArtists({
    per_page: 100,
    q: debouncedQuery || undefined,
  });

  const artists = artistPage?.items ?? [];

  return (
    <div className="admin-layout">
      <div className="admin-panel">
        <div className="admin-panel-header">
          <input
            type="search"
            className="form-input"
            style={{ flex: 1 }}
            placeholder="Search artists…"
            value={searchInput}
            onChange={(event) => {
              setSearchInput(event.target.value);
            }}
            aria-label="Search artists"
          />
          <button
            className="btn btn-primary"
            onClick={() => {
              setCreateOpen(true);
            }}
          >
            New
          </button>
        </div>
        <ul className="admin-user-list">
          {artistsLoading && <li className="admin-empty">Loading…</li>}
          {!artistsLoading && artists.length === 0 && (
            <li className="admin-empty">No artists found.</li>
          )}
          {artists.map((artist) => (
            <li key={artist.id}>
              <button
                className={`admin-user-item${selectedArtist?.id === artist.id ? " admin-user-item--active" : ""}`}
                onClick={() => {
                  setSelectedArtist(artist);
                }}
              >
                <div className="admin-item-title">{artist.name}</div>
                {artist.description && <div className="admin-item-sub">{artist.description}</div>}
              </button>
            </li>
          ))}
        </ul>
      </div>

      <div className="admin-panel">
        {selectedArtist ? (
          <ArtistDetailPanel
            key={selectedArtist.id}
            artist={selectedArtist}
            onDeleted={() => {
              setSelectedArtist(null);
            }}
          />
        ) : (
          <p className="admin-empty">Select an artist to view details.</p>
        )}
      </div>

      <CreateArtistDialog open={createOpen} onOpenChange={setCreateOpen} />
    </div>
  );
}
