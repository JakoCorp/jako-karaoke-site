use std::path::PathBuf;

use backup::Manifest;
use backup::dump::{self, DumpOptions};
use backup::manifest::{self, FORMAT_VERSION};
use chrono::{DateTime, TimeDelta, Utc};
use db::models::asset::NewInternalAsset;
use db::queries::assets;
use server::backups::{BackupEntry, BackupKind};
use sqlx::MySqlPool;

fn internal_asset(hash_char: &str) -> NewInternalAsset {
    NewInternalAsset {
        title: None,
        credits: None,
        source_url: None,
        hash: hash_char.repeat(64),
        storage_url: format!("https://cdn.example.com/{hash_char}.png"),
        internal_path: Some(format!("/data/storage/images/{hash_char}.png")),
        length: Some(212),
    }
}

fn entry(kind: BackupKind, created_at: Option<DateTime<Utc>>) -> BackupEntry {
    BackupEntry {
        path: PathBuf::from("backup.zip"),
        kind,
        size: 0,
        manifest: created_at.map(|created_at| Manifest {
            format_version: FORMAT_VERSION,
            created_at,
            commit: "unknown".to_string(),
            storage_path: "/data/storage".to_string(),
            storage_base_url: "https://cdn.example.com".to_string(),
            migrations: Vec::new(),
            file_count: 0,
        }),
    }
}

#[test]
fn scheduled_backups_expire_after_the_cutoff() {
    let cutoff = Utc::now();
    let before = Some(cutoff - TimeDelta::days(1));
    let after = Some(cutoff + TimeDelta::days(1));

    assert!(entry(BackupKind::Scheduled, before).is_expired(cutoff));
    assert!(!entry(BackupKind::Scheduled, after).is_expired(cutoff));
    assert!(!entry(BackupKind::Manual, before).is_expired(cutoff));
    assert!(!entry(BackupKind::Scheduled, None).is_expired(cutoff));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn application_schema_round_trips_through_a_dump(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let asset = assets::create_internal(&mut conn, &internal_asset("a"))
        .await
        .unwrap();
    drop(conn);
    let migrations = manifest::applied_migrations(&pool).await.unwrap();
    assert!(!migrations.is_empty());

    let mut output = Vec::new();
    dump::dump(&pool, &DumpOptions::default(), &mut output)
        .await
        .unwrap();
    dump::restore(&pool, output.as_slice()).await.unwrap();

    assert_eq!(
        manifest::applied_migrations(&pool).await.unwrap(),
        migrations
    );
    let restored = assets::get_by_id(&pool, asset.id).await.unwrap().unwrap();
    assert_eq!(restored.hash, asset.hash);
    assert_eq!(restored.internal_path, asset.internal_path);
    assert_eq!(restored.length, Some(212));
    let mut conn = pool.acquire().await.unwrap();
    assets::create_internal(&mut conn, &internal_asset("b"))
        .await
        .unwrap();
}
