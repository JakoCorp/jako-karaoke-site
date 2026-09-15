//! Query functions for the `songs` table and its related join tables.
//!
//! M2M relations (artists, tags, images) are managed via full replace helpers
//! (e.g., `set_original_artists`, `set_tags`, `set_images`) that delete existing
//! rows and reinsert.
use std::collections::HashMap;

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::artist::Artist;
use crate::models::image::Image;
use crate::models::song::{NewSong, Song, UpdateSong};
use crate::models::tag::TagWithKind;

type Result<T> = std::result::Result<T, DbError>;

/// Fetches a song by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<Song>> {
    sqlx::query_as::<_, Song>(
        "SELECT id, title, created_by, lyrics_id, \
         (SELECT COUNT(*) FROM performance_songs WHERE song_id = s.id) AS performance_count \
         FROM songs s WHERE s.id = ?",
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Returns songs matching an optional text query with a caller-specified sort order.
///
/// When `q` is `Some`, results are filtered by a case-insensitive substring match
/// against the song title or any linked original artist name.
/// `order_by` must be a trusted SQL fragment (e.g. `"performance_count DESC, s.id DESC"`).
pub async fn search(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
    order_by: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<Song>> {
    let sql = if q.is_some() {
        format!(
            "SELECT id, title, created_by, lyrics_id, \
             (SELECT COUNT(*) FROM performance_songs WHERE song_id = s.id) AS performance_count \
             FROM songs s WHERE s.id IN ( \
               SELECT id FROM songs WHERE title LIKE ? \
               UNION \
               SELECT soa.song_id FROM song_original_artists soa \
               JOIN artists a ON a.id = soa.artist_id WHERE a.name LIKE ? \
             ) ORDER BY {order_by} LIMIT ? OFFSET ?"
        )
    } else {
        format!(
            "SELECT id, title, created_by, lyrics_id, \
             (SELECT COUNT(*) FROM performance_songs WHERE song_id = s.id) AS performance_count \
             FROM songs s ORDER BY {order_by} LIMIT ? OFFSET ?"
        )
    };

    let query = sqlx::query_as::<_, Song>(sqlx::AssertSqlSafe(sql.as_str()));
    let query = if let Some(pattern) = q.map(|q| format!("%{q}%")) {
        query.bind(pattern.clone()).bind(pattern)
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

/// Returns the total number of songs matching the optional text query.
pub async fn search_count(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
) -> Result<u64> {
    match q {
        None => sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM songs")
            .fetch_one(executor)
            .await
            .map(|n| n as u64)
            .map_err(DbError::from),
        Some(q) => {
            let pattern = format!("%{q}%");
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM ( \
                   SELECT id FROM songs WHERE title LIKE ? \
                   UNION \
                   SELECT soa.song_id FROM song_original_artists soa \
                   JOIN artists a ON a.id = soa.artist_id WHERE a.name LIKE ? \
                 ) AS matched",
            )
            .bind(&pattern)
            .bind(&pattern)
            .fetch_one(executor)
            .await
            .map(|n| n as u64)
            .map_err(DbError::from)
        }
    }
}

/// Inserts a new song and returns the created row.
///
/// Returns [`DbError::Conflict`] if a song with the same title already exists.
pub async fn create(conn: &mut MySqlConnection, new: &NewSong) -> Result<Song> {
    sqlx::query_as::<_, Song>(
        "INSERT INTO songs (title, created_by, lyrics_id) VALUES (?, ?, ?) \
         RETURNING id, title, created_by, lyrics_id, 0 AS performance_count",
    )
    .bind(&new.title)
    .bind(new.created_by)
    .bind(new.lyrics_id)
    .fetch_one(conn)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db) if db.is_unique_violation() => DbError::Conflict,
        other => DbError::Sqlx(other),
    })
}

/// Updates a song's mutable fields. Returns `None` if the ID does not exist.
///
/// Returns [`DbError::Conflict`] if the new title is already taken by another song.
pub async fn update(
    conn: &mut MySqlConnection,
    id: Uuid,
    upd: &UpdateSong,
) -> Result<Option<Song>> {
    sqlx::query("UPDATE songs SET title = ? WHERE id = ?")
        .bind(&upd.title)
        .bind(id)
        .execute(&mut *conn)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db) if db.is_unique_violation() => DbError::Conflict,
            other => DbError::Sqlx(other),
        })?;
    get_by_id(&mut *conn, id).await
}

