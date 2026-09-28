//! Performance CRUD handlers, media upload/delete, and the `PerformancesApi` OpenAPI spec struct.

pub(crate) mod audio;
pub(crate) mod lyrics;
pub(crate) mod video;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use uuid::Uuid;

use api_types::{
    assets::AssetInfo,
    common::{ArtistInfo, ErrorResponse, TagInfo},
    lyrics::{LyricsResponse, UpdateLyricsRequest},
    pagination::{PagedResponse, defaults as pagination_defaults},
    performances::{
        AddAudioLinkRequest, AddVideoLinkRequest, AudioInfo, AudioKind, CreatePerformanceRequest,
        PerformanceResponse, PerformanceSummary, PerformanceTagAssignment, UpdateAudioKindRequest,
        UpdatePerformanceRequest, UpdateVideoKindRequest, VideoInfo, VideoKind,
    },
    songs::{SongRef, SongSummary},
    tags::PerformanceTagKind,
};
use db::{
    MySqlPool,
    error::DbError,
    models::{NewLyrics, NewPerformance, UpdatePerformance, performance::Performance},
    queries,
};

use crate::{
    auth::middleware::AuthUser, capabilities, convert, error::ApiError, pagination,
    routes::common::SortDir, state::AppState,
};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_performances,
        get_performance,
        create_performance,
        update_performance,
        delete_performance,
        audio::upload_audio,
        audio::add_audio_link,
        audio::update_audio_kind,
        audio::delete_audio,
        video::upload_video,
        video::add_video_link,
        video::update_video_kind,
        video::delete_video,
        lyrics::get_performance_lyrics,
        lyrics::put_performance_lyrics,
        lyrics::delete_performance_lyrics,
    ),
    components(schemas(
        PerformanceSummary,
        PerformanceResponse,
        CreatePerformanceRequest,
        UpdatePerformanceRequest,
        PerformanceTagAssignment,
        PerformanceTagKind,
        PerformanceSort,
        SortDir,
        SongRef,
        SongSummary,
        ArtistInfo,
        TagInfo,
        AssetInfo,
        AudioInfo,
        AudioKind,
        VideoInfo,
        VideoKind,
        audio::AudioUpload,
        video::VideoUpload,
        AddAudioLinkRequest,
        AddVideoLinkRequest,
        UpdateAudioKindRequest,
        UpdateVideoKindRequest,
        LyricsResponse,
        UpdateLyricsRequest,
        ErrorResponse,
        PagedResponse<PerformanceSummary>,
    ))
)]
pub(crate) struct PerformancesApi;

/// Query parameters for `GET /api/performances`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct PerformanceListParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "pagination_defaults::page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "pagination_defaults::per_page")]
    pub per_page: u32,
    /// Text search across performance title, song title, and singer names.
    pub q: Option<String>,
    /// Filter by song ID. Returns only performances that include this song.
    pub song_id: Option<Uuid>,
    /// Field to sort by. Defaults to `performance_date`.
    pub sort: Option<PerformanceSort>,
    /// Sort direction. Defaults to `desc`.
    pub sort_dir: Option<SortDir>,
}

impl PerformanceListParams {
    fn sort_dir_str(&self) -> &'static str {
        self.sort_dir.as_ref().map_or("DESC", SortDir::as_str)
    }

    fn order_by_clause(&self) -> String {
        let dir = self.sort_dir_str();
        let id_tiebreak =
            format!("performance_date {dir}, stream_number {dir}, performance_number {dir}");
        match &self.sort {
            Some(PerformanceSort::PlayCount) => format!("play_count {dir}, {id_tiebreak}"),
            Some(PerformanceSort::Duration) => format!("duration {dir}, {id_tiebreak}"),
            _ => id_tiebreak,
        }
    }
}

/// Field to sort performances by in list endpoints.
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PerformanceSort {
    PerformanceDate,
    PlayCount,
    Duration,
}

