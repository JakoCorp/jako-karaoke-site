//! Query functions for the `soundbites` table and its artist join table.
//!
//! Artists are managed via the full replace helper `set_artists` that deletes existing rows
//! and reinserts.

use std::collections::HashMap;

use sqlx::{Executor, MySql, MySqlConnection, QueryBuilder};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::artist::Artist;
use crate::models::soundbite::{NewSoundbite, Soundbite, UpdateSoundbite};
use crate::search::{WhereClause, like_pattern, push_text_or_id};

type Result<T> = std::result::Result<T, DbError>;

/// Column list matching the fields of [`Soundbite`].
macro_rules! soundbite_columns {
    () => {
        "id, created_by, title, asset_id, soundbite_date, stream_number, soundbite_number"
    };
}

/// Newest first, with every tie broken chronologically in the same direction.
const ORDER_NEWEST_FIRST: &str =
    " ORDER BY soundbite_date DESC, stream_number DESC, soundbite_number DESC, id DESC";

/// Maps a unique index violation to [`DbError::Conflict`].
fn map_write_error(error: sqlx::Error) -> DbError {
    match error {
        sqlx::Error::Database(ref db) if db.is_unique_violation() => DbError::Conflict,
        other => DbError::Sqlx(other),
    }
}

/// Fetches a soundbite by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<Soundbite>> {
    sqlx::query_as::<_, Soundbite>(concat!(
        "SELECT ",
        soundbite_columns!(),
        " FROM soundbites WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Appends the `WHERE` clause for the optional query, binding its values. Appends nothing for `None`.
fn push_filters(builder: &mut QueryBuilder<MySql>, q: Option<&str>) {
    let mut conditions = WhereClause::new(builder);

    if let Some(q) = q {
        let builder = conditions.condition();
        builder.push("id IN (SELECT id FROM soundbites WHERE ");
        push_text_or_id(builder, "title", q);
        builder
            .push(
                " UNION SELECT sa.soundbite_id FROM soundbite_artists sa \
                 JOIN artists a ON a.id = sa.artist_id WHERE a.name LIKE ",
            )
            .push_bind(like_pattern(q))
            .push(")");
    }
}

/// Returns a page of soundbites, newest first.
///
/// When `q` is `Some`, results are filtered by a case insensitive substring match against the
/// title and any linked artist name, or by an exact match when `q` is the soundbite id.
pub async fn search(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Soundbite>> {
    let mut builder =
        QueryBuilder::new(concat!("SELECT ", soundbite_columns!(), " FROM soundbites"));
    push_filters(&mut builder, q);
    builder
        .push(ORDER_NEWEST_FIRST)
        .push(" LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    builder
        .build_query_as::<Soundbite>()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)
}

/// Returns the total number of soundbites matching the optional query.
pub async fn search_count(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
) -> Result<u64> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM soundbites");
    push_filters(&mut builder, q);

    builder
        .build_query_scalar::<i64>()
        .fetch_one(executor)
        .await
        .map(|n| n as u64)
        .map_err(DbError::from)
}

/// Inserts a new soundbite and returns the created row.
///
/// Returns [`DbError::Conflict`] if a soundbite with the same date, stream number and soundbite
/// number already exists.
pub async fn create(conn: &mut MySqlConnection, new: &NewSoundbite) -> Result<Soundbite> {
    sqlx::query_as::<_, Soundbite>(concat!(
        "INSERT INTO soundbites \
         (created_by, title, soundbite_date, stream_number, soundbite_number) \
         VALUES (?, ?, ?, ?, ?) RETURNING ",
        soundbite_columns!()
    ))
    .bind(new.created_by)
    .bind(&new.title)
    .bind(new.soundbite_date)
    .bind(new.stream_number)
    .bind(new.soundbite_number)
    .fetch_one(conn)
    .await
    .map_err(map_write_error)
}

/// Updates a soundbite's mutable fields. Returns `None` if the ID does not exist.
///
/// Returns [`DbError::Conflict`] if the new date, stream number and soundbite number are already
/// taken by another soundbite.
pub async fn update(
    conn: &mut MySqlConnection,
    id: Uuid,
    upd: &UpdateSoundbite,
) -> Result<Option<Soundbite>> {
    sqlx::query(
        "UPDATE soundbites \
         SET title = ?, soundbite_date = ?, stream_number = ?, soundbite_number = ? \
         WHERE id = ?",
    )
    .bind(&upd.title)
    .bind(upd.soundbite_date)
    .bind(upd.stream_number)
    .bind(upd.soundbite_number)
    .bind(id)
    .execute(&mut *conn)
    .await
    .map_err(map_write_error)?;
    get_by_id(&mut *conn, id).await
}

/// Sets the audio asset of a soundbite, replacing any previous one.
pub async fn set_asset(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
    asset_id: Uuid,
) -> Result<()> {
    sqlx::query("UPDATE soundbites SET asset_id = ? WHERE id = ?")
        .bind(asset_id)
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Deletes a soundbite by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<bool> {
    sqlx::query("DELETE FROM soundbites WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Returns artists for multiple soundbites, keyed by soundbite ID.
///
/// Soundbites with no artists are absent from the returned map.
pub async fn get_artists_batch(
    executor: impl Executor<'_, Database = MySql>,
    soundbite_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<Artist>>> {
    if soundbite_ids.is_empty() {
        return Ok(HashMap::new());
    }

    #[derive(sqlx::FromRow)]
    struct Row {
        soundbite_id: Uuid,
        id: Uuid,
        name: String,
        description: Option<String>,
    }

    let mut builder = QueryBuilder::new(
        "SELECT sa.soundbite_id, a.id, a.name, a.description \
         FROM artists a \
         JOIN soundbite_artists sa ON sa.artist_id = a.id \
         WHERE sa.soundbite_id IN (",
    );
    let mut separated = builder.separated(", ");
    for soundbite_id in soundbite_ids {
        separated.push_bind(soundbite_id);
    }
    builder.push(")");

    let rows: Vec<Row> = builder
        .build_query_as()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)?;

    let mut by_soundbite: HashMap<Uuid, Vec<Artist>> = HashMap::new();
    for row in rows {
        by_soundbite
            .entry(row.soundbite_id)
            .or_default()
            .push(Artist {
                id: row.id,
                name: row.name,
                description: row.description,
                song_count: 0,
            });
    }
    Ok(by_soundbite)
}

/// Replaces the full set of artists for a soundbite.
///
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_artists(
    conn: &mut MySqlConnection,
    soundbite_id: Uuid,
    artist_ids: &[Uuid],
) -> Result<()> {
    sqlx::query("DELETE FROM soundbite_artists WHERE soundbite_id = ?")
        .bind(soundbite_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for &artist_id in artist_ids {
        sqlx::query("INSERT INTO soundbite_artists (soundbite_id, artist_id) VALUES (?, ?)")
            .bind(soundbite_id)
            .bind(artist_id)
            .execute(&mut *conn)
            .await
            .map_err(DbError::from)?;
    }
    Ok(())
}
