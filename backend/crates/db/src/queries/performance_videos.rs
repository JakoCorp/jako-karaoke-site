//! Query functions for the `performance_videos` join table.

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::performance_video::{NewPerformanceVideo, PerformanceVideoRow};

type Result<T> = std::result::Result<T, DbError>;

/// Returns all video rows for a given performance, joined with asset fields.
pub async fn list_for_performance(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
) -> Result<Vec<PerformanceVideoRow>> {
    sqlx::query_as::<_, PerformanceVideoRow>(
        "SELECT pv.performance_id, pv.asset_id, pv.kind, \
         a.title, a.credits, a.source_url, a.storage_url, a.internal_path, a.external_url \
         FROM performance_videos pv \
         JOIN assets a ON a.id = pv.asset_id \
         WHERE pv.performance_id = ?",
    )
    .bind(performance_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Inserts a join row linking an asset to a performance as video and returns the joined row.
pub async fn link(
    conn: &mut MySqlConnection,
    performance_id: Uuid,
    new: &NewPerformanceVideo,
) -> Result<PerformanceVideoRow> {
    sqlx::query("INSERT INTO performance_videos (performance_id, asset_id, kind) VALUES (?, ?, ?)")
        .bind(performance_id)
        .bind(new.asset_id)
        .bind(&new.kind)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;

    sqlx::query_as::<_, PerformanceVideoRow>(
        "SELECT pv.performance_id, pv.asset_id, pv.kind, \
         a.title, a.credits, a.source_url, a.storage_url, a.internal_path, a.external_url \
         FROM performance_videos pv \
         JOIN assets a ON a.id = pv.asset_id \
         WHERE pv.performance_id = ? AND pv.asset_id = ?",
    )
    .bind(performance_id)
    .bind(new.asset_id)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Updates the kind of a video join row. Returns `true` if a row was updated.
pub async fn update_kind(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
    asset_id: Uuid,
    kind: &str,
) -> Result<bool> {
    sqlx::query(
        "UPDATE performance_videos SET kind = ? \
         WHERE performance_id = ? AND asset_id = ?",
    )
    .bind(kind)
    .bind(performance_id)
    .bind(asset_id)
    .execute(executor)
    .await
    .map(|r| r.rows_affected() > 0)
    .map_err(DbError::from)
}

/// Removes the join row linking an asset to a performance as video.
///
/// Returns `true` if a row was deleted.
pub async fn unlink(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
    asset_id: Uuid,
) -> Result<bool> {
    sqlx::query("DELETE FROM performance_videos WHERE performance_id = ? AND asset_id = ?")
        .bind(performance_id)
        .bind(asset_id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
