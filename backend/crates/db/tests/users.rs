use db::error::DbError;
use db::models::user::NewUser;
use db::queries::users;
use sqlx::MySqlPool;

async fn create_user(pool: &MySqlPool, username: &str) -> db::models::user::User {
    let mut conn = pool.acquire().await.unwrap();
    let new_user = NewUser {
        username: username.to_string(),
        twitch_id: None,
        discord_id: None,
    };
    users::create(&mut conn, &new_user).await.unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_and_get_by_id(pool: MySqlPool) {
    let created = create_user(&pool, "user_a").await;
    let fetched = users::get_by_id(&pool, created.id).await.unwrap().unwrap();

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.username, "user_a");
    assert!(fetched.twitch_id.is_none());
    assert!(fetched.discord_id.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_duplicate_username_conflicts(pool: MySqlPool) {
    create_user(&pool, "user_a").await;

    let mut conn = pool.acquire().await.unwrap();
    let second = users::create(
        &mut conn,
        &NewUser {
            username: "user_a".to_string(),
            twitch_id: None,
            discord_id: None,
        },
    )
    .await;

    assert!(matches!(second, Err(DbError::Conflict)));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_no_filter_returns_all(pool: MySqlPool) {
    create_user(&pool, "user_alpha").await;
    create_user(&pool, "user_beta").await;
    create_user(&pool, "user_gamma").await;

    let results = users::search(&pool, None).await.unwrap();
    let names: Vec<&str> = results.iter().map(|u| u.username.as_str()).collect();

    assert_eq!(names, vec!["user_alpha", "user_beta", "user_gamma"]);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_with_filter(pool: MySqlPool) {
    create_user(&pool, "user_alpha").await;
    create_user(&pool, "user_beta").await;
    create_user(&pool, "user_gamma").await;

    let results = users::search(&pool, Some("alpha")).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].username, "user_alpha");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_no_matches(pool: MySqlPool) {
    create_user(&pool, "user_alpha").await;
    create_user(&pool, "user_beta").await;

    let results = users::search(&pool, Some("user_zzz")).await.unwrap();
    assert!(results.is_empty());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_removes_user(pool: MySqlPool) {
    let user = create_user(&pool, "user_a").await;

    let deleted = users::delete(&pool, user.id).await.unwrap();
    assert!(deleted);

    let fetched = users::get_by_id(&pool, user.id).await.unwrap();
    assert!(fetched.is_none());
}
