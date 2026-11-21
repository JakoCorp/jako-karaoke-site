//! Metadata stored alongside every backup.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::Result;

/// Archive layout version written by this crate.
pub const FORMAT_VERSION: u32 = 1;

/// Describes where a backup came from and what it contains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Archive layout version, see [`FORMAT_VERSION`].
    pub format_version: u32,
    /// When the backup was taken.
    pub created_at: DateTime<Utc>,
    /// Source commit of the build that took the backup.
    pub commit: String,
    /// Storage directory of the server the backup was taken from.
    pub storage_path: String,
    /// Public URL of that storage directory.
    pub storage_base_url: String,
    /// Migrations applied to the database at backup time, oldest first.
    pub migrations: Vec<MigrationRecord>,
    /// Number of stored files in the archive.
    pub file_count: u64,
}

/// One applied sqlx migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub version: i64,
    /// Lowercase hex of the migration checksum.
    pub checksum: String,
}

/// Reads the successfully applied migrations from the sqlx `_sqlx_migrations` table.
///
/// Returns an empty list when the table does not exist.
///
/// # Errors
///
/// Returns [`crate::BackupError::Sqlx`] if a query fails.
pub async fn applied_migrations(pool: &MySqlPool) -> Result<Vec<MigrationRecord>> {
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM information_schema.TABLES \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await?;
    if exists == 0 {
        return Ok(Vec::new());
    }

    let rows = sqlx::query_as::<_, (i64, String)>(
        "SELECT version, LOWER(HEX(checksum)) FROM _sqlx_migrations \
         WHERE success = 1 ORDER BY version",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(version, checksum)| MigrationRecord { version, checksum })
        .collect())
}
