use db::models::user::NewUser;
use db::queries::{capabilities, users};
use sqlx::MySqlPool;
use uuid::Uuid;

async fn create_user(pool: &MySqlPool, username: &str) -> Uuid {
    let mut conn = pool.acquire().await.unwrap();
    let new_user = NewUser {
        username: username.to_string(),
        twitch_id: None,
        discord_id: None,
    };
    users::create(&mut conn, &new_user).await.unwrap().id
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn list_returns_alphabetical_order(pool: MySqlPool) {
    let user_id = create_user(&pool, "alice").await;
    let mut conn = pool.acquire().await.unwrap();

    capabilities::add_to_user(&mut conn, user_id, "songs:manage_any")
        .await
        .unwrap();
    capabilities::add_to_user(&mut conn, user_id, "capabilities:manage")
        .await
        .unwrap();

    let caps = capabilities::list_for_user(&pool, user_id).await.unwrap();
    assert_eq!(caps, vec!["capabilities:manage", "songs:manage_any"]);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn add_duplicate_is_idempotent(pool: MySqlPool) {
    let user_id = create_user(&pool, "alice").await;
    let mut conn = pool.acquire().await.unwrap();

    capabilities::add_to_user(&mut conn, user_id, "capabilities:manage")
        .await
        .unwrap();
    capabilities::add_to_user(&mut conn, user_id, "capabilities:manage")
        .await
        .unwrap();

    let caps = capabilities::list_for_user(&pool, user_id).await.unwrap();
    assert_eq!(caps.len(), 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn remove_existing(pool: MySqlPool) {
    let user_id = create_user(&pool, "alice").await;
    let mut conn = pool.acquire().await.unwrap();

    capabilities::add_to_user(&mut conn, user_id, "capabilities:manage")
        .await
        .unwrap();
    capabilities::remove_from_user(&pool, user_id, "capabilities:manage")
        .await
        .unwrap();

    let caps = capabilities::list_for_user(&pool, user_id).await.unwrap();
    assert!(caps.is_empty());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn remove_nonexistent_is_ok(pool: MySqlPool) {
    let user_id = create_user(&pool, "alice").await;
    let result = capabilities::remove_from_user(&pool, user_id, "capabilities:manage").await;
    assert!(result.is_ok());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn list_for_different_users(pool: MySqlPool) {
    let alice_id = create_user(&pool, "alice").await;
    let bob_id = create_user(&pool, "bob").await;
    let mut conn = pool.acquire().await.unwrap();

    capabilities::add_to_user(&mut conn, alice_id, "capabilities:manage")
        .await
        .unwrap();
    capabilities::add_to_user(&mut conn, bob_id, "songs:manage_any")
        .await
        .unwrap();

    let alice_caps = capabilities::list_for_user(&pool, alice_id).await.unwrap();
    let bob_caps = capabilities::list_for_user(&pool, bob_id).await.unwrap();

    assert_eq!(alice_caps, vec!["capabilities:manage"]);
    assert_eq!(bob_caps, vec!["songs:manage_any"]);
}
