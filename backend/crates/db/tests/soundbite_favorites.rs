use chrono::NaiveDate;
use db::models::soundbite::{NewSoundbite, Soundbite};
use db::models::user::{NewUser, User};
use db::queries::soundbites::{SortKey, SoundbiteFilter};
use db::queries::{soundbite_favorites, soundbites, users};
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

async fn create_soundbite(pool: &MySqlPool, title: &str) -> Soundbite {
    let mut conn = pool.acquire().await.unwrap();
    soundbites::create(
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
    .unwrap()
}

fn sorted(mut ids: Vec<Uuid>) -> Vec<Uuid> {
    ids.sort();
    ids
}

async fn favorite_search(pool: &MySqlPool, filter: SoundbiteFilter<'_>) -> (Vec<Uuid>, u64) {
    let found = soundbites::search(
        pool,
        filter,
        SortKey::SoundbiteDate,
        SortDirection::Desc,
        10,
        0,
    )
    .await
    .unwrap();
    let total = soundbites::search_count(pool, filter).await.unwrap();
    (
        sorted(found.into_iter().map(|soundbite| soundbite.id).collect()),
        total,
    )
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn favorite_lifecycle_and_filter(pool: MySqlPool) {
    let user_a = create_user(&pool, "user_a").await;
    let user_b = create_user(&pool, "user_b").await;
    let soundbite_a = create_soundbite(&pool, "soundbite_a").await;
    let soundbite_b = create_soundbite(&pool, "soundbite_b").await;
    let soundbite_c = create_soundbite(&pool, "soundbite_c").await;

    soundbite_favorites::add(&pool, user_a.id, soundbite_a.id)
        .await
        .unwrap();
    soundbite_favorites::add(&pool, user_a.id, soundbite_a.id)
        .await
        .unwrap();
    soundbite_favorites::add(&pool, user_a.id, soundbite_b.id)
        .await
        .unwrap();
    soundbite_favorites::add(&pool, user_b.id, soundbite_c.id)
        .await
        .unwrap();

    let ids_a = sorted(
        soundbite_favorites::list_ids(&pool, user_a.id)
            .await
            .unwrap(),
    );
    assert_eq!(ids_a, sorted(vec![soundbite_a.id, soundbite_b.id]));
    assert_eq!(
        soundbite_favorites::list_ids(&pool, user_b.id)
            .await
            .unwrap(),
        vec![soundbite_c.id]
    );

    let favorites_of_a = SoundbiteFilter {
        favorited_by: Some(user_a.id),
        ..SoundbiteFilter::default()
    };
    assert_eq!(favorite_search(&pool, favorites_of_a).await, (ids_a, 2));

    let with_text = SoundbiteFilter {
        q: Some("soundbite_b"),
        ..favorites_of_a
    };
    assert_eq!(
        favorite_search(&pool, with_text).await,
        (vec![soundbite_b.id], 1)
    );
    let outside_favorites = SoundbiteFilter {
        q: Some("soundbite_c"),
        ..favorites_of_a
    };
    assert_eq!(favorite_search(&pool, outside_favorites).await, (vec![], 0));

    soundbite_favorites::remove(&pool, user_a.id, soundbite_a.id)
        .await
        .unwrap();
    assert_eq!(
        soundbite_favorites::list_ids(&pool, user_a.id)
            .await
            .unwrap(),
        vec![soundbite_b.id]
    );

    soundbites::delete(&pool, soundbite_b.id).await.unwrap();
    assert!(
        soundbite_favorites::list_ids(&pool, user_a.id)
            .await
            .unwrap()
            .is_empty()
    );
}
