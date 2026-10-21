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
    let found = playlists::search(pool, filter, 10, 0).await.unwrap();
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
