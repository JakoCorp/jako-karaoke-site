//! Query functions for the `sessions` table.

use chrono::{DateTime, Utc};
use sqlx::{Executor, MySql};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::session::Session;

type Result<T> = std::result::Result<T, DbError>;

/// Inserts a new session row. `id` is the SHA256 hash of the session token.
pub async fn create(
    executor: impl Executor<'_, Database = MySql>,
    id: &str,
    user_id: Uuid,
    expires_at: DateTime<Utc>,
) -> Result<()> {
    sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(id)
        .bind(user_id)
        .bind(expires_at)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Fetches a session by ID, returning `None` if it is missing, expired, or belongs to a
/// suspended user.
pub async fn get_valid(
    executor: impl Executor<'_, Database = MySql>,
    id: &str,
) -> Result<Option<Session>> {
    sqlx::query_as::<_, Session>(
        "SELECT s.id, s.user_id, s.created_at, s.expires_at FROM sessions s \
         JOIN users u ON u.id = s.user_id \
         WHERE s.id = ? AND s.expires_at > UTC_TIMESTAMP() AND u.suspended_at IS NULL",
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Deletes a session by ID. Missing IDs are a no-op.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Deletes every session of a user, returning the number of rows removed.
pub async fn delete_for_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
) -> Result<u64> {
    sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(user_id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected())
        .map_err(DbError::from)
}

/// Deletes all expired sessions, returning the number of rows removed.
pub async fn delete_expired(executor: impl Executor<'_, Database = MySql>) -> Result<u64> {
    sqlx::query("DELETE FROM sessions WHERE expires_at <= UTC_TIMESTAMP()")
        .execute(executor)
        .await
        .map(|r| r.rows_affected())
        .map_err(DbError::from)
}
