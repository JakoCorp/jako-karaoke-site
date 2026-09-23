//! User request and response types for admin endpoints.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// A minimal user projection returned by user search endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserSummary {
    /// Unique identifier.
    pub id: Uuid,
    /// Display name chosen during registration.
    pub username: String,
}

/// Request body for `POST /api/admin/users/{id}/capabilities`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GrantCapabilityRequest {
    /// Capability string to grant. Must be one of the values in `ALL_CAPABILITIES`.
    pub capability: String,
}
