//! Query functions for the `playlists` table and its `playlist_performances` join table.

use sqlx::{Executor, MySql, MySqlConnection, QueryBuilder};
use uuid::Uuid;

use crate::error::DbError;
use crate::filter::WhereClause;
use crate::models::playlist::{NewPlaylist, Playlist, PlaylistPerformanceRow, UpdatePlaylist};

type Result<T> = std::result::Result<T, DbError>;

/// Column list matching the fields of [`Playlist`], for a query over `playlists`.
///
/// The `inserted` form is for a freshly inserted row, which has no performances yet.
macro_rules! playlist_columns {
    () => {
        playlist_columns!(@with "(SELECT COUNT(*) FROM playlist_performances WHERE playlist_id = playlists.id)")
    };
    (inserted) => {
        playlist_columns!(@with "0")
    };
    (@with $performance_count:literal) => {
        concat!(
            "id, title, description, kind, is_public, created_by, ",
            $performance_count,
            " AS performance_count"
        )
    };
}

/// Fetches a playlist by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<Playlist>> {
    sqlx::query_as::<_, Playlist>(concat!(
        "SELECT ",
        playlist_columns!(),
        " FROM playlists WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Filters applied by [`search`] and [`search_count`]. Every set field must match.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlaylistFilter<'a> {
    /// Case insensitive substring matched against the playlist title.
    pub q: Option<&'a str>,
    /// Whether private playlists are included. Only public playlists are returned when `false`.
    pub include_private: bool,
    /// Only playlists owned by this user. Otherwise favorites playlists are left out, since they
    /// are personal and never part of a global listing.
    pub created_by: Option<Uuid>,
    /// Only playlists of this kind.
    pub kind: Option<&'a str>,
}

/// Appends the `WHERE` clause for `filter`, binding its values.
fn push_filters(builder: &mut QueryBuilder<MySql>, filter: PlaylistFilter<'_>) {
    let mut conditions = WhereClause::new(builder);

    if !filter.include_private {
        conditions.condition().push("is_public = TRUE");
    }

    if let Some(user_id) = filter.created_by {
        conditions
            .condition()
            .push("created_by = ")
            .push_bind(user_id);
    } else {
        conditions.condition().push("kind <> 'favorites'");
    }

    if let Some(kind) = filter.kind {
        conditions.condition().push("kind = ").push_bind(kind);
    }

    if let Some(q) = filter.q {
        conditions
            .condition()
            .push("title LIKE ")
            .push_bind(format!("%{q}%"));
    }
}

/// Returns a page of playlists matching `filter`, ordered by title.
pub async fn search(
    executor: impl Executor<'_, Database = MySql>,
    filter: PlaylistFilter<'_>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Playlist>> {
    let mut builder = QueryBuilder::new(concat!("SELECT ", playlist_columns!(), " FROM playlists"));
    push_filters(&mut builder, filter);
    builder
        .push(" ORDER BY title LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    builder
        .build_query_as::<Playlist>()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)
}

/// Returns the total number of playlists matching `filter`.
pub async fn search_count(
    executor: impl Executor<'_, Database = MySql>,
    filter: PlaylistFilter<'_>,
) -> Result<u64> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM playlists");
    push_filters(&mut builder, filter);

    builder
        .build_query_scalar::<i64>()
        .fetch_one(executor)
        .await
        .map(|n| n as u64)
        .map_err(DbError::from)
}

