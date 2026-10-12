//! Query functions for the `artists` table and its related join tables.

use std::collections::HashMap;

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::artist::{Artist, ArtistLink, NewArtist, NewArtistLink, UpdateArtist};

type Result<T> = std::result::Result<T, DbError>;

/// Flat query result joining an artist image join row with its asset fields.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ArtistImageRow {
    pub asset_id: Uuid,
    pub kind: String,
    pub title: Option<String>,
    pub credits: Option<String>,
    pub source_url: Option<String>,
    pub storage_url: Option<String>,
    pub internal_path: Option<String>,
    pub external_url: Option<String>,
}

/// Fetches an artist by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<Artist>> {
    sqlx::query_as::<_, Artist>(
        "SELECT id, name, description, \
         (SELECT COUNT(*) FROM song_original_artists WHERE artist_id = a.id) AS song_count \
         FROM artists a WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Returns the number of performances in which the artist appears as a singer.
pub async fn performance_count(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<u64> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM performance_singers WHERE artist_id = ?")
        .bind(id)
        .fetch_one(executor)
        .await
        .map(|n| n as u64)
        .map_err(DbError::from)
}

/// Returns the total number of artists.
async fn count(executor: impl Executor<'_, Database = MySql>) -> Result<u64> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM artists")
        .fetch_one(executor)
        .await
        .map(|n| n as u64)
        .map_err(DbError::from)
}

/// Returns artists matching an optional name query, with the given sort order.
///
/// When `q` is `None`, all artists are returned. When `q` is `Some`, results are
/// filtered by a case-insensitive substring match against the artist name.
pub async fn search(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
    order_by: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<Artist>> {
    let sql = if q.is_some() {
        format!(
            "SELECT id, name, description, \
             (SELECT COUNT(*) FROM song_original_artists WHERE artist_id = a.id) AS song_count \
             FROM artists a WHERE name LIKE ? ORDER BY {order_by} LIMIT ? OFFSET ?"
        )
    } else {
        format!(
            "SELECT id, name, description, \
             (SELECT COUNT(*) FROM song_original_artists WHERE artist_id = a.id) AS song_count \
             FROM artists a ORDER BY {order_by} LIMIT ? OFFSET ?"
        )
    };

    let query = sqlx::query_as::<_, Artist>(sqlx::AssertSqlSafe(sql.as_str()));
    let query = if let Some(pattern) = q.map(|q| format!("%{q}%")) {
        query.bind(pattern)
    } else {
        query
    };

    query
        .bind(limit)
        .bind(offset)
        .fetch_all(executor)
        .await
        .map_err(DbError::from)
}

/// Returns the total number of artists matching the optional name query.
pub async fn search_count(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
) -> Result<u64> {
    let Some(q) = q else {
        return count(executor).await;
    };
    let pattern = format!("%{q}%");
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM artists WHERE name LIKE ?")
        .bind(&pattern)
        .fetch_one(executor)
        .await
        .map(|n| n as u64)
        .map_err(DbError::from)
}

/// Inserts a new artist and returns the created row.
pub async fn create(conn: &mut MySqlConnection, new: &NewArtist) -> Result<Artist> {
    sqlx::query_as::<_, Artist>(
        "INSERT INTO artists (name, description) VALUES (?, ?) \
         RETURNING id, name, description, 0 AS song_count",
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

/// Returns the images for an artist joined with asset fields.
pub async fn get_images(
    executor: impl Executor<'_, Database = MySql>,
    artist_id: Uuid,
) -> Result<Vec<ArtistImageRow>> {
    sqlx::query_as::<_, ArtistImageRow>(
        "SELECT aimg.asset_id, aimg.kind, \
         a.title, a.credits, a.source_url, a.storage_url, a.internal_path, a.external_url \
         FROM assets a \
         JOIN artist_images aimg ON aimg.asset_id = a.id \
         WHERE aimg.artist_id = ?",
    )
    .bind(artist_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Returns images for multiple artists, keyed by artist ID.
///
/// Artists with no images are absent from the returned map.
pub async fn get_images_batch(
    executor: impl Executor<'_, Database = MySql>,
    artist_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<ArtistImageRow>>> {
    if artist_ids.is_empty() {
        return Ok(HashMap::new());
    }

    #[derive(sqlx::FromRow)]
    struct BatchRow {
        artist_id: Uuid,
        asset_id: Uuid,
        kind: String,
        title: Option<String>,
        credits: Option<String>,
        source_url: Option<String>,
        storage_url: Option<String>,
        internal_path: Option<String>,
        external_url: Option<String>,
    }

    let mut builder = sqlx::QueryBuilder::new(
        "SELECT aimg.artist_id, aimg.asset_id, aimg.kind, \
         a.title, a.credits, a.source_url, a.storage_url, a.internal_path, a.external_url \
         FROM assets a \
         JOIN artist_images aimg ON aimg.asset_id = a.id \
         WHERE aimg.artist_id IN (",
    );
    let mut separated = builder.separated(", ");
    for artist_id in artist_ids {
        separated.push_bind(artist_id);
    }
    builder.push(")");

    let rows: Vec<BatchRow> = builder
        .build_query_as()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)?;

    let mut by_artist: HashMap<Uuid, Vec<ArtistImageRow>> = HashMap::new();
    for row in rows {
        by_artist
            .entry(row.artist_id)
            .or_default()
            .push(ArtistImageRow {
                asset_id: row.asset_id,
                kind: row.kind,
                title: row.title,
                credits: row.credits,
                source_url: row.source_url,
                storage_url: row.storage_url,
                internal_path: row.internal_path,
                external_url: row.external_url,
            });
    }
    Ok(by_artist)
}

/// Inserts a single `artist_images` join row.
pub async fn link_image(
    conn: &mut MySqlConnection,
    artist_id: Uuid,
    asset_id: Uuid,
    kind: &str,
) -> Result<()> {
    sqlx::query("INSERT IGNORE INTO artist_images (artist_id, asset_id, kind) VALUES (?, ?, ?)")
        .bind(artist_id)
        .bind(asset_id)
        .bind(kind)
        .execute(conn)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Updates the kind of an `artist_images` join row. Returns `true` if a row was updated.
pub async fn update_image_kind(
    executor: impl Executor<'_, Database = MySql>,
    artist_id: Uuid,
    asset_id: Uuid,
    kind: &str,
) -> Result<bool> {
    sqlx::query("UPDATE artist_images SET kind = ? WHERE artist_id = ? AND asset_id = ?")
        .bind(kind)
        .bind(artist_id)
        .bind(asset_id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Removes a single `artist_images` join row. Returns `true` if a row was deleted.
pub async fn unlink_image(
    executor: impl Executor<'_, Database = MySql>,
    artist_id: Uuid,
    asset_id: Uuid,
) -> Result<bool> {
    sqlx::query("DELETE FROM artist_images WHERE artist_id = ? AND asset_id = ?")
        .bind(artist_id)
        .bind(asset_id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Replaces the full set of images for an artist.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_images(
    conn: &mut MySqlConnection,
    artist_id: Uuid,
    images: &[(Uuid, &str)],
) -> Result<()> {
    sqlx::query("DELETE FROM artist_images WHERE artist_id = ?")
        .bind(artist_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for &(asset_id, kind) in images {
        sqlx::query("INSERT INTO artist_images (artist_id, asset_id, kind) VALUES (?, ?, ?)")
            .bind(artist_id)
            .bind(asset_id)
            .bind(kind)
            .execute(&mut *conn)
            .await
            .map_err(DbError::from)?;
    }
    Ok(())
}

/// Returns all external links for an artist.
pub async fn get_links(
    executor: impl Executor<'_, Database = MySql>,
    artist_id: Uuid,
) -> Result<Vec<ArtistLink>> {
    sqlx::query_as::<_, ArtistLink>(
        "SELECT id, artist_id, url, kind, label \
         FROM artist_links WHERE artist_id = ?",
    )
    .bind(artist_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Creates a single external link for an artist.
pub async fn create_link(
    conn: &mut MySqlConnection,
    artist_id: Uuid,
    link: &NewArtistLink,
) -> Result<ArtistLink> {
    sqlx::query_as::<_, ArtistLink>(
        "INSERT INTO artist_links (artist_id, url, kind, label) VALUES (?, ?, ?, ?) \
         RETURNING id, artist_id, url, kind, label",
    )
    .bind(artist_id)
    .bind(&link.url)
    .bind(&link.kind)
    .bind(&link.label)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Replaces the url, kind, and label of an artist link by its ID.
pub async fn update_link(
    executor: impl Executor<'_, Database = MySql>,
    link_id: Uuid,
    link: &NewArtistLink,
) -> Result<Option<ArtistLink>> {
    sqlx::query_as::<_, ArtistLink>(
        "UPDATE artist_links SET url = ?, kind = ?, label = ? WHERE id = ? \
         RETURNING id, artist_id, url, kind, label",
    )
    .bind(&link.url)
    .bind(&link.kind)
    .bind(&link.label)
    .bind(link_id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Deletes a single artist link by its ID. Returns `true` if the row existed.
pub async fn delete_link(
    executor: impl Executor<'_, Database = MySql>,
    link_id: Uuid,
) -> Result<bool> {
    let result = sqlx::query("DELETE FROM artist_links WHERE id = ?")
        .bind(link_id)
        .execute(executor)
        .await
        .map_err(DbError::from)?;
    Ok(result.rows_affected() > 0)
}
