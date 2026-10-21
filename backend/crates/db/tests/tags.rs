use db::models::tag::NewTag;
use db::queries::tags;
use sqlx::MySqlPool;

fn new_tag(name: &str) -> NewTag {
    NewTag {
        name: name.to_string(),
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn tag_lifecycle(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    let tag_c = tags::get_or_create(&mut conn, &new_tag("tag_c"))
        .await
        .unwrap();
    tags::get_or_create(&mut conn, &new_tag("tag_a"))
        .await
        .unwrap();
    let again = tags::get_or_create(&mut conn, &new_tag("tag_c"))
        .await
        .unwrap();
    assert_eq!(again.id, tag_c.id);

    let listed = tags::list(&pool).await.unwrap();
    let names: Vec<&str> = listed.iter().map(|tag| tag.name.as_str()).collect();
    assert_eq!(names, vec!["tag_a", "tag_c"]);

    assert!(tags::delete(&pool, tag_c.id).await.unwrap());
    assert!(tags::get_by_id(&pool, tag_c.id).await.unwrap().is_none());
}
