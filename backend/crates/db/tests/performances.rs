use chrono::NaiveDate;
use db::error::DbError;
use db::filter::{self, Clause, FilterTarget, Subject, TagScope};
use db::models::artist::{Artist, NewArtist};
use db::models::performance::{NewPerformance, UpdatePerformance};
use db::models::song::NewSong;
use db::models::tag::NewTag;
use db::queries::performances::{self, PerformanceFilter};
use db::queries::{artists, songs, tags};
use sqlx::MySqlPool;
use uuid::Uuid;

const ORDER_BY: &str =
    "performance_date DESC, stream_number DESC, performance_number DESC, id DESC";

fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, day).unwrap()
}

fn singer_clause(ids: &[Uuid], exclude: bool) -> Clause {
    Clause {
        target: FilterTarget::Singer,
        ids: ids.to_vec(),
        exclude,
    }
}

fn tag_clause(kind: &'static str, scope: TagScope, ids: &[Uuid]) -> Clause {
    Clause {
        target: FilterTarget::Tag { kind, scope },
        ids: ids.to_vec(),
        exclude: false,
    }
}

async fn create_artist(pool: &MySqlPool, name: &str) -> Artist {
    let mut conn = pool.acquire().await.unwrap();
    artists::create(
        &mut conn,
        &NewArtist {
            name: name.to_string(),
            description: None,
        },
    )
    .await
    .unwrap()
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
async fn search_matches_exact_id(pool: MySqlPool) {
    let target = create_performance(&pool, date(1), Some(1), Some(1))
        .await
        .unwrap();
    create_performance(&pool, date(1), Some(1), Some(2))
        .await
        .unwrap();

    let query = target.id.to_string();
    let filter = PerformanceFilter {
        q: Some(&query),
        ..PerformanceFilter::default()
    };
    let results = performances::search(&pool, filter, ORDER_BY, 10, 0)
        .await
        .unwrap();
    let ids: Vec<_> = results.iter().map(|performance| performance.id).collect();

    assert_eq!(ids, vec![target.id]);
    assert_eq!(performances::search_count(&pool, filter).await.unwrap(), 1);
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

    let clauses = [singer_clause(&[artist_b.id], false)];
    let filter = PerformanceFilter {
        clauses: &clauses,
        ..PerformanceFilter::default()
    };
    let results = performances::search(&pool, filter, ORDER_BY, 10, 0)
        .await
        .unwrap();
    let total = performances::search_count(&pool, filter).await.unwrap();

    assert_eq!(total, 1);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, sung_by_b.id);

    let clauses = [singer_clause(&[artist_a.id], false)];
    let filter = PerformanceFilter {
        clauses: &clauses,
        ..PerformanceFilter::default()
    };
    assert_eq!(performances::search_count(&pool, filter).await.unwrap(), 2);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn clause_values_match_any_and_clauses_match_all(pool: MySqlPool) {
    let only_a = create_performance(&pool, date(1), Some(1), Some(1))
        .await
        .unwrap();
    let only_b = create_performance(&pool, date(1), Some(1), Some(2))
        .await
        .unwrap();
    let both = create_performance(&pool, date(1), Some(1), Some(3))
        .await
        .unwrap();
    let neither = create_performance(&pool, date(1), Some(1), Some(4))
        .await
        .unwrap();

    let artist_a = create_artist(&pool, "artist_a").await;
    let artist_b = create_artist(&pool, "artist_b").await;
    let mut conn = pool.acquire().await.unwrap();
    performances::set_singers(&mut conn, only_a.id, &[artist_a.id])
        .await
        .unwrap();
    performances::set_singers(&mut conn, only_b.id, &[artist_b.id])
        .await
        .unwrap();
    performances::set_singers(&mut conn, both.id, &[artist_a.id, artist_b.id])
        .await
        .unwrap();

    let any_of = [singer_clause(&[artist_a.id, artist_b.id], false)];
    let filter = PerformanceFilter {
        clauses: &any_of,
        ..PerformanceFilter::default()
    };
    assert_eq!(performances::search_count(&pool, filter).await.unwrap(), 3);

    let all_of = [
        singer_clause(&[artist_a.id], false),
        singer_clause(&[artist_b.id], false),
    ];
    let filter = PerformanceFilter {
        clauses: &all_of,
        ..PerformanceFilter::default()
    };
    let results = performances::search(&pool, filter, ORDER_BY, 10, 0)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, both.id);

    let excluded = [singer_clause(&[artist_a.id], true)];
    let filter = PerformanceFilter {
        clauses: &excluded,
        ..PerformanceFilter::default()
    };
    let results = performances::search(&pool, filter, ORDER_BY, 10, 0)
        .await
        .unwrap();
    let ids: Vec<_> = results.iter().map(|performance| performance.id).collect();
    assert_eq!(ids, vec![neither.id, only_b.id]);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_filters_by_date_range(pool: MySqlPool) {
    create_performance(&pool, date(1), None, None)
        .await
        .unwrap();
    let middle = create_performance(&pool, date(5), None, None)
        .await
        .unwrap();
    create_performance(&pool, date(9), None, None)
        .await
        .unwrap();

    let filter = PerformanceFilter {
        date_from: Some(date(2)),
        date_to: Some(date(8)),
        ..PerformanceFilter::default()
    };
    let results = performances::search(&pool, filter, ORDER_BY, 10, 0)
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, middle.id);
    assert_eq!(performances::search_count(&pool, filter).await.unwrap(), 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn misc_tag_clause_matches_performance_and_song_tags(pool: MySqlPool) {
    let tagged_directly = create_performance(&pool, date(1), None, None)
        .await
        .unwrap();
    let tagged_by_song = create_performance(&pool, date(2), None, None)
        .await
        .unwrap();
    let genre_only = create_performance(&pool, date(3), None, None)
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let tag = tags::get_or_create(
        &mut conn,
        &NewTag {
            name: "tag_a".to_string(),
        },
    )
    .await
    .unwrap();
    let song = songs::create(
        &mut conn,
        &NewSong {
            title: "song_a".to_string(),
            created_by: None,
            lyrics_id: None,
        },
    )
    .await
    .unwrap();
    performances::set_tags(&mut conn, tagged_directly.id, &[(tag.id, "misc")])
        .await
        .unwrap();
    performances::set_songs(&mut conn, tagged_by_song.id, &[song.id])
        .await
        .unwrap();
    performances::set_songs(&mut conn, genre_only.id, &[song.id])
        .await
        .unwrap();
    songs::set_tags(&mut conn, song.id, &[(tag.id, "misc")])
        .await
        .unwrap();

    let clauses = [tag_clause("misc", TagScope::Either, &[tag.id])];
    let filter = PerformanceFilter {
        clauses: &clauses,
        ..PerformanceFilter::default()
    };
    assert_eq!(performances::search_count(&pool, filter).await.unwrap(), 3);

    let performance_only = [tag_clause("misc", TagScope::Performance, &[tag.id])];
    let filter = PerformanceFilter {
        clauses: &performance_only,
        ..PerformanceFilter::default()
    };
    let results = performances::search(&pool, filter, ORDER_BY, 10, 0)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, tagged_directly.id);

    let wrong_kind = [tag_clause("modifier", TagScope::Either, &[tag.id])];
    let filter = PerformanceFilter {
        clauses: &wrong_kind,
        ..PerformanceFilter::default()
    };
    assert_eq!(performances::search_count(&pool, filter).await.unwrap(), 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn catalog_lists_used_tags_most_used_first(pool: MySqlPool) {
    let first = create_performance(&pool, date(1), None, None)
        .await
        .unwrap();
    let second = create_performance(&pool, date(2), None, None)
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let mut create_tag = async |name: &str| {
        tags::get_or_create(
            &mut conn,
            &NewTag {
                name: name.to_string(),
            },
        )
        .await
        .unwrap()
    };
    let rare = create_tag("tag_a").await;
    let common = create_tag("tag_b").await;
    create_tag("tag_unused").await;

    performances::set_tags(
        &mut conn,
        first.id,
        &[(rare.id, "instrument"), (common.id, "instrument")],
    )
    .await
    .unwrap();
    performances::set_tags(&mut conn, second.id, &[(common.id, "instrument")])
        .await
        .unwrap();

    let target = FilterTarget::Tag {
        kind: "instrument",
        scope: TagScope::Performance,
    };
    let values = filter::catalog_values(&pool, Subject::Performance, target)
        .await
        .unwrap();
    let ids: Vec<_> = values.iter().map(|value| value.id).collect();

    assert_eq!(ids, vec![common.id, rare.id]);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn years_lists_each_year_once_newest_first(pool: MySqlPool) {
    for (year, day) in [(2025, 1), (2026, 1), (2026, 2)] {
        let mut conn = pool.acquire().await.unwrap();
        performances::create(
            &mut conn,
            &NewPerformance {
                created_by: None,
                title: None,
                lyrics_id: None,
                stream_time: None,
                performance_date: NaiveDate::from_ymd_opt(year, 1, day).unwrap(),
                stream_number: None,
                performance_number: None,
            },
        )
        .await
        .unwrap();
    }

    let years = performances::years(&pool).await.unwrap();

    assert_eq!(years, vec![2026, 2025]);
}
