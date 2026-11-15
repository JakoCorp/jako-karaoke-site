//! User suspension model.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A suspension of a user with the admins who started and lifted it.
///
/// An admin is `None` when their account has since been deleted. The suspension is still in
/// effect while `lifted_at` is `None`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserSuspensionRow {
    pub id: Uuid,
    pub reason: String,
    pub suspended_at: DateTime<Utc>,
    pub suspended_by: Option<Uuid>,
    pub suspended_by_username: Option<String>,
    pub lifted_at: Option<DateTime<Utc>>,
    pub lifted_by: Option<Uuid>,
    pub lifted_by_username: Option<String>,
}
