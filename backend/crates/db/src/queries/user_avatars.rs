//! Query functions for the `user_avatars` join table.

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;

type Result<T> = std::result::Result<T, DbError>;

/// Returns the public storage URL of a user's avatar, or `None` if they have none.
pub async fn get_storage_url(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
) -> Result<Option<String>> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT a.storage_url FROM user_avatars ua \
         JOIN assets a ON a.id = ua.asset_id \
         WHERE ua.user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(executor)
    .await
    .map(Option::flatten)
    .map_err(DbError::from)
}

/// Sets a user's avatar to the given asset, replacing any existing link.
pub async fn set(conn: &mut MySqlConnection, user_id: Uuid, asset_id: Uuid) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_avatars (user_id, asset_id) VALUES (?, ?) \
         ON DUPLICATE KEY UPDATE asset_id = VALUES(asset_id)",
    )
    .bind(user_id)
    .bind(asset_id)
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(DbError::from)
}
