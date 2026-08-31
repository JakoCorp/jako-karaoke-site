//! Query functions for the `images` table.

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::image::{Image, NewImage, UpdateImage};

type Result<T> = std::result::Result<T, DbError>;

/// Fetches an image by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<Image>> {
    sqlx::query_as::<_, Image>(
        "SELECT id, public_url, internal_path, credits FROM images WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Inserts a new image record and returns the created row.
pub async fn create(conn: &mut MySqlConnection, new: &NewImage) -> Result<Image> {
    sqlx::query_as::<_, Image>(
        "INSERT INTO images (public_url, internal_path, credits) VALUES (?, ?, ?) \
         RETURNING id, public_url, internal_path, credits",
    )
    .bind(&new.public_url)
    .bind(&new.internal_path)
    .bind(&new.credits)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Updates an image record's mutable fields. Returns `None` if the ID does not exist.
pub async fn update(
    conn: &mut MySqlConnection,
    id: Uuid,
    upd: &UpdateImage,
) -> Result<Option<Image>> {
    sqlx::query("UPDATE images SET public_url = ?, internal_path = ?, credits = ? WHERE id = ?")
        .bind(&upd.public_url)
        .bind(&upd.internal_path)
        .bind(&upd.credits)
        .bind(id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    get_by_id(&mut *conn, id).await
}

/// Deletes an image record by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<bool> {
    sqlx::query("DELETE FROM images WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
