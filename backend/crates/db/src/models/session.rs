//! Session model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A server side session row, keyed by the SHA256 hash of the session token.
///
/// A request is authenticated by rehashing its cookie value and
/// looking up the matching row.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Session {
    /// SHA-256 hex digest of the raw session token.
    pub id: String,
    /// User this session belongs to.
    pub user_id: Uuid,
    /// When the session was issued.
    pub created_at: DateTime<Utc>,
    /// When the session expires. Requests after this time are rejected.
    pub expires_at: DateTime<Utc>,
}
