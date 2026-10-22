//! Query parameters of the song list endpoint.

use api_types::pagination::defaults as pagination_defaults;
use db::{queries::songs::SortKey, sort::SortDirection};

use crate::routes::common::SortDir;

/// Field to sort songs by in list endpoints.
#[derive(Debug, Clone, Copy, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SongSort {
    PerformanceCount,
}

impl From<SongSort> for SortKey {
    fn from(sort: SongSort) -> Self {
        match sort {
            SongSort::PerformanceCount => Self::PerformanceCount,
        }
    }
}

/// Query parameters for `GET /api/songs`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct SongListParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "pagination_defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "pagination_defaults::per_page")]
    pub per_page: u32,
    /// Text search by title or original artist name.
    pub q: Option<String>,
    /// Filter expressions like `original_artist:<id>,<id>` or `-genre:<id>`, repeated for each
    /// condition. All expressions must hold, values within one are alternatives.
    #[serde(default)]
    pub filter: Vec<String>,
    /// Field to sort by. Defaults to `performance_count`.
    pub sort: Option<SongSort>,
    /// Sort direction. Defaults to `desc`.
    pub sort_dir: Option<SortDir>,
}

impl SongListParams {
    /// Returns the requested sort with the endpoint defaults applied.
    pub(crate) fn order(&self) -> (SortKey, SortDirection) {
        (
            self.sort.map_or(SortKey::PerformanceCount, Into::into),
            self.sort_dir.map_or(SortDirection::Desc, Into::into),
        )
    }
}
