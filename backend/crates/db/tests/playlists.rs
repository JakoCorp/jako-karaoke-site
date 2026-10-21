use chrono::NaiveDate;
use db::models::performance::NewPerformance;
use db::models::playlist::{NewPlaylist, Playlist};
use db::models::user::NewUser;
use db::queries::playlists::{self, PlaylistFilter};
use db::queries::{performances, users};
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

async fn titles(pool: &MySqlPool, filter: PlaylistFilter<'_>) -> Vec<String> {
    playlists::search(pool, filter, 10, 0)
        .await
        .unwrap()
        .into_iter()
        .map(|playlist| playlist.title)
        .collect()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_hides_private_and_favorites_by_default(pool: MySqlPool) {
    let user_id = create_user(&pool).await;
    create_playlist(&pool, "playlist_a", "user", true, Some(user_id)).await;
    create_playlist(&pool, "playlist_b", "user", false, Some(user_id)).await;
    create_playlist(&pool, "playlist_c", "favorites", true, Some(user_id)).await;

    let filter = PlaylistFilter::default();
    assert_eq!(titles(&pool, filter).await, vec!["playlist_a"]);
    assert_eq!(playlists::search_count(&pool, filter).await.unwrap(), 1);

    let filter = PlaylistFilter {
        include_private: true,
        ..PlaylistFilter::default()
    };
    assert_eq!(
        titles(&pool, filter).await,
        vec!["playlist_a", "playlist_b"]
    );
    assert_eq!(playlists::search_count(&pool, filter).await.unwrap(), 2);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_filters_by_owner_kind_and_title(pool: MySqlPool) {
    let user_id = create_user(&pool).await;
    create_playlist(&pool, "needle_a", "user", true, Some(user_id)).await;
    create_playlist(&pool, "playlist_b", "favorites", true, Some(user_id)).await;
    create_playlist(&pool, "needle_c", "user", true, None).await;

    let owned = PlaylistFilter {
        include_private: true,
        created_by: Some(user_id),
        ..PlaylistFilter::default()
    };
    assert_eq!(titles(&pool, owned).await, vec!["needle_a", "playlist_b"]);

    let favorites = PlaylistFilter {
        kind: Some("favorites"),
        ..owned
    };
    assert_eq!(titles(&pool, favorites).await, vec!["playlist_b"]);

    let by_title = PlaylistFilter {
        q: Some("needle"),
        ..PlaylistFilter::default()
    };
    assert_eq!(titles(&pool, by_title).await, vec!["needle_a", "needle_c"]);
    assert_eq!(playlists::search_count(&pool, by_title).await.unwrap(), 2);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn remove_performances_ignores_missing_entries(pool: MySqlPool) {
    let playlist = create_playlist(&pool, "playlist_a", "user", true, None).await;
    let performance_a = create_performance(&pool).await;
    let performance_b = create_performance(&pool).await;
    let performance_c = create_performance(&pool).await;

    let mut conn = pool.acquire().await.unwrap();
    playlists::add_performances(
        &mut conn,
        playlist.id,
        &[performance_a, performance_b, performance_c],
    )
    .await
    .unwrap();

    playlists::remove_performances(&pool, playlist.id, &[performance_a, Uuid::new_v4()])
        .await
        .unwrap();

    let remaining = playlists::get_performances_in_playlist(&pool, playlist.id)
        .await
        .unwrap();
    let ids: Vec<_> = remaining.iter().map(|row| row.performance.id).collect();
    assert_eq!(ids, vec![performance_b, performance_c]);
}
