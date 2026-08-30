//! Query functions for the `performances` table and its related join tables.
//!
//! M2M relations (songs, singers) are managed via full replace helpers
//! (`set_songs`, `set_singers`) that delete existing rows and reinsert.

use std::collections::HashMap;

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::artist::Artist;
use crate::models::performance::{NewPerformance, Performance, UpdatePerformance};
use crate::models::song::Song;
use crate::models::tag::TagWithKind;

type Result<T> = std::result::Result<T, DbError>;

/// Fetches a performance by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<Performance>> {
    sqlx::query_as::<_, Performance>(
        "SELECT id, created_by, title, lyrics_id, play_count, duration, performance_date \
         FROM performances WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Returns the total number of performances.
pub async fn count(executor: impl Executor<'_, Database = MySql>) -> Result<u64> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM performances")
        .fetch_one(executor)
        .await
        .map(|n| n as u64)
        .map_err(DbError::from)
}

/// Returns a filtered, sorted page of performances.
///
/// When `q` is `Some`, results are filtered by a case-insensitive substring match
/// against the performance title, any linked song title, or any linked singer name.
/// `sort_col` and `sort_dir` must be derived from validated enums to prevent injection.
pub async fn search(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
    sort_col: &str,
    sort_dir: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<Performance>> {
    let sql = if q.is_some() {
        format!(
            "SELECT id, created_by, title, lyrics_id, play_count, duration, performance_date \
             FROM performances WHERE id IN ( \
               SELECT id FROM performances WHERE title LIKE ? \
               UNION \
               SELECT ps.performance_id FROM performance_songs ps \
               JOIN songs s ON s.id = ps.song_id WHERE s.title LIKE ? \
               UNION \
               SELECT ps.performance_id FROM performance_singers ps \
               JOIN artists a ON a.id = ps.artist_id WHERE a.name LIKE ? \
             ) ORDER BY {sort_col} {sort_dir} LIMIT ? OFFSET ?"
        )
    } else {
        format!(
            "SELECT id, created_by, title, lyrics_id, play_count, duration, performance_date \
             FROM performances ORDER BY {sort_col} {sort_dir} LIMIT ? OFFSET ?"
        )
    };

    let query = sqlx::query_as::<_, Performance>(sqlx::AssertSqlSafe(sql.as_str()));
    let query = if let Some(pattern) = q.map(|q| format!("%{q}%")) {
        query
            .bind(pattern.clone())
            .bind(pattern.clone())
            .bind(pattern)
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

/// Returns the total number of performances matching the optional text query.
pub async fn search_count(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
) -> Result<u64> {
    if let Some(q) = q {
        let pattern = format!("%{q}%");
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM ( \
               SELECT id FROM performances WHERE title LIKE ? \
               UNION \
               SELECT ps.performance_id FROM performance_songs ps \
               JOIN songs s ON s.id = ps.song_id WHERE s.title LIKE ? \
               UNION \
               SELECT ps.performance_id FROM performance_singers ps \
               JOIN artists a ON a.id = ps.artist_id WHERE a.name LIKE ? \
             ) AS matched",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_one(executor)
        .await
        .map(|n| n as u64)
        .map_err(DbError::from)
    } else {
        count(executor).await
    }
}

/// Returns songs for multiple performances, keyed by performance ID.
///
/// Each value is a vec of `(song_id, song_title)` tuples.
/// Performances with no linked songs are absent from the map.
pub async fn get_songs_batch(
    executor: impl Executor<'_, Database = MySql>,
    performance_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<(Uuid, String)>>> {
    if performance_ids.is_empty() {
        return Ok(HashMap::new());
    }

    #[derive(sqlx::FromRow)]
    struct Row {
        performance_id: Uuid,
        id: Uuid,
        title: String,
    }

    let mut builder = sqlx::QueryBuilder::new(
        "SELECT ps.performance_id, s.id, s.title \
         FROM songs s \
         JOIN performance_songs ps ON ps.song_id = s.id \
         WHERE ps.performance_id IN (",
    );
    let mut separated = builder.separated(", ");
    for performance_id in performance_ids {
        separated.push_bind(performance_id);
    }
    builder.push(")");

    let rows: Vec<Row> = builder
        .build_query_as()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)?;

    let mut by_performance: HashMap<Uuid, Vec<(Uuid, String)>> = HashMap::new();
    for row in rows {
        by_performance
            .entry(row.performance_id)
            .or_default()
            .push((row.id, row.title));
    }
    Ok(by_performance)
}

/// Inserts a new performance and returns the created row.
pub async fn create(conn: &mut MySqlConnection, new: &NewPerformance) -> Result<Performance> {
    sqlx::query_as::<_, Performance>(
        "INSERT INTO performances \
         (created_by, title, lyrics_id, duration, performance_date) \
         VALUES (?, ?, ?, ?, ?) \
         RETURNING id, created_by, title, lyrics_id, play_count, duration, performance_date",
    )
    .bind(new.created_by)
    .bind(&new.title)
    .bind(new.lyrics_id)
    .bind(new.duration)
    .bind(new.performance_date)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Updates a performance's mutable fields. Returns `None` if the ID does not exist.
pub async fn update(
    conn: &mut MySqlConnection,
    id: Uuid,
    upd: &UpdatePerformance,
) -> Result<Option<Performance>> {
    sqlx::query(
        "UPDATE performances \
         SET title = ?, duration = ?, performance_date = ? \
         WHERE id = ?",
    )
    .bind(&upd.title)
    .bind(upd.duration)
    .bind(upd.performance_date)
    .bind(id)
    .execute(&mut *conn)
    .await
    .map_err(DbError::from)?;
    get_by_id(&mut *conn, id).await
}

/// Sets the `lyrics_id` foreign key on a performance, or clears it with `None`.
pub async fn update_lyrics_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
    lyrics_id: Option<Uuid>,
) -> Result<()> {
    sqlx::query("UPDATE performances SET lyrics_id = ? WHERE id = ?")
        .bind(lyrics_id)
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Returns the lyrics content from the first linked song that has lyrics.
///
/// Used as a fallback when the performance itself has no `lyrics_id` set.
/// Songs are ordered by ID to give a deterministic result when multiple are linked.
pub async fn get_fallback_song_lyrics(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
) -> Result<Option<String>> {
    sqlx::query_scalar::<_, String>(
        "SELECT l.content \
         FROM lyrics l \
         JOIN songs s ON s.lyrics_id = l.id \
         JOIN performance_songs ps ON ps.song_id = s.id \
         WHERE ps.performance_id = ? \
         ORDER BY ps.song_id \
         LIMIT 1",
    )
    .bind(performance_id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Deletes a performance by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<bool> {
    sqlx::query("DELETE FROM performances WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Atomically increments the play count for a performance.
pub async fn increment_play_count(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<()> {
    sqlx::query("UPDATE performances SET play_count = play_count + 1 WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Returns the songs linked to a performance via the `performance_songs` join table.
pub async fn get_songs(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
) -> Result<Vec<Song>> {
    sqlx::query_as::<_, Song>(
        "SELECT s.id, s.title, s.created_by, s.lyrics_id, s.date_added \
         FROM songs s \
         JOIN performance_songs ps ON ps.song_id = s.id \
         WHERE ps.performance_id = ?",
    )
    .bind(performance_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Replaces the full set of songs for a performance.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_songs(
    conn: &mut MySqlConnection,
    performance_id: Uuid,
    song_ids: &[Uuid],
) -> Result<()> {
    sqlx::query("DELETE FROM performance_songs WHERE performance_id = ?")
        .bind(performance_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for &song_id in song_ids {
        sqlx::query("INSERT INTO performance_songs (performance_id, song_id) VALUES (?, ?)")
            .bind(performance_id)
            .bind(song_id)
            .execute(&mut *conn)
            .await
            .map_err(DbError::from)?;
    }
    Ok(())
}

/// Returns the singers for a performance via the `performance_singers` join table.
pub async fn get_singers(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
) -> Result<Vec<Artist>> {
    sqlx::query_as::<_, Artist>(
        "SELECT a.id, a.name, a.description \
         FROM artists a \
         JOIN performance_singers ps ON ps.artist_id = a.id \
         WHERE ps.performance_id = ?",
    )
    .bind(performance_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Returns singers for multiple performances, keyed by performance ID.
///
/// Performances with no singers are absent from the returned map.
pub async fn get_singers_batch(
    executor: impl Executor<'_, Database = MySql>,
    performance_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<Artist>>> {
    if performance_ids.is_empty() {
        return Ok(HashMap::new());
    }

    #[derive(sqlx::FromRow)]
    struct Row {
        performance_id: Uuid,
        id: Uuid,
        name: String,
        description: Option<String>,
    }

    let mut builder = sqlx::QueryBuilder::new(
        "SELECT ps.performance_id, a.id, a.name, a.description \
         FROM artists a \
         JOIN performance_singers ps ON ps.artist_id = a.id \
         WHERE ps.performance_id IN (",
    );
    let mut separated = builder.separated(", ");
    for performance_id in performance_ids {
        separated.push_bind(performance_id);
    }
    builder.push(")");

    let rows: Vec<Row> = builder
        .build_query_as()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)?;

    let mut by_performance: HashMap<Uuid, Vec<Artist>> = HashMap::new();
    for row in rows {
        by_performance
            .entry(row.performance_id)
            .or_default()
            .push(Artist {
                id: row.id,
                name: row.name,
                description: row.description,
            });
    }
    Ok(by_performance)
}

/// Returns the tags for a performance with their kind from the `performance_tags` join table.
pub async fn get_tags(
    executor: impl Executor<'_, Database = MySql>,
    performance_id: Uuid,
) -> Result<Vec<TagWithKind>> {
    sqlx::query_as::<_, TagWithKind>(
        "SELECT t.id, t.name, pt.kind \
         FROM tags t \
         JOIN performance_tags pt ON pt.tag_id = t.id \
         WHERE pt.performance_id = ?",
    )
    .bind(performance_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Replaces the full set of tags for a performance.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_tags(
    conn: &mut MySqlConnection,
    performance_id: Uuid,
    tags: &[(Uuid, &str)],
) -> Result<()> {
    sqlx::query("DELETE FROM performance_tags WHERE performance_id = ?")
        .bind(performance_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for &(tag_id, kind) in tags {
        sqlx::query("INSERT INTO performance_tags (performance_id, tag_id, kind) VALUES (?, ?, ?)")
            .bind(performance_id)
            .bind(tag_id)
            .bind(kind)
            .execute(&mut *conn)
            .await
            .map_err(DbError::from)?;
    }
    Ok(())
}

/// Replaces the full set of singers for a performance.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_singers(
    conn: &mut MySqlConnection,
    performance_id: Uuid,
    artist_ids: &[Uuid],
) -> Result<()> {
    sqlx::query("DELETE FROM performance_singers WHERE performance_id = ?")
        .bind(performance_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for &artist_id in artist_ids {
        sqlx::query("INSERT INTO performance_singers (performance_id, artist_id) VALUES (?, ?)")
            .bind(performance_id)
            .bind(artist_id)
            .execute(&mut *conn)
            .await
            .map_err(DbError::from)?;
    }
    Ok(())
}
