//! Query functions for the `songs` table and its related join tables.
//!
//! M2M relations (artists, tags, images) are managed via full replace helpers
//! (e.g., `set_original_artists`, `set_tags`, `set_images`) that delete existing
//! rows and reinsert.
use std::collections::HashMap;

use sqlx::{Executor, MySql, MySqlConnection, QueryBuilder};
use uuid::Uuid;

use crate::error::DbError;
use crate::filter::{Clause, Subject, WhereClause, push_clauses};
use crate::models::artist::Artist;
use crate::models::song::{NewSong, Song, UpdateSong};
use crate::models::tag::TagWithKind;
use crate::sort::SortDirection;

/// Flat query result joining a song image join row with its asset fields.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SongImageRow {
    pub asset_id: Uuid,
    pub kind: String,
    pub title: Option<String>,
    pub credits: Option<String>,
    pub source_url: Option<String>,
    pub storage_url: Option<String>,
    pub internal_path: Option<String>,
    pub external_url: Option<String>,
}

type Result<T> = std::result::Result<T, DbError>;

/// Column list matching the fields of [`Song`], for a query over `songs s`.
///
/// The `inserted` form is for a freshly inserted row, which has no performances yet.
macro_rules! song_columns {
    () => {
        song_columns!(@with "(SELECT COUNT(*) FROM performance_songs WHERE song_id = s.id)")
    };
    (inserted) => {
        song_columns!(@with "0")
    };
    (@with $performance_count:literal) => {
        concat!(
            "id, title, created_by, lyrics_id, ",
            $performance_count,
            " AS performance_count"
        )
    };
}

/// Fetches a song by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<Song>> {
    sqlx::query_as::<_, Song>(concat!(
        "SELECT ",
        song_columns!(),
        " FROM songs s WHERE s.id = ?"
    ))
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Filters applied by [`search`] and [`search_count`]. Every set field must match.
#[derive(Debug, Clone, Copy, Default)]
pub struct SongFilter<'a> {
    /// Case insensitive substring matched against the song title or any original artist name,
    /// or the song id matched exactly.
    pub q: Option<&'a str>,
    /// Relation conditions, all of which must hold.
    pub clauses: &'a [Clause],
}

/// Appends the `WHERE` clause for `filter`, binding its values. Appends nothing for an empty filter.
fn push_filters(builder: &mut QueryBuilder<MySql>, filter: SongFilter<'_>) {
    let mut conditions = WhereClause::new(builder);

    if let Some(q) = filter.q {
        let pattern = format!("%{q}%");
        conditions
            .condition()
            .push("s.id IN (SELECT id FROM songs WHERE title LIKE ")
            .push_bind(pattern.clone())
            .push(" OR id = ")
            .push_bind(Uuid::parse_str(q).ok())
            .push(
                " UNION SELECT soa.song_id FROM song_original_artists soa \
                 JOIN artists a ON a.id = soa.artist_id WHERE a.name LIKE ",
            )
            .push_bind(pattern)
            .push(")");
    }

    push_clauses(&mut conditions, Subject::Song, "s.id", filter.clauses);
}

/// Field a song list is sorted by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    PerformanceCount,
}

/// Appends the `ORDER BY` clause. Ties are always broken by newest song first.
fn push_order(builder: &mut QueryBuilder<MySql>, sort: SortKey, direction: SortDirection) {
    let column = match sort {
        SortKey::PerformanceCount => "performance_count",
    };
    builder
        .push(" ORDER BY ")
        .push(column)
        .push(" ")
        .push(direction.as_sql())
        .push(", s.id DESC");
}

/// Returns a filtered page of songs sorted by `sort` in `direction`.
pub async fn search(
    executor: impl Executor<'_, Database = MySql>,
    filter: SongFilter<'_>,
    sort: SortKey,
    direction: SortDirection,
    limit: u32,
    offset: u32,
) -> Result<Vec<Song>> {
    let mut builder = QueryBuilder::new(concat!("SELECT ", song_columns!(), " FROM songs s"));
    push_filters(&mut builder, filter);
    push_order(&mut builder, sort, direction);
    builder
        .push(" LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    builder
        .build_query_as::<Song>()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)
}

