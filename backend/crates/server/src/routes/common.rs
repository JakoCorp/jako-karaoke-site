//! Shared types used across multiple route modules.

use db::sort::SortDirection;

use crate::error::ApiError;

/// Returns the search text of a list request, treating an empty string as no search.
pub(crate) fn search_text(q: Option<&str>) -> Option<&str> {
    q.filter(|text| !text.is_empty())
}

/// Rejects a stream number without a position number, or the reverse.
///
/// `number_field` is the API name of the position number, used in the error message.
pub(crate) fn validate_stream_position(
    stream_number: Option<u8>,
    number: Option<u16>,
    number_field: &str,
) -> Result<(), ApiError> {
    if stream_number.is_some() == number.is_some() {
        return Ok(());
    }
    Err(ApiError::BadRequest(format!(
        "stream_number and {number_field} must be provided together"
    )))
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
