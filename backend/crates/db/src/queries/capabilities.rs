//! Query functions for the `user_capabilities` table.

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;

type Result<T> = std::result::Result<T, DbError>;

/// Returns the capability strings assigned to a user.
pub async fn list_for_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
) -> Result<Vec<String>> {
    sqlx::query_scalar::<_, String>(
        "SELECT capability FROM user_capabilities WHERE user_id = ? ORDER BY capability",
    )
    .bind(user_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Grants a capability to a user. No ops if the grant already exists.
pub async fn add_to_user(
    conn: &mut MySqlConnection,
    user_id: Uuid,
    capability: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_capabilities (user_id, capability) VALUES (?, ?) \
         ON DUPLICATE KEY UPDATE user_id = user_id",
    )
    .bind(user_id)
    .bind(capability)
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(DbError::from)
}

/// Revokes a capability from a user.
pub async fn remove_from_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
    capability: &str,
) -> Result<()> {
    sqlx::query("DELETE FROM user_capabilities WHERE user_id = ? AND capability = ?")
        .bind(user_id)
        .bind(capability)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}
