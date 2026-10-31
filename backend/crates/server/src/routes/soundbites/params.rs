//! Query parameters of the soundbite list endpoint.

use api_types::pagination::defaults as pagination_defaults;

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
}
