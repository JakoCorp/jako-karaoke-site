//! Query functions for the `user_suspensions` table.

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::UserSuspensionRow;

type Result<T> = std::result::Result<T, DbError>;

/// Records the current suspension of a user, taking its time from `users.suspended_at`.
///
/// Does nothing if the user is not suspended.
pub async fn create(
    conn: &mut MySqlConnection,
    user_id: Uuid,
    suspended_by: Uuid,
    reason: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_suspensions (user_id, suspended_by, reason, suspended_at) \
         SELECT id, ?, ?, suspended_at FROM users WHERE id = ? AND suspended_at IS NOT NULL",
    )
    .bind(suspended_by)
    .bind(reason)
    .bind(user_id)
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(DbError::from)
}

/// Marks the open suspension of a user as lifted. No ops if there is none.
pub async fn lift(conn: &mut MySqlConnection, user_id: Uuid, lifted_by: Uuid) -> Result<()> {
    sqlx::query(
        "UPDATE user_suspensions SET lifted_by = ?, lifted_at = UTC_TIMESTAMP() \
         WHERE user_id = ? AND lifted_at IS NULL",
    )
    .bind(lifted_by)
    .bind(user_id)
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(DbError::from)
}

/// Returns the suspensions of a user, newest first.
pub async fn list_for_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
) -> Result<Vec<UserSuspensionRow>> {
    sqlx::query_as::<_, UserSuspensionRow>(
        "SELECT s.id, s.reason, s.suspended_at, s.suspended_by, \
         suspender.username AS suspended_by_username, \
         s.lifted_at, s.lifted_by, lifter.username AS lifted_by_username \
         FROM user_suspensions s \
         LEFT JOIN users suspender ON suspender.id = s.suspended_by \
         LEFT JOIN users lifter ON lifter.id = s.lifted_by \
         WHERE s.user_id = ? \
         ORDER BY s.suspended_at DESC, s.id DESC",
    )
    .bind(user_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}
