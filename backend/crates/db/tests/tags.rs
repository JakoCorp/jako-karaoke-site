use db::models::tag::NewTag;
use db::queries::tags;
use sqlx::MySqlPool;
use uuid::Uuid;

fn new_tag(name: &str) -> NewTag {
    NewTag {
        name: name.to_string(),
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_or_create_new_tag(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let tag = tags::get_or_create(&mut conn, &new_tag("tag_a"))
        .await
        .unwrap();
    assert_eq!(tag.name, "tag_a");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_or_create_existing_returns_same_id(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let first = tags::get_or_create(&mut conn, &new_tag("tag_a"))
        .await
        .unwrap();
    let second = tags::get_or_create(&mut conn, &new_tag("tag_a"))
        .await
        .unwrap();
    assert_eq!(first.id, second.id);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn list_ordered_by_name(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    tags::get_or_create(&mut conn, &new_tag("tag_c"))
        .await
        .unwrap();
    tags::get_or_create(&mut conn, &new_tag("tag_a"))
        .await
        .unwrap();
    tags::get_or_create(&mut conn, &new_tag("tag_b"))
        .await
        .unwrap();

    let results = tags::list(&pool).await.unwrap();
    let names: Vec<&str> = results.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["tag_a", "tag_b", "tag_c"]);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_removes_tag(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let tag = tags::get_or_create(&mut conn, &new_tag("tag_a"))
        .await
        .unwrap();

    let deleted = tags::delete(&pool, tag.id).await.unwrap();
    assert!(deleted);

    let fetched = tags::get_by_id(&pool, tag.id).await.unwrap();
    assert!(fetched.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_nonexistent_returns_false(pool: MySqlPool) {
    let deleted = tags::delete(&pool, Uuid::new_v4()).await.unwrap();
    assert!(!deleted);
}
