//! Application configuration loaded from environment variables.

use std::env;

use chrono::TimeDelta;
use croner::Cron;

const DEFAULT_PORT: u16 = 3000;
const DEFAULT_BACKUP_SCHEDULE: &str = "0 3 * * *";
const DEFAULT_BACKUP_KEEP_DAYS: u32 = 14;

/// Runtime configuration for the server.
///
/// All fields are populated from environment variables by [`Config::from_env`].
#[derive(Clone, Debug)]
pub struct Config {
    /// MySQL connection string.
    pub database_url: String,
    /// TCP port the HTTP server listens on. Defaults to 3000.
    pub port: u16,
    /// Absolute filesystem path where uploaded files are stored.
    pub storage_path: String,
    /// Public base URL prepended to stored file paths when building asset storage URLs.
    pub storage_base_url: String,
    /// Directory where backup archives are written. Defaults to `backups`.
    pub backup_path: String,
    /// Scheduled backups, `None` unless enabled.
    pub backup_schedule: Option<BackupSchedule>,
    /// Source commit of this build, recorded in backups. Defaults to `unknown`.
    pub commit: String,
    /// Twitch OAuth application client ID.
    pub twitch_client_id: String,
    /// Twitch OAuth application client secret.
    pub twitch_client_secret: String,
    /// Discord OAuth application client ID.
    pub discord_client_id: String,
    /// Discord OAuth application client secret.
    pub discord_client_secret: String,
    /// Google Cloud API key shared by Google API integrations.
    ///
    /// Integrations that need it are skipped when unset.
    pub google_api_key: Option<String>,
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

/// When scheduled backups run and how long they are kept.
#[derive(Clone, Debug)]
pub struct BackupSchedule {
    /// Cron expression evaluated in UTC.
    pub cron: Cron,
    /// Age after which scheduled backups are deleted, `None` keeps them all.
    pub retention: Option<TimeDelta>,
}

/// Errors that can occur when loading [`Config`].
#[derive(Debug)]
pub enum ConfigError {
    /// A required environment variable was absent.
    Missing(&'static str),
    /// An environment variable was present but its value could not be parsed.
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
    /// `PORT` defaults to `3000` if unset.
    /// `FRONTEND_URL` defaults to `/`.
    /// `BACKUP_PATH` defaults to `backups`.
    /// `BACKUP_SCHEDULE` defaults to `0 3 * * *` and `BACKUP_KEEP_DAYS` to `14`, and both are
    /// only read when `BACKUP_SCHEDULE_ENABLED` is set.
    /// `GIT_COMMIT` defaults to `unknown`.
    /// `GOOGLE_API_KEY` is optional.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Missing`] for any required variable that is absent,
    /// or [`ConfigError::Invalid`] if `PORT`, `BACKUP_SCHEDULE` or `BACKUP_KEEP_DAYS` cannot be
    /// parsed.
    pub fn from_env() -> Result<Self, ConfigError> {
        let port = match optional("PORT") {
            Some(port) => port
                .parse::<u16>()
                .map_err(|_| ConfigError::Invalid("PORT"))?,
            None => DEFAULT_PORT,
        };

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL"))?,
            port,
            storage_path: env::var("STORAGE_PATH")
                .map_err(|_| ConfigError::Missing("STORAGE_PATH"))?,
            storage_base_url: env::var("STORAGE_BASE_URL")
                .map_err(|_| ConfigError::Missing("STORAGE_BASE_URL"))?,
            backup_path: env::var("BACKUP_PATH").unwrap_or_else(|_| "backups".into()),
            backup_schedule: backup_schedule()?,
            commit: optional("GIT_COMMIT").unwrap_or_else(|| "unknown".into()),
            twitch_client_id: env::var("TWITCH_CLIENT_ID")
                .map_err(|_| ConfigError::Missing("TWITCH_CLIENT_ID"))?,
            twitch_client_secret: env::var("TWITCH_CLIENT_SECRET")
                .map_err(|_| ConfigError::Missing("TWITCH_CLIENT_SECRET"))?,
            discord_client_id: env::var("DISCORD_CLIENT_ID")
                .map_err(|_| ConfigError::Missing("DISCORD_CLIENT_ID"))?,
            discord_client_secret: env::var("DISCORD_CLIENT_SECRET")
                .map_err(|_| ConfigError::Missing("DISCORD_CLIENT_SECRET"))?,
            google_api_key: optional("GOOGLE_API_KEY"),
            base_url: env::var("BASE_URL").map_err(|_| ConfigError::Missing("BASE_URL"))?,
            frontend_url: env::var("FRONTEND_URL").unwrap_or_else(|_| "/".into()),
            dev_auth: flag("DEV_AUTH"),
        })
    }
}

fn backup_schedule() -> Result<Option<BackupSchedule>, ConfigError> {
    if !flag("BACKUP_SCHEDULE_ENABLED") {
        return Ok(None);
    }

    let cron = optional("BACKUP_SCHEDULE")
        .as_deref()
        .unwrap_or(DEFAULT_BACKUP_SCHEDULE)
        .parse::<Cron>()
        .map_err(|_| ConfigError::Invalid("BACKUP_SCHEDULE"))?;
    let keep_days = optional("BACKUP_KEEP_DAYS")
        .map_or(Ok(DEFAULT_BACKUP_KEEP_DAYS), |days| days.parse::<u32>())
        .map_err(|_| ConfigError::Invalid("BACKUP_KEEP_DAYS"))?;
    let retention = (keep_days > 0).then(|| TimeDelta::days(i64::from(keep_days)));
    Ok(Some(BackupSchedule { cron, retention }))
}

/// Reads a variable, treating an empty value as unset.
fn optional(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

/// Reads a boolean variable, which is on when set to `true` or `1`.
fn flag(name: &str) -> bool {
    env::var(name).is_ok_and(|value| value == "true" || value == "1")
}
