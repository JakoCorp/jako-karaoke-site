//! Query parameters of the artist list endpoint.

use api_types::pagination::defaults as pagination_defaults;
use db::{queries::artists::SortKey, sort::SortDirection};

use crate::routes::common::SortDir;

/// Field to sort artists by in list endpoints.
#[derive(Debug, Clone, Copy, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ArtistSort {
    Name,
    SongCount,
}

impl From<ArtistSort> for SortKey {
    fn from(sort: ArtistSort) -> Self {
        match sort {
            ArtistSort::Name => Self::Name,
            ArtistSort::SongCount => Self::SongCount,
        }
    }
}

/// Query parameters for `GET /api/artists`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct ArtistListParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "pagination_defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "pagination_defaults::per_page")]
    pub per_page: u32,
    /// Text search by artist name.
    pub q: Option<String>,
    /// Field to sort by. Defaults to `name`.
    pub sort: Option<ArtistSort>,
    /// Sort direction. Defaults to `asc`.
    pub sort_dir: Option<SortDir>,
}

impl ArtistListParams {
    /// Returns the requested sort with the endpoint defaults applied.
    pub(crate) fn order(&self) -> (SortKey, SortDirection) {
        (
            self.sort.map_or(SortKey::Name, Into::into),
            self.sort_dir.map_or(SortDirection::Asc, Into::into),
        )
    }
}
