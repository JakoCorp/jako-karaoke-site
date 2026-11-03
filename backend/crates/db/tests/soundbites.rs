use chrono::NaiveDate;
use db::error::DbError;
use db::models::artist::{Artist, NewArtist};
use db::models::asset::{Asset, NewInternalAsset};
use db::models::soundbite::{NewSoundbite, Soundbite, UpdateSoundbite};
use db::queries::{artists, assets, soundbites};
use sqlx::MySqlPool;
use uuid::Uuid;

fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, day).unwrap()
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

async fn create_asset(pool: &MySqlPool, hash_char: &str) -> Asset {
    let mut conn = pool.acquire().await.unwrap();
    assets::create_internal(
        &mut conn,
        &NewInternalAsset {
            title: None,
            credits: None,
            source_url: None,
            hash: hash_char.repeat(64),
            storage_url: format!("https://cdn.example.com/{hash_char}.mp3"),
            internal_path: None,
            length: Some(3),
        },
    )
    .await
    .unwrap()
}

async fn create_soundbite(
    pool: &MySqlPool,
    title: &str,
    soundbite_date: NaiveDate,
    stream_number: Option<u8>,
    soundbite_number: Option<u16>,
) -> Result<Soundbite, DbError> {
    let mut conn = pool.acquire().await.unwrap();
    soundbites::create(
        &mut conn,
        &NewSoundbite {
            created_by: None,
            title: title.to_string(),
            soundbite_date,
            stream_number,
            soundbite_number,
        },
    )
    .await
}

fn update_to(
    title: &str,
    soundbite_date: NaiveDate,
    stream_number: Option<u8>,
    soundbite_number: Option<u16>,
) -> UpdateSoundbite {
    UpdateSoundbite {
        title: title.to_string(),
        soundbite_date,
        stream_number,
        soundbite_number,
    }
}

async fn search_ids(pool: &MySqlPool, q: Option<&str>, limit: u32, offset: u32) -> Vec<Uuid> {
    soundbites::search(pool, q, limit, offset)
        .await
        .unwrap()
        .into_iter()
        .map(|soundbite| soundbite.id)
        .collect()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn lifecycle_and_stream_position_rules(pool: MySqlPool) {
    let asset_a = create_asset(&pool, "a").await;
    let asset_b = create_asset(&pool, "b").await;
    let artist_a = create_artist(&pool, "artist_a").await;
    let artist_b = create_artist(&pool, "artist_b").await;

    let created = create_soundbite(&pool, "soundbite_a", date(1), Some(1), Some(1))
        .await
        .unwrap();
    let fetched = soundbites::get_by_id(&pool, created.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.title, "soundbite_a");
    assert_eq!(fetched.asset_id, None);
    assert_eq!(fetched.soundbite_date, date(1));
    assert_eq!(fetched.stream_number, Some(1));
    assert_eq!(fetched.soundbite_number, Some(1));

    let duplicate = create_soundbite(&pool, "soundbite_b", date(1), Some(1), Some(1)).await;
    assert!(matches!(duplicate, Err(DbError::Conflict)));
    assert!(
        create_soundbite(&pool, "soundbite_b", date(1), Some(1), None)
            .await
            .is_err()
    );
    assert!(
        create_soundbite(&pool, "soundbite_b", date(1), None, Some(1))
            .await
            .is_err()
    );

    create_soundbite(&pool, "soundbite_b", date(1), None, None)
        .await
        .unwrap();
    create_soundbite(&pool, "soundbite_c", date(1), None, None)
        .await
        .unwrap();
    let other = create_soundbite(&pool, "soundbite_d", date(1), Some(1), Some(2))
        .await
        .unwrap();

    soundbites::set_asset(&pool, created.id, asset_a.id)
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let taken = soundbites::update(
        &mut conn,
        other.id,
        &update_to("soundbite_d", date(1), Some(1), Some(1)),
    )
    .await;
    assert!(matches!(taken, Err(DbError::Conflict)));

    let updated = soundbites::update(
        &mut conn,
        created.id,
        &update_to("soundbite_e", date(3), None, None),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(updated.title, "soundbite_e");
    assert_eq!(updated.soundbite_date, date(3));
    assert_eq!(updated.stream_number, None);
    assert_eq!(updated.soundbite_number, None);
    assert_eq!(updated.asset_id, Some(asset_a.id));

    soundbites::set_artists(&mut conn, created.id, &[artist_a.id])
        .await
        .unwrap();
    soundbites::set_artists(&mut conn, created.id, &[artist_b.id])
        .await
        .unwrap();
    let artists_by_soundbite = soundbites::get_artists_batch(&pool, &[created.id, other.id])
        .await
        .unwrap();
    assert_eq!(artists_by_soundbite.len(), 1);
    let names: Vec<_> = artists_by_soundbite[&created.id]
        .iter()
        .map(|artist| artist.name.as_str())
        .collect();
    assert_eq!(names, ["artist_b"]);

    soundbites::set_asset(&pool, created.id, asset_b.id)
        .await
        .unwrap();
    let swapped = soundbites::get_by_id(&pool, created.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(swapped.asset_id, Some(asset_b.id));

    assert!(soundbites::delete(&pool, created.id).await.unwrap());
    assert!(
        soundbites::get_by_id(&pool, created.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        soundbites::get_artists_batch(&pool, &[created.id])
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_orders_newest_first_and_matches_title_artist_and_id(pool: MySqlPool) {
    let first = create_soundbite(&pool, "alpha", date(1), Some(1), Some(1))
        .await
        .unwrap();
    let second = create_soundbite(&pool, "beta", date(1), Some(1), Some(2))
        .await
        .unwrap();
    let upload_a = create_soundbite(&pool, "gamma", date(1), None, None)
        .await
        .unwrap();
    let upload_b = create_soundbite(&pool, "delta", date(1), None, None)
        .await
        .unwrap();
    let later = create_soundbite(&pool, "epsilon", date(2), None, None)
        .await
        .unwrap();
    let expected = vec![later.id, second.id, first.id, upload_b.id, upload_a.id];

    assert_eq!(search_ids(&pool, None, 10, 0).await, expected);
    assert_eq!(search_ids(&pool, None, 2, 1).await, expected[1..3]);
    assert_eq!(soundbites::search_count(&pool, None).await.unwrap(), 5);

    let artist = create_artist(&pool, "artist_a").await;
    let mut conn = pool.acquire().await.unwrap();
    soundbites::set_artists(&mut conn, second.id, &[artist.id])
        .await
        .unwrap();

    let id_query = first.id.to_string();
    for (q, expected) in [
        ("alp", vec![first.id]),
        ("ARTIST_A", vec![second.id]),
        (id_query.as_str(), vec![first.id]),
        ("a", vec![second.id, first.id, upload_b.id, upload_a.id]),
        ("missing", vec![]),
    ] {
        assert_eq!(search_ids(&pool, Some(q), 10, 0).await, expected, "q = {q}");
        assert_eq!(
            soundbites::search_count(&pool, Some(q)).await.unwrap(),
            expected.len() as u64,
            "q = {q}"
        );
    }
}
