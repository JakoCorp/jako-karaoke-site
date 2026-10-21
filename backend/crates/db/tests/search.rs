use db::models::artist::NewArtist;
use db::queries::artists::{self, SortKey};
use db::sort::SortDirection;
use sqlx::MySqlPool;

async fn create_artist(pool: &MySqlPool, name: &str) {
    let mut conn = pool.acquire().await.unwrap();
    artists::create(
        &mut conn,
        &NewArtist {
            name: name.to_string(),
            description: None,
        },
    )
    .await
    .unwrap();
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_treats_wildcards_as_text(pool: MySqlPool) {
    create_artist(&pool, "100% artist").await;
    create_artist(&pool, "1000 artist").await;
    create_artist(&pool, "snake_case").await;
    create_artist(&pool, "snakeXcase").await;

    for (query, expected) in [("100%", "100% artist"), ("snake_case", "snake_case")] {
        let results = artists::search(&pool, Some(query), SortKey::Name, SortDirection::Asc, 10, 0)
            .await
            .unwrap();
        let names: Vec<&str> = results.iter().map(|artist| artist.name.as_str()).collect();

        assert_eq!(names, vec![expected]);
    }
}
