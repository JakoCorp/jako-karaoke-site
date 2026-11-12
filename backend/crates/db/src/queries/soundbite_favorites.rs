//! Query functions for the `soundbite_favorites` table.

use sqlx::{Executor, MySql};
use uuid::Uuid;

use crate::error::DbError;

type Result<T> = std::result::Result<T, DbError>;

/// Marks a soundbite as a favorite of a user. Does nothing if it already is.
pub async fn add(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
    soundbite_id: Uuid,
) -> Result<()> {
    sqlx::query("INSERT IGNORE INTO soundbite_favorites (user_id, soundbite_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(soundbite_id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Removes a soundbite from the favorites of a user. Does nothing if it is not one.
pub async fn remove(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
    soundbite_id: Uuid,
) -> Result<()> {
    sqlx::query("DELETE FROM soundbite_favorites WHERE user_id = ? AND soundbite_id = ?")
        .bind(user_id)
        .bind(soundbite_id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Returns the IDs of every soundbite a user has marked as a favorite.
pub async fn list_ids(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
) -> Result<Vec<Uuid>> {
    sqlx::query_scalar::<_, Uuid>("SELECT soundbite_id FROM soundbite_favorites WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(executor)
        .await
        .map_err(DbError::from)
}
