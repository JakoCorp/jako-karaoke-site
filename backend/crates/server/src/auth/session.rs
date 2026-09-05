//! Server side session issuance, verification, and the `session` cookie.
//!
//! Sessions are opaque random tokens; only a SHA256 hash of the token is
//! stored in the `sessions` table (see [`db::queries::sessions`]).

use chrono::{Duration, Utc};
use cookie::{Cookie, SameSite};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use db::{MySqlPool, error::DbError, queries};

const SESSION_DAYS: i64 = 30;
const TOKEN_BYTES: usize = 32;
const SESSION_MAX_AGE: time::Duration = time::Duration::days(SESSION_DAYS);

/// Generates a cryptographically random 64-char hex token.
pub(crate) fn generate_token() -> String {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).expect("OS RNG failed");
    hex::encode(bytes)
}

/// Creates a new session for `user_id` and returns the raw token to store in the cookie.
pub async fn issue(pool: &MySqlPool, user_id: Uuid) -> Result<String, DbError> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).expect("OS RNG failed");
    let token = hex::encode(bytes);
    let expires_at = Utc::now() + Duration::days(SESSION_DAYS);
    queries::sessions::create(pool, &hash(&token), user_id, expires_at).await?;
    Ok(token)
}

/// Looks up the session for `token`, returning its user ID if present and unexpired.
pub async fn verify(pool: &MySqlPool, token: &str) -> Result<Option<Uuid>, DbError> {
    Ok(queries::sessions::get_valid(pool, &hash(token))
        .await?
        .map(|s| s.user_id))
}

/// Deletes the session for `token`, if any.
pub async fn revoke(pool: &MySqlPool, token: &str) -> Result<(), DbError> {
    queries::sessions::delete(pool, &hash(token)).await
}

fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// Builds the HTTP only `session` cookie carrying the given token.
///
/// `Max-Age` is set to [`SESSION_MAX_AGE`] so the cookie persists across browser
/// restarts. The server enforces the same lifetime via `sessions.expires_at`.
pub fn session_cookie(token: String) -> Cookie<'static> {
    let mut c = Cookie::new("session", token);
    c.set_http_only(true);
    c.set_same_site(SameSite::Lax);
    c.set_path("/");
    c.set_secure(true);
    c.set_max_age(SESSION_MAX_AGE);
    c
}
