use db::models::artist::NewArtist;
use db::models::song::{NewSong, UpdateSong};
use db::models::tag::NewTag;
use db::models::user::NewUser;
use db::queries::{artists, songs, tags, users};
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

async fn create_song(
    pool: &MySqlPool,
    title: &str,
    created_by: Option<Uuid>,
) -> db::models::song::Song {
    let mut conn = pool.acquire().await.unwrap();
    songs::create(
        &mut conn,
        &NewSong {
            title: title.to_string(),
            created_by,
            lyrics_id: None,
        },
    )
    .await
    .unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_and_get_by_id(pool: MySqlPool) {
    let user_id = create_user(&pool).await;
    let created = create_song(&pool, "song_a", Some(user_id)).await;

    let fetched = songs::get_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.title, "song_a");
    assert_eq!(fetched.created_by, Some(user_id));
    assert!(fetched.lyrics_id.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn list_returns_newest_first(pool: MySqlPool) {
    let song_a = create_song(&pool, "song_a", None).await;
    let song_b = create_song(&pool, "song_b", None).await;

    let results = songs::list(&pool, 10, 0).await.unwrap();
    assert_eq!(results[0].id, song_b.id);
    assert_eq!(results[1].id, song_a.id);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn update_title(pool: MySqlPool) {
    let created = create_song(&pool, "title_a", None).await;

    let mut conn = pool.acquire().await.unwrap();
    let updated = songs::update(
        &mut conn,
        created.id,
        &UpdateSong {
            title: "title_b".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(updated.title, "title_b");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn set_and_get_original_artists(pool: MySqlPool) {
    let song = create_song(&pool, "song_a", None).await;
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

    songs::set_original_artists(&mut conn, song.id, &[artist_a.id, artist_b.id])
        .await
        .unwrap();

    let result = songs::get_original_artists(&pool, song.id).await.unwrap();
    let mut names: Vec<&str> = result.iter().map(|a| a.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(names, vec!["artist_a", "artist_b"]);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn set_original_artists_replaces_existing(pool: MySqlPool) {
    let song = create_song(&pool, "song_a", None).await;
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

    songs::set_original_artists(&mut conn, song.id, &[artist_a.id])
        .await
        .unwrap();
    songs::set_original_artists(&mut conn, song.id, &[artist_b.id])
        .await
        .unwrap();

    let result = songs::get_original_artists(&pool, song.id).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "artist_b");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn set_and_get_tags(pool: MySqlPool) {
    let song = create_song(&pool, "song_a", None).await;
    let mut conn = pool.acquire().await.unwrap();

    let tag = tags::get_or_create(
        &mut conn,
        &NewTag {
            name: "tag_a".to_string(),
        },
    )
    .await
    .unwrap();
    songs::set_tags(&mut conn, song.id, &[(tag.id, "kind_a")])
        .await
        .unwrap();

    let result = songs::get_tags(&pool, song.id).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "tag_a");
    assert_eq!(result[0].kind, "kind_a");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn delete_removes_song(pool: MySqlPool) {
    let created = create_song(&pool, "song_a", None).await;

    let deleted = songs::delete(&pool, created.id).await.unwrap();
    assert!(deleted);

    let fetched = songs::get_by_id(&pool, created.id).await.unwrap();
    assert!(fetched.is_none());
}
