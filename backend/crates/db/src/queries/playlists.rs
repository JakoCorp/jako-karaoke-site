//! Query functions for the `playlists` table and its `playlist_performances` join table.

use sqlx::{Executor, MySql, MySqlConnection, QueryBuilder};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::playlist::{NewPlaylist, Playlist, PlaylistPerformanceRow, UpdatePlaylist};

type Result<T> = std::result::Result<T, DbError>;

/// Fetches a playlist by ID.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<Playlist>> {
    sqlx::query_as::<_, Playlist>(
        "SELECT id, title, description, kind, is_public, created_by, \
         (SELECT COUNT(*) FROM playlist_performances WHERE playlist_id = playlists.id) AS performance_count \
         FROM playlists WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Returns a page of playlists matching the given filters.
///
/// When `include_private` is `false`, only public playlists are returned.
/// When `created_by` is `Some`, only playlists owned by that user are returned.
/// When `q` is `Some`, results are filtered by a case-insensitive substring match against the title.
pub async fn search(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
    include_private: bool,
    created_by: Option<Uuid>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Playlist>> {
    let mut builder = QueryBuilder::new(
        "SELECT id, title, description, kind, is_public, created_by, \
         (SELECT COUNT(*) FROM playlist_performances WHERE playlist_id = playlists.id) AS performance_count \
         FROM playlists",
    );
    let mut where_started = false;
    let mut push_where = |builder: &mut QueryBuilder<MySql>| {
        if where_started {
            builder.push(" AND");
        } else {
            builder.push(" WHERE");
            where_started = true;
        }
    };
    if !include_private {
        push_where(&mut builder);
        builder.push(" is_public = TRUE");
    }
    if let Some(user_id) = created_by {
        push_where(&mut builder);
        builder.push(" created_by = ");
        builder.push_bind(user_id);
    }
    if let Some(query) = q {
        push_where(&mut builder);
        builder.push(" title LIKE ");
        builder.push_bind(format!("%{query}%"));
    }
    builder.push(" ORDER BY title LIMIT ");
    builder.push_bind(limit);
    builder.push(" OFFSET ");
    builder.push_bind(offset);
    builder
        .build_query_as::<Playlist>()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)
}

/// Returns the total count of playlists matching the given filters.
///
/// Parameters have the same semantics as [`search`].
pub async fn search_count(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
    include_private: bool,
    created_by: Option<Uuid>,
) -> Result<i64> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM playlists");
    let mut where_started = false;
    let mut push_where = |builder: &mut QueryBuilder<MySql>| {
        if where_started {
            builder.push(" AND");
        } else {
            builder.push(" WHERE");
            where_started = true;
        }
    };
    if !include_private {
        push_where(&mut builder);
        builder.push(" is_public = TRUE");
    }
    if let Some(user_id) = created_by {
        push_where(&mut builder);
        builder.push(" created_by = ");
        builder.push_bind(user_id);
    }
    if let Some(query) = q {
        push_where(&mut builder);
        builder.push(" title LIKE ");
        builder.push_bind(format!("%{query}%"));
    }
    builder
        .build_query_scalar()
        .fetch_one(executor)
        .await
        .map_err(DbError::from)
}

/// Fetches the favorites playlist for a user, returning `None` if it does not exist.
pub async fn get_favorites_by_user(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
) -> Result<Option<Playlist>> {
    sqlx::query_as::<_, Playlist>(
        "SELECT id, title, description, kind, is_public, created_by, \
         (SELECT COUNT(*) FROM playlist_performances WHERE playlist_id = playlists.id) AS performance_count \
         FROM playlists WHERE created_by = ? AND kind = 'favorites'",
    )
    .bind(user_id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Inserts a new playlist and returns the created row.
pub async fn create(conn: &mut MySqlConnection, new: &NewPlaylist) -> Result<Playlist> {
    sqlx::query_as::<_, Playlist>(
        "INSERT INTO playlists (title, description, kind, is_public, created_by) \
         VALUES (?, ?, ?, ?, ?) \
         RETURNING id, title, description, kind, is_public, created_by, 0 AS performance_count",
    )
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

/// Inserts favorites playlist for a new user.
///
/// Favorites playlists are private by default.
pub async fn create_favorites(conn: &mut MySqlConnection, user_id: Uuid) -> Result<Playlist> {
    sqlx::query_as::<_, Playlist>(
        "INSERT INTO playlists (title, kind, is_public, created_by) \
         VALUES ('Favorites', 'favorites', FALSE, ?) \
         RETURNING id, title, description, kind, is_public, created_by, 0 AS performance_count",
    )
    .bind(user_id)
    .fetch_one(conn)
    .await
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

/// Returns performances in a playlist, ordered by `sort_order`.
pub async fn get_performances_in_playlist(
    executor: impl Executor<'_, Database = MySql>,
    playlist_id: Uuid,
) -> Result<Vec<PlaylistPerformanceRow>> {
    sqlx::query_as::<_, PlaylistPerformanceRow>(
        "SELECT p.id, p.created_by, p.title, p.lyrics_id, p.play_count, p.duration, p.stream_time, \
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

/// Replaces the full ordered set of performances in a playlist.
///
/// The position in `performance_ids` becomes the `sort_order` value.
/// Must be called within a caller provided transaction for atomicity.
pub async fn set_performances(
    conn: &mut MySqlConnection,
    playlist_id: Uuid,
    performance_ids: &[Uuid],
) -> Result<()> {
    sqlx::query("DELETE FROM playlist_performances WHERE playlist_id = ?")
        .bind(playlist_id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    for (pos, &performance_id) in performance_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO playlist_performances (playlist_id, performance_id, sort_order) \
             VALUES (?, ?, ?)",
        )
        .bind(playlist_id)
        .bind(performance_id)
        .bind(pos as u32)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    }
    Ok(())
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
    let placeholders = performance_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "DELETE FROM playlist_performances \
         WHERE playlist_id = ? AND performance_id IN ({placeholders})"
    );
    let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).bind(playlist_id);
    for &id in performance_ids {
        query = query.bind(id);
    }
    query
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(DbError::from)
}
