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

/// Request body for `PUT /api/users/{id}/suspension`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SuspendUserRequest {
    /// Why the account is suspended, max 500 chars.
    pub reason: String,
}

/// A suspension of a user, returned by `GET /api/users/{id}/suspensions`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SuspensionResponse {
    /// Unique identifier.
    pub id: Uuid,
    /// Why the account was suspended.
    pub reason: String,
    /// When the suspension started.
    pub suspended_at: DateTime<Utc>,
    /// Admin who suspended the account. Absent when that account was deleted.
    pub suspended_by: Option<UserSummary>,
    /// When the suspension was lifted. Absent while it is in effect.
    pub lifted_at: Option<DateTime<Utc>>,
    /// Admin who lifted the suspension. Absent while it is in effect or when that account was deleted.
    pub lifted_by: Option<UserSummary>,
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
