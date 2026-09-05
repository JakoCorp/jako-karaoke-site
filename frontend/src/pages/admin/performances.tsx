import { useState } from "react";

import { type PerformanceSummary } from "@/api/performances";
import { usePerformances } from "@/hooks/api/performances";
import { useDebounced } from "@/hooks/use-debounced";
import { formatDate } from "@/lib/format";

import { ImportPerformancesDialog } from "./import-performances-dialog";
import { PerformanceDetailPanel } from "./performance-detail";

export function PerformancesAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [selectedPerformance, setSelectedPerformance] = useState<PerformanceSummary | null>(null);
  const [creating, setCreating] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [importKey, setImportKey] = useState(0);

  const debouncedQuery = useDebounced(searchInput.trim());

  const { data: performancePage, isLoading: performancesLoading } = usePerformances({
    per_page: 100,
    q: debouncedQuery || undefined,
  });

  const performances = performancePage?.items ?? [];

  return (
    <div className="admin-layout">
      <div className="admin-panel">
        <div style={{ display: "flex", flexDirection: "column", gap: "0.25rem" }}>
          <div className="admin-panel-header">
            <input
              type="search"
              className="form-input"
              style={{ flex: 1 }}
              placeholder="Search performances…"
              value={searchInput}
              onChange={(event) => {
                setSearchInput(event.target.value);
              }}
              aria-label="Search performances"
            />
            <button
              className="btn btn-primary"
              onClick={() => {
                setSelectedPerformance(null);
                setCreating(true);
              }}
            >
              New
            </button>
          </div>
          <button
            className="btn btn-secondary"
            style={{ width: "100%" }}
            onClick={() => {
              setImportKey((k) => k + 1);
              setImportOpen(true);
            }}
          >
            Import CSV
          </button>
        </div>
        <ul className="admin-user-list">
          {performancesLoading && <li className="admin-empty">Loading…</li>}
          {!performancesLoading && performances.length === 0 && (
            <li className="admin-empty">No performances found.</li>
          )}
          {performances.map((performance) => (
            <li key={performance.id}>
              <button
                className={
                  selectedPerformance?.id === performance.id && !creating
                    ? "admin-user-item admin-user-item--active"
                    : "admin-user-item"
                }
                onClick={() => {
                  setCreating(false);
                  setSelectedPerformance(performance);
                }}
              >
                <div className="admin-item-title">
                  {performance.title ??
                    `${formatDate(performance.performance_date)} · S${String(performance.stream_number)} #${String(performance.performance_number)}`}
                </div>
                {performance.singers.length > 0 && (
                  <div className="admin-item-sub">
                    {performance.singers.map((singer) => singer.name).join(", ")}
                  </div>
                )}
              </button>
            </li>
          ))}
        </ul>
      </div>

      <div className="admin-panel">
        {creating ? (
          <PerformanceDetailPanel key="new" performance={null} onClose={() => setCreating(false)} />
        ) : selectedPerformance ? (
          <PerformanceDetailPanel
            key={selectedPerformance.id}
            performance={selectedPerformance}
            onClose={() => setSelectedPerformance(null)}
          />
        ) : (
          <p className="admin-empty">Select a performance to view details.</p>
        )}
      </div>

      <ImportPerformancesDialog
        key={importKey}
        open={importOpen}
        onClose={() => {
          setImportOpen(false);
        }}
        onSuccess={() => {
          setImportOpen(false);
        }}
      />
    </div>
  );
}
