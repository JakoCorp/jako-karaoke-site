//! Error type returned by backup operations.

/// Errors that can be returned while dumping, archiving or restoring.
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    /// A database query failed.
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// A filesystem operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// The zip archive could not be read or written.
    #[error("archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// The manifest could not be serialized or parsed.
    #[error("manifest error: {0}")]
    Manifest(#[from] serde_json::Error),

    /// The data does not have the layout a backup requires.
    #[error("invalid backup: {0}")]
    Invalid(String),
}
