use chrono::{Duration, Utc};
use db::queries::pending_oauth;
use sqlx::MySqlPool;

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_and_get(pool: MySqlPool) {
    let expires_at = Utc::now() + Duration::minutes(15);
    pending_oauth::create(&pool, "token_a", "provider_a", 1001, "user_a", expires_at)
        .await
        .unwrap();

    let record = pending_oauth::get(&pool, "token_a").await.unwrap().unwrap();
    assert_eq!(record.token, "token_a");
    assert_eq!(record.provider, "provider_a");
    assert_eq!(record.provider_id, 1001);
    assert_eq!(record.suggested_username, "user_a");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_returns_none_for_expired(pool: MySqlPool) {
    let expires_at = Utc::now() - Duration::seconds(1);
    pending_oauth::create(&pool, "token_a", "provider_a", 1001, "user_a", expires_at)
        .await
        .unwrap();

    let record = pending_oauth::get(&pool, "token_a").await.unwrap();
    assert!(record.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_returns_none_for_missing(pool: MySqlPool) {
    let record = pending_oauth::get(&pool, "token_a").await.unwrap();
    assert!(record.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_removes_record(pool: MySqlPool) {
    let expires_at = Utc::now() + Duration::minutes(15);
    pending_oauth::create(&pool, "token_a", "provider_a", 1001, "user_a", expires_at)
        .await
        .unwrap();

    pending_oauth::delete(&pool, "token_a").await.unwrap();

    let record = pending_oauth::get(&pool, "token_a").await.unwrap();
    assert!(record.is_none());
}
