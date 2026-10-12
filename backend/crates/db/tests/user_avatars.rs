use db::models::asset::NewInternalAsset;
use db::models::user::NewUser;
use db::queries::{assets, user_avatars, users};
use sqlx::MySqlPool;
use uuid::Uuid;

async fn create_user(pool: &MySqlPool, username: &str) -> Uuid {
    let mut conn = pool.acquire().await.unwrap();
    users::create(
        &mut conn,
        &NewUser {
            username: username.to_string(),
            twitch_id: None,
            discord_id: None,
        },
    )
    .await
    .unwrap()
    .id
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
            storage_url: format!("https://cdn.example.com/{hash_char}.png"),
            internal_path: None,
        },
    )
    .await
    .unwrap()
    .id
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn no_avatar_returns_none(pool: MySqlPool) {
    let user_id = create_user(&pool, "user_a").await;

    assert!(
        user_avatars::get_asset_id(&pool, user_id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        user_avatars::get_storage_url(&pool, user_id)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn set_links_asset_and_resolves_url(pool: MySqlPool) {
    let user_id = create_user(&pool, "user_a").await;
    let asset_id = create_asset(&pool, "a").await;

    let mut conn = pool.acquire().await.unwrap();
    user_avatars::set(&mut conn, user_id, asset_id)
        .await
        .unwrap();

    assert_eq!(
        user_avatars::get_asset_id(&pool, user_id).await.unwrap(),
        Some(asset_id)
    );
    assert_eq!(
        user_avatars::get_storage_url(&pool, user_id)
            .await
            .unwrap()
            .as_deref(),
        Some("https://cdn.example.com/a.png")
    );
    assert_eq!(assets::reference_count(&pool, asset_id).await.unwrap(), 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn set_replaces_existing_avatar(pool: MySqlPool) {
    let user_id = create_user(&pool, "user_a").await;
    let first = create_asset(&pool, "a").await;
    let second = create_asset(&pool, "b").await;

    let mut conn = pool.acquire().await.unwrap();
    user_avatars::set(&mut conn, user_id, first).await.unwrap();
    user_avatars::set(&mut conn, user_id, second).await.unwrap();

    assert_eq!(
        user_avatars::get_asset_id(&pool, user_id).await.unwrap(),
        Some(second)
    );
    assert_eq!(assets::reference_count(&pool, first).await.unwrap(), 0);
    assert_eq!(assets::reference_count(&pool, second).await.unwrap(), 1);
}
