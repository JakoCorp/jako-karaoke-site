//! Shared application state passed to every route handler.

use std::sync::Arc;

use db::MySqlPool;
use oauth2::{EndpointNotSet, EndpointSet, basic::BasicClient};

use crate::config::Config;
use crate::storage::FileStore;

/// OAuth2 client with both auth URI and token URI configured.
pub(crate) type OAuthClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

/// Shared state injected into all Axum route handlers via [`axum::extract::State`].
///
/// The struct is [`Clone`] so Axum can hand a copy to each request; heavy
/// resources are wrapped in [`Arc`] to keep cloning cheap.
#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub store: FileStore,
    pub config: Config,
    pub twitch_oauth: Arc<OAuthClient>,
    pub discord_oauth: Arc<OAuthClient>,
    pub http_client: reqwest::Client,
}
