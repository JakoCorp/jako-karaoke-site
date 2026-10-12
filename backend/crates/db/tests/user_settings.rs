use db::models::{NewUser, UserSettings};
use db::queries::{user_settings, users};
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

fn custom_settings() -> UserSettings {
    UserSettings {
        download_filename_template: "{date} {title}".to_string(),
        download_include_cover_art: false,
        download_include_lyrics: true,
        download_include_date: false,
        download_include_singers: false,
        download_include_original_artists: true,
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn missing_row_returns_none(pool: MySqlPool) {
    let user_id = create_user(&pool, "user_a").await;

    assert!(user_settings::get(&pool, user_id).await.unwrap().is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn upsert_inserts_then_replaces(pool: MySqlPool) {
    let user_id = create_user(&pool, "user_a").await;
    let mut conn = pool.acquire().await.unwrap();

    user_settings::upsert(&mut conn, user_id, &UserSettings::default())
        .await
        .unwrap();
    assert_eq!(
        user_settings::get(&pool, user_id).await.unwrap(),
        Some(UserSettings::default())
    );

    user_settings::upsert(&mut conn, user_id, &custom_settings())
        .await
        .unwrap();
    assert_eq!(
        user_settings::get(&pool, user_id).await.unwrap(),
        Some(custom_settings())
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn settings_are_scoped_per_user(pool: MySqlPool) {
    let user_a = create_user(&pool, "user_a").await;
    let user_b = create_user(&pool, "user_b").await;
    let mut conn = pool.acquire().await.unwrap();

    user_settings::upsert(&mut conn, user_a, &custom_settings())
        .await
        .unwrap();

    assert!(user_settings::get(&pool, user_b).await.unwrap().is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn schema_defaults_match_model_defaults(pool: MySqlPool) {
    let user_id = create_user(&pool, "user_a").await;

    sqlx::query("INSERT INTO user_settings (user_id) VALUES (?)")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(
        user_settings::get(&pool, user_id).await.unwrap(),
        Some(UserSettings::default())
    );
}
