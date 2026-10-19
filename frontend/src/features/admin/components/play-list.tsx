import type { ReactNode } from "react";

export interface PlayListItem {
  id: string;
  title: string;
  meta: string;
  onRemove: () => void;
}

/** Rows of plays with a remove button each, shared by the admin plays dialogs. */
export function PlayList({ items }: { items: readonly PlayListItem[] }) {
  return (
    <ul className="admin-play-list">
      {items.map((item) => (
        <li key={item.id} className="admin-play-row">
          <div className="admin-play-info">
            <span className="admin-play-title">{item.title}</span>
            <span className="admin-play-meta">{item.meta}</span>
          </div>
          <button type="button" className="btn btn-secondary" onClick={item.onRemove}>
            Remove
          </button>
        </li>
      ))}
    </ul>
  );
}

interface PlayPagerProps {
  page: number;
  lastPage: number;
  onPageChange: (page: number) => void;
  children?: ReactNode;
}

/** Previous and next buttons for the admin plays dialogs, with room for extra actions. */
export function PlayPager({ page, lastPage, onPageChange, children }: PlayPagerProps) {
  return (
    <div className="dialog-actions">
      <button
        type="button"
        className="btn btn-secondary"
        disabled={page <= 1}
        onClick={() => {
          onPageChange(page - 1);
        }}
      >
        Previous
      </button>
      <button
        type="button"
        className="btn btn-secondary"
        disabled={page >= lastPage}
        onClick={() => {
          onPageChange(page + 1);
        }}
      >
        Next
      </button>
      {children}
    </div>
  );
}
