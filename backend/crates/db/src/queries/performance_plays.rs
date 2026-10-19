//! Query functions for the `performance_plays` table.
//!
//! A performance's play count is the number of its counted rows, it is not stored separately.

use std::collections::HashMap;

use sqlx::{Executor, MySql, MySqlConnection, QueryBuilder};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::performance_play::{
    PerformancePlay, PerformancePlayerRow, PlayProgress, UserPlayRow,
};

type Result<T> = std::result::Result<T, DbError>;

/// SQL expression for the play count of the row in an unaliased `performances` table.
///
/// Meant for `ORDER BY` in queries on `performances`.
pub const PLAY_COUNT_EXPR: &str = "(SELECT COUNT(*) FROM performance_plays \
     WHERE performance_plays.performance_id = performances.id \
     AND performance_plays.counted_at IS NOT NULL)";

/// Opens a pending play session and returns the created row.
pub async fn create(
    conn: &mut MySqlConnection,
    performance_id: Uuid,
    user_id: Uuid,
) -> Result<PerformancePlay> {
    sqlx::query_as::<_, PerformancePlay>(
        "INSERT INTO performance_plays (performance_id, user_id) VALUES (?, ?) \
         RETURNING id, performance_id, user_id, started_at, counted_at",
    )
    .bind(performance_id)
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
    sqlx::query("DELETE FROM performance_plays WHERE user_id = ? AND counted_at IS NULL")
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
) -> Result<Option<PlayProgress>> {
    sqlx::query_as::<_, PlayProgress>(
        "SELECT performance_id, counted_at IS NOT NULL AS counted, \
         TIMESTAMPDIFF(SECOND, started_at, CURRENT_TIMESTAMP) AS elapsed_seconds \
         FROM performance_plays WHERE id = ? AND user_id = ? FOR UPDATE",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(conn)
    .await
    .map_err(DbError::from)
}

/// Returns the seconds since the user's latest counted play of a performance, if any.
pub async fn seconds_since_last_counted(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
    performance_id: Uuid,
) -> Result<Option<i64>> {
    sqlx::query_scalar::<_, Option<i64>>(
        "SELECT TIMESTAMPDIFF(SECOND, MAX(counted_at), CURRENT_TIMESTAMP) \
         FROM performance_plays \
         WHERE user_id = ? AND performance_id = ? AND counted_at IS NOT NULL",
    )
    .bind(user_id)
    .bind(performance_id)
    .fetch_one(executor)
    .await
    .map_err(DbError::from)
}

/// Marks a session as counted.
pub async fn mark_counted(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<()> {
    sqlx::query("UPDATE performance_plays SET counted_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Deletes a session. Missing IDs are a no-op.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM performance_plays WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Returns the play count of one performance.
pub async fn count_for_performance(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
) -> Result<u64> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM performance_plays \
         WHERE performance_id = ? AND counted_at IS NOT NULL",
    )
    .bind(performance_id)
    .fetch_one(executor)
    .await
    .map(|n| n as u64)
    .map_err(DbError::from)
}

/// Returns play counts for multiple performances, keyed by performance ID.
///
/// Performances with no counted plays are absent from the returned map.
pub async fn counts_batch(
    executor: impl Executor<'_, Database = MySql>,
    performance_ids: &[Uuid],
) -> Result<HashMap<Uuid, u64>> {
    if performance_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::new(
        "SELECT performance_id, COUNT(*) FROM performance_plays \
         WHERE counted_at IS NOT NULL AND performance_id IN (",
    );
    let mut separated = builder.separated(", ");
    for performance_id in performance_ids {
        separated.push_bind(performance_id);
    }
    builder.push(") GROUP BY performance_id");

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

/// Returns a page of the users who have counted plays on a performance, most plays first.
pub async fn list_players_for_performance(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
    limit: u32,
    offset: u32,
) -> Result<Vec<PerformancePlayerRow>> {
    sqlx::query_as::<_, PerformancePlayerRow>(
        "SELECT u.id AS user_id, u.username, CAST(COUNT(*) AS UNSIGNED) AS play_count, \
         MAX(pp.counted_at) AS last_played_at \
         FROM performance_plays pp \
         JOIN users u ON u.id = pp.user_id \
         WHERE pp.performance_id = ? AND pp.counted_at IS NOT NULL \
         GROUP BY u.id, u.username \
         ORDER BY play_count DESC, last_played_at DESC, u.id DESC \
         LIMIT ? OFFSET ?",
    )
    .bind(performance_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Returns the number of users who have counted plays on a performance.
pub async fn count_players_for_performance(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
) -> Result<u64> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT user_id) FROM performance_plays \
         WHERE performance_id = ? AND counted_at IS NOT NULL",
    )
    .bind(performance_id)
    .fetch_one(executor)
    .await
    .map(|n| n as u64)
    .map_err(DbError::from)
}

/// Returns a page of the performances a user has counted plays on, most recently played first.
pub async fn list_for_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
    limit: u32,
    offset: u32,
) -> Result<Vec<UserPlayRow>> {
    sqlx::query_as::<_, UserPlayRow>(
        "SELECT p.id, p.created_by, p.title, p.lyrics_id, p.stream_time, \
         p.performance_date, p.stream_number, p.performance_number, \
         CAST(COUNT(*) AS UNSIGNED) AS user_play_count, MAX(pp.counted_at) AS last_played_at \
         FROM performance_plays pp \
         JOIN performances p ON p.id = pp.performance_id \
         WHERE pp.user_id = ? AND pp.counted_at IS NOT NULL \
         GROUP BY p.id \
         ORDER BY last_played_at DESC, p.id DESC \
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Returns the number of performances a user has counted plays on.
pub async fn count_for_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
) -> Result<u64> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT performance_id) FROM performance_plays \
         WHERE user_id = ? AND counted_at IS NOT NULL",
    )
    .bind(user_id)
    .fetch_one(executor)
    .await
    .map(|n| n as u64)
    .map_err(DbError::from)
}

/// Deletes a user's counted plays, optionally limited to one performance.
///
/// Returns the number of plays removed.
pub async fn delete_counted_for_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
    performance_id: Option<Uuid>,
) -> Result<u64> {
    sqlx::query(
        "DELETE FROM performance_plays \
         WHERE user_id = ? AND counted_at IS NOT NULL AND (? IS NULL OR performance_id = ?)",
    )
    .bind(user_id)
    .bind(performance_id)
    .bind(performance_id)
    .execute(executor)
    .await
    .map(|r| r.rows_affected())
    .map_err(DbError::from)
}
