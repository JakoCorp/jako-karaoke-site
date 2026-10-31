//! Query functions for the `soundbite_plays` table.
//!
//! A soundbite's play count is the number of its counted rows, it is not stored separately.

use std::collections::HashMap;

use sqlx::{Executor, MySql, MySqlConnection, QueryBuilder};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::soundbite_play::{SoundbitePlay, SoundbitePlayProgress};

type Result<T> = std::result::Result<T, DbError>;

/// SQL expression for the play count of the row in an unaliased `soundbites` table.
///
/// Meant for `ORDER BY` in queries on `soundbites`.
pub(crate) const PLAY_COUNT_EXPR: &str = "(SELECT COUNT(*) FROM soundbite_plays \
     WHERE soundbite_plays.soundbite_id = soundbites.id \
     AND soundbite_plays.counted_at IS NOT NULL)";

/// Opens a pending play session and returns the created row.
pub async fn create(
    conn: &mut MySqlConnection,
    soundbite_id: Uuid,
    user_id: Uuid,
) -> Result<SoundbitePlay> {
    sqlx::query_as::<_, SoundbitePlay>(
        "INSERT INTO soundbite_plays (soundbite_id, user_id) VALUES (?, ?) \
         RETURNING id, soundbite_id, user_id, started_at, counted_at",
    )
    .bind(soundbite_id)
    .bind(user_id)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Deletes every pending session of a user.
pub async fn delete_pending_for_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
) -> Result<()> {
    sqlx::query("DELETE FROM soundbite_plays WHERE user_id = ? AND counted_at IS NULL")
        .bind(user_id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Locks and returns a user's session, or `None` if it does not exist or belongs to someone else.
pub async fn get_progress_for_update(
    conn: &mut MySqlConnection,
    id: Uuid,
    user_id: Uuid,
) -> Result<Option<SoundbitePlayProgress>> {
    sqlx::query_as::<_, SoundbitePlayProgress>(
        "SELECT soundbite_id, counted_at IS NOT NULL AS counted, \
         TIMESTAMPDIFF(SECOND, started_at, CURRENT_TIMESTAMP) AS elapsed_seconds \
         FROM soundbite_plays WHERE id = ? AND user_id = ? FOR UPDATE",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(conn)
    .await
    .map_err(DbError::from)
}

/// Returns the seconds since the user's latest counted play of a soundbite, if any.
pub async fn seconds_since_last_counted(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
    soundbite_id: Uuid,
) -> Result<Option<i64>> {
    sqlx::query_scalar::<_, Option<i64>>(
        "SELECT TIMESTAMPDIFF(SECOND, MAX(counted_at), CURRENT_TIMESTAMP) \
         FROM soundbite_plays \
         WHERE user_id = ? AND soundbite_id = ? AND counted_at IS NOT NULL",
    )
    .bind(user_id)
    .bind(soundbite_id)
    .fetch_one(executor)
    .await
    .map_err(DbError::from)
}

/// Marks a session as counted.
pub async fn mark_counted(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<()> {
    sqlx::query("UPDATE soundbite_plays SET counted_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Deletes a session. Missing IDs are a no-op.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM soundbite_plays WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Returns the play count of one soundbite.
pub async fn count_for_soundbite(
    executor: impl Executor<'_, Database = MySql>,
    soundbite_id: Uuid,
) -> Result<u64> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM soundbite_plays \
         WHERE soundbite_id = ? AND counted_at IS NOT NULL",
    )
    .bind(soundbite_id)
    .fetch_one(executor)
    .await
    .map(|n| n as u64)
    .map_err(DbError::from)
}

/// Returns play counts for multiple soundbites, keyed by soundbite ID.
///
/// Soundbites with no counted plays are absent from the returned map.
pub async fn counts_batch(
    executor: impl Executor<'_, Database = MySql>,
    soundbite_ids: &[Uuid],
) -> Result<HashMap<Uuid, u64>> {
    if soundbite_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::new(
        "SELECT soundbite_id, COUNT(*) FROM soundbite_plays \
         WHERE counted_at IS NOT NULL AND soundbite_id IN (",
    );
    let mut separated = builder.separated(", ");
    for soundbite_id in soundbite_ids {
        separated.push_bind(soundbite_id);
    }
    builder.push(") GROUP BY soundbite_id");

    let rows: Vec<(Uuid, i64)> = builder
        .build_query_as()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)?;
    Ok(rows
        .into_iter()
        .map(|(id, count)| (id, count as u64))
        .collect())
}
