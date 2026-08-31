use chrono::{Duration, Utc};
use db::models::user::NewUser;
use db::queries::{sessions, users};
use sqlx::MySqlPool;
use uuid::Uuid;

async fn create_user(pool: &MySqlPool) -> Uuid {
    let mut conn = pool.acquire().await.unwrap();
    users::create(
        &mut conn,
        &NewUser {
            username: "testuser".to_string(),
            twitch_id: None,
            discord_id: None,
        },
    )
    .await
    .unwrap()
    .id
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_and_get_valid(pool: MySqlPool) {
    let user_id = create_user(&pool).await;
    let expires_at = Utc::now() + Duration::hours(1);

    sessions::create(&pool, "token_hash_abc", user_id, expires_at)
        .await
        .unwrap();

    let session = sessions::get_valid(&pool, "token_hash_abc")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.id, "token_hash_abc");
    assert_eq!(session.user_id, user_id);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_valid_returns_none_for_expired(pool: MySqlPool) {
    let user_id = create_user(&pool).await;
    let expires_at = Utc::now() - Duration::seconds(1);

    sessions::create(&pool, "expired_token", user_id, expires_at)
        .await
        .unwrap();

    let session = sessions::get_valid(&pool, "expired_token").await.unwrap();
    assert!(session.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_valid_returns_none_for_missing(pool: MySqlPool) {
    let session = sessions::get_valid(&pool, "nonexistent_token")
        .await
        .unwrap();
    assert!(session.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_removes_session(pool: MySqlPool) {
    let user_id = create_user(&pool).await;
    let expires_at = Utc::now() + Duration::hours(1);

    sessions::create(&pool, "token_to_delete", user_id, expires_at)
        .await
        .unwrap();
    sessions::delete(&pool, "token_to_delete").await.unwrap();

    let session = sessions::get_valid(&pool, "token_to_delete").await.unwrap();
    assert!(session.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_expired_removes_only_expired(pool: MySqlPool) {
    let user_id = create_user(&pool).await;

    sessions::create(
        &pool,
        "valid_token",
        user_id,
        Utc::now() + Duration::hours(1),
    )
    .await
    .unwrap();
    sessions::create(
        &pool,
        "expired_token",
        user_id,
        Utc::now() - Duration::seconds(1),
    )
    .await
    .unwrap();

    let removed = sessions::delete_expired(&pool).await.unwrap();
    assert_eq!(removed, 1);

    assert!(
        sessions::get_valid(&pool, "valid_token")
            .await
            .unwrap()
            .is_some()
    );
}
