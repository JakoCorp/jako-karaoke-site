use db::models::user::NewUser;
use db::queries::{user_suspensions, users};
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

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn suspensions_record_who_and_why_and_outlive_lifting(pool: MySqlPool) {
    let target = create_user(&pool, "target").await;
    let admin = create_user(&pool, "admin_a").await;
    let other_admin = create_user(&pool, "admin_b").await;
    let mut conn = pool.acquire().await.unwrap();

    user_suspensions::create(&mut conn, target, admin, "ignored")
        .await
        .unwrap();
    assert!(
        user_suspensions::list_for_user(&pool, target)
            .await
            .unwrap()
            .is_empty()
    );

    let suspended = users::suspend(&mut conn, target).await.unwrap();
    user_suspensions::create(&mut conn, target, admin, "spam")
        .await
        .unwrap();
    let rows = user_suspensions::list_for_user(&pool, target)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].reason, "spam");
    assert_eq!(rows[0].suspended_by, Some(admin));
    assert_eq!(rows[0].suspended_by_username.as_deref(), Some("admin_a"));
    assert_eq!(Some(rows[0].suspended_at), suspended.suspended_at);
    assert!(rows[0].lifted_at.is_none());

    users::unsuspend(&mut conn, target).await.unwrap();
    user_suspensions::lift(&mut conn, target, other_admin)
        .await
        .unwrap();
    let rows = user_suspensions::list_for_user(&pool, target)
        .await
        .unwrap();
    assert!(rows[0].lifted_at.is_some());
    assert_eq!(rows[0].lifted_by, Some(other_admin));
    assert_eq!(rows[0].lifted_by_username.as_deref(), Some("admin_b"));

    sqlx::query("UPDATE user_suspensions SET suspended_at = suspended_at - INTERVAL 1 DAY")
        .execute(&pool)
        .await
        .unwrap();
    users::suspend(&mut conn, target).await.unwrap();
    user_suspensions::create(&mut conn, target, admin, "again")
        .await
        .unwrap();
    let reasons: Vec<String> = user_suspensions::list_for_user(&pool, target)
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.reason)
        .collect();
    assert_eq!(reasons, vec!["again", "spam"]);

    users::delete(&pool, admin).await.unwrap();
    let rows = user_suspensions::list_for_user(&pool, target)
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row.suspended_by.is_none()));
    assert!(rows.iter().all(|row| row.suspended_by_username.is_none()));
    assert_eq!(rows[1].lifted_by, Some(other_admin));
}
