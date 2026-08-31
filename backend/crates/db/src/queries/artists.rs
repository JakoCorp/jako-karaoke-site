//! Query functions for the `artists` table.

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::artist::{Artist, NewArtist, UpdateArtist};

type Result<T> = std::result::Result<T, DbError>;

/// Fetches an artist by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<Artist>> {
    sqlx::query_as::<_, Artist>("SELECT id, name, description FROM artists WHERE id = ?")
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(DbError::from)
}

/// Returns the total number of artists.
pub async fn count(executor: impl Executor<'_, Database = MySql>) -> Result<u64> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM artists")
        .fetch_one(executor)
        .await
        .map(|n| n as u64)
        .map_err(DbError::from)
}

/// Returns a page of artists ordered by name.
pub async fn list(
    executor: impl Executor<'_, Database = MySql>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Artist>> {
    sqlx::query_as::<_, Artist>(
        "SELECT id, name, description FROM artists ORDER BY name LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Inserts a new artist and returns the created row.
pub async fn create(conn: &mut MySqlConnection, new: &NewArtist) -> Result<Artist> {
    sqlx::query_as::<_, Artist>(
        "INSERT INTO artists (name, description) VALUES (?, ?) \
         RETURNING id, name, description",
    )
    .bind(&new.name)
    .bind(&new.description)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Updates an artist's mutable fields. Returns `None` if the ID does not exist.
pub async fn update(
    conn: &mut MySqlConnection,
    id: Uuid,
    upd: &UpdateArtist,
) -> Result<Option<Artist>> {
    sqlx::query("UPDATE artists SET name = ?, description = ? WHERE id = ?")
        .bind(&upd.name)
        .bind(&upd.description)
        .bind(id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    get_by_id(&mut *conn, id).await
}

/// Deletes an artist by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<bool> {
    sqlx::query("DELETE FROM artists WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
