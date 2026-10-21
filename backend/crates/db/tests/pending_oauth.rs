use chrono::{Duration, Utc};
use db::queries::pending_oauth;
use sqlx::MySqlPool;

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn pending_record_roundtrip(pool: MySqlPool) {
    let expires_at = Utc::now() + Duration::minutes(15);
    pending_oauth::create(
        &pool,
        "token_a",
        "provider_a",
        1001,
        "user_a",
        None,
        expires_at,
    )
    .await
    .unwrap();

    let record = pending_oauth::get(&pool, "token_a").await.unwrap().unwrap();
    assert_eq!(record.provider, "provider_a");
    assert_eq!(record.provider_id, 1001);
    assert_eq!(record.suggested_username, "user_a");

    pending_oauth::delete(&pool, "token_a").await.unwrap();
    assert!(
        pending_oauth::get(&pool, "token_a")
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn expired_record_is_hidden(pool: MySqlPool) {
    let expires_at = Utc::now() - Duration::seconds(1);
    pending_oauth::create(
        &pool,
        "token_a",
        "provider_a",
        1001,
        "user_a",
        None,
        expires_at,
    )
    .await
    .unwrap();

    assert!(
        pending_oauth::get(&pool, "token_a")
            .await
            .unwrap()
            .is_none()
    );
}
