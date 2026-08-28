//! Authentication routes: OAuth2 via Twitch and Discord.
//!
//! All auth methods converge on a single `users` table and issue the same
//! session cookie on success. See [`session`] for token issuance and storage,
//! and [`middleware`] for the `AuthUser` extractor used by protected handlers.

pub(crate) mod discord;
pub(crate) mod middleware;
pub(crate) mod session;
pub(crate) mod twitch;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use cookie::Cookie;

use api_types::{
    auth::{ClaimRequest, MeResponse},
    common::ErrorResponse,
};
use db::{error::DbError, models::NewUser, queries};

use crate::{error::ApiError, state::AppState};
use middleware::AuthUser;

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

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        pending_check,
        claim,
        me,
        logout,
        twitch::initiate,
        twitch::callback,
        discord::initiate,
        discord::callback,
    ),
    components(schemas(ClaimRequest, MeResponse, ErrorResponse,))
)]
pub(crate) struct AuthApi;

/// Builds the `/auth` subrouter.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/pending", get(pending_check))
        .route("/claim", post(claim))
        .route("/twitch", get(twitch::initiate))
        .route("/twitch/callback", get(twitch::callback))
        .route("/discord", get(discord::initiate))
        .route("/discord/callback", get(discord::callback))
        .route("/me", get(me))
        .route("/logout", post(logout))
}

#[utoipa::path(
    get,
    path = "/auth/pending",
    responses(
        (status = 204, description = "Valid pending signup session exists"),
        (status = 404, description = "No pending signup or session expired"),
    ),
    tag = "auth"
)]
async fn pending_check(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<StatusCode, ApiError> {
    let token = jar
        .get("oauth_pending")
        .map(|c| c.value().to_owned())
        .ok_or(ApiError::NotFound)?;

    queries::pending_oauth::get(&state.pool, &token)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/auth/claim",
    request_body = ClaimRequest,
    responses(
        (status = 200, description = "Account created, session cookie set", body = MeResponse),
        (status = 400, description = "Invalid username or no pending signup"),
        (status = 409, description = "Username already taken"),
    ),
    tag = "auth"
)]
async fn claim(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<ClaimRequest>,
) -> Result<(CookieJar, Json<MeResponse>), ApiError> {
    validate_username(&req.username)?;

    let pending_token = jar
        .get("oauth_pending")
        .map(|c| c.value().to_owned())
        .ok_or_else(|| ApiError::BadRequest("no pending signup".to_string()))?;

    let pending = queries::pending_oauth::get(&state.pool, &pending_token)
        .await?
        .ok_or_else(|| ApiError::BadRequest("signup session expired".to_string()))?;

    let (twitch_id, discord_id) = match pending.provider.as_str() {
        "twitch" => (Some(pending.provider_id), None),
        "discord" => (None, Some(pending.provider_id)),
        other => return Err(ApiError::Internal(format!("unknown provider: {other}"))),
    };

    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;

    let user = queries::users::create(
        &mut tx,
        &NewUser {
            username: req.username,
            twitch_id,
            discord_id,
        },
    )
    .await
    .map_err(|e| match e {
        DbError::Conflict => ApiError::Conflict("username already taken".into()),
        other => ApiError::from(other),
    })?;

    queries::playlists::create_favorites(&mut tx, user.id).await?;
    queries::pending_oauth::delete(&mut *tx, &pending_token).await?;

    tx.commit().await.map_err(DbError::Sqlx)?;

    let session_token = session::issue(&state.pool, user.id).await?;

    let capabilities = queries::capabilities::list_for_user(&state.pool, user.id)
        .await?
        .into_iter()
        .collect();

    let mut rm_pending = Cookie::new("oauth_pending", "");
    rm_pending.set_path("/");

    Ok((
        jar.remove(rm_pending)
            .add(session::session_cookie(session_token)),
        Json(MeResponse {
            id: user.id,
            username: user.username,
            capabilities,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Current user", body = MeResponse),
        (status = 401, description = "Not authenticated", body = ErrorResponse),
    ),
    tag = "auth"
)]
pub(crate) async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<MeResponse>, ApiError> {
    let user = queries::users::get_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(MeResponse {
        id: user.id,
        username: user.username,
        capabilities: auth.capabilities.into_iter().collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 204, description = "Logged out, session cookie cleared"),
    ),
    tag = "auth"
)]
pub(crate) async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), ApiError> {
    if let Some(token) = jar.get("session").map(|c| c.value().to_owned()) {
        session::revoke(&state.pool, &token).await?;
    }
    let mut removal = Cookie::new("session", "");
    removal.set_path("/");
    Ok((jar.remove(removal), StatusCode::NO_CONTENT))
}
