use db::models::artist::{NewArtist, NewArtistLink, UpdateArtist};
use db::models::image::NewImage;
use db::queries::{artists, images};
use sqlx::MySqlPool;
use uuid::Uuid;

fn new_artist(name: &str) -> NewArtist {
    NewArtist {
        name: name.to_string(),
        description: None,
    }
}

async fn create_image(pool: &MySqlPool, url: &str) -> Uuid {
    let mut conn = pool.acquire().await.unwrap();
    images::create(
        &mut conn,
        &NewImage {
            public_url: url.to_string(),
            internal_path: None,
            credits: None,
        },
    )
    .await
    .unwrap()
    .id
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_and_get_by_id(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let created = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let fetched = artists::get_by_id(&pool, created.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "artist_a");
    assert!(fetched.description.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn list_ordered_by_name(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    artists::create(&mut conn, &new_artist("name_c"))
        .await
        .unwrap();
    artists::create(&mut conn, &new_artist("name_a"))
        .await
        .unwrap();
    artists::create(&mut conn, &new_artist("name_b"))
        .await
        .unwrap();

    let results = artists::list(&pool, 10, 0).await.unwrap();
    let names: Vec<&str> = results.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, vec!["name_a", "name_b", "name_c"]);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn update_returns_updated_fields(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let created = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

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

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "artist_b");
    assert_eq!(updated.description.as_deref(), Some("description_a"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn update_nonexistent_returns_none(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let result = artists::update(
        &mut conn,
        Uuid::new_v4(),
        &UpdateArtist {
            name: "artist_a".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    assert!(result.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_removes_artist(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let created = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let deleted = artists::delete(&pool, created.id).await.unwrap();
    assert!(deleted);

    let fetched = artists::get_by_id(&pool, created.id).await.unwrap();
    assert!(fetched.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_nonexistent_returns_false(pool: MySqlPool) {
    let deleted = artists::delete(&pool, Uuid::new_v4()).await.unwrap();
    assert!(!deleted);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn set_and_get_images(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let artist = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let image_id = create_image(&pool, "https://example.com/avatar.png").await;

    let mut tx = pool.begin().await.unwrap();
    artists::set_images(&mut tx, artist.id, &[(image_id, "avatar")])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let images = artists::get_images(&pool, artist.id).await.unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].0.id, image_id);
    assert_eq!(images[0].1, "avatar");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn set_images_replaces_existing(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let artist = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let image_a = create_image(&pool, "https://example.com/a.png").await;
    let image_b = create_image(&pool, "https://example.com/b.png").await;

    let mut tx = pool.begin().await.unwrap();
    artists::set_images(&mut tx, artist.id, &[(image_a, "avatar")])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    artists::set_images(&mut tx, artist.id, &[(image_b, "avatar")])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let images = artists::get_images(&pool, artist.id).await.unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].0.id, image_b);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_images_returns_empty_for_artist_with_no_images(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let artist = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let images = artists::get_images(&pool, artist.id).await.unwrap();
    assert!(images.is_empty());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_images_batch_groups_by_artist(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let artist_a = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();
    let artist_b = artists::create(&mut conn, &new_artist("artist_b"))
        .await
        .unwrap();

    let image_a = create_image(&pool, "https://example.com/a.png").await;
    let image_b = create_image(&pool, "https://example.com/b.png").await;

    let mut tx = pool.begin().await.unwrap();
    artists::set_images(&mut tx, artist_a.id, &[(image_a, "avatar")])
        .await
        .unwrap();
    artists::set_images(&mut tx, artist_b.id, &[(image_b, "avatar")])
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let batch = artists::get_images_batch(&pool, &[artist_a.id, artist_b.id])
        .await
        .unwrap();

    assert_eq!(batch[&artist_a.id].len(), 1);
    assert_eq!(batch[&artist_a.id][0].0.id, image_a);
    assert_eq!(batch[&artist_b.id].len(), 1);
    assert_eq!(batch[&artist_b.id][0].0.id, image_b);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn set_and_get_links(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let artist = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    let created = artists::set_links(
        &mut tx,
        artist.id,
        &[
            NewArtistLink {
                url: "https://youtube.com/channel_a".to_string(),
                kind: "youtube".to_string(),
                label: Some("YouTube".to_string()),
            },
            NewArtistLink {
                url: "https://example.com".to_string(),
                kind: "website".to_string(),
                label: None,
            },
        ],
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(created.len(), 2);

    let fetched = artists::get_links(&pool, artist.id).await.unwrap();
    assert_eq!(fetched.len(), 2);
    assert!(fetched.iter().any(|l| l.kind == "youtube"));
    assert!(fetched.iter().any(|l| l.kind == "website"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn set_links_replaces_existing(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let artist = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    artists::set_links(
        &mut tx,
        artist.id,
        &[NewArtistLink {
            url: "https://youtube.com/old".to_string(),
            kind: "youtube".to_string(),
            label: None,
        }],
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    artists::set_links(
        &mut tx,
        artist.id,
        &[NewArtistLink {
            url: "https://example.com".to_string(),
            kind: "website".to_string(),
            label: None,
        }],
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let links = artists::get_links(&pool, artist.id).await.unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].kind, "website");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn set_links_clear(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let artist = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    artists::set_links(
        &mut tx,
        artist.id,
        &[NewArtistLink {
            url: "https://youtube.com/channel".to_string(),
            kind: "youtube".to_string(),
            label: None,
        }],
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    artists::set_links(&mut tx, artist.id, &[]).await.unwrap();
    tx.commit().await.unwrap();

    let links = artists::get_links(&pool, artist.id).await.unwrap();
    assert!(links.is_empty());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_links_returns_empty_for_artist_with_no_links(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let artist = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let links = artists::get_links(&pool, artist.id).await.unwrap();
    assert!(links.is_empty());
}
