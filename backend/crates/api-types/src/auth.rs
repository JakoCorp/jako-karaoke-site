//! Authentication request and response types.

use chrono::{DateTime, Utc};
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
    /// Unique identifier of the authenticated user.
    pub id: Uuid,
    /// Display name chosen during registration.
    pub username: String,
    /// URL of the user's avatar. Absent when the user has no avatar.
    pub avatar_url: Option<String>,
    /// Earliest time the username can be changed again. Absent when it can be changed now.
    pub username_changeable_at: Option<DateTime<Utc>>,
    /// Capability titles embedded in the session JWT.
    pub capabilities: Vec<String>,
}

/// Request body for `PATCH /auth/me`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateMeRequest {
    /// New username. Alphanumeric with `_` and `.` allowed, max 64 chars, case insensitive unique.
    pub username: String,
}
