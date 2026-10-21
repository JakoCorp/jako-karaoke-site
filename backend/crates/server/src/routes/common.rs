//! Shared types used across multiple route modules.

use db::sort::SortDirection;

/// Returns the search text of a list request, treating an empty string as no search.
pub(crate) fn search_text(q: Option<&str>) -> Option<&str> {
    q.filter(|text| !text.is_empty())
}

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