/// Sets the `lyrics_id` foreign key on a song, or clears it with `None`.
pub async fn update_lyrics_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
    lyrics_id: Option<Uuid>,
) -> Result<()> {
    sqlx::query("UPDATE songs SET lyrics_id = ? WHERE id = ?")
        .bind(lyrics_id)
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Deletes a song by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<bool> {
    sqlx::query("DELETE FROM songs WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Returns the original artists for a song via the `song_original_artists` join table.
pub async fn get_original_artists(
    executor: impl Executor<'_, Database = MySql>,
    song_id: Uuid,
) -> Result<Vec<Artist>> {
    sqlx::query_as::<_, Artist>(
        "SELECT a.id, a.name, a.description, 0 AS song_count \
         FROM artists a \
         JOIN song_original_artists soa ON soa.artist_id = a.id \
         WHERE soa.song_id = ?",
    )
    .bind(song_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Returns original artists for multiple songs, keyed by song ID.
///
/// Songs with no original artists are absent from the returned map.
pub async fn get_original_artists_batch(
    executor: impl Executor<'_, Database = MySql>,
    song_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<Artist>>> {
    if song_ids.is_empty() {
        return Ok(HashMap::new());
    }

    #[derive(sqlx::FromRow)]
    struct Row {
        song_id: Uuid,
        id: Uuid,
        name: String,
        description: Option<String>,
    }

    let mut builder = sqlx::QueryBuilder::new(
        "SELECT soa.song_id, a.id, a.name, a.description \
         FROM artists a \
         JOIN song_original_artists soa ON soa.artist_id = a.id \
         WHERE soa.song_id IN (",
    );
    let mut separated = builder.separated(", ");
    for song_id in song_ids {
        separated.push_bind(song_id);
    }
    builder.push(")");

    let rows: Vec<Row> = builder
        .build_query_as()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)?;

    let mut by_song: HashMap<Uuid, Vec<Artist>> = HashMap::new();
    for row in rows {
        by_song.entry(row.song_id).or_default().push(Artist {
            id: row.id,
            name: row.name,
            description: row.description,
            song_count: 0,
        });
    }
    Ok(by_song)
}

/// Replaces the full set of original artists for a song.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_original_artists(
    conn: &mut MySqlConnection,
    song_id: Uuid,
    artist_ids: &[Uuid],
) -> Result<()> {
    sqlx::query("DELETE FROM song_original_artists WHERE song_id = ?")
        .bind(song_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for &artist_id in artist_ids {
        sqlx::query("INSERT INTO song_original_artists (song_id, artist_id) VALUES (?, ?)")
            .bind(song_id)
            .bind(artist_id)
            .execute(&mut *conn)
            .await
            .map_err(DbError::from)?;
    }
    Ok(())
}

/// Returns the tags for a song with their kind from the `song_tags` join table.
pub async fn get_tags(
    executor: impl Executor<'_, Database = MySql>,
    song_id: Uuid,
) -> Result<Vec<TagWithKind>> {
    sqlx::query_as::<_, TagWithKind>(
        "SELECT t.id, t.name, st.kind \
         FROM tags t \
         JOIN song_tags st ON st.tag_id = t.id \
         WHERE st.song_id = ?",
    )
    .bind(song_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Replaces the full set of tags for a song.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_tags(
    conn: &mut MySqlConnection,
    song_id: Uuid,
    tags: &[(Uuid, &str)],
) -> Result<()> {
    sqlx::query("DELETE FROM song_tags WHERE song_id = ?")
        .bind(song_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for &(tag_id, kind) in tags {
        sqlx::query("INSERT INTO song_tags (song_id, tag_id, kind) VALUES (?, ?, ?)")
            .bind(song_id)
            .bind(tag_id)
            .bind(kind)
            .execute(&mut *conn)
            .await
            .map_err(DbError::from)?;
    }
    Ok(())
}

/// Returns the images for a song with their kind from the `song_images` join table.
pub async fn get_images(
    executor: impl Executor<'_, Database = MySql>,
    song_id: Uuid,
) -> Result<Vec<(Image, String)>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        hash: String,
        public_url: String,
        internal_path: Option<String>,
        credits: Option<String>,
        kind: String,
    }

    sqlx::query_as::<_, Row>(
        "SELECT i.id, i.hash, i.public_url, i.internal_path, i.credits, si.kind \
         FROM images i \
         JOIN song_images si ON si.image_id = i.id \
         WHERE si.song_id = ?",
    )
    .bind(song_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
    .map(|rows| {
        rows.into_iter()
            .map(|r| {
                (
                    Image {
                        id: r.id,
                        hash: r.hash,
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

/// Inserts a single `song_images` join row.
pub async fn link_image(
    conn: &mut MySqlConnection,
    song_id: Uuid,
    image_id: Uuid,
    kind: &str,
) -> Result<()> {
    sqlx::query("INSERT IGNORE INTO song_images (song_id, image_id, kind) VALUES (?, ?, ?)")
        .bind(song_id)
        .bind(image_id)
        .bind(kind)
        .execute(conn)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Removes a single `song_images` join row. Returns `true` if a row was deleted.
pub async fn unlink_image(
    executor: impl Executor<'_, Database = MySql>,
    song_id: Uuid,
    image_id: Uuid,
) -> Result<bool> {
    sqlx::query("DELETE FROM song_images WHERE song_id = ? AND image_id = ?")
        .bind(song_id)
        .bind(image_id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Updates the kind of a `song_images` join row. Returns `true` if a row was updated.
pub async fn update_image_kind(
    conn: &mut MySqlConnection,
    song_id: Uuid,
    image_id: Uuid,
    kind: &str,
) -> Result<bool> {
    sqlx::query("UPDATE song_images SET kind = ? WHERE song_id = ? AND image_id = ?")
        .bind(kind)
        .bind(song_id)
        .bind(image_id)
        .execute(conn)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Replaces the full set of images for a song.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_images(
    conn: &mut MySqlConnection,
    song_id: Uuid,
    images: &[(Uuid, &str)],
) -> Result<()> {
    sqlx::query("DELETE FROM song_images WHERE song_id = ?")
        .bind(song_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for &(image_id, kind) in images {
        sqlx::query("INSERT INTO song_images (song_id, image_id, kind) VALUES (?, ?, ?)")
            .bind(song_id)
            .bind(image_id)
            .bind(kind)
            .execute(&mut *conn)
            .await
            .map_err(DbError::from)?;
    }
    Ok(())
}
