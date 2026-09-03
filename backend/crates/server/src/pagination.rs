//! Server side pagination policy and SQL parameter derivation.

/// Maximum items a client may request in a single page.
pub(crate) const MAX_PER_PAGE: u32 = 200;

/// Returns the `(limit, offset)` pair for a SQL query, clamping `per_page` to [`MAX_PER_PAGE`].
pub(crate) fn limit_offset(page: u32, per_page: u32) -> (u32, u32) {
    let per_page = per_page.min(MAX_PER_PAGE);
    let offset = page.saturating_sub(1).saturating_mul(per_page);
    (per_page, offset)
}
