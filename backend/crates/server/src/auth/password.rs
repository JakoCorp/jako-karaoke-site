//! Username and password authentication handlers.

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;

use api_types::auth::{LoginRequest, MeResponse, RegisterRequest};
use db::{error::DbError, models::NewUser, queries};

use crate::{error::ApiError, state::AppState};

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Registered, session cookie set", body = MeResponse),
        (status = 400, description = "Invalid username or password"),
        (status = 409, description = "Username already taken"),
    ),
    tag = "auth"
)]
pub(crate) async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<RegisterRequest>,
) -> Result<(CookieJar, (StatusCode, Json<MeResponse>)), ApiError> {
    validate_username(&req.username)?;
    validate_password(&req.password)?;

    let hash = hash_password(&req.password)?;

    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;

    let user = queries::users::create(
        &mut tx,
        &NewUser {
            username: req.username,
            twitch_id: None,
            discord_id: None,
        },
    )
    .await
    .map_err(|e| match e {
        db::error::DbError::Conflict => ApiError::Conflict("username already taken".into()),
        other => ApiError::from(other),
    })?;

    queries::user_credentials::create(&mut *tx, user.id, &hash).await?;
    queries::playlists::create_favorites(&mut tx, user.id).await?;

    tx.commit().await.map_err(DbError::Sqlx)?;

    let token = super::session::issue(&state.pool, user.id).await?;

    Ok((
        jar.add(super::session::session_cookie(token)),
        (
            StatusCode::CREATED,
            Json(MeResponse {
                id: user.id,
                username: user.username,
                capabilities: vec![],
            }),
        ),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Logged in, session cookie set", body = MeResponse),
        (status = 401, description = "Invalid credentials"),
    ),
    tag = "auth"
)]
pub(crate) async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<MeResponse>), ApiError> {
    let user = queries::users::get_by_username(&state.pool, &req.username)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    let cred = queries::user_credentials::get_by_user_id(&state.pool, user.id)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    verify_password(&req.password, &cred.password_hash)?;

    let token = super::session::issue(&state.pool, user.id).await?;
    let capabilities = queries::capabilities::list_for_user(&state.pool, user.id)
        .await?
        .into_iter()
        .map(|capability| capability.title)
        .collect();

    Ok((
        jar.add(super::session::session_cookie(token)),
        Json(MeResponse {
            id: user.id,
            username: user.username,
            capabilities,
        }),
    ))
}

/// Validates that a username meets length and character constraints.
///
/// Allowed characters: ASCII alphanumeric, `_`, `.`. Max length 64.
fn validate_username(username: &str) -> Result<(), ApiError> {
    if username.is_empty() || username.len() > 64 {
        return Err(ApiError::BadRequest(
            "username must be between 1 and 64 characters".into(),
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return Err(ApiError::BadRequest(
            "username may only contain letters, numbers, underscores, and periods".into(),
        ));
    }
    Ok(())
}

/// Validates that a password meets the minimum length requirement.
fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.len() < 8 {
        return Err(ApiError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| ApiError::Internal(e.to_string()))
}

fn verify_password(password: &str, hash: &str) -> Result<(), ApiError> {
    let parsed = PasswordHash::new(hash).map_err(|e| ApiError::Internal(e.to_string()))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| ApiError::Unauthorized)
}
