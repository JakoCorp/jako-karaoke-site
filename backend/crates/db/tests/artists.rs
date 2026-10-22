use db::models::artist::{Artist, NewArtist, NewArtistLink, UpdateArtist};
use db::models::asset::NewInternalAsset;
use db::queries::artists::{self, SortKey};
use db::queries::assets;
use db::sort::SortDirection;
use sqlx::MySqlPool;
use uuid::Uuid;

async fn create_artist(pool: &MySqlPool, name: &str) -> Artist {
    let mut conn = pool.acquire().await.unwrap();
    artists::create(
        &mut conn,
        &NewArtist {
            name: name.to_string(),
            description: None,
        },
    )
    .await
    .unwrap()
}

async fn create_asset(pool: &MySqlPool, hash_char: &str) -> Uuid {
    let mut conn = pool.acquire().await.unwrap();
    assets::create_internal(
        &mut conn,
        &NewInternalAsset {
            title: None,
            credits: None,
            source_url: None,
            hash: hash_char.repeat(64),
            storage_url: format!("https://example.com/{hash_char}.png"),
            internal_path: None,
            length: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn set_images(pool: &MySqlPool, artist_id: Uuid, asset_id: Uuid) {
    let mut tx = pool.begin().await.unwrap();
    artists::set_images(&mut tx, artist_id, &[(asset_id, "avatar")])
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

async fn search_ids(
    pool: &MySqlPool,
    q: Option<&str>,
    sort: SortKey,
    direction: SortDirection,
) -> Vec<Uuid> {
    artists::search(pool, q, sort, direction, 10, 0)
        .await
        .unwrap()
        .into_iter()
        .map(|artist| artist.id)
        .collect()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn crud_roundtrip(pool: MySqlPool) {
    let created = create_artist(&pool, "artist_a").await;

    let fetched = artists::get_by_id(&pool, created.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.name, "artist_a");
    assert!(fetched.description.is_none());

    let mut conn = pool.acquire().await.unwrap();
    let updated = artists::update(
        &mut conn,
        created.id,
        &UpdateArtist {
            name: "artist_b".to_string(),
            description: Some("description_a".to_string()),
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(updated.name, "artist_b");
    assert_eq!(updated.description.as_deref(), Some("description_a"));

    assert!(artists::delete(&pool, created.id).await.unwrap());
    assert!(
        artists::get_by_id(&pool, created.id)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_sorts_and_matches_text_and_id(pool: MySqlPool) {
    let artist_b = create_artist(&pool, "name_b").await;
    let artist_a = create_artist(&pool, "name_a").await;
    let artist_c = create_artist(&pool, "name_c").await;

    let by_name = |direction| search_ids(&pool, None, SortKey::Name, direction);
    assert_eq!(
        by_name(SortDirection::Asc).await,
        vec![artist_a.id, artist_b.id, artist_c.id]
    );
    assert_eq!(
        by_name(SortDirection::Desc).await,
        vec![artist_c.id, artist_b.id, artist_a.id]
    );

    let by_song_count = |direction| search_ids(&pool, None, SortKey::SongCount, direction);
    assert_eq!(
        by_song_count(SortDirection::Asc).await,
        vec![artist_b.id, artist_a.id, artist_c.id]
    );
    assert_eq!(
        by_song_count(SortDirection::Desc).await,
        vec![artist_c.id, artist_a.id, artist_b.id]
    );

    let id_query = artist_c.id.to_string();
    for (query, expected) in [("name_a", artist_a.id), (id_query.as_str(), artist_c.id)] {
        let ids = search_ids(&pool, Some(query), SortKey::Name, SortDirection::Asc).await;
        assert_eq!(ids, vec![expected]);
        assert_eq!(artists::search_count(&pool, Some(query)).await.unwrap(), 1);
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn images_are_replaced_and_batched(pool: MySqlPool) {
    let artist_a = create_artist(&pool, "artist_a").await;
    let artist_b = create_artist(&pool, "artist_b").await;
    let asset_a = create_asset(&pool, "a").await;
    let asset_b = create_asset(&pool, "b").await;

    set_images(&pool, artist_a.id, asset_a).await;
    set_images(&pool, artist_a.id, asset_b).await;
    set_images(&pool, artist_b.id, asset_a).await;

    let images = artists::get_images(&pool, artist_a.id).await.unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].asset_id, asset_b);
    assert_eq!(images[0].kind, "avatar");

    let batch = artists::get_images_batch(&pool, &[artist_a.id, artist_b.id])
        .await
        .unwrap();
    assert_eq!(batch[&artist_a.id][0].asset_id, asset_b);
    assert_eq!(batch[&artist_b.id][0].asset_id, asset_a);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn links_lifecycle(pool: MySqlPool) {
    let artist = create_artist(&pool, "artist_a").await;
    let mut conn = pool.acquire().await.unwrap();

    let youtube = artists::create_link(
        &mut conn,
        artist.id,
        &NewArtistLink {
            url: "https://youtube.com/channel_a".to_string(),
            kind: "youtube".to_string(),
            label: Some("YouTube".to_string()),
        },
    )
    .await
    .unwrap();
    let website = artists::create_link(
        &mut conn,
        artist.id,
        &NewArtistLink {
            url: "https://example.com".to_string(),
            kind: "website".to_string(),
            label: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(artists::get_links(&pool, artist.id).await.unwrap().len(), 2);

    let updated = artists::update_link(
        &pool,
        youtube.id,
        &NewArtistLink {
            url: "https://example.com/new".to_string(),
            kind: "website".to_string(),
            label: Some("Site".to_string()),
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(updated.url, "https://example.com/new");
    assert_eq!(updated.kind, "website");
    assert_eq!(updated.label.as_deref(), Some("Site"));

    assert!(artists::delete_link(&pool, website.id).await.unwrap());
    let links = artists::get_links(&pool, artist.id).await.unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].id, youtube.id);
}
