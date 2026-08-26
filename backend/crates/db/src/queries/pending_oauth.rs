//! Query functions for the `pending_oauth` table.

use chrono::{DateTime, Utc};
use sqlx::{Executor, MySql};

use crate::error::DbError;
use crate::models::pending_oauth::PendingOAuth;

type Result<T> = std::result::Result<T, DbError>;

/// Inserts a pending OAuth claim record.
///
/// `token` is a caller-generated 64-char hex string. `expires_at` should be
/// set to a short horizon (15 minutes is typical) so stale records do not
/// accumulate indefinitely.
pub async fn create(
    executor: impl Executor<'_, Database = MySql>,
    token: &str,
    provider: &str,
    provider_id: u64,
    suggested_username: &str,
    expires_at: DateTime<Utc>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO pending_oauth (token, provider, provider_id, suggested_username, expires_at) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(token)
    .bind(provider)
    .bind(provider_id)
    .bind(suggested_username)
    .bind(expires_at)
    .execute(executor)
    .await
    .map(|_| ())
    .map_err(DbError::from)
}

/// Fetches a pending claim by token, returning `None` if missing or expired.
pub async fn get(
    executor: impl Executor<'_, Database = MySql>,
    token: &str,
) -> Result<Option<PendingOAuth>> {
    sqlx::query_as::<_, PendingOAuth>(
        "SELECT token, provider, provider_id, suggested_username \
         FROM pending_oauth WHERE token = ? AND expires_at > UTC_TIMESTAMP()",
    )
    .bind(token)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Deletes a pending claim by token.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, token: &str) -> Result<()> {
    sqlx::query("DELETE FROM pending_oauth WHERE token = ?")
        .bind(token)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}
