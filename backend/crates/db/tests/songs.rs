use db::filter::{self, Clause, FilterTarget, Subject, TagScope};
use db::models::artist::{Artist, NewArtist};
use db::models::song::{NewSong, Song, UpdateSong};
use db::models::tag::NewTag;
use db::models::user::NewUser;
use db::queries::songs::{SongFilter, SortKey};
use db::queries::{artists, songs, tags, users};
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

async fn create_song(pool: &MySqlPool, title: &str, created_by: Option<Uuid>) -> Song {
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

async fn search_ids(
    pool: &MySqlPool,
    filter: SongFilter<'_>,
    direction: SortDirection,
) -> Vec<Uuid> {
    songs::search(pool, filter, SortKey::PerformanceCount, direction, 10, 0)
        .await
        .unwrap()
        .into_iter()
        .map(|song| song.id)
        .collect()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn crud_roundtrip(pool: MySqlPool) {
    let user_id = create_user(&pool).await;
    let created = create_song(&pool, "song_a", Some(user_id)).await;

    let fetched = songs::get_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(fetched.title, "song_a");
    assert_eq!(fetched.created_by, Some(user_id));
    assert!(fetched.lyrics_id.is_none());

    let mut conn = pool.acquire().await.unwrap();
    let updated = songs::update(
        &mut conn,
        created.id,
        &UpdateSong {
            title: "song_b".to_string(),
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(updated.title, "song_b");

    assert!(songs::delete(&pool, created.id).await.unwrap());
    assert!(songs::get_by_id(&pool, created.id).await.unwrap().is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_matches_title_artist_and_id(pool: MySqlPool) {
    let by_title = create_song(&pool, "needle_title", None).await;
    let by_artist = create_song(&pool, "song_b", None).await;
    let other = create_song(&pool, "song_c", None).await;

    let artist = create_artist(&pool, "needle_artist").await;
    let mut conn = pool.acquire().await.unwrap();
    songs::set_original_artists(&mut conn, by_artist.id, &[artist.id])
        .await
        .unwrap();

    let by_text = SongFilter {
        q: Some("needle"),
        ..SongFilter::default()
    };
    assert_eq!(
        search_ids(&pool, by_text, SortDirection::Desc).await,
        vec![by_artist.id, by_title.id]
    );
    assert_eq!(
        search_ids(&pool, by_text, SortDirection::Asc).await,
        vec![by_title.id, by_artist.id]
    );
    assert_eq!(songs::search_count(&pool, by_text).await.unwrap(), 2);

    let id_query = other.id.to_string();
    let by_id = SongFilter {
        q: Some(&id_query),
        ..SongFilter::default()
    };
    assert_eq!(
        search_ids(&pool, by_id, SortDirection::Desc).await,
        vec![other.id]
    );
    assert_eq!(songs::search_count(&pool, by_id).await.unwrap(), 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_filters_by_clauses_and_lists_used_catalog_values(pool: MySqlPool) {
    let tagged = create_song(&pool, "song_a", None).await;
    let untagged = create_song(&pool, "song_b", None).await;

    let mut conn = pool.acquire().await.unwrap();
    let tag = tags::get_or_create(
        &mut conn,
        &NewTag {
            name: "tag_a".to_string(),
        },
    )
    .await
    .unwrap();
    songs::set_tags(&mut conn, tagged.id, &[(tag.id, "genre")])
        .await
        .unwrap();

    let target = FilterTarget::Tag {
        kind: "genre",
        scope: TagScope::Song,
    };
    for (exclude, expected) in [(false, tagged.id), (true, untagged.id)] {
        let clauses = [Clause {
            target,
            ids: vec![tag.id],
            exclude,
        }];
        let filter = SongFilter {
            clauses: &clauses,
            ..SongFilter::default()
        };
        assert_eq!(
            search_ids(&pool, filter, SortDirection::Desc).await,
            vec![expected]
        );
    }

    let values = filter::catalog_values(&pool, Subject::Song, target)
        .await
        .unwrap();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].id, tag.id);

    let performance_only = FilterTarget::Tag {
        kind: "instrument",
        scope: TagScope::Performance,
    };
    let values = filter::catalog_values(&pool, Subject::Song, performance_only)
        .await
        .unwrap();
    assert!(values.is_empty());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn relations_are_replaced_on_set(pool: MySqlPool) {
    let song = create_song(&pool, "song_a", None).await;
    let artist_a = create_artist(&pool, "artist_a").await;
    let artist_b = create_artist(&pool, "artist_b").await;
    let mut conn = pool.acquire().await.unwrap();

    songs::set_original_artists(&mut conn, song.id, &[artist_a.id, artist_b.id])
        .await
        .unwrap();
    let both = songs::get_original_artists(&pool, song.id).await.unwrap();
    assert_eq!(both.len(), 2);

    songs::set_original_artists(&mut conn, song.id, &[artist_b.id])
        .await
        .unwrap();
    let only_b = songs::get_original_artists(&pool, song.id).await.unwrap();
    assert_eq!(only_b.len(), 1);
    assert_eq!(only_b[0].name, "artist_b");

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
    let song_tags = songs::get_tags(&pool, song.id).await.unwrap();
    assert_eq!(song_tags.len(), 1);
    assert_eq!(song_tags[0].name, "tag_a");
    assert_eq!(song_tags[0].kind, "kind_a");
}
