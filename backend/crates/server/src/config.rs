//! Application configuration loaded from environment variables.

use std::env;

/// Runtime configuration for the server.
///
/// All fields are populated from environment variables by [`Config::from_env`].
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub storage_path: String,
    pub storage_base_url: String,
    pub twitch_client_id: String,
    pub twitch_client_secret: String,
    pub discord_client_id: String,
    pub discord_client_secret: String,
    /// Public base URL of this server (e.g. `http://localhost:3000`).
    /// Used to build OAuth callback URIs.
    pub base_url: String,
    /// Origin of the frontend app. Used for CORS and post OAuth redirects.
    pub frontend_url: String,
    /// Enables the `GET /auth/dev-login` endpoint for local development.
    ///
    /// Must never be enabled in production.
    pub dev_auth: bool,
}

/// Errors that can occur when loading [`Config`].
#[derive(Debug)]
pub enum ConfigError {
    Missing(&'static str),
    Invalid(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing(var) => write!(f, "missing required env var: {var}"),
            ConfigError::Invalid(var) => write!(f, "invalid value for env var: {var}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Reads all required environment variables and returns a [`Config`].
    ///
    /// `PORT` defaults to `3000` if unset. `FRONTEND_URL` defaults to `/`.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Missing`] for any required variable that is absent,
    /// or [`ConfigError::Invalid`] if `PORT` cannot be parsed as a `u16`.
    pub fn from_env() -> Result<Self, ConfigError> {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse::<u16>()
            .map_err(|_| ConfigError::Invalid("PORT"))?;

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL"))?,
            port,
            storage_path: env::var("STORAGE_PATH")
                .map_err(|_| ConfigError::Missing("STORAGE_PATH"))?,
            storage_base_url: env::var("STORAGE_BASE_URL")
                .map_err(|_| ConfigError::Missing("STORAGE_BASE_URL"))?,
            twitch_client_id: env::var("TWITCH_CLIENT_ID")
                .map_err(|_| ConfigError::Missing("TWITCH_CLIENT_ID"))?,
            twitch_client_secret: env::var("TWITCH_CLIENT_SECRET")
                .map_err(|_| ConfigError::Missing("TWITCH_CLIENT_SECRET"))?,
            discord_client_id: env::var("DISCORD_CLIENT_ID")
                .map_err(|_| ConfigError::Missing("DISCORD_CLIENT_ID"))?,
            discord_client_secret: env::var("DISCORD_CLIENT_SECRET")
                .map_err(|_| ConfigError::Missing("DISCORD_CLIENT_SECRET"))?,
            base_url: env::var("BASE_URL").map_err(|_| ConfigError::Missing("BASE_URL"))?,
            frontend_url: env::var("FRONTEND_URL").unwrap_or_else(|_| "/".into()),
            dev_auth: env::var("DEV_AUTH")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
        })
    }
}
