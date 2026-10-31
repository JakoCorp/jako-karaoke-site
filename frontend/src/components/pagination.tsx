interface Props {
  page: number;
  totalPages: number;
  onPageChange: (page: number) => void;
}

/** Page buttons with previous and next arrows. Renders nothing when there is only one page. */
export function Pagination({ page, totalPages, onPageChange }: Props) {
  if (totalPages <= 1) return null;

  return (
    <div className="search-pagination">
      <button
        className="page-btn"
        disabled={page <= 1}
        onClick={() => {
          onPageChange(page - 1);
        }}
        aria-label="Previous page"
      >
        ‹
      </button>
      {getPageWindow(page, totalPages).map((pageNum) => (
        <button
          key={pageNum}
          className={pageNum === page ? "page-btn page-btn--active" : "page-btn"}
          onClick={() => {
            onPageChange(pageNum);
          }}
          aria-current={pageNum === page ? "page" : undefined}
        >
          {pageNum}
        </button>
      ))}
      <button
        className="page-btn"
        disabled={page >= totalPages}
        onClick={() => {
          onPageChange(page + 1);
        }}
        aria-label="Next page"
      >
        ›
      </button>
    </div>
  );
}

/** Returns up to 7 page numbers centered around the current page. */
function getPageWindow(current: number, total: number): number[] {
  const half = 3;
  let start = Math.max(1, current - half);
  const end = Math.min(total, start + 6);
  start = Math.max(1, end - 6);
  return Array.from({ length: end - start + 1 }, (_, i) => start + i);
}
