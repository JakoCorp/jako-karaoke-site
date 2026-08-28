//! Top level router composition and middleware stack.

pub(crate) mod artists;
pub(crate) mod capabilities;
pub(crate) mod performances;
pub(crate) mod playlists;
pub(crate) mod songs;
pub(crate) mod tags;
pub(crate) mod users;

use axum::{
    Router,
    http::{HeaderValue, Method, header::CONTENT_TYPE},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{auth, docs, state::AppState};

/// Assembles the full application router with CORS, tracing, and all subrouters.
pub fn build_router(state: AppState) -> Router {
    let cors_origin = state
        .config
        .frontend_url
        .parse::<HeaderValue>()
        .expect("FRONTEND_URL must be a valid HTTP origin");
    let cors = CorsLayer::new()
        .allow_origin(cors_origin)
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);
    Router::new()
        .nest("/api/artists", artists::router())
        .nest("/api/capabilities", capabilities::router())
        .nest("/api/performances", performances::router())
        .nest("/api/playlists", playlists::router())
        .nest("/api/songs", songs::router())
        .nest("/api/tags", tags::router())
        .nest("/api/users", users::router())
        .nest("/auth", auth::router())
        .merge(docs::router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
