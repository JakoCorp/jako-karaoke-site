//! Query functions for the `assets` table.

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::asset::{Asset, NewExternalAsset, NewInternalAsset};

type Result<T> = std::result::Result<T, DbError>;

/// Fetches an asset by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<Asset>> {
    sqlx::query_as::<_, Asset>(
        "SELECT id, title, credits, source_url, hash, storage_url, internal_path, external_url \
         FROM assets WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Fetches an internal asset by its SHA-256 hash. Used for deduplication on upload.
pub async fn get_by_hash(
    executor: impl Executor<'_, Database = MySql>,
    hash: &str,
) -> Result<Option<Asset>> {
    sqlx::query_as::<_, Asset>(
        "SELECT id, title, credits, source_url, hash, storage_url, internal_path, external_url \
         FROM assets WHERE hash = ?",
    )
    .bind(hash)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Returns the total number of join rows referencing this asset across all resource types.
///
/// Used to determine whether an asset row and its associated file can be safely removed.
pub async fn reference_count(
    executor: impl Executor<'_, Database = MySql>,
    asset_id: Uuid,
) -> Result<i64> {
    sqlx::query_scalar::<_, i64>(
        "SELECT \
            (SELECT COUNT(*) FROM song_images WHERE asset_id = ?) + \
            (SELECT COUNT(*) FROM artist_images WHERE asset_id = ?) + \
            (SELECT COUNT(*) FROM performance_audios WHERE asset_id = ?) + \
            (SELECT COUNT(*) FROM performance_videos WHERE asset_id = ?)",
    )
    .bind(asset_id)
    .bind(asset_id)
    .bind(asset_id)
    .bind(asset_id)
    .fetch_one(executor)
    .await
    .map_err(DbError::from)
}

/// Inserts a new internal asset record and returns the created row.
pub async fn create_internal(conn: &mut MySqlConnection, new: &NewInternalAsset) -> Result<Asset> {
    sqlx::query_as::<_, Asset>(
        "INSERT INTO assets (title, credits, source_url, hash, storage_url, internal_path) \
         VALUES (?, ?, ?, ?, ?, ?) \
         RETURNING id, title, credits, source_url, hash, storage_url, internal_path, external_url",
    )
    .bind(&new.title)
    .bind(&new.credits)
    .bind(&new.source_url)
    .bind(&new.hash)
    .bind(&new.storage_url)
    .bind(&new.internal_path)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Inserts a new external asset record and returns the created row.
pub async fn create_external(conn: &mut MySqlConnection, new: &NewExternalAsset) -> Result<Asset> {
    sqlx::query_as::<_, Asset>(
        "INSERT INTO assets (title, credits, source_url, external_url) \
         VALUES (?, ?, ?, ?) \
         RETURNING id, title, credits, source_url, hash, storage_url, internal_path, external_url",
    )
    .bind(&new.title)
    .bind(&new.credits)
    .bind(&new.source_url)
    .bind(&new.external_url)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Deletes an asset record by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<bool> {
    sqlx::query("DELETE FROM assets WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
