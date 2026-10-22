use db::error::DbError;
use db::models::user::{NewUser, User};
use db::queries::users;
use sqlx::MySqlPool;

fn new_user(username: &str) -> NewUser {
    NewUser {
        username: username.to_string(),
        twitch_id: None,
        discord_id: None,
    }
}

async fn create_user(pool: &MySqlPool, username: &str) -> User {
    let mut conn = pool.acquire().await.unwrap();
    users::create(&mut conn, &new_user(username)).await.unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_get_and_delete(pool: MySqlPool) {
    let created = create_user(&pool, "user_a").await;

    let fetched = users::get_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(fetched.username, "user_a");
    assert!(fetched.twitch_id.is_none());
    assert!(fetched.discord_id.is_none());

    let mut conn = pool.acquire().await.unwrap();
    let duplicate = users::create(&mut conn, &new_user("user_a")).await;
    assert!(matches!(duplicate, Err(DbError::Conflict)));

    assert!(users::delete(&pool, created.id).await.unwrap());
    assert!(users::get_by_id(&pool, created.id).await.unwrap().is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn rename_rejects_taken_names_and_records_change_time(pool: MySqlPool) {
    let user = create_user(&pool, "user_a").await;
    create_user(&pool, "user_b").await;
    assert!(user.username_changed_at.is_none());
    let mut conn = pool.acquire().await.unwrap();

    let taken = users::update_username(&mut conn, user.id, "USER_B").await;
    assert!(matches!(taken, Err(DbError::Conflict)));
    let unchanged = users::get_by_id(&pool, user.id).await.unwrap().unwrap();
    assert_eq!(unchanged.username, "user_a");
    assert!(unchanged.username_changed_at.is_none());

    let renamed = users::update_username(&mut conn, user.id, "user_c")
        .await
        .unwrap();
    assert_eq!(renamed.username, "user_c");
    assert!(renamed.username_changed_at.is_some());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_matches_text_and_id(pool: MySqlPool) {
    let alpha = create_user(&pool, "user_alpha").await;
    create_user(&pool, "user_beta").await;
    create_user(&pool, "user_gamma").await;

    let all = users::search(&pool, None, 10, 0).await.unwrap();
    let names: Vec<&str> = all.iter().map(|user| user.username.as_str()).collect();
    assert_eq!(names, vec!["user_alpha", "user_beta", "user_gamma"]);
    assert_eq!(users::search_count(&pool, None).await.unwrap(), 3);

    let id_query = alpha.id.to_string();
    for query in ["alpha", id_query.as_str()] {
        let results = users::search(&pool, Some(query), 10, 0).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, alpha.id);
        assert_eq!(users::search_count(&pool, Some(query)).await.unwrap(), 1);
    }
}
