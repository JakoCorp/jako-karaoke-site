//! Shared types used across multiple route modules.

use db::sort::SortDirection;

/// Sort direction for list endpoints.
#[derive(Debug, Clone, Copy, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SortDir {
    Asc,
    Desc,
}

impl From<SortDir> for SortDirection {
    fn from(dir: SortDir) -> Self {
        match dir {
            SortDir::Asc => Self::Asc,
            SortDir::Desc => Self::Desc,
        }
    }
}
