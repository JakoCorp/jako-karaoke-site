use db::models::lyrics::NewLyrics;
use db::queries::lyrics;
use sqlx::MySqlPool;

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn lyrics_lifecycle(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let created = lyrics::create(
        &mut conn,
        &NewLyrics {
            content: "content_a".to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(created.content, "content_a");
    assert_eq!(lyrics::reference_count(&pool, created.id).await.unwrap(), 0);

    lyrics::update(&pool, created.id, "content_b")
        .await
        .unwrap();
    let fetched = lyrics::get_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(fetched.content, "content_b");

    assert!(lyrics::delete(&pool, created.id).await.unwrap());
    assert!(
        lyrics::get_by_id(&pool, created.id)
            .await
            .unwrap()
            .is_none()
    );
}
