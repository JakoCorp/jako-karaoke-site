//! Query parameters of the playlist list endpoints.

use api_types::{pagination::defaults as pagination_defaults, playlists::PlaylistKind};
use db::{queries::playlists::SortKey, sort::SortDirection};

use crate::routes::common::SortDir;

/// Field to sort playlists by in list endpoints.
#[derive(Debug, Clone, Copy, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PlaylistSort {
    Created,
    Title,
    PerformanceCount,
}

impl From<PlaylistSort> for SortKey {
    fn from(sort: PlaylistSort) -> Self {
        match sort {
            PlaylistSort::Created => Self::Created,
            PlaylistSort::Title => Self::Title,
            PlaylistSort::PerformanceCount => Self::PerformanceCount,
        }
    }
}

/// Query parameters for `GET /api/playlists` and `GET /api/users/{id}/playlists`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct PlaylistListParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "pagination_defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "pagination_defaults::per_page")]
    pub per_page: u32,
    /// Optional text search filter against playlist title.
    pub q: Option<String>,
    /// Optional filter returning only playlists of this kind.
    pub kind: Option<PlaylistKind>,
    /// Field to sort by. Defaults to `created`.
    pub sort: Option<PlaylistSort>,
    /// Sort direction. Defaults to `desc`.
    pub sort_dir: Option<SortDir>,
}

impl PlaylistListParams {
    /// Returns the requested sort with the endpoint defaults applied.
    pub(crate) fn order(&self) -> (SortKey, SortDirection) {
        (
            self.sort.map_or(SortKey::Created, Into::into),
            self.sort_dir.map_or(SortDirection::Desc, Into::into),
        )
    }
}
