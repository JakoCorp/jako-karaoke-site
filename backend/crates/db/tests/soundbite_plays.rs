use chrono::NaiveDate;
use db::models::asset::NewInternalAsset;
use db::models::soundbite::{NewSoundbite, Soundbite};
use db::models::user::{NewUser, User};
use db::queries::soundbites::{SortKey, SoundbiteFilter};
use db::queries::{assets, soundbite_plays, soundbites, users};
use db::sort::SortDirection;
use sqlx::MySqlPool;
use uuid::Uuid;

async fn create_user(pool: &MySqlPool, username: &str) -> User {
    let mut conn = pool.acquire().await.unwrap();
    users::create(
        &mut conn,
        &NewUser {
            username: username.to_string(),
            twitch_id: None,
            discord_id: None,
        },
    )
    .await
    .unwrap()
}

async fn create_soundbite(pool: &MySqlPool, title: &str, length: Option<u32>) -> Soundbite {
    let mut conn = pool.acquire().await.unwrap();
    let soundbite = soundbites::create(
        &mut conn,
        &NewSoundbite {
            created_by: None,
            title: title.to_string(),
            soundbite_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            stream_number: None,
            soundbite_number: None,
        },
    )
    .await
    .unwrap();

    if let Some(length) = length {
        let asset = assets::create_internal(
            &mut conn,
            &NewInternalAsset {
                title: None,
                credits: None,
                source_url: None,
                hash: format!("{title:0>64}"),
                storage_url: format!("https://cdn.example.com/{title}.mp3"),
                internal_path: None,
                length: Some(length),
            },
        )
        .await
        .unwrap();
        soundbites::set_asset(&mut *conn, soundbite.id, asset.id)
            .await
            .unwrap();
    }
    soundbite
}

async fn count_play(pool: &MySqlPool, soundbite_id: Uuid, user_id: Uuid) {
    let mut conn = pool.acquire().await.unwrap();
    let play = soundbite_plays::create(&mut conn, soundbite_id, user_id)
        .await
        .unwrap();
    soundbite_plays::mark_counted(&mut *conn, play.id)
        .await
        .unwrap();
}

async fn sorted_ids(pool: &MySqlPool, sort: SortKey, direction: SortDirection) -> Vec<Uuid> {
    soundbites::search(pool, SoundbiteFilter::default(), sort, direction, 10, 0)
        .await
        .unwrap()
        .into_iter()
        .map(|soundbite| soundbite.id)
        .collect()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn session_lifecycle_and_cooldown_scope(pool: MySqlPool) {
    let user_a = create_user(&pool, "user_a").await;
    let user_b = create_user(&pool, "user_b").await;
    let soundbite_a = create_soundbite(&pool, "soundbite_a", None).await;
    let soundbite_b = create_soundbite(&pool, "soundbite_b", None).await;

    let mut conn = pool.acquire().await.unwrap();
    let first = soundbite_plays::create(&mut conn, soundbite_a.id, user_a.id)
        .await
        .unwrap();
    assert_eq!(first.counted_at, None);

    let progress = soundbite_plays::get_progress_for_update(&mut conn, first.id, user_a.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(progress.soundbite_id, soundbite_a.id);
    assert!(!progress.counted);
    assert!(
        soundbite_plays::get_progress_for_update(&mut conn, first.id, user_b.id)
            .await
            .unwrap()
            .is_none()
    );

    soundbite_plays::delete_pending_for_user(&pool, user_a.id)
        .await
        .unwrap();
    assert!(
        soundbite_plays::get_progress_for_update(&mut conn, first.id, user_a.id)
            .await
            .unwrap()
            .is_none()
    );

    let second = soundbite_plays::create(&mut conn, soundbite_a.id, user_a.id)
        .await
        .unwrap();
    soundbite_plays::mark_counted(&pool, second.id)
        .await
        .unwrap();
    let counts = soundbite_plays::counts_batch(&pool, &[soundbite_a.id, soundbite_b.id])
        .await
        .unwrap();
    assert_eq!(counts.len(), 1);
    assert_eq!(counts[&soundbite_a.id], 1);
    assert_eq!(
        soundbite_plays::count_for_soundbite(&pool, soundbite_a.id)
            .await
            .unwrap(),
        1
    );

    assert!(
        soundbite_plays::seconds_since_last_counted(&pool, user_a.id, soundbite_a.id)
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        soundbite_plays::seconds_since_last_counted(&pool, user_a.id, soundbite_b.id)
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        soundbite_plays::seconds_since_last_counted(&pool, user_b.id, soundbite_a.id)
            .await
            .unwrap(),
        None
    );

    soundbite_plays::delete_pending_for_user(&pool, user_a.id)
        .await
        .unwrap();
    soundbite_plays::delete(&pool, second.id).await.unwrap();
    assert_eq!(
        soundbite_plays::count_for_soundbite(&pool, soundbite_a.id)
            .await
            .unwrap(),
        0
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn sorts_by_play_count_and_duration(pool: MySqlPool) {
    let user_a = create_user(&pool, "user_a").await;
    let user_b = create_user(&pool, "user_b").await;
    let long = create_soundbite(&pool, "long", Some(10)).await;
    let short = create_soundbite(&pool, "short", Some(5)).await;
    let unknown = create_soundbite(&pool, "unknown", None).await;

    count_play(&pool, short.id, user_a.id).await;
    count_play(&pool, short.id, user_b.id).await;
    count_play(&pool, unknown.id, user_a.id).await;

    let by_plays = [short.id, unknown.id, long.id];
    assert_eq!(
        sorted_ids(&pool, SortKey::PlayCount, SortDirection::Desc).await,
        by_plays
    );
    let mut ascending = sorted_ids(&pool, SortKey::PlayCount, SortDirection::Asc).await;
    ascending.reverse();
    assert_eq!(ascending, by_plays);

    let by_length = [long.id, short.id, unknown.id];
    assert_eq!(
        sorted_ids(&pool, SortKey::Duration, SortDirection::Desc).await,
        by_length
    );
    let mut ascending = sorted_ids(&pool, SortKey::Duration, SortDirection::Asc).await;
    ascending.reverse();
    assert_eq!(ascending, by_length);
}
