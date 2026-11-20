use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use backup::manifest::FORMAT_VERSION;
use backup::{BackupError, Manifest, MigrationRecord, archive};
use chrono::Utc;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn manifest() -> Manifest {
    Manifest {
        format_version: FORMAT_VERSION,
        created_at: Utc::now(),
        commit: "21948c4".to_string(),
        storage_path: "/data/storage".to_string(),
        storage_base_url: "https://api.example.com/files".to_string(),
        migrations: vec![MigrationRecord {
            version: 1,
            checksum: "ab".repeat(48),
        }],
        file_count: 0,
    }
}

fn write_file(path: &Path, contents: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn archive_round_trips_manifest_database_and_storage() {
    let dir = tempfile::tempdir().unwrap();
    let storage = dir.path().join("storage");
    let files: [(&str, &[u8]); 3] = [
        ("images/a.png", &[1, 2, 3]),
        ("audio/nested/b.mp3", b"audio"),
        ("empty.txt", b""),
    ];
    for (relative, contents) in files {
        write_file(&storage.join(relative), contents);
    }
    let database_sql = dir.path().join("database.sql");
    fs::write(&database_sql, "SET NAMES utf8mb4;\n\n").unwrap();
    let dest = dir.path().join("backup.zip");

    let written = archive::write(&dest, manifest(), &database_sql, &storage).unwrap();

    assert_eq!(written.file_count, 3);
    assert!(!dir.path().join("backup.zip.partial").exists());
    let read = archive::read_manifest(&dest).unwrap();
    assert_eq!(read.file_count, 3);
    assert_eq!(read.commit, "21948c4");
    assert_eq!(read.migrations, manifest().migrations);

    let restored_sql = dir.path().join("restored.sql");
    archive::extract_database(&dest, &restored_sql).unwrap();
    assert_eq!(
        fs::read(&restored_sql).unwrap(),
        fs::read(&database_sql).unwrap()
    );

    let target = dir.path().join("target");
    archive::extract_storage(&dest, &target).unwrap();
    for (relative, contents) in files {
        assert_eq!(fs::read(target.join(relative)).unwrap(), contents);
    }
}

#[test]
fn extract_storage_rejects_entries_outside_the_destination() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["../evil.txt", "storage/../../evil.txt"] {
        let dest = dir.path().join("malicious.zip");
        let mut zip = ZipWriter::new(File::create(&dest).unwrap());
        zip.start_file(name, SimpleFileOptions::default()).unwrap();
        zip.write_all(b"evil").unwrap();
        zip.finish().unwrap();

        let target = dir.path().join("target");
        let result = archive::extract_storage(&dest, &target);

        assert!(matches!(result, Err(BackupError::Invalid(_))), "{name}");
        assert!(!dir.path().join("evil.txt").exists());
        assert!(!target.join("evil.txt").exists());
    }
}
