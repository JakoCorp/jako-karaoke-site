import { useEffect, useState } from "react";

import { ConfirmDialog } from "@/components/confirm-dialog";
import { formatBytes } from "@/lib/format";
import { selectTotalBytes, useOfflineStore } from "@/store/offline";

import { clearOffline } from "../lib/save-offline";

export function StorageHeader() {
  const totalBytes = useOfflineStore(selectTotalBytes);
  const hasEntries = useOfflineStore((s) => Object.keys(s.entries).length > 0);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [quotaBytes, setQuotaBytes] = useState<number | null>(null);

  useEffect(() => {
    async function loadQuota() {
      const estimate = await navigator.storage.estimate();
      setQuotaBytes(estimate.quota ?? null);
    }
    void loadQuota();
  }, []);

  return (
    <div className="local-storage-header">
      <div className="local-storage-usage">
        <span className="local-storage-total">{formatBytes(totalBytes)} used</span>
        {quotaBytes !== null && (
          <span className="playlist-detail-sub">of {formatBytes(quotaBytes)} available</span>
        )}
      </div>
      <button
        className="btn btn-danger"
        disabled={!hasEntries}
        onClick={() => {
          setConfirmOpen(true);
        }}
      >
        Clear all
      </button>
      <ConfirmDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        onConfirm={() => {
          void clearOffline();
          setConfirmOpen(false);
        }}
        title="Clear all saved performances?"
        description="Every offline copy will be deleted from this browser."
        confirmLabel="Clear all"
        variant="danger"
      />
    </div>
  );
}
