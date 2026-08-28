//! Axum extractor for authenticated requests.

use std::collections::HashSet;

use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use db::queries;

use crate::{error::ApiError, state::AppState};

use super::session;

/// Axum extractor that requires a valid session cookie.
///
/// Add `auth: AuthUser` as a handler parameter to protect a route. Axum calls
/// [`FromRequestParts`] before the handler body runs; a missing, invalid, or
/// expired session cookie returns `401` before the handler is reached.
///
/// Capabilities are looked up fresh on every request.
pub struct AuthUser {
    pub user_id: Uuid,
    pub capabilities: HashSet<String>,
}

impl OptionalFromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
        let Some(token) = jar.get("session").map(|c| c.value().to_owned()) else {
            return Ok(None);
        };
        let Some(user_id) = session::verify(&state.pool, &token).await? else {
            return Ok(None);
        };
        let capabilities = queries::capabilities::list_for_user(&state.pool, user_id)
            .await?
            .into_iter()
            .collect();
        Ok(Some(AuthUser {
            user_id,
            capabilities,
        }))
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
        let token = jar
            .get("session")
            .map(|c| c.value().to_owned())
            .ok_or(ApiError::Unauthorized)?;
        let user_id = session::verify(&state.pool, &token)
            .await?
            .ok_or(ApiError::Unauthorized)?;
        let capabilities = queries::capabilities::list_for_user(&state.pool, user_id)
            .await?
            .into_iter()
            .collect();
        Ok(AuthUser {
            user_id,
            capabilities,
        })
    }
}
