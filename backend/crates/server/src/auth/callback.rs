//! Shared tail of the provider callbacks: signs in an existing user or starts a signup.

use axum::response::Redirect;
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use cookie::{Cookie, SameSite};

use db::{models::User, queries};

use crate::{error::ApiError, state::AppState};

use super::session;

const PENDING_MINUTES: i64 = 15;

/// How a provider callback ended, reported to the frontend as the `auth` query parameter.
enum Outcome {
    SignedIn,
    ClaimRequired,
    Suspended,
}

impl Outcome {
    fn redirect(&self, frontend_url: &str) -> Redirect {
        let base = frontend_url.trim_end_matches('/');
        let url = match self {
            Self::SignedIn => format!("{base}/"),
            Self::ClaimRequired => format!("{base}/?auth=claim"),
            Self::Suspended => format!("{base}/?auth=suspended"),
        };
        Redirect::to(&url)
    }
}

/// A user a provider has authenticated.
pub(crate) struct ProviderIdentity<'a> {
    /// Provider name stored on the pending signup, `twitch` or `discord`.
    pub provider: &'static str,
    pub provider_id: u64,
    /// Handle at the provider, offered as the default username.
    pub username: &'a str,
    pub avatar_url: Option<&'a str>,
}

/// Finishes a provider callback by redirecting to the frontend with the outcome.
///
/// A suspended user gets no session. A new user gets a pending signup cookie.
pub(crate) async fn finish(
    state: &AppState,
    jar: CookieJar,
    existing: Option<User>,
    identity: ProviderIdentity<'_>,
) -> Result<(CookieJar, Redirect), ApiError> {
    let frontend_url = &state.config.frontend_url;

    if let Some(user) = existing {
        if user.suspended_at.is_some() {
            return Ok((jar, Outcome::Suspended.redirect(frontend_url)));
        }
        let session_token = session::issue(&state.pool, user.id).await?;
        return Ok((
            jar.add(session::session_cookie(session_token)),
            Outcome::SignedIn.redirect(frontend_url),
        ));
    }

    let pending_token = session::generate_token();
    let expires_at = Utc::now() + Duration::minutes(PENDING_MINUTES);
    queries::pending_oauth::create(
        &state.pool,
        &pending_token,
        identity.provider,
        identity.provider_id,
        identity.username,
        identity.avatar_url,
        expires_at,
    )
    .await?;

    let mut pending_cookie = Cookie::new("oauth_pending", pending_token);
    pending_cookie.set_http_only(true);
    pending_cookie.set_same_site(SameSite::Lax);
    pending_cookie.set_path("/");
    pending_cookie.set_secure(true);
    pending_cookie.set_max_age(time::Duration::minutes(PENDING_MINUTES));

    Ok((
        jar.add(pending_cookie),
        Outcome::ClaimRequired.redirect(frontend_url),
    ))
}
