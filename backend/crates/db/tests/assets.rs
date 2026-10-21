use db::models::artist::NewArtist;
use db::models::asset::{NewExternalAsset, NewInternalAsset};
use db::queries::{artists, assets};
use sqlx::MySqlPool;

fn internal_asset(hash_char: &str) -> NewInternalAsset {
    NewInternalAsset {
        title: None,
        credits: None,
        source_url: None,
        hash: hash_char.repeat(64),
        storage_url: format!("https://cdn.example.com/{hash_char}.png"),
        internal_path: None,
        length: None,
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn internal_and_external_assets_roundtrip(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();

    let internal = assets::create_internal(
        &mut conn,
        &NewInternalAsset {
            title: Some("cover.png".to_string()),
            credits: Some("artist_name".to_string()),
            source_url: Some("https://example.com/post/1".to_string()),
            internal_path: Some("/files/cover.png".to_string()),
            ..internal_asset("a")
        },
    )
    .await
    .unwrap();
    assert!(internal.external_url.is_none());
    assert_eq!(internal.hash.as_deref(), Some(&*"a".repeat(64)));
    assert_eq!(
        internal.storage_url.as_deref(),
        Some("https://cdn.example.com/a.png")
    );
    assert_eq!(internal.title.as_deref(), Some("cover.png"));
    assert_eq!(internal.credits.as_deref(), Some("artist_name"));

    let by_hash = assets::get_by_hash(&pool, &"a".repeat(64))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(by_hash.id, internal.id);

    let external = assets::create_external(
        &mut conn,
        &NewExternalAsset {
            title: Some("VOD clip".to_string()),
            credits: None,
            source_url: None,
            external_url: "https://youtube.com/watch?v=abc".to_string(),
            length: Some(212),
        },
    )
    .await
    .unwrap();
    assert!(external.hash.is_none());
    assert!(external.storage_url.is_none());
    assert_eq!(
        external.external_url.as_deref(),
        Some("https://youtube.com/watch?v=abc")
    );
    assert_eq!(external.length, Some(212));

    let fetched = assets::get_by_id(&pool, external.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.id, external.id);

    assert!(assets::delete(&pool, internal.id).await.unwrap());
    assert!(
        assets::get_by_id(&pool, internal.id)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn reference_count_follows_join_rows(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let asset = assets::create_internal(&mut conn, &internal_asset("b"))
        .await
        .unwrap();
    let artist = artists::create(
        &mut conn,
        &NewArtist {
            name: "artist_a".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(assets::reference_count(&pool, asset.id).await.unwrap(), 0);

    let mut tx = pool.begin().await.unwrap();
    artists::set_images(&mut tx, artist.id, &[(asset.id, "avatar")])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(assets::reference_count(&pool, asset.id).await.unwrap(), 1);
}
