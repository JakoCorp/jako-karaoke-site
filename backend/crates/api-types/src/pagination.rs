//! Pagination envelope and query parameter types shared across all paginated endpoints.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Generic envelope returned by all paginated list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PagedResponse<T: ToSchema> {
    /// Items on the current page.
    pub items: Vec<T>,
    /// Total number of matching items across all pages.
    pub total: u64,
    /// The page that was returned, 1-indexed.
    pub page: u32,
    /// Items per page as applied by the server (may be less than requested if capped).
    pub per_page: u32,
}

/// Default values for serde field attributes on pagination params.
pub mod defaults {
    /// Returns the default page number (`1`).
    pub fn page() -> u32 {
        1
    }

    /// Returns the default items per page (`20`).
    pub fn per_page() -> u32 {
        20
    }
}
