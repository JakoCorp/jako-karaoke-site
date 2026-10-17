use chrono::NaiveDate;
use db::error::DbError;
use db::models::artist::NewArtist;
use db::models::performance::{NewPerformance, UpdatePerformance};
use db::queries::artists;
use db::queries::performances::{self, PerformanceFilter};
use sqlx::MySqlPool;

const ORDER_BY: &str =
    "performance_date DESC, stream_number DESC, performance_number DESC, id DESC";

fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, day).unwrap()
}

async fn create_performance(
    pool: &MySqlPool,
    performance_date: NaiveDate,
    stream_number: Option<u8>,
    performance_number: Option<u16>,
) -> Result<db::models::Performance, DbError> {
    let mut conn = pool.acquire().await.unwrap();
    performances::create(
        &mut conn,
        &NewPerformance {
            created_by: None,
            title: None,
            lyrics_id: None,
            duration: None,
            stream_time: None,
            performance_date,
            stream_number,
            performance_number,
        },
    )
    .await
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_without_stream_position(pool: MySqlPool) {
    let created = create_performance(&pool, date(1), None, None)
        .await
        .unwrap();

    let fetched = performances::get_by_id(&pool, created.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.stream_number, None);
    assert_eq!(fetched.performance_number, None);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn same_date_without_stream_position_does_not_conflict(pool: MySqlPool) {
    create_performance(&pool, date(1), None, None)
        .await
        .unwrap();
    let second = create_performance(&pool, date(1), None, None).await;

    assert!(second.is_ok());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn duplicate_stream_position_is_rejected(pool: MySqlPool) {
    create_performance(&pool, date(1), Some(1), Some(1))
        .await
        .unwrap();
    let second = create_performance(&pool, date(1), Some(1), Some(1)).await;

    assert!(second.is_err());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn partial_stream_position_is_rejected(pool: MySqlPool) {
    let only_stream = create_performance(&pool, date(1), Some(1), None).await;
    let only_number = create_performance(&pool, date(1), None, Some(1)).await;

    assert!(only_stream.is_err());
    assert!(only_number.is_err());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn update_clears_stream_position(pool: MySqlPool) {
    let created = create_performance(&pool, date(1), Some(1), Some(2))
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let updated = performances::update(
        &mut conn,
        created.id,
        &UpdatePerformance {
            title: None,
            duration: None,
            stream_time: None,
            performance_date: date(1),
            stream_number: None,
            performance_number: None,
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(updated.stream_number, None);
    assert_eq!(updated.performance_number, None);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_sorts_without_stream_position_last_within_date(pool: MySqlPool) {
    let streamed_a = create_performance(&pool, date(1), Some(1), Some(1))
        .await
        .unwrap();
    let streamed_b = create_performance(&pool, date(1), Some(1), Some(2))
        .await
        .unwrap();
    let upload_a = create_performance(&pool, date(1), None, None)
        .await
        .unwrap();
    let upload_b = create_performance(&pool, date(1), None, None)
        .await
        .unwrap();
    let later = create_performance(&pool, date(2), None, None)
        .await
        .unwrap();

    let results = performances::search(&pool, PerformanceFilter::default(), ORDER_BY, 10, 0)
        .await
        .unwrap();
    let ids: Vec<_> = results.iter().map(|performance| performance.id).collect();

    assert_eq!(
        ids,
        vec![
            later.id,
            streamed_b.id,
            streamed_a.id,
            upload_b.id,
            upload_a.id
        ]
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_filters_by_artist(pool: MySqlPool) {
    let sung_by_a = create_performance(&pool, date(1), Some(1), Some(1))
        .await
        .unwrap();
    let sung_by_b = create_performance(&pool, date(1), Some(1), Some(2))
        .await
        .unwrap();
    create_performance(&pool, date(1), Some(1), Some(3))
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let artist_a = artists::create(
        &mut conn,
        &NewArtist {
            name: "artist_a".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();
    let artist_b = artists::create(
        &mut conn,
        &NewArtist {
            name: "artist_b".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();
    performances::set_singers(&mut conn, sung_by_a.id, &[artist_a.id])
        .await
        .unwrap();
    performances::set_singers(&mut conn, sung_by_b.id, &[artist_a.id, artist_b.id])
        .await
        .unwrap();

    let filter = PerformanceFilter {
        artist_id: Some(artist_b.id),
        ..PerformanceFilter::default()
    };
    let results = performances::search(&pool, filter, ORDER_BY, 10, 0)
        .await
        .unwrap();
    let total = performances::search_count(&pool, filter).await.unwrap();

    assert_eq!(total, 1);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, sung_by_b.id);

    let filter = PerformanceFilter {
        artist_id: Some(artist_a.id),
        ..PerformanceFilter::default()
    };
    assert_eq!(performances::search_count(&pool, filter).await.unwrap(), 2);
}
