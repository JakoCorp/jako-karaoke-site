//! Query parameters of the performance list endpoint.

use api_types::pagination::defaults as pagination_defaults;
use db::{queries::performances::SortKey, sort::SortDirection};

use crate::routes::common::SortDir;

/// Field to sort performances by in list endpoints.
#[derive(Debug, Clone, Copy, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PerformanceSort {
    PerformanceDate,
    PlayCount,
    Duration,
}

impl From<PerformanceSort> for SortKey {
    fn from(sort: PerformanceSort) -> Self {
        match sort {
            PerformanceSort::PerformanceDate => Self::PerformanceDate,
            PerformanceSort::PlayCount => Self::PlayCount,
            PerformanceSort::Duration => Self::Duration,
        }
    }
}

/// Query parameters for `GET /api/performances`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct PerformanceListParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "pagination_defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "pagination_defaults::per_page")]
    pub per_page: u32,
    /// Text search across performance title, song title, and singer names.
    pub q: Option<String>,
    /// Filter expressions like `singer:<id>,<id>` or `-genre:<id>`, repeated for each condition.
    /// All expressions must hold, values within one are alternatives.
    #[serde(default)]
    pub filter: Vec<String>,
    /// Field to sort by. Defaults to `performance_date`.
    pub sort: Option<PerformanceSort>,
    /// Sort direction. Defaults to `desc`.
    pub sort_dir: Option<SortDir>,
}

impl PerformanceListParams {
    /// Returns the requested sort with the endpoint defaults applied.
    pub(crate) fn order(&self) -> (SortKey, SortDirection) {
        (
            self.sort.map_or(SortKey::PerformanceDate, Into::into),
            self.sort_dir.map_or(SortDirection::Desc, Into::into),
        )
    }
}
