import type { OfflineEntry } from "@/store/offline";

import { SavedRow } from "./saved-row";

interface Props {
  entries: OfflineEntry[];
}

export function SavedTable({ entries }: Props) {
  return (
    <div>
      <div className="local-perf-row-header">
        <div />
        <div className="perf-header-label">Title</div>
        <div className="perf-header-label--right perf-header-label">Duration</div>
        <div className="perf-header-label--right perf-header-label">Size</div>
        <div className="perf-header-label--right perf-header-label">Saved</div>
        <div />
      </div>
      {entries.map((entry, index) => (
        <SavedRow key={entry.performance.id} entry={entry} queue={entries} index={index} />
      ))}
    </div>
  );
}
