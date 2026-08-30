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
        "SELECT id, title, created_by, lyrics_id, date_added FROM songs WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Returns the total number of songs.
pub async fn count(executor: impl Executor<'_, Database = MySql>) -> Result<u64> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM songs")
        .fetch_one(executor)
        .await
        .map(|n| n as u64)
        .map_err(DbError::from)
}

/// Returns a page of songs ordered by date added descending.
pub async fn list(
    executor: impl Executor<'_, Database = MySql>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Song>> {
    sqlx::query_as::<_, Song>(
        "SELECT id, title, created_by, lyrics_id, date_added \
         FROM songs ORDER BY date_added DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Inserts a new song and returns the created row.
pub async fn create(conn: &mut MySqlConnection, new: &NewSong) -> Result<Song> {
    sqlx::query_as::<_, Song>(
        "INSERT INTO songs (title, created_by, lyrics_id) VALUES (?, ?, ?) \
         RETURNING id, title, created_by, lyrics_id, date_added",
    )
    .bind(&new.title)
    .bind(new.created_by)
    .bind(new.lyrics_id)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Updates a song's mutable fields. Returns `None` if the ID does not exist.
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
        .map_err(DbError::from)?;
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
        "SELECT a.id, a.name, a.description \
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

/// Returns the images for a song via the `song_images` join table.
pub async fn get_images(
    executor: impl Executor<'_, Database = MySql>,
    song_id: Uuid,
) -> Result<Vec<Image>> {
    sqlx::query_as::<_, Image>(
        "SELECT i.id, i.public_url, i.internal_path, i.credits \
         FROM images i \
         JOIN song_images si ON si.image_id = i.id \
         WHERE si.song_id = ?",
    )
    .bind(song_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Replaces the full set of images for a song.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_images(
    conn: &mut MySqlConnection,
    song_id: Uuid,
    image_ids: &[Uuid],
) -> Result<()> {
    sqlx::query("DELETE FROM song_images WHERE song_id = ?")
        .bind(song_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for &image_id in image_ids {
        sqlx::query("INSERT INTO song_images (song_id, image_id) VALUES (?, ?)")
            .bind(song_id)
            .bind(image_id)
            .execute(&mut *conn)
            .await
            .map_err(DbError::from)?;
    }
    Ok(())
}
