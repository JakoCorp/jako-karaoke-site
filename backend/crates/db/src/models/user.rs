//! User model.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user record fetched from the database.
///
/// All auth methods (password, Twitch, Discord) share this single row.
/// A `None` provider ID means the user has not linked that provider.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub twitch_id: Option<u64>,
    pub discord_id: Option<u64>,
}

/// Input for creating a new user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub twitch_id: Option<u64>,
    pub discord_id: Option<u64>,
}

/// A minimal user projection used for search results.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
}

/// Input for replacing a user's mutable fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub username: String,
    pub twitch_id: Option<u64>,
    pub discord_id: Option<u64>,
}
