//! Query functions for the `user_settings` table.

use sqlx::{Executor, MySql, MySqlConnection};
use uuid::Uuid;

use crate::{error::DbError, models::UserSettings};

type Result<T> = std::result::Result<T, DbError>;

/// Returns a user's stored settings, or `None` if they have never saved any.
pub async fn get(
    executor: impl Executor<'_, Database = MySql>,
    user_id: Uuid,
) -> Result<Option<UserSettings>> {
    sqlx::query_as::<_, UserSettings>(
        "SELECT download_filename_template, download_include_cover_art, \
                download_include_lyrics, download_include_date, \
                download_include_singers, download_include_original_artists \
         FROM user_settings WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::from)
}

/// Stores a user's settings, replacing any existing row.
pub async fn upsert(
    conn: &mut MySqlConnection,
    user_id: Uuid,
    settings: &UserSettings,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_settings (user_id, download_filename_template, \
                download_include_cover_art, download_include_lyrics, \
                download_include_date, download_include_singers, \
                download_include_original_artists) \
         VALUES (?, ?, ?, ?, ?, ?, ?) \
         ON DUPLICATE KEY UPDATE \
                download_filename_template = VALUES(download_filename_template), \
                download_include_cover_art = VALUES(download_include_cover_art), \
                download_include_lyrics = VALUES(download_include_lyrics), \
                download_include_date = VALUES(download_include_date), \
                download_include_singers = VALUES(download_include_singers), \
                download_include_original_artists = VALUES(download_include_original_artists)",
    )
    .bind(user_id)
    .bind(&settings.download_filename_template)
    .bind(settings.download_include_cover_art)
    .bind(settings.download_include_lyrics)
    .bind(settings.download_include_date)
    .bind(settings.download_include_singers)
    .bind(settings.download_include_original_artists)
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(DbError::from)
}
