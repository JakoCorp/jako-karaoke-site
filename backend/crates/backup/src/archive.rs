//! Zip archive holding a manifest, a database dump and a copy of a storage directory.

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{BackupError, Manifest, Result};

/// Archive entry holding the [`Manifest`] as JSON.
pub const MANIFEST_ENTRY: &str = "manifest.json";
/// Archive entry holding the SQL dump.
pub const DATABASE_ENTRY: &str = "database.sql";
/// Archive directory holding the stored files.
pub const STORAGE_DIR: &str = "storage";

/// Writes a backup archive to `dest` and returns the manifest with `file_count` filled in.
///
/// The archive is written to a `.partial` file first and renamed once complete.
///
/// # Errors
///
/// Returns [`BackupError::Invalid`] if a stored file name is not valid UTF-8, and other
/// variants if writing fails.
pub fn write(
    dest: &Path,
    mut manifest: Manifest,
    database_sql: &Path,
    storage_dir: &Path,
) -> Result<Manifest> {
    let mut partial = OsString::from(dest.as_os_str());
    partial.push(".partial");
    let partial = PathBuf::from(partial);

    match write_to(&partial, &mut manifest, database_sql, storage_dir) {
        Ok(()) => {
            fs::rename(&partial, dest)?;
            Ok(manifest)
        }
        Err(error) => {
            let _ = fs::remove_file(&partial);
            Err(error)
        }
    }
}

/// Reads the manifest of the archive at `path`.
///
/// # Errors
///
/// Returns [`BackupError::Zip`] if the file is not a zip archive or has no manifest, and
/// [`BackupError::Manifest`] if the manifest cannot be parsed.
pub fn read_manifest(path: &Path) -> Result<Manifest> {
    let mut archive = open(path)?;
    let entry = archive.by_name(MANIFEST_ENTRY)?;
    Ok(serde_json::from_reader(entry)?)
}

/// Copies the SQL dump of the archive at `path` to the file `dest`.
///
/// # Errors
///
/// Returns [`BackupError::Zip`] if the archive cannot be read and [`BackupError::Io`] if
/// writing fails.
pub fn extract_database(path: &Path, dest: &Path) -> Result<()> {
    let mut archive = open(path)?;
    let mut entry = archive.by_name(DATABASE_ENTRY)?;
    let mut output = BufWriter::new(File::create(dest)?);
    io::copy(&mut entry, &mut output)?;
    output.flush()?;
    Ok(())
}

/// Extracts the stored files of the archive at `path` into `dest_dir`.
///
/// # Errors
///
/// Returns [`BackupError::Invalid`] if an entry path would escape `dest_dir`, and other
/// variants if reading or writing fails.
pub fn extract_storage(path: &Path, dest_dir: &Path) -> Result<()> {
    let mut archive = open(path)?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(name) = entry.enclosed_name() else {
            return Err(BackupError::Invalid(format!(
                "unsafe entry path: {}",
                entry.name_raw().escape_ascii()
            )));
        };
        let Ok(relative) = name.strip_prefix(STORAGE_DIR) else {
            continue;
        };
        if entry.is_dir() || relative.as_os_str().is_empty() {
            continue;
        }

        let target = dest_dir.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = BufWriter::new(File::create(&target)?);
        io::copy(&mut entry, &mut output)?;
        output.flush()?;
    }
    Ok(())
}

fn open(path: &Path) -> Result<ZipArchive<BufReader<File>>> {
    Ok(ZipArchive::new(BufReader::new(File::open(path)?))?)
}

fn write_to(
    path: &Path,
    manifest: &mut Manifest,
    database_sql: &Path,
    storage_dir: &Path,
) -> Result<()> {
    let mut zip = ZipWriter::new(BufWriter::new(File::create(path)?));
    let deflated = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .large_file(true);
    // Stored files are mostly audio and images, which are already compressed.
    let stored = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .large_file(true);

    zip.start_file(DATABASE_ENTRY, deflated)?;
    io::copy(&mut File::open(database_sql)?, &mut zip)?;

    let mut files = Vec::new();
    collect_files(storage_dir, Path::new(""), &mut files)?;
    for relative in &files {
        zip.start_file(format!("{STORAGE_DIR}/{}", entry_name(relative)?), stored)?;
        io::copy(&mut File::open(storage_dir.join(relative))?, &mut zip)?;
    }

    manifest.file_count = files.len() as u64;
    zip.start_file(MANIFEST_ENTRY, deflated)?;
    serde_json::to_writer_pretty(&mut zip, manifest)?;

    let file = zip
        .finish()?
        .into_inner()
        .map_err(|error| error.into_error())?;
    file.sync_all()?;
    Ok(())
}

/// Appends the paths of all regular files under `root/relative`, relative to `root`, in name order.
fn collect_files(root: &Path, relative: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(root.join(relative))?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry.file_type()?;
        let path = relative.join(entry.file_name());
        if file_type.is_dir() {
            collect_files(root, &path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn entry_name(relative: &Path) -> Result<String> {
    let parts = relative
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| {
            BackupError::Invalid(format!("file name is not UTF-8: {}", relative.display()))
        })?;
    Ok(parts.join("/"))
}
