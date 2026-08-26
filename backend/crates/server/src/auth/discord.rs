//! Discord OAuth2 Authorization Code flow handlers.

use axum::{
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::extract::CookieJar;
use cookie::{Cookie, SameSite};
use oauth2::{CsrfToken, Scope};
use serde::Deserialize;

use db::queries;

use crate::{error::ApiError, state::AppState};

const DISCORD_TOKEN_URL: &str = "https://discord.com/api/v10/oauth2/token";
const DISCORD_USERS_URL: &str = "https://discord.com/api/v10/users/@me";

const PENDING_MINUTES: i64 = 15;

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
}

#[utoipa::path(
    get,
    operation_id = "discord_initiate",
    path = "/auth/discord",
    responses(
        (status = 302, description = "Redirect to Discord OAuth, sets CSRF cookie"),
    ),
    tag = "auth"
)]
pub(crate) async fn initiate(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Redirect) {
    let (auth_url, csrf_token) = state
        .discord_oauth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .url();

    let mut csrf_cookie = Cookie::new("oauth_csrf_discord", csrf_token.secret().to_owned());
    csrf_cookie.set_http_only(true);
    csrf_cookie.set_same_site(SameSite::Lax);
    csrf_cookie.set_path("/");

    (jar.add(csrf_cookie), Redirect::to(auth_url.as_str()))
}

#[utoipa::path(
    get,
    operation_id = "discord_callback",
    path = "/auth/discord/callback",
    params(
        ("code" = String, Query, description = "Authorization code from Discord"),
        ("state" = String, Query, description = "CSRF state token"),
    ),
    responses(
        (status = 302, description = "Redirect to frontend on success"),
        (status = 400, description = "Missing or invalid OAuth state"),
    ),
    tag = "auth"
)]
pub(crate) async fn callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(params): Query<CallbackParams>,
) -> Result<(CookieJar, Redirect), ApiError> {
    let csrf_value = jar
        .get("oauth_csrf_discord")
        .map(|c| c.value().to_owned())
        .ok_or_else(|| ApiError::BadRequest("missing OAuth state".to_string()))?;
    if csrf_value != params.state {
        return Err(ApiError::BadRequest("invalid OAuth state".to_string()));
    }

    let redirect_uri = format!("{}/auth/discord/callback", state.config.base_url);
    let token: TokenResponse = state
        .http_client
        .post(DISCORD_TOKEN_URL)
        .form(&[
            ("client_id", state.config.discord_client_id.as_str()),
            ("client_secret", state.config.discord_client_secret.as_str()),
            ("code", params.code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .json()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let discord_user: DiscordUser = state
        .http_client
        .get(DISCORD_USERS_URL)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .json()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let discord_id: u64 = discord_user
        .id
        .parse()
        .map_err(|_| ApiError::Internal("invalid Discord user ID".to_string()))?;

    let mut rm_csrf = Cookie::new("oauth_csrf_discord", "");
    rm_csrf.set_path("/");

    if let Some(user) = queries::users::get_by_discord_id(&state.pool, discord_id).await? {
        let session_token = super::session::issue(&state.pool, user.id).await?;
        return Ok((
            jar.remove(rm_csrf)
                .add(super::session::session_cookie(session_token)),
            Redirect::to(&state.config.frontend_url),
        ));
    }

    let pending_token = super::session::generate_token();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(PENDING_MINUTES);
    queries::pending_oauth::create(
        &state.pool,
        &pending_token,
        "discord",
        discord_id,
        &discord_user.username,
        expires_at,
    )
    .await?;

    let mut pending_cookie = Cookie::new("oauth_pending", pending_token);
    pending_cookie.set_http_only(true);
    pending_cookie.set_same_site(SameSite::Lax);
    pending_cookie.set_path("/");
    pending_cookie.set_secure(true);
    pending_cookie.set_max_age(time::Duration::minutes(PENDING_MINUTES));

    let redirect_url = format!(
        "{}/?username={}",
        state.config.frontend_url.trim_end_matches('/'),
        discord_user.username
    );

    Ok((
        jar.remove(rm_csrf).add(pending_cookie),
        Redirect::to(&redirect_url),
    ))
}
