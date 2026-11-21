//! Creating, listing and restoring whole server backups.
//!
//! Restoring replaces the database and the storage directory in full, so the server must be
//! stopped first.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use backup::dump::{self, DumpOptions};
use backup::manifest::{self, FORMAT_VERSION};
use backup::{Manifest, archive};
use chrono::{DateTime, Utc};
use db::{MySqlPool, error::DbError, queries};
use thiserror::Error;
use tokio::fs::{self, File};
use tokio::io::{BufReader, BufWriter};

use crate::config::Config;

/// Tables backed up without rows.
const SCHEMA_ONLY_TABLES: &[&str] = &["sessions", "pending_oauth"];
const FILE_PREFIX: &str = "jako-backup-";
const FILE_EXTENSION: &str = ".zip";
const SCHEDULED_SUFFIX: &str = "-scheduled";
const TIMESTAMP_FORMAT: &str = "%Y%m%dT%H%M%SZ";

/// Errors returned by backup operations.
#[derive(Debug, Error)]
pub enum BackupError {
    #[error(transparent)]
    Backup(#[from] backup::BackupError),
    #[error(transparent)]
    Database(#[from] DbError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// The backup cannot be restored by this build.
    #[error("{0}")]
    Incompatible(String),
    /// The configuration or the requested backup is not usable.
    #[error("{0}")]
    Invalid(String),
}

type Result<T> = std::result::Result<T, BackupError>;

/// How a backup was started, recorded in its file name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackupKind {
    /// Taken on request or before a restore. Never deleted automatically.
    Manual,
    /// Taken by the backup schedule and deleted once expired.
    Scheduled,
}

/// A backup archive in the backup directory.
pub struct BackupEntry {
    pub path: PathBuf,
    pub kind: BackupKind,
    /// Archive size in bytes.
    pub size: u64,
    /// `None` when the manifest cannot be read.
    pub manifest: Option<Manifest>,
}

impl BackupEntry {
    /// Whether this is a scheduled backup taken before `cutoff`.
    ///
    /// Backups with an unreadable manifest never expire.
    pub fn is_expired(&self, cutoff: DateTime<Utc>) -> bool {
        self.kind == BackupKind::Scheduled
            && self
                .manifest
                .as_ref()
                .is_some_and(|manifest| manifest.created_at < cutoff)
    }
}

/// Options for [`restore`].
pub struct RestoreOptions {
    /// Takes a backup of the current state before anything is replaced.
    pub safety_backup: bool,
}

/// Outcome of a [`restore`].
pub struct RestoreReport {
    /// Backup of the replaced state, unless it was skipped.
    pub safety_backup: Option<PathBuf>,
    /// Whether asset URLs and paths were moved to this server's storage location.
    pub rebased: bool,
    /// Internal paths of assets whose file is missing after the restore.
    pub missing_files: Vec<String>,
}

/// Writes a backup to the backup directory and returns its path.
///
/// Archives are named `jako-backup-<UTC timestamp>-<short commit>[-scheduled].zip`.
///
/// # Errors
///
/// Returns [`BackupError::Invalid`] if the backup directory is inside the storage directory,
/// and other variants if the dump or the archive fails.
pub async fn create(config: &Config, pool: &MySqlPool, kind: BackupKind) -> Result<PathBuf> {
    let backup_dir = PathBuf::from(&config.backup_path);
    if std::path::absolute(&backup_dir)?.starts_with(std::path::absolute(&config.storage_path)?) {
        return Err(BackupError::Invalid(
            "BACKUP_PATH must not be inside STORAGE_PATH".to_string(),
        ));
    }
    fs::create_dir_all(&backup_dir).await?;

    let created_at = Utc::now();
    let dest = backup_dir.join(file_name(created_at, &config.commit, kind));
    if fs::try_exists(&dest).await? {
        return Err(BackupError::Invalid(format!(
            "{} already exists",
            dest.display()
        )));
    }

    let database_sql =
        backup_dir.join(format!(".dump-{}.sql", created_at.format(TIMESTAMP_FORMAT)));
    let result = write_backup(config, pool, created_at, &dest, &database_sql).await;
    let _ = fs::remove_file(&database_sql).await;
    result.map(|()| dest)
}

/// Lists the backups in the backup directory, oldest first.
///
/// # Errors
///
/// Returns [`BackupError::Backup`] if the directory cannot be read.
pub async fn list(config: &Config) -> Result<Vec<BackupEntry>> {
    let backup_dir = PathBuf::from(&config.backup_path);
    blocking(move || list_dir(&backup_dir)).await
}

/// Deletes the scheduled backups taken before `cutoff` and returns their paths.
///
/// # Errors
///
/// Returns [`BackupError::Backup`] if the directory cannot be read and [`BackupError::Io`] if a
/// backup cannot be deleted.
pub async fn prune(config: &Config, cutoff: DateTime<Utc>) -> Result<Vec<PathBuf>> {
    let mut deleted = Vec::new();
    for entry in list(config).await? {
        if entry.is_expired(cutoff) {
            fs::remove_file(&entry.path).await?;
            deleted.push(entry.path);
        }
    }
    Ok(deleted)
}

/// Finds a backup by `latest`, a file name in the backup directory, or a path.
///
/// # Errors
///
/// Returns [`BackupError::Invalid`] if no matching backup exists.
pub async fn resolve(config: &Config, backup: &str) -> Result<PathBuf> {
    if backup == "latest" {
        return list(config)
            .await?
            .pop()
            .map(|entry| entry.path)
            .ok_or_else(|| BackupError::Invalid(format!("no backups in {}", config.backup_path)));
    }
    for path in [
        Path::new(&config.backup_path).join(backup),
        PathBuf::from(backup),
    ] {
        if fs::try_exists(&path).await? {
            return Ok(path);
        }
    }
    Err(BackupError::Invalid(format!("backup not found: {backup}")))
}

/// Reads the manifest of the archive at `archive_path` and checks this build can restore it.
///
/// # Errors
///
/// Returns [`BackupError::Incompatible`] if the archive format or a migration of the backup
/// differs from this build, and [`BackupError::Backup`] if the manifest cannot be read.
pub async fn inspect(archive_path: &Path) -> Result<Manifest> {
    let path = archive_path.to_path_buf();
    let manifest = blocking(move || archive::read_manifest(&path)).await?;

    if manifest.format_version != FORMAT_VERSION {
        return Err(BackupError::Incompatible(format!(
            "backup format version {} is not supported, this build reads version {FORMAT_VERSION}",
            manifest.format_version
        )));
    }
    let known: HashMap<i64, String> = db::MIGRATOR
        .iter()
        .filter(|migration| !migration.migration_type.is_down_migration())
        .map(|migration| (migration.version, hex::encode(&migration.checksum)))
        .collect();
    if let Some(record) = manifest
        .migrations
        .iter()
        .find(|record| known.get(&record.version) != Some(&record.checksum))
    {
        return Err(BackupError::Incompatible(format!(
            "migration {} of the backup does not match this build, restore it with commit {}",
            record.version, manifest.commit
        )));
    }
    Ok(manifest)
}

/// Replaces the database and the contents of the storage directory with a backup.
///
/// Assets are moved to this server's storage location when the backup came from another one.
///
/// # Errors
///
/// Returns the errors of [`inspect`] before anything is changed. Other errors can leave a
/// partial restore, in which case the safety backup holds the previous state.
pub async fn restore(
    config: &Config,
    pool: &MySqlPool,
    archive_path: &Path,
    options: &RestoreOptions,
) -> Result<RestoreReport> {
    let manifest = inspect(archive_path).await?;
    let safety_backup = if options.safety_backup {
        Some(create(config, pool, BackupKind::Manual).await?)
    } else {
        None
    };

    restore_database(config, pool, archive_path).await?;
    restore_storage(config, archive_path).await?;

    let rebased = !same_location(&manifest.storage_base_url, &config.storage_base_url)
        || !same_location(&manifest.storage_path, &config.storage_path);
    if rebased {
        let mut conn = pool.acquire().await.map_err(DbError::Sqlx)?;
        queries::assets::rebase_storage(
            &mut conn,
            &manifest.storage_base_url,
            &config.storage_base_url,
            &manifest.storage_path,
            &config.storage_path,
        )
        .await?;
    }

    let mut missing_files = Vec::new();
    for path in queries::assets::list_internal_paths(pool).await? {
        if !fs::try_exists(&path).await? {
            missing_files.push(path);
        }
    }
    Ok(RestoreReport {
        safety_backup,
        rebased,
        missing_files,
    })
}

async fn write_backup(
    config: &Config,
    pool: &MySqlPool,
    created_at: DateTime<Utc>,
    dest: &Path,
    database_sql: &Path,
) -> Result<()> {
    let mut writer = BufWriter::new(File::create(database_sql).await?);
    dump::dump(
        pool,
        &DumpOptions {
            skip_rows: SCHEMA_ONLY_TABLES,
        },
        &mut writer,
    )
    .await?;

    let manifest = Manifest {
        format_version: FORMAT_VERSION,
        created_at,
        commit: config.commit.clone(),
        storage_path: config.storage_path.clone(),
        storage_base_url: config.storage_base_url.clone(),
        migrations: manifest::applied_migrations(pool).await?,
        file_count: 0,
    };
    let dest = dest.to_path_buf();
    let database_sql = database_sql.to_path_buf();
    let storage_dir = PathBuf::from(&config.storage_path);
    blocking(move || archive::write(&dest, manifest, &database_sql, &storage_dir)).await?;
    Ok(())
}

async fn restore_database(config: &Config, pool: &MySqlPool, archive_path: &Path) -> Result<()> {
    fs::create_dir_all(&config.backup_path).await?;
    let database_sql = Path::new(&config.backup_path).join(format!(
        ".restore-{}.sql",
        Utc::now().format(TIMESTAMP_FORMAT)
    ));

    let result = load_database(pool, archive_path, &database_sql).await;
    let _ = fs::remove_file(&database_sql).await;
    result
}

async fn load_database(pool: &MySqlPool, archive_path: &Path, database_sql: &Path) -> Result<()> {
    let source = archive_path.to_path_buf();
    let dest = database_sql.to_path_buf();
    blocking(move || archive::extract_database(&source, &dest)).await?;
    let reader = BufReader::new(File::open(database_sql).await?);
    dump::restore(pool, reader).await?;
    Ok(())
}

async fn restore_storage(config: &Config, archive_path: &Path) -> Result<()> {
    let source = archive_path.to_path_buf();
    let storage_dir = PathBuf::from(&config.storage_path);
    blocking(move || {
        std::fs::create_dir_all(&storage_dir)?;
        // Only the contents are replaced, since the directory itself is usually a mount point.
        for entry in std::fs::read_dir(&storage_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                std::fs::remove_dir_all(entry.path())?;
            } else {
                std::fs::remove_file(entry.path())?;
            }
        }
        archive::extract_storage(&source, &storage_dir)
    })
    .await
}

