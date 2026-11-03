//! Query parameters of the soundbite list endpoint.

use api_types::pagination::defaults as pagination_defaults;
use db::{queries::soundbites::SortKey, sort::SortDirection};

use crate::routes::common::SortDir;

/// Field to sort soundbites by in list endpoints.
#[derive(Debug, Clone, Copy, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SoundbiteSort {
    SoundbiteDate,
    PlayCount,
    Duration,
}

impl From<SoundbiteSort> for SortKey {
    fn from(sort: SoundbiteSort) -> Self {
        match sort {
            SoundbiteSort::SoundbiteDate => Self::SoundbiteDate,
            SoundbiteSort::PlayCount => Self::PlayCount,
            SoundbiteSort::Duration => Self::Duration,
        }
    }
}

/// Query parameters for `GET /api/soundbites`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct SoundbiteListParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "pagination_defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "pagination_defaults::per_page")]
    pub per_page: u32,
    /// Text search by title or artist name.
    pub q: Option<String>,
    /// Field to sort by. Defaults to `soundbite_date`.
    pub sort: Option<SoundbiteSort>,
    /// Sort direction. Defaults to `desc`.
    pub sort_dir: Option<SortDir>,
}

impl SoundbiteListParams {
    /// Returns the requested sort with the endpoint defaults applied.
    pub(crate) fn order(&self) -> (SortKey, SortDirection) {
        (
            self.sort.map_or(SortKey::SoundbiteDate, Into::into),
            self.sort_dir.map_or(SortDirection::Desc, Into::into),
        )
    }
}
