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
async fn session_roundtrip(pool: MySqlPool) {
    let user_id = create_user(&pool).await;
    let expires_at = Utc::now() + Duration::hours(1);
    sessions::create(&pool, "token_a", user_id, expires_at)
        .await
        .unwrap();

    let session = sessions::get_valid(&pool, "token_a")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.user_id, user_id);

    sessions::delete(&pool, "token_a").await.unwrap();
    assert!(
        sessions::get_valid(&pool, "token_a")
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn expired_sessions_are_hidden_and_purged(pool: MySqlPool) {
    let user_id = create_user(&pool).await;
    let valid = Utc::now() + Duration::hours(1);
    let expired = Utc::now() - Duration::seconds(1);
    sessions::create(&pool, "valid_token", user_id, valid)
        .await
        .unwrap();
    sessions::create(&pool, "expired_token", user_id, expired)
        .await
        .unwrap();

    assert!(
        sessions::get_valid(&pool, "expired_token")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(sessions::delete_expired(&pool).await.unwrap(), 1);
    assert!(
        sessions::get_valid(&pool, "valid_token")
            .await
            .unwrap()
            .is_some()
    );
}
