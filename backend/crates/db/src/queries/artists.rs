//! Query functions for the `artists` table and its related join tables.

use std::collections::HashMap;

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::artist::{Artist, ArtistLink, NewArtist, NewArtistLink, UpdateArtist};
use crate::models::image::Image;

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

/// Returns artists matching an optional name query, ordered by name.
///
/// When `q` is `Some`, results are filtered by a case-insensitive substring match
/// against the artist name. Falls through to [`list`] when `q` is `None`.
pub async fn search(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Artist>> {
    let Some(q) = q else {
        return list(executor, limit, offset).await;
    };
    let pattern = format!("%{q}%");
    sqlx::query_as::<_, Artist>(
        "SELECT id, name, description FROM artists WHERE name LIKE ? ORDER BY name LIMIT ? OFFSET ?",
    )
    .bind(&pattern)
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

/// Returns the images for an artist with their kind from the `artist_images` join table.
pub async fn get_images(
    executor: impl Executor<'_, Database = MySql>,
    artist_id: Uuid,
) -> Result<Vec<(Image, String)>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        public_url: String,
        internal_path: Option<String>,
        credits: Option<String>,
        kind: String,
    }

    sqlx::query_as::<_, Row>(
        "SELECT i.id, i.public_url, i.internal_path, i.credits, aimg.kind \
         FROM images i \
         JOIN artist_images aimg ON aimg.image_id = i.id \
         WHERE aimg.artist_id = ?",
    )
    .bind(artist_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
    .map(|rows| {
        rows.into_iter()
            .map(|r| {
                (
                    Image {
                        id: r.id,
                        public_url: r.public_url,
                        internal_path: r.internal_path,
                        credits: r.credits,
                    },
                    r.kind,
                )
            })
            .collect()
    })
}

/// Returns images for multiple artists, keyed by artist ID.
///
/// Artists with no images are absent from the returned map.
pub async fn get_images_batch(
    executor: impl Executor<'_, Database = MySql>,
    artist_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<(Image, String)>>> {
    if artist_ids.is_empty() {
        return Ok(HashMap::new());
    }

    #[derive(sqlx::FromRow)]
    struct Row {
        artist_id: Uuid,
        id: Uuid,
        public_url: String,
        internal_path: Option<String>,
        credits: Option<String>,
        kind: String,
    }

    let mut builder = sqlx::QueryBuilder::new(
        "SELECT aimg.artist_id, i.id, i.public_url, i.internal_path, i.credits, aimg.kind \
         FROM images i \
         JOIN artist_images aimg ON aimg.image_id = i.id \
         WHERE aimg.artist_id IN (",
    );
    let mut separated = builder.separated(", ");
    for artist_id in artist_ids {
        separated.push_bind(artist_id);
    }
    builder.push(")");

    let rows: Vec<Row> = builder
        .build_query_as()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)?;

    let mut by_artist: HashMap<Uuid, Vec<(Image, String)>> = HashMap::new();
    for row in rows {
        by_artist.entry(row.artist_id).or_default().push((
            Image {
                id: row.id,
                public_url: row.public_url,
                internal_path: row.internal_path,
                credits: row.credits,
            },
            row.kind,
        ));
    }
    Ok(by_artist)
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
    for &(image_id, kind) in images {
        sqlx::query("INSERT INTO artist_images (artist_id, image_id, kind) VALUES (?, ?, ?)")
            .bind(artist_id)
            .bind(image_id)
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

/// Replaces the full set of external links for an artist.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_links(
    conn: &mut MySqlConnection,
    artist_id: Uuid,
    links: &[NewArtistLink],
) -> Result<Vec<ArtistLink>> {
    sqlx::query("DELETE FROM artist_links WHERE artist_id = ?")
        .bind(artist_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    let mut result = Vec::with_capacity(links.len());
    for link in links {
        let inserted = sqlx::query_as::<_, ArtistLink>(
            "INSERT INTO artist_links (artist_id, url, kind, label) VALUES (?, ?, ?, ?) \
             RETURNING id, artist_id, url, kind, label",
        )
        .bind(artist_id)
        .bind(&link.url)
        .bind(&link.kind)
        .bind(&link.label)
        .fetch_one(&mut *conn)
        .await
        .map_err(DbError::from)?;
        result.push(inserted);
    }
    Ok(result)
}
