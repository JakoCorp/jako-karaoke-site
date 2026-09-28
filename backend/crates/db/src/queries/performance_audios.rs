//! Query functions for the `performance_audios` join table.

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::performance_audio::{NewPerformanceAudio, PerformanceAudioRow};

type Result<T> = std::result::Result<T, DbError>;

/// Returns all audio rows for a given performance, joined with asset fields.
pub async fn list_for_performance(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
) -> Result<Vec<PerformanceAudioRow>> {
    sqlx::query_as::<_, PerformanceAudioRow>(
        "SELECT pa.performance_id, pa.asset_id, pa.kind, \
         a.title, a.credits, a.source_url, a.storage_url, a.internal_path, a.external_url \
         FROM performance_audios pa \
         JOIN assets a ON a.id = pa.asset_id \
         WHERE pa.performance_id = ?",
    )
    .bind(performance_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Inserts a join row linking an asset to a performance as audio and returns the joined row.
pub async fn link(
    conn: &mut MySqlConnection,
    performance_id: Uuid,
    new: &NewPerformanceAudio,
) -> Result<PerformanceAudioRow> {
    sqlx::query("INSERT INTO performance_audios (performance_id, asset_id, kind) VALUES (?, ?, ?)")
        .bind(performance_id)
        .bind(new.asset_id)
        .bind(&new.kind)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;

    sqlx::query_as::<_, PerformanceAudioRow>(
        "SELECT pa.performance_id, pa.asset_id, pa.kind, \
         a.title, a.credits, a.source_url, a.storage_url, a.internal_path, a.external_url \
         FROM performance_audios pa \
         JOIN assets a ON a.id = pa.asset_id \
         WHERE pa.performance_id = ? AND pa.asset_id = ?",
    )
    .bind(performance_id)
    .bind(new.asset_id)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Demotes any existing `"primary"` audio for the given performance to `"misc"`.
pub async fn unset_primary(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
) -> Result<()> {
    sqlx::query(
        "UPDATE performance_audios SET kind = 'misc' \
         WHERE performance_id = ? AND kind = 'primary'",
    )
    .bind(performance_id)
    .execute(executor)
    .await
    .map(|_| ())
    .map_err(DbError::from)
}

/// Updates the kind of an audio join row. Returns `true` if a row was updated.
pub async fn update_kind(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
    asset_id: Uuid,
    kind: &str,
) -> Result<bool> {
    sqlx::query(
        "UPDATE performance_audios SET kind = ? \
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

/// Removes the join row linking an asset to a performance as audio.
///
/// Returns `true` if a row was deleted.
pub async fn unlink(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
    asset_id: Uuid,
) -> Result<bool> {
    sqlx::query("DELETE FROM performance_audios WHERE performance_id = ? AND asset_id = ?")
        .bind(performance_id)
        .bind(asset_id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
