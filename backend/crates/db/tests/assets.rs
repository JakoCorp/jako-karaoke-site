use db::models::artist::NewArtist;
use db::models::asset::{NewExternalAsset, NewInternalAsset};
use db::queries::{artists, assets};
use sqlx::MySqlPool;

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_internal_and_get_by_id(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let new = NewInternalAsset {
        title: Some("cover.png".to_string()),
        credits: Some("artist_name".to_string()),
        source_url: Some("https://example.com/post/1".to_string()),
        hash: "a".repeat(64),
        storage_url: "https://cdn.example.com/cover.png".to_string(),
        internal_path: Some("/files/cover.png".to_string()),
    };
    let created = assets::create_internal(&mut conn, &new).await.unwrap();

    assert!(created.external_url.is_none());
    assert_eq!(created.hash.as_deref(), Some(&*"a".repeat(64)));
    assert_eq!(
        created.storage_url.as_deref(),
        Some("https://cdn.example.com/cover.png")
    );
    assert_eq!(created.title.as_deref(), Some("cover.png"));
    assert_eq!(created.credits.as_deref(), Some("artist_name"));

    let fetched = assets::get_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, created.id);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_external_and_get_by_id(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let new = NewExternalAsset {
        title: Some("VOD clip".to_string()),
        credits: None,
        source_url: None,
        external_url: "https://youtube.com/watch?v=abc".to_string(),
    };
    let created = assets::create_external(&mut conn, &new).await.unwrap();

    assert!(created.hash.is_none());
    assert!(created.storage_url.is_none());
    assert_eq!(
        created.external_url.as_deref(),
        Some("https://youtube.com/watch?v=abc")
    );

    let fetched = assets::get_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, created.id);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_by_hash_returns_existing(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let hash = "b".repeat(64);
    let created = assets::create_internal(
        &mut conn,
        &NewInternalAsset {
            title: None,
            credits: None,
            source_url: None,
            hash: hash.clone(),
            storage_url: "https://cdn.example.com/file.png".to_string(),
            internal_path: None,
        },
    )
    .await
    .unwrap();

    let found = assets::get_by_hash(&pool, &hash).await.unwrap().unwrap();
    assert_eq!(found.id, created.id);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_by_hash_returns_none_for_missing(pool: MySqlPool) {
    let result = assets::get_by_hash(&pool, &"c".repeat(64)).await.unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn reference_count_sums_all_join_tables(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let asset = assets::create_internal(
        &mut conn,
        &NewInternalAsset {
            title: None,
            credits: None,
            source_url: None,
            hash: "d".repeat(64),
            storage_url: "https://cdn.example.com/img.png".to_string(),
            internal_path: None,
        },
    )
    .await
    .unwrap();

    let artist = db::queries::artists::create(
        &mut conn,
        &NewArtist {
            name: "artist_ref".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    let count_before = assets::reference_count(&pool, asset.id).await.unwrap();
    assert_eq!(count_before, 0);

    let mut tx = pool.begin().await.unwrap();
    artists::set_images(&mut tx, artist.id, &[(asset.id, "avatar")])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let count_after = assets::reference_count(&pool, asset.id).await.unwrap();
    assert_eq!(count_after, 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_removes_asset(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let asset = assets::create_internal(
        &mut conn,
        &NewInternalAsset {
            title: None,
            credits: None,
            source_url: None,
            hash: "e".repeat(64),
            storage_url: "https://cdn.example.com/del.png".to_string(),
            internal_path: None,
        },
    )
    .await
    .unwrap();

    let deleted = assets::delete(&pool, asset.id).await.unwrap();
    assert!(deleted);

    let fetched = assets::get_by_id(&pool, asset.id).await.unwrap();
    assert!(fetched.is_none());
}
