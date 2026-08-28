//! Shared request and response types used by both the server and any future clients.
//!
//! All types derive [`utoipa::ToSchema`] so they can be referenced directly in
//! OpenAPI path annotations without redeclaration.

pub mod artists;
pub mod auth;
pub mod common;
pub mod lyrics;
pub mod pagination;
pub mod performances;
pub mod playlists;
pub mod songs;
pub mod tags;
pub mod users;
