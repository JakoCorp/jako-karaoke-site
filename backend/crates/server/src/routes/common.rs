//! Shared types used across multiple route modules.

use db::sort::SortDirection;

use crate::error::ApiError;

/// Returns the search text of a list request, treating an empty string as no search.
pub(crate) fn search_text(q: Option<&str>) -> Option<&str> {
    q.filter(|text| !text.is_empty())
}

/// Longest title accepted for titled resources, matching their `VARCHAR(256)` columns.
const MAX_TITLE_CHARS: usize = 256;

/// Returns the title as it is stored, without surrounding whitespace.
pub(crate) fn normalize_title(title: &str) -> String {
    title.trim().to_string()
}

/// Rejects a blank or overlong title.
///
/// Callers pass the [`normalize_title`] result, since that is the value stored.
pub(crate) fn validate_title(title: &str) -> Result<(), ApiError> {
    if title.trim().is_empty() {
        return Err(ApiError::BadRequest("title must not be blank".to_string()));
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err(ApiError::BadRequest(format!(
            "title must be at most {MAX_TITLE_CHARS} characters"
        )));
    }
    Ok(())
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
