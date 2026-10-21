use db::models::user::NewUser;
use db::queries::{capabilities, users};
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
async fn capabilities_are_sorted_unique_and_scoped_per_user(pool: MySqlPool) {
    let alice = create_user(&pool, "alice").await;
    let bob = create_user(&pool, "bob").await;
    let mut conn = pool.acquire().await.unwrap();

    for capability in [
        "songs:manage_any",
        "capabilities:manage",
        "songs:manage_any",
    ] {
        capabilities::add_to_user(&mut conn, alice, capability)
            .await
            .unwrap();
    }
    capabilities::add_to_user(&mut conn, bob, "songs:manage_any")
        .await
        .unwrap();

    assert_eq!(
        capabilities::list_for_user(&pool, alice).await.unwrap(),
        vec!["capabilities:manage", "songs:manage_any"]
    );
    assert_eq!(
        capabilities::list_for_user(&pool, bob).await.unwrap(),
        vec!["songs:manage_any"]
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn remove_deletes_only_the_given_capability(pool: MySqlPool) {
    let alice = create_user(&pool, "alice").await;
    let mut conn = pool.acquire().await.unwrap();
    for capability in ["capabilities:manage", "songs:manage_any"] {
        capabilities::add_to_user(&mut conn, alice, capability)
            .await
            .unwrap();
    }

    capabilities::remove_from_user(&pool, alice, "capabilities:manage")
        .await
        .unwrap();
    capabilities::remove_from_user(&pool, alice, "capabilities:manage")
        .await
        .unwrap();

    assert_eq!(
        capabilities::list_for_user(&pool, alice).await.unwrap(),
        vec!["songs:manage_any"]
    );
}
