//! Authentication request and response types.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Request body for `POST /auth/claim`.
///
/// Completes a new OAuth signup by finalizing the username. Requires an active
/// `oauth_pending` cookie set during the OAuth callback.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClaimRequest {
    /// Alphanumeric with `_` and `.` allowed, max 64 chars, case insensitive unique.
    pub username: String,
}

/// Response body for `GET /auth/me`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MeResponse {
    pub id: Uuid,
    pub username: String,
    /// Capability titles embedded in the session JWT.
    pub capabilities: Vec<String>,
}
