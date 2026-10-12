//! Per user preferences for the authenticated user.

use axum::{Json, extract::State};

use api_types::{
    common::ErrorResponse,
    settings::{DownloadSettings, UpdateUserSettingsRequest, UserSettingsResponse},
};
use db::{error::DbError, models::UserSettings, queries};

use super::middleware::AuthUser;
use crate::{error::ApiError, state::AppState};

const MAX_FILENAME_TEMPLATE_CHARS: usize = 128;

fn settings_response(settings: UserSettings) -> UserSettingsResponse {
    UserSettingsResponse {
        download: DownloadSettings {
            filename_template: settings.download_filename_template,
            include_cover_art: settings.download_include_cover_art,
            include_lyrics: settings.download_include_lyrics,
            include_date: settings.download_include_date,
            include_singers: settings.download_include_singers,
            include_original_artists: settings.download_include_original_artists,
        },
    }
}

fn validate_filename_template(template: &str) -> Result<(), ApiError> {
    let length = template.chars().count();
    if template.trim().is_empty() || length > MAX_FILENAME_TEMPLATE_CHARS {
        return Err(ApiError::BadRequest(format!(
            "filename template must be between 1 and {MAX_FILENAME_TEMPLATE_CHARS} characters"
        )));
    }
    Ok(())
}

#[utoipa::path(
    get,
    path = "/auth/me/settings",
    responses(
        (status = 200, description = "Current settings, defaults when never saved", body = UserSettingsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "auth",
    security(("session" = []))
)]
pub(crate) async fn get_settings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserSettingsResponse>, ApiError> {
    let settings = queries::user_settings::get(&state.pool, auth.user_id)
        .await?
        .unwrap_or_default();

    Ok(Json(settings_response(settings)))
}

#[utoipa::path(
    put,
    path = "/auth/me/settings",
    request_body = UpdateUserSettingsRequest,
    responses(
        (status = 200, description = "Settings replaced", body = UserSettingsResponse),
        (status = 400, description = "Invalid filename template", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "auth",
    security(("session" = []))
)]
pub(crate) async fn update_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateUserSettingsRequest>,
) -> Result<Json<UserSettingsResponse>, ApiError> {
    validate_filename_template(&req.download.filename_template)?;

    let settings = UserSettings {
        download_filename_template: req.download.filename_template.trim().to_string(),
        download_include_cover_art: req.download.include_cover_art,
        download_include_lyrics: req.download.include_lyrics,
        download_include_date: req.download.include_date,
        download_include_singers: req.download.include_singers,
        download_include_original_artists: req.download.include_original_artists,
    };

    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    queries::user_settings::upsert(&mut conn, auth.user_id, &settings).await?;

    let stored = queries::user_settings::get(&state.pool, auth.user_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(settings_response(stored)))
}
