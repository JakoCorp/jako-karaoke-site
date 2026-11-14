//! User request and response types for admin endpoints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// A minimal user projection embedded in other responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserSummary {
    /// Unique identifier.
    pub id: Uuid,
    /// Display name chosen during registration.
    pub username: String,
}

/// A user with their account status, returned by the user management endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminUserSummary {
    /// Unique identifier.
    pub id: Uuid,
    /// Display name chosen during registration.
    pub username: String,
    /// When the account was suspended. Absent when active.
    pub suspended_at: Option<DateTime<Utc>>,
}

/// Request body for `PATCH /api/users/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    /// New username. Alphanumeric with `_` and `.` allowed, max 64 chars, case insensitive unique.
    pub username: String,
}

/// Request body for `POST /api/admin/users/{id}/capabilities`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GrantCapabilityRequest {
    /// Capability string to grant. Must be one of the values in `ALL_CAPABILITIES`.
    pub capability: String,
}
