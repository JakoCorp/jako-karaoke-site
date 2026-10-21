use chrono::NaiveDate;
use db::error::DbError;
use db::filter::{self, Clause, FilterTarget, Subject, TagScope};
use db::models::artist::{Artist, NewArtist};
use db::models::performance::{NewPerformance, Performance, UpdatePerformance};
use db::models::song::NewSong;
use db::models::tag::{NewTag, Tag};
use db::queries::performances::{self, PerformanceFilter, SortKey};
use db::queries::{artists, songs, tags};
use db::sort::SortDirection;
use sqlx::MySqlPool;
use uuid::Uuid;

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

fn with_clauses(clauses: &[Clause]) -> PerformanceFilter<'_> {
    PerformanceFilter {
        clauses,
        ..PerformanceFilter::default()
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

async fn create_tag(pool: &MySqlPool, name: &str) -> Tag {
    let mut conn = pool.acquire().await.unwrap();
    tags::get_or_create(
        &mut conn,
        &NewTag {
            name: name.to_string(),
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
) -> Result<Performance, DbError> {
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

async fn search_ids(
    pool: &MySqlPool,
    filter: PerformanceFilter<'_>,
    sort: SortKey,
    direction: SortDirection,
) -> Vec<Uuid> {
    performances::search(pool, filter, sort, direction, 10, 0)
        .await
        .unwrap()
        .into_iter()
        .map(|performance| performance.id)
        .collect()
}

async fn newest_first(pool: &MySqlPool, filter: PerformanceFilter<'_>) -> Vec<Uuid> {
    search_ids(pool, filter, SortKey::PerformanceDate, SortDirection::Desc).await
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn stream_position_rules(pool: MySqlPool) {
    let streamed = create_performance(&pool, date(1), Some(1), Some(2))
        .await
        .unwrap();
    assert!(
        create_performance(&pool, date(1), Some(1), Some(2))
            .await
            .is_err()
    );
    assert!(
        create_performance(&pool, date(1), Some(1), None)
            .await
            .is_err()
    );
    assert!(
        create_performance(&pool, date(1), None, Some(1))
            .await
            .is_err()
    );

    create_performance(&pool, date(1), None, None)
        .await
        .unwrap();
    create_performance(&pool, date(1), None, None)
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let cleared = performances::update(
        &mut conn,
        streamed.id,
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
    assert_eq!(cleared.stream_number, None);
    assert_eq!(cleared.performance_number, None);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_orders_chronologically_for_every_sort_key(pool: MySqlPool) {
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
    let expected = vec![
        later.id,
        streamed_b.id,
        streamed_a.id,
        upload_b.id,
        upload_a.id,
    ];

    for sort in [
        SortKey::PerformanceDate,
        SortKey::PlayCount,
        SortKey::Duration,
    ] {
        let filter = PerformanceFilter::default();
        let descending = search_ids(&pool, filter, sort, SortDirection::Desc).await;
        let mut ascending = search_ids(&pool, filter, sort, SortDirection::Asc).await;

        assert_eq!(descending, expected);
        ascending.reverse();
        assert_eq!(ascending, expected);
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_filters_by_singers(pool: MySqlPool) {
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
    assert_eq!(
        newest_first(&pool, with_clauses(&any_of)).await,
        vec![both.id, only_b.id, only_a.id]
    );
    assert_eq!(
        performances::search_count(&pool, with_clauses(&any_of))
            .await
            .unwrap(),
        3
    );

    let all_of = [
        singer_clause(&[artist_a.id], false),
        singer_clause(&[artist_b.id], false),
    ];
    assert_eq!(
        newest_first(&pool, with_clauses(&all_of)).await,
        vec![both.id]
    );

    let excluded = [singer_clause(&[artist_a.id], true)];
    assert_eq!(
        newest_first(&pool, with_clauses(&excluded)).await,
        vec![neither.id, only_b.id]
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_filters_by_date_range_and_id(pool: MySqlPool) {
    let first = create_performance(&pool, date(1), None, None)
        .await
        .unwrap();
    let middle = create_performance(&pool, date(5), None, None)
        .await
        .unwrap();
    create_performance(&pool, date(9), None, None)
        .await
        .unwrap();

    let in_range = PerformanceFilter {
        date_from: Some(date(2)),
        date_to: Some(date(8)),
        ..PerformanceFilter::default()
    };
    assert_eq!(newest_first(&pool, in_range).await, vec![middle.id]);
    assert_eq!(
        performances::search_count(&pool, in_range).await.unwrap(),
        1
    );

    let id_query = first.id.to_string();
    let by_id = PerformanceFilter {
        q: Some(&id_query),
        ..PerformanceFilter::default()
    };
    assert_eq!(newest_first(&pool, by_id).await, vec![first.id]);
    assert_eq!(performances::search_count(&pool, by_id).await.unwrap(), 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn tag_clause_matches_performance_and_song_tags(pool: MySqlPool) {
    let tagged_directly = create_performance(&pool, date(1), None, None)
        .await
        .unwrap();
    let tagged_by_song = create_performance(&pool, date(2), None, None)
        .await
        .unwrap();

    let tag = create_tag(&pool, "tag_a").await;
    let mut conn = pool.acquire().await.unwrap();
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
    songs::set_tags(&mut conn, song.id, &[(tag.id, "misc")])
        .await
        .unwrap();

    let either = [tag_clause("misc", TagScope::Either, &[tag.id])];
    assert_eq!(
        newest_first(&pool, with_clauses(&either)).await,
        vec![tagged_by_song.id, tagged_directly.id]
    );

    let performance_only = [tag_clause("misc", TagScope::Performance, &[tag.id])];
    assert_eq!(
        newest_first(&pool, with_clauses(&performance_only)).await,
        vec![tagged_directly.id]
    );

    let wrong_kind = [tag_clause("modifier", TagScope::Either, &[tag.id])];
    assert!(
        newest_first(&pool, with_clauses(&wrong_kind))
            .await
            .is_empty()
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn catalog_lists_used_tags_most_used_first(pool: MySqlPool) {
    let first = create_performance(&pool, date(1), None, None)
        .await
        .unwrap();
    let second = create_performance(&pool, date(2), None, None)
        .await
        .unwrap();
    let rare = create_tag(&pool, "tag_a").await;
    let common = create_tag(&pool, "tag_b").await;
    create_tag(&pool, "tag_unused").await;

    let mut conn = pool.acquire().await.unwrap();
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
        let performance_date = NaiveDate::from_ymd_opt(year, 1, day).unwrap();
        create_performance(&pool, performance_date, None, None)
            .await
            .unwrap();
    }

    let years = performances::years(&pool).await.unwrap();

    assert_eq!(years, vec![2026, 2025]);
}
