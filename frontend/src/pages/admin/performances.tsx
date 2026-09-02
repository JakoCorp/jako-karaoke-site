import { useState } from "react";

import { type PerformanceSummary } from "@/api/performances";
import { usePerformances } from "@/hooks/api/performances";
import { useDebounced } from "@/hooks/use-debounced";
import { formatDate } from "@/lib/format";

import { PerformanceDetailPanel } from "./performance-detail";

export function PerformancesAdminTab() {
  const [searchInput, setSearchInput] = useState("");
  const [selectedPerformance, setSelectedPerformance] = useState<PerformanceSummary | null>(null);
  const [creating, setCreating] = useState(false);

  const debouncedQuery = useDebounced(searchInput.trim());

  const { data: performancePage, isLoading: performancesLoading } = usePerformances({
    per_page: 100,
    q: debouncedQuery || undefined,
  });

  const performances = performancePage?.items ?? [];

  return (
    <div className="admin-layout">
      <div className="admin-panel">
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
        <ul className="admin-user-list">
          {performancesLoading && <li className="admin-empty">Loading…</li>}
          {!performancesLoading && performances.length === 0 && (
            <li className="admin-empty">No performances found.</li>
          )}
          {performances.map((performance) => (
            <li key={performance.id}>
              <button
                className={`admin-user-item${selectedPerformance?.id === performance.id && !creating ? " admin-user-item--active" : ""}`}
                onClick={() => {
                  setCreating(false);
                  setSelectedPerformance(performance);
                }}
              >
                <div className="admin-item-title">
                  {performance.title ?? formatDate(performance.performance_date)}
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
    </div>
  );
}
