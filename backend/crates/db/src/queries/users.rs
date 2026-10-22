//! Query functions for the `users` table.

use sqlx::{Executor, MySql, MySqlConnection, QueryBuilder};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::user::{NewUser, User, UserSummary};
use crate::search::{WhereClause, push_text_or_id};

type Result<T> = std::result::Result<T, DbError>;

/// Column list matching the fields of [`User`].
macro_rules! user_columns {
    () => {
        "id, username, twitch_id, discord_id, username_changed_at"
    };
}

/// Fetches a user by primary key.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(concat!(
        "SELECT ",
        user_columns!(),
        " FROM users WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Fetches a user by primary key and locks the row until the transaction ends.
pub async fn get_by_id_for_update(conn: &mut MySqlConnection, id: Uuid) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(concat!(
        "SELECT ",
        user_columns!(),
        " FROM users WHERE id = ? FOR UPDATE"
    ))
    .bind(id)
    .fetch_optional(conn)
    .await
    .map_err(DbError::from)
}

/// Fetches a user by Twitch provider ID.
pub async fn get_by_twitch_id(
    executor: impl Executor<'_, Database = MySql>,
    twitch_id: u64,
) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(concat!(
        "SELECT ",
        user_columns!(),
        " FROM users WHERE twitch_id = ?"
    ))
    .bind(twitch_id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Fetches a user by Discord provider ID.
pub async fn get_by_discord_id(
    executor: impl Executor<'_, Database = MySql>,
    discord_id: u64,
) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(concat!(
        "SELECT ",
        user_columns!(),
        " FROM users WHERE discord_id = ?"
    ))
    .bind(discord_id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Fetches a user by username (case insensitive via DB collation).
pub async fn get_by_username(
    executor: impl Executor<'_, Database = MySql>,
    username: &str,
) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(concat!(
        "SELECT ",
        user_columns!(),
        " FROM users WHERE username = ?"
    ))
    .bind(username)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Appends the `WHERE` clause for the optional query, binding its values. Appends nothing for `None`.
fn push_filters(builder: &mut QueryBuilder<MySql>, q: Option<&str>) {
    let mut conditions = WhereClause::new(builder);

    if let Some(q) = q {
        let builder = conditions.condition();
        builder.push("(");
        push_text_or_id(builder, "username", q);
        builder.push(")");
    }
}

/// Returns a page of users ordered by username.
///
/// When `q` is `None`, all users are returned. When `q` is `Some`, results are filtered by a
/// case-insensitive substring match against the username, or by an exact match when `q` is the
/// user id.
pub async fn search(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<UserSummary>> {
    let mut builder = QueryBuilder::new("SELECT id, username FROM users");
    push_filters(&mut builder, q);
    builder
        .push(" ORDER BY username LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    builder
        .build_query_as::<UserSummary>()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)
}

/// Returns the total number of users matching the optional query.
pub async fn search_count(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
) -> Result<u64> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM users");
    push_filters(&mut builder, q);

    builder
        .build_query_scalar::<i64>()
        .fetch_one(executor)
        .await
        .map(|n| n as u64)
        .map_err(DbError::from)
}

/// Inserts a new user and returns the created row.
///
/// # Errors
///
/// Returns [`DbError::Conflict`] if the username is already taken.
pub async fn create(conn: &mut MySqlConnection, new: &NewUser) -> Result<User> {
    let result = sqlx::query_as::<_, User>(concat!(
        "INSERT INTO users (username, twitch_id, discord_id) VALUES (?, ?, ?) RETURNING ",
        user_columns!()
    ))
    .bind(&new.username)
    .bind(new.twitch_id)
    .bind(new.discord_id)
    .fetch_one(conn)
    .await;

    match result {
        Ok(user) => Ok(user),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Err(DbError::Conflict),
        Err(e) => Err(DbError::Sqlx(e)),
    }
}

/// Changes a user's username and records the change time, then returns the updated row.
///
/// # Errors
///
/// Returns [`DbError::Conflict`] if the username is already taken, and
/// [`DbError::NotFound`] if the user does not exist.
pub async fn update_username(conn: &mut MySqlConnection, id: Uuid, username: &str) -> Result<User> {
    let result = sqlx::query(
        "UPDATE users SET username = ?, username_changed_at = UTC_TIMESTAMP() WHERE id = ?",
    )
    .bind(username)
    .bind(id)
    .execute(&mut *conn)
    .await;

    match result {
        Ok(_) => get_by_id(&mut *conn, id).await?.ok_or(DbError::NotFound),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Err(DbError::Conflict),
        Err(e) => Err(DbError::Sqlx(e)),
    }
}

/// Deletes a user by ID. Returns `true` if a row was deleted.
pub async fn delete(executor: impl Executor<'_, Database = MySql>, id: Uuid) -> Result<bool> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await
        .map(|r| r.rows_affected() > 0)
        .map_err(DbError::from)
}
