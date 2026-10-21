use chrono::NaiveDate;
use db::models::performance::NewPerformance;
use db::models::playlist::{NewPlaylist, Playlist};
use db::models::user::NewUser;
use db::queries::playlists::{self, PlaylistFilter, SortKey};
use db::queries::{performances, users};
use db::sort::SortDirection;
use sqlx::MySqlPool;
use uuid::Uuid;

async fn create_user(pool: &MySqlPool) -> Uuid {
    let mut conn = pool.acquire().await.unwrap();
    users::create(
        &mut conn,
        &NewUser {
            username: "user_a".to_string(),
            twitch_id: None,
            discord_id: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_playlist(
    pool: &MySqlPool,
    title: &str,
    kind: &str,
    is_public: bool,
    created_by: Option<Uuid>,
) -> Playlist {
    let mut conn = pool.acquire().await.unwrap();
    playlists::create(
        &mut conn,
        &NewPlaylist {
            title: title.to_string(),
            description: None,
            kind: kind.to_string(),
            is_public,
            created_by,
        },
    )
    .await
    .unwrap()
}

async fn create_performance(pool: &MySqlPool) -> Uuid {
    let mut conn = pool.acquire().await.unwrap();
    performances::create(
        &mut conn,
        &NewPerformance {
            created_by: None,
            title: None,
            lyrics_id: None,
            stream_time: None,
            performance_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            stream_number: None,
            performance_number: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn add_performances(pool: &MySqlPool, playlist_id: Uuid, performance_ids: &[Uuid]) {
    let mut conn = pool.acquire().await.unwrap();
    playlists::add_performances(&mut conn, playlist_id, performance_ids)
        .await
        .unwrap();
}

async fn search_ids(pool: &MySqlPool, sort: SortKey, direction: SortDirection) -> Vec<Uuid> {
    playlists::search(pool, PlaylistFilter::default(), sort, direction, 10, 0)
        .await
        .unwrap()
        .into_iter()
        .map(|playlist| playlist.id)
        .collect()
}

async fn titles(pool: &MySqlPool, filter: PlaylistFilter<'_>) -> Vec<String> {
    let found = playlists::search(pool, filter, SortKey::Title, SortDirection::Asc, 10, 0)
        .await
        .unwrap();
    assert_eq!(
        playlists::search_count(pool, filter).await.unwrap(),
        found.len() as u64
    );
    found.into_iter().map(|playlist| playlist.title).collect()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_applies_visibility_owner_kind_and_title(pool: MySqlPool) {
    let user_id = create_user(&pool).await;
    create_playlist(&pool, "playlist_a", "user", true, Some(user_id)).await;
    create_playlist(&pool, "playlist_b", "user", false, Some(user_id)).await;
    create_playlist(&pool, "playlist_c", "favorites", true, Some(user_id)).await;
    create_playlist(&pool, "needle_d", "user", true, None).await;

    let public = PlaylistFilter::default();
    assert_eq!(titles(&pool, public).await, vec!["needle_d", "playlist_a"]);

    let with_private = PlaylistFilter {
        include_private: true,
        ..public
    };
    assert_eq!(
        titles(&pool, with_private).await,
        vec!["needle_d", "playlist_a", "playlist_b"]
    );

    let owned = PlaylistFilter {
        created_by: Some(user_id),
        ..with_private
    };
    assert_eq!(
        titles(&pool, owned).await,
        vec!["playlist_a", "playlist_b", "playlist_c"]
    );

    let owned_favorites = PlaylistFilter {
        kind: Some("favorites"),
        ..owned
    };
    assert_eq!(titles(&pool, owned_favorites).await, vec!["playlist_c"]);

    let by_title = PlaylistFilter {
        q: Some("needle"),
        ..public
    };
    assert_eq!(titles(&pool, by_title).await, vec!["needle_d"]);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_sorts_by_every_key_in_both_directions(pool: MySqlPool) {
    let playlist_b = create_playlist(&pool, "playlist_b", "user", true, None).await;
    let playlist_c = create_playlist(&pool, "playlist_c", "user", true, None).await;
    let playlist_a = create_playlist(&pool, "playlist_a", "user", true, None).await;
    let playlist_d = create_playlist(&pool, "playlist_d", "user", true, None).await;

    let performance_ids = [
        create_performance(&pool).await,
        create_performance(&pool).await,
        create_performance(&pool).await,
    ];
    add_performances(&pool, playlist_b.id, &performance_ids[..1]).await;
    add_performances(&pool, playlist_c.id, &performance_ids).await;
    add_performances(&pool, playlist_a.id, &performance_ids[..2]).await;
    add_performances(&pool, playlist_d.id, &performance_ids[..1]).await;

    let by_created = |direction| search_ids(&pool, SortKey::Created, direction);
    assert_eq!(
        by_created(SortDirection::Asc).await,
        vec![playlist_b.id, playlist_c.id, playlist_a.id, playlist_d.id]
    );
    assert_eq!(
        by_created(SortDirection::Desc).await,
        vec![playlist_d.id, playlist_a.id, playlist_c.id, playlist_b.id]
    );

    let by_title = |direction| search_ids(&pool, SortKey::Title, direction);
    assert_eq!(
        by_title(SortDirection::Asc).await,
        vec![playlist_a.id, playlist_b.id, playlist_c.id, playlist_d.id]
    );
    assert_eq!(
        by_title(SortDirection::Desc).await,
        vec![playlist_d.id, playlist_c.id, playlist_b.id, playlist_a.id]
    );

    let by_performance_count = |direction| search_ids(&pool, SortKey::PerformanceCount, direction);
    assert_eq!(
        by_performance_count(SortDirection::Asc).await,
        vec![playlist_b.id, playlist_d.id, playlist_a.id, playlist_c.id]
    );
    assert_eq!(
        by_performance_count(SortDirection::Desc).await,
        vec![playlist_c.id, playlist_a.id, playlist_d.id, playlist_b.id]
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn remove_performances_ignores_missing_entries(pool: MySqlPool) {
    let playlist = create_playlist(&pool, "playlist_a", "user", true, None).await;
    let performance_a = create_performance(&pool).await;
    let performance_b = create_performance(&pool).await;

    let mut conn = pool.acquire().await.unwrap();
    playlists::add_performances(&mut conn, playlist.id, &[performance_a, performance_b])
        .await
        .unwrap();

    playlists::remove_performances(&pool, playlist.id, &[performance_a, Uuid::new_v4()])
        .await
        .unwrap();

    let remaining = playlists::get_performances_in_playlist(&pool, playlist.id)
        .await
        .unwrap();
    let ids: Vec<_> = remaining.iter().map(|row| row.performance.id).collect();
    assert_eq!(ids, vec![performance_b]);
}
