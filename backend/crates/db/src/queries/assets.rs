//! Query functions for the `assets` table.

use std::collections::HashMap;

use sqlx::{Executor, MySql, MySqlConnection, QueryBuilder};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::asset::{Asset, NewExternalAsset, NewInternalAsset};

type Result<T> = std::result::Result<T, DbError>;

/// Column list matching the fields of [`Asset`].
macro_rules! asset_columns {
    () => {
        "id, title, credits, source_url, hash, storage_url, internal_path, external_url, length"
    };
}

/// Fetches an asset by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<Asset>> {
    sqlx::query_as::<_, Asset>(concat!(
        "SELECT ",
        asset_columns!(),
        " FROM assets WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Fetches multiple assets by ID, keyed by asset ID.
///
/// IDs that do not exist are absent from the returned map.
pub async fn get_by_ids(
    executor: impl Executor<'_, Database = MySql>,
    ids: &[Uuid],
) -> Result<HashMap<Uuid, Asset>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::new(concat!(
        "SELECT ",
        asset_columns!(),
        " FROM assets WHERE id IN ("
    ));
    let mut separated = builder.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    builder.push(")");

    let assets: Vec<Asset> = builder
        .build_query_as()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)?;
    Ok(assets.into_iter().map(|asset| (asset.id, asset)).collect())
}

/// Fetches an internal asset by its SHA-256 hash. Used for deduplication on upload.
pub async fn get_by_hash(
    executor: impl Executor<'_, Database = MySql>,
    hash: &str,
) -> Result<Option<Asset>> {
    sqlx::query_as::<_, Asset>(concat!(
        "SELECT ",
        asset_columns!(),
        " FROM assets WHERE hash = ?"
    ))
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
            (SELECT COUNT(*) FROM performance_videos WHERE asset_id = ?) + \
            (SELECT COUNT(*) FROM soundbites WHERE asset_id = ?) + \
            (SELECT COUNT(*) FROM user_avatars WHERE asset_id = ?)",
    )
    .bind(asset_id)
    .bind(asset_id)
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
    sqlx::query_as::<_, Asset>(concat!(
        "INSERT INTO assets (title, credits, source_url, hash, storage_url, internal_path, length) \
         VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING ",
        asset_columns!()
    ))
    .bind(&new.title)
    .bind(&new.credits)
    .bind(&new.source_url)
    .bind(&new.hash)
    .bind(&new.storage_url)
    .bind(&new.internal_path)
    .bind(new.length)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Inserts a new external asset record and returns the created row.
pub async fn create_external(conn: &mut MySqlConnection, new: &NewExternalAsset) -> Result<Asset> {
    sqlx::query_as::<_, Asset>(concat!(
        "INSERT INTO assets (title, credits, source_url, external_url, length) \
         VALUES (?, ?, ?, ?, ?) RETURNING ",
        asset_columns!()
    ))
    .bind(&new.title)
    .bind(&new.credits)
    .bind(&new.source_url)
    .bind(&new.external_url)
    .bind(new.length)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Returns the internal path of every stored asset.
pub async fn list_internal_paths(
    executor: impl Executor<'_, Database = MySql>,
) -> Result<Vec<String>> {
    sqlx::query_scalar::<_, String>(
        "SELECT internal_path FROM assets WHERE internal_path IS NOT NULL ORDER BY internal_path",
    )
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Moves stored assets to a new storage location by rewriting their URL and path prefixes.
///
/// Prefixes match whole path segments and trailing slashes are ignored.
pub async fn rebase_storage(
    conn: &mut MySqlConnection,
    old_base_url: &str,
    new_base_url: &str,
    old_path: &str,
    new_path: &str,
) -> Result<()> {
    for (sql, old, new) in [
        (
            "UPDATE assets SET storage_url = CONCAT(?, SUBSTRING(storage_url, CHAR_LENGTH(?) + 1)) \
             WHERE LEFT(storage_url, CHAR_LENGTH(?)) = ?",
            old_base_url,
            new_base_url,
        ),
        (
            "UPDATE assets SET internal_path = CONCAT(?, SUBSTRING(internal_path, CHAR_LENGTH(?) + 1)) \
             WHERE LEFT(internal_path, CHAR_LENGTH(?)) = ?",
            old_path,
            new_path,
        ),
    ] {
        let old = format!("{}/", old.trim_end_matches('/'));
        let new = format!("{}/", new.trim_end_matches('/'));
        sqlx::query(sql)
            .bind(&new)
            .bind(&old)
            .bind(&old)
            .bind(&old)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
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
