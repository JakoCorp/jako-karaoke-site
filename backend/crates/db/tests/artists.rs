use db::models::artist::{NewArtist, UpdateArtist};
use db::queries::artists;
use sqlx::MySqlPool;
use uuid::Uuid;

fn new_artist(name: &str) -> NewArtist {
    NewArtist {
        name: name.to_string(),
        description: None,
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_and_get_by_id(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let created = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let fetched = artists::get_by_id(&pool, created.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "artist_a");
    assert!(fetched.description.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn list_ordered_by_name(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    artists::create(&mut conn, &new_artist("name_c"))
        .await
        .unwrap();
    artists::create(&mut conn, &new_artist("name_a"))
        .await
        .unwrap();
    artists::create(&mut conn, &new_artist("name_b"))
        .await
        .unwrap();

    let results = artists::list(&pool, 10, 0).await.unwrap();
    let names: Vec<&str> = results.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, vec!["name_a", "name_b", "name_c"]);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn update_returns_updated_fields(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let created = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let updated = artists::update(
        &mut conn,
        created.id,
        &UpdateArtist {
            name: "artist_b".to_string(),
            description: Some("description_a".to_string()),
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "artist_b");
    assert_eq!(updated.description.as_deref(), Some("description_a"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn update_nonexistent_returns_none(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let result = artists::update(
        &mut conn,
        Uuid::new_v4(),
        &UpdateArtist {
            name: "artist_a".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    assert!(result.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_removes_artist(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let created = artists::create(&mut conn, &new_artist("artist_a"))
        .await
        .unwrap();

    let deleted = artists::delete(&pool, created.id).await.unwrap();
    assert!(deleted);

    let fetched = artists::get_by_id(&pool, created.id).await.unwrap();
    assert!(fetched.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_nonexistent_returns_false(pool: MySqlPool) {
    let deleted = artists::delete(&pool, Uuid::new_v4()).await.unwrap();
    assert!(!deleted);
}