fn list_dir(backup_dir: &Path) -> backup::Result<Vec<BackupEntry>> {
    let entries = match std::fs::read_dir(backup_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };

    let mut backups = Vec::new();
    for entry in entries {
        let entry = entry?;
        let Some(kind) = entry.file_name().to_str().and_then(backup_kind) else {
            continue;
        };
        let path = entry.path();
        backups.push(BackupEntry {
            kind,
            size: entry.metadata()?.len(),
            manifest: archive::read_manifest(&path).ok(),
            path,
        });
    }
    backups.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(backups)
}

/// Runs blocking archive work off the async runtime.
async fn blocking<T: Send + 'static>(
    task: impl FnOnce() -> backup::Result<T> + Send + 'static,
) -> Result<T> {
    let result = tokio::task::spawn_blocking(task)
        .await
        .map_err(std::io::Error::other)?;
    Ok(result?)
}

fn file_name(created_at: DateTime<Utc>, commit: &str, kind: BackupKind) -> String {
    let short_commit: String = commit
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(7)
        .collect();
    let suffix = match kind {
        BackupKind::Manual => "",
        BackupKind::Scheduled => SCHEDULED_SUFFIX,
    };
    format!(
        "{FILE_PREFIX}{}-{short_commit}{suffix}{FILE_EXTENSION}",
        created_at.format(TIMESTAMP_FORMAT)
    )
}

/// Returns the kind of a backup from its file name, or `None` if it is not a backup.
fn backup_kind(file_name: &str) -> Option<BackupKind> {
    let stem = file_name
        .strip_prefix(FILE_PREFIX)?
        .strip_suffix(FILE_EXTENSION)?;
    Some(if stem.ends_with(SCHEDULED_SUFFIX) {
        BackupKind::Scheduled
    } else {
        BackupKind::Manual
    })
}

fn same_location(left: &str, right: &str) -> bool {
    left.trim_end_matches('/') == right.trim_end_matches('/')
}
