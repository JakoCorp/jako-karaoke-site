//! Server side pagination policy and SQL parameter derivation.

/// Maximum items a client may request in a single page.
pub(crate) const MAX_PER_PAGE: u32 = 200;