/// Hydrates a list of performances with singers and songs for use in summary responses.
pub(crate) async fn build_performance_summaries(
    pool: &MySqlPool,
    performances: Vec<Performance>,
) -> Result<Vec<PerformanceSummary>, ApiError> {
    let perf_ids: Vec<Uuid> = performances.iter().map(|p| p.id).collect();

    let (mut singers_by_perf, mut songs_by_perf) = tokio::try_join!(
        queries::performances::get_singers_batch(pool, &perf_ids),
        queries::performances::get_songs_batch(pool, &perf_ids),
    )?;

    let items = performances
        .into_iter()
        .map(|p| {
            let singers = singers_by_perf
                .remove(&p.id)
                .unwrap_or_default()
                .into_iter()
                .map(|a| ArtistInfo {
                    id: a.id,
                    name: a.name,
                    description: a.description,
                })
                .collect();

            let songs = songs_by_perf
                .remove(&p.id)
                .unwrap_or_default()
                .into_iter()
                .map(|(id, title)| SongRef { id, title })
                .collect();

            PerformanceSummary {
                id: p.id,
                title: p.title,
                play_count: p.play_count,
                duration: p.duration,
                performance_date: p.performance_date,
                stream_number: p.stream_number,
                performance_number: p.performance_number,
                singers,
                songs,
            }
        })
        .collect();

    Ok(items)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_performances).post(create_performance))
        .route(
            "/{id}",
            get(get_performance)
                .put(update_performance)
                .delete(delete_performance),
        )
        .route(
            "/{id}/audio",
            post(audio::upload_audio).layer(DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route("/{id}/audio/link", post(audio::add_audio_link))
        .route(
            "/{id}/audio/{asset_id}",
            patch(audio::update_audio_kind).delete(audio::delete_audio),
        )
        .route(
            "/{id}/video",
            post(video::upload_video).layer(DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route("/{id}/video/link", post(video::add_video_link))
        .route(
            "/{id}/video/{asset_id}",
            patch(video::update_video_kind).delete(video::delete_video),
        )
        .route(
            "/{id}/lyrics",
            get(lyrics::get_performance_lyrics)
                .put(lyrics::put_performance_lyrics)
                .delete(lyrics::delete_performance_lyrics),
        )
}

async fn hydrate(
    pool: &MySqlPool,
    perf: db::models::Performance,
) -> Result<PerformanceResponse, ApiError> {
    let (songs, singers, tags, audio_rows, video_rows) = tokio::try_join!(
        queries::performances::get_songs(pool, perf.id),
        queries::performances::get_singers(pool, perf.id),
        queries::performances::get_tags(pool, perf.id),
        queries::performance_audios::list_for_performance(pool, perf.id),
        queries::performance_videos::list_for_performance(pool, perf.id),
    )?;

    let song_ids: Vec<Uuid> = songs.iter().map(|s| s.id).collect();
    let mut images_by_song = queries::songs::get_images_batch(pool, &song_ids).await?;

    let songs = songs
        .into_iter()
        .map(|s| {
            let images = images_by_song
                .remove(&s.id)
                .unwrap_or_default()
                .into_iter()
                .map(convert::song_image_info)
                .collect();
            SongSummary {
                id: s.id,
                title: s.title,
                artists: vec![],
                images,
                performance_count: 0,
            }
        })
        .collect();

    let singers = singers
        .into_iter()
        .map(|a| ArtistInfo {
            id: a.id,
            name: a.name,
            description: a.description,
        })
        .collect();

    let tags = tags
        .into_iter()
        .map(|t| TagInfo {
            id: t.id,
            name: t.name,
            kind: t.kind,
        })
        .collect();

    let audio = audio_rows.into_iter().map(convert::audio_info).collect();
    let video = video_rows.into_iter().map(convert::video_info).collect();

    Ok(PerformanceResponse {
        id: perf.id,
        title: perf.title,
        play_count: perf.play_count,
        duration: perf.duration,
        stream_time: perf.stream_time,
        performance_date: perf.performance_date,
        stream_number: perf.stream_number,
        performance_number: perf.performance_number,
        songs,
        singers,
        tags,
        audio,
        video,
    })
}

fn tag_pairs(assignments: &[PerformanceTagAssignment]) -> Vec<(Uuid, &str)> {
    assignments
        .iter()
        .map(|a| (a.tag_id, a.kind.as_str()))
        .collect()
}

#[utoipa::path(
    get,
    path = "/api/performances",
    params(PerformanceListParams),
    responses(
        (status = 200, description = "Paged list of performances", body = PagedResponse<PerformanceSummary>),
    ),
    tag = "performances"
)]
pub(crate) async fn list_performances(
    State(state): State<AppState>,
    Query(params): Query<PerformanceListParams>,
) -> Result<Json<PagedResponse<PerformanceSummary>>, ApiError> {
    let (limit, offset) = pagination::limit_offset(params.page, params.per_page);
    let q = params.q.as_deref().filter(|s| !s.is_empty());

    let order_by = params.order_by_clause();
    let (total, perfs) = tokio::try_join!(
        queries::performances::search_count(&state.pool, q, params.song_id),
        queries::performances::search(&state.pool, q, params.song_id, &order_by, limit, offset),
    )?;

    let items = build_performance_summaries(&state.pool, perfs).await?;

    Ok(Json(PagedResponse {
        items,
        total,
        page: params.page,
        per_page: limit,
    }))
}

#[utoipa::path(
    get,
    path = "/api/performances/{id}",
    params(("id" = Uuid, Path, description = "Performance ID")),
    responses(
        (status = 200, description = "Performance detail", body = PerformanceResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances"
)]
pub(crate) async fn get_performance(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PerformanceResponse>, ApiError> {
    let perf = queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(hydrate(&state.pool, perf).await?))
}

#[utoipa::path(
    post,
    path = "/api/performances",
    request_body = CreatePerformanceRequest,
    responses(
        (status = 201, description = "Created performance", body = PerformanceResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn create_performance(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreatePerformanceRequest>,
) -> Result<(StatusCode, Json<PerformanceResponse>), ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;

    let lyrics_id = match req.lyrics {
        Some(content) => {
            let l = queries::lyrics::create(&mut tx, &NewLyrics { content }).await?;
            Some(l.id)
        }
        None => None,
    };

    let perf = queries::performances::create(
        &mut tx,
        &NewPerformance {
            created_by: None,
            title: req.title,
            lyrics_id,
            duration: req.duration,
            stream_time: req.stream_time,
            performance_date: req.performance_date,
            stream_number: req.stream_number,
            performance_number: req.performance_number,
        },
    )
    .await?;

    let tag_pairs = tag_pairs(&req.tags);
    queries::performances::set_songs(&mut tx, perf.id, &req.song_ids).await?;
    queries::performances::set_singers(&mut tx, perf.id, &req.singer_ids).await?;
    queries::performances::set_tags(&mut tx, perf.id, &tag_pairs).await?;

    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok((StatusCode::CREATED, Json(hydrate(&state.pool, perf).await?)))
}

#[utoipa::path(
    put,
    path = "/api/performances/{id}",
    params(("id" = Uuid, Path, description = "Performance ID")),
    request_body = UpdatePerformanceRequest,
    responses(
        (status = 200, description = "Updated performance", body = PerformanceResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn update_performance(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePerformanceRequest>,
) -> Result<Json<PerformanceResponse>, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;

    let perf = queries::performances::update(
        &mut tx,
        id,
        &UpdatePerformance {
            title: req.title,
            duration: req.duration,
            stream_time: req.stream_time,
            performance_date: req.performance_date,
            stream_number: req.stream_number,
            performance_number: req.performance_number,
        },
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    let tag_pairs = tag_pairs(&req.tags);
    queries::performances::set_songs(&mut tx, id, &req.song_ids).await?;
    queries::performances::set_singers(&mut tx, id, &req.singer_ids).await?;
    queries::performances::set_tags(&mut tx, id, &tag_pairs).await?;

    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok(Json(hydrate(&state.pool, perf).await?))
}

#[utoipa::path(
    delete,
    path = "/api/performances/{id}",
    params(("id" = Uuid, Path, description = "Performance ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn delete_performance(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    let found = queries::performances::delete(&state.pool, id).await?;
    if found {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}