/// Fetches the favorites playlist for a user, returning `None` if it does not exist.
pub async fn get_favorites_by_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
) -> Result<Option<Playlist>> {
    sqlx::query_as::<_, Playlist>(concat!(
        "SELECT ",
        playlist_columns!(),
        " FROM playlists WHERE created_by = ? AND kind = 'favorites'"
    ))
    .bind(user_id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Counts the `user` kind playlists owned by `user_id`.
pub async fn count_user_playlists(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
) -> Result<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM playlists WHERE created_by = ? AND kind = 'user'")
        .bind(user_id)
        .fetch_one(executor)
        .await
        .map_err(DbError::from)
}

/// Inserts a new playlist and returns the created row.
pub async fn create(conn: &mut MySqlConnection, new: &NewPlaylist) -> Result<Playlist> {
    sqlx::query_as::<_, Playlist>(concat!(
        "INSERT INTO playlists (title, description, kind, is_public, created_by) \
         VALUES (?, ?, ?, ?, ?) RETURNING ",
        playlist_columns!(inserted)
    ))
    .bind(&new.title)
    .bind(&new.description)
    .bind(&new.kind)
    .bind(new.is_public)
    .bind(new.created_by)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Updates a playlist's mutable fields. Returns `None` if the ID does not exist.
pub async fn update(
    conn: &mut MySqlConnection,
    id: Uuid,
    upd: &UpdatePlaylist,
) -> Result<Option<Playlist>> {
    sqlx::query(
        "UPDATE playlists SET title = ?, description = ?, kind = ?, is_public = ? WHERE id = ?",
    )
    .bind(&upd.title)
    .bind(&upd.description)
    .bind(&upd.kind)
    .bind(upd.is_public)
    .bind(id)
    .execute(&mut *conn)
    .await
    .map_err(DbError::from)?;
    get_by_id(&mut *conn, id).await
}

/// Returns the title of a user's favorites playlist, which is always "{username}'s Favorites".
fn favorites_title(username: &str) -> String {
    format!("{username}'s Favorites")
}

/// Inserts favorites playlist for a new user.
///
/// Favorites playlists are private by default.
pub async fn create_favorites(
    conn: &mut MySqlConnection,
    user_id: Uuid,
    username: &str,
) -> Result<Playlist> {
    sqlx::query_as::<_, Playlist>(concat!(
        "INSERT INTO playlists (title, kind, is_public, created_by) \
         VALUES (?, 'favorites', FALSE, ?) RETURNING ",
        playlist_columns!(inserted)
    ))
    .bind(favorites_title(username))
    .bind(user_id)
    .fetch_one(conn)
    .await
    .map_err(DbError::from)
}

/// Retitles a user's favorites playlist after a username change.
pub async fn retitle_favorites(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
    username: &str,
) -> Result<()> {
    sqlx::query("UPDATE playlists SET title = ? WHERE created_by = ? AND kind = 'favorites'")
        .bind(favorites_title(username))
        .bind(user_id)
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}

/// Deletes a playlist by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<bool> {
    sqlx::query("DELETE FROM playlists WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}

/// Returns the IDs of playlists owned by `user_id` that contain `performance_id`.
pub async fn get_user_playlist_ids_containing_performance(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
    performance_id: Uuid,
) -> Result<Vec<Uuid>> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT playlist_id FROM playlist_performances \
         WHERE performance_id = ? \
         AND playlist_id IN (SELECT id FROM playlists WHERE created_by = ?)",
    )
    .bind(performance_id)
    .bind(user_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Returns performances in a playlist, ordered by `sort_order`.
pub async fn get_performances_in_playlist(
    executor: impl Executor<'_, Database = MySql>,
    playlist_id: Uuid,
) -> Result<Vec<PlaylistPerformanceRow>> {
    sqlx::query_as::<_, PlaylistPerformanceRow>(
        "SELECT p.id, p.created_by, p.title, p.lyrics_id, p.stream_time, \
         p.performance_date, p.stream_number, p.performance_number, pp.added_at \
         FROM performances p \
         JOIN playlist_performances pp ON pp.performance_id = p.id \
         WHERE pp.playlist_id = ? \
         ORDER BY pp.sort_order",
    )
    .bind(playlist_id)
    .fetch_all(executor)
    .await
    .map_err(DbError::from)
}

/// Appends multiple performances to the end of a playlist, skipping any already present.
///
/// `sort_order` increments from the current maximum.
pub async fn add_performances(
    conn: &mut MySqlConnection,
    playlist_id: Uuid,
    performance_ids: &[Uuid],
) -> Result<()> {
    if performance_ids.is_empty() {
        return Ok(());
    }
    let base: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order) + 1, 0) \
         FROM playlist_performances WHERE playlist_id = ?",
    )
    .bind(playlist_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(DbError::from)?;
    for (pos, &performance_id) in performance_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO playlist_performances (playlist_id, performance_id, sort_order) \
             VALUES (?, ?, ?) \
             ON DUPLICATE KEY UPDATE playlist_id = playlist_id",
        )
        .bind(playlist_id)
        .bind(performance_id)
        .bind(base + pos as i32)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    }
    Ok(())
}

/// Removes multiple performances from a playlist, ignoring any not present.
pub async fn remove_performances(
    executor: impl Executor<'_, Database = MySql>,
    playlist_id: Uuid,
    performance_ids: &[Uuid],
) -> Result<()> {
    if performance_ids.is_empty() {
        return Ok(());
    }
    let mut builder = QueryBuilder::new("DELETE FROM playlist_performances WHERE playlist_id = ");
    builder
        .push_bind(playlist_id)
        .push(" AND performance_id IN (");
    let mut separated = builder.separated(", ");
    for performance_id in performance_ids {
        separated.push_bind(performance_id);
    }
    builder.push(")");

    builder
        .build()
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}