/// Returns the total number of songs matching `filter`.
pub async fn search_count(
    executor: impl Executor<'_, Database = MySql>,
    filter: SongFilter<'_>,
) -> Result<u64> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM songs s");
    push_filters(&mut builder, filter);

    builder
        .build_query_scalar::<i64>()
        .fetch_one(executor)
        .await
        .map(|n| n as u64)
        .map_err(DbError::from)
}

/// Inserts a new song and returns the created row.
///
/// Returns [`DbError::Conflict`] if a song with the same title already exists.
pub async fn create(conn: &mut MySqlConnection, new: &NewSong) -> Result<Song> {
    sqlx::query_as::<_, Song>(concat!(
        "INSERT INTO songs (title, created_by, lyrics_id) VALUES (?, ?, ?) RETURNING ",
        song_columns!(inserted)
    ))
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

/// Returns the images for a song joined with asset fields.
pub async fn get_images(
    executor: impl Executor<'_, Database = MySql>,
    song_id: Uuid,
) -> Result<Vec<SongImageRow>> {
    sqlx::query_as::<_, SongImageRow>(
        "SELECT si.asset_id, si.kind, \
         a.title, a.credits, a.source_url, a.storage_url, a.internal_path, a.external_url \
         FROM assets a \
         JOIN song_images si ON si.asset_id = a.id \
         WHERE si.song_id = ?",
    )
    .bind(song_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Returns images for multiple songs, keyed by song ID.
///
/// Songs with no images are absent from the returned map.
pub async fn get_images_batch(
    executor: impl Executor<'_, Database = MySql>,
    song_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<SongImageRow>>> {
    if song_ids.is_empty() {
        return Ok(HashMap::new());
    }

    #[derive(sqlx::FromRow)]
    struct BatchRow {
        song_id: Uuid,
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
        "SELECT si.song_id, si.asset_id, si.kind, \
         a.title, a.credits, a.source_url, a.storage_url, a.internal_path, a.external_url \
         FROM assets a \
         JOIN song_images si ON si.asset_id = a.id \
         WHERE si.song_id IN (",
    );
    let mut separated = builder.separated(", ");
    for song_id in song_ids {
        separated.push_bind(song_id);
    }
    builder.push(")");

    let rows: Vec<BatchRow> = builder
        .build_query_as()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)?;

    let mut by_song: HashMap<Uuid, Vec<SongImageRow>> = HashMap::new();
    for row in rows {
        by_song.entry(row.song_id).or_default().push(SongImageRow {
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
    Ok(by_song)
}

/// Inserts a single `song_images` join row.
pub async fn link_image(
    conn: &mut MySqlConnection,
    song_id: Uuid,
    asset_id: Uuid,
    kind: &str,
) -> Result<()> {
    sqlx::query("INSERT IGNORE INTO song_images (song_id, asset_id, kind) VALUES (?, ?, ?)")
        .bind(song_id)
        .bind(asset_id)
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
    asset_id: Uuid,
) -> Result<bool> {
    sqlx::query("DELETE FROM song_images WHERE song_id = ? AND asset_id = ?")
        .bind(song_id)
        .bind(asset_id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Updates the kind of a `song_images` join row. Returns `true` if a row was updated.
pub async fn update_image_kind(
    executor: impl Executor<'_, Database = MySql>,
    song_id: Uuid,
    asset_id: Uuid,
    kind: &str,
) -> Result<bool> {
    sqlx::query("UPDATE song_images SET kind = ? WHERE song_id = ? AND asset_id = ?")
        .bind(kind)
        .bind(song_id)
        .bind(asset_id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
