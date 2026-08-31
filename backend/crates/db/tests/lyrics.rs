use db::models::lyrics::NewLyrics;
use db::queries::lyrics;
use sqlx::MySqlPool;
use uuid::Uuid;

fn new_lyrics(content: &str) -> NewLyrics {
    NewLyrics {
        content: content.to_string(),
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_and_get_by_id(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let created = lyrics::create(&mut conn, &new_lyrics("content_a"))
        .await
        .unwrap();

    let fetched = lyrics::get_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.content, "content_a");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn update_changes_content(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let created = lyrics::create(&mut conn, &new_lyrics("content_a"))
        .await
        .unwrap();

    lyrics::update(&pool, created.id, "content_b")
        .await
        .unwrap();

    let fetched = lyrics::get_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(fetched.content, "content_b");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn reference_count_with_no_references(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let created = lyrics::create(&mut conn, &new_lyrics("content_a"))
        .await
        .unwrap();

    let count = lyrics::reference_count(&pool, created.id).await.unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_removes_lyrics(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let created = lyrics::create(&mut conn, &new_lyrics("content_a"))
        .await
        .unwrap();

    let deleted = lyrics::delete(&pool, created.id).await.unwrap();
    assert!(deleted);

    let fetched = lyrics::get_by_id(&pool, created.id).await.unwrap();
    assert!(fetched.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_nonexistent_returns_false(pool: MySqlPool) {
    let deleted = lyrics::delete(&pool, Uuid::new_v4()).await.unwrap();
    assert!(!deleted);
}
