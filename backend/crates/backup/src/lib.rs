//! Whole server backups: a logical MariaDB dump and a storage directory packed into one zip archive.

pub mod archive;
pub mod dump;
pub mod error;
pub mod manifest;

pub use error::BackupError;
pub use manifest::{Manifest, MigrationRecord};

pub type Result<T> = std::result::Result<T, BackupError>;
