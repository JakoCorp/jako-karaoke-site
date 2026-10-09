import { useShallow } from "zustand/react/shallow";

import { selectEntryList, useOfflineStore } from "@/store/offline";

import { SavedTable } from "../components/saved-table";
import { StorageHeader } from "../components/storage-header";

export function LocalPage() {
  const entries = useOfflineStore(useShallow(selectEntryList));

  return (
    <div>
      <div className="page-header">
        <div className="page-title">Local</div>
        <div className="playlist-detail-sub">Performances saved for offline playback.</div>
      </div>
      <StorageHeader />
      {entries.length === 0 ? (
        <div className="page-empty">
          Nothing saved yet. Use "Save offline" on a performance to add it here.
        </div>
      ) : (
        <SavedTable entries={entries} />
      )}
    </div>
  );
}
