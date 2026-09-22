//! Lyrics subresource request and response types.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response body for lyrics GET endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LyricsResponse {
    /// The full lyrics text.
    pub content: String,
}

/// Request body for lyrics PUT endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateLyricsRequest {
    /// The full lyrics text to store, replacing any existing content.
    pub content: String,
}
