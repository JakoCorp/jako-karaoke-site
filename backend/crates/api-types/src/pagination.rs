//! Pagination envelope and query parameter types shared across all paginated endpoints.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

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

/// Query parameters accepted by paginated list endpoints.
#[derive(Debug, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "defaults::per_page")]
    pub per_page: u32,
}

/// Query parameters for list endpoints that support text search.
#[derive(Debug, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchPaginationParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "defaults::per_page")]
    pub per_page: u32,
    /// Optional text search filter.
    pub q: Option<String>,
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
