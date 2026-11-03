//! Play moderation handlers for users, one module per kind of played item.

pub(crate) mod performances;
pub(crate) mod soundbites;

use api_types::pagination::defaults as pagination_defaults;

/// Query parameters for the endpoints listing the items a user has played.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct UserPlayListParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "pagination_defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "pagination_defaults::per_page")]
    pub per_page: u32,
}
