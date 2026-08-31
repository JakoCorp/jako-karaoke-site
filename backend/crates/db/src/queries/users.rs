//! Query functions for the `users` table.

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::error::DbError;
use crate::models::user::{NewUser, UpdateUser, User, UserSummary};

type Result<T> = std::result::Result<T, DbError>;

/// Fetches a user by primary key.
pub async fn get_by_id(
    executor: impl Executor<'_, Database = MySql>,
    id: Uuid,
) -> Result<Option<User>> {
    sqlx::query_as::<_, User>("SELECT id, username, twitch_id, discord_id FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(DbError::from)
}

/// Fetches a user by Twitch provider ID.
pub async fn get_by_twitch_id(
    executor: impl Executor<'_, Database = MySql>,
    twitch_id: u64,
) -> Result<Option<User>> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, twitch_id, discord_id FROM users WHERE twitch_id = ?",
    )
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
    sqlx::query_as::<_, User>(
        "SELECT id, username, twitch_id, discord_id FROM users WHERE discord_id = ?",
    )
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
    sqlx::query_as::<_, User>(
        "SELECT id, username, twitch_id, discord_id FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Returns all users ordered by ID.
pub async fn list(executor: impl Executor<'_, Database = MySql>) -> Result<Vec<User>> {
    sqlx::query_as::<_, User>("SELECT id, username, twitch_id, discord_id FROM users ORDER BY id")
        .fetch_all(executor)
        .await
        .map_err(DbError::from)
}

/// Searches users by optional username substring. Returns all users when `q` is `None`.
pub async fn search(
    executor: impl Executor<'_, Database = MySql>,
    q: Option<&str>,
) -> Result<Vec<UserSummary>> {
    match q {
        Some(term) => {
            let pattern = format!("%{term}%");
            sqlx::query_as::<_, UserSummary>(
                "SELECT id, username FROM users WHERE username LIKE ? ORDER BY username",
            )
            .bind(pattern)
            .fetch_all(executor)
            .await
            .map_err(DbError::from)
        }
        None => {
            sqlx::query_as::<_, UserSummary>("SELECT id, username FROM users ORDER BY username")
                .fetch_all(executor)
                .await
                .map_err(DbError::from)
        }
    }
}

/// Inserts a new user and returns the created row.
///
/// # Errors
///
/// Returns [`DbError::Conflict`] if the username is already taken.
pub async fn create(conn: &mut MySqlConnection, new: &NewUser) -> Result<User> {
    let result = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, twitch_id, discord_id) VALUES (?, ?, ?) \
         RETURNING id, username, twitch_id, discord_id",
    )
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

/// Inserts a user keyed on `twitch_id`, updating `username` on conflict.
///
/// Used on every successful Twitch OAuth login so the username stays in sync
/// with the user's current Twitch display name.
///
/// Returns the user and `true` if a new row was inserted, `false` if an existing
/// row was updated.
pub async fn upsert_by_twitch(conn: &mut MySqlConnection, new: &NewUser) -> Result<(User, bool)> {
    let result = sqlx::query(
        "INSERT INTO users (username, twitch_id, discord_id) VALUES (?, ?, ?) \
         ON DUPLICATE KEY UPDATE username = VALUES(username)",
    )
    .bind(&new.username)
    .bind(new.twitch_id)
    .bind(new.discord_id)
    .execute(&mut *conn)
    .await
    .map_err(DbError::from)?;
    let is_new = result.rows_affected() == 1;
    let user = get_by_twitch_id(&mut *conn, new.twitch_id.ok_or(DbError::NotFound)?)
        .await?
        .ok_or(DbError::NotFound)?;
    Ok((user, is_new))
}

/// Inserts a user keyed on `discord_id`, updating `username` on conflict.
///
/// Used on every successful Discord OAuth login so the username stays in sync
/// with the user's current Discord username.
///
/// Returns the user and `true` if a new row was inserted, `false` if an existing
/// row was updated.
pub async fn upsert_by_discord(conn: &mut MySqlConnection, new: &NewUser) -> Result<(User, bool)> {
    let result = sqlx::query(
        "INSERT INTO users (username, twitch_id, discord_id) VALUES (?, ?, ?) \
         ON DUPLICATE KEY UPDATE username = VALUES(username)",
    )
    .bind(&new.username)
    .bind(new.twitch_id)
    .bind(new.discord_id)
    .execute(&mut *conn)
    .await
    .map_err(DbError::from)?;
    let is_new = result.rows_affected() == 1;
    let user = get_by_discord_id(&mut *conn, new.discord_id.ok_or(DbError::NotFound)?)
        .await?
        .ok_or(DbError::NotFound)?;
    Ok((user, is_new))
}

/// Replaces all mutable fields on a user. Returns `None` if the ID does not exist.
pub async fn update(
    conn: &mut MySqlConnection,
    id: Uuid,
    upd: &UpdateUser,
) -> Result<Option<User>> {
    sqlx::query("UPDATE users SET username = ?, twitch_id = ?, discord_id = ? WHERE id = ?")
        .bind(&upd.username)
        .bind(upd.twitch_id)
        .bind(upd.discord_id)
        .bind(id)
        .execute(&mut *conn)
        .await
        .map_err(DbError::from)?;
    get_by_id(&mut *conn, id).await
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
