//! Performance CRUD handlers, media upload/delete, and the `PerformancesApi` OpenAPI spec struct.

pub(crate) mod lyrics;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use tracing::error;
use uuid::Uuid;

use api_types::{
    common::{ArtistInfo, ErrorResponse, MediaInfo, TagInfo},
    lyrics::{LyricsResponse, UpdateLyricsRequest},
    pagination::{PagedResponse, defaults as pagination_defaults},
    performances::{
        CreatePerformanceRequest, PerformanceResponse, PerformanceSummary,
        PerformanceTagAssignment, UpdatePerformanceRequest,
    },
    songs::{SongRef, SongSummary},
    tags::PerformanceTagKind,
};
use db::{
    MySqlPool,
    error::DbError,
    models::{
        NewLyrics, NewPerformance, NewPerformanceAudio, NewPerformanceVideo, UpdatePerformance,
        performance::Performance,
    },
    queries,
};

use crate::{
    auth::middleware::AuthUser, capabilities, error::ApiError, media, pagination, state::AppState,
};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_performances,
        get_performance,
        create_performance,
        update_performance,
        delete_performance,
        upload_audio,
        delete_audio,
        upload_video,
        delete_video,
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
        MediaInfo,
        FileUpload,
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
    /// Field to sort by. Defaults to `performance_date`.
    pub sort: Option<PerformanceSort>,
    /// Sort direction. Defaults to `desc`.
    pub sort_dir: Option<SortDir>,
}

impl PerformanceListParams {
    fn sort_col(&self) -> &'static str {
        match &self.sort {
            Some(PerformanceSort::PlayCount) => "play_count",
            Some(PerformanceSort::Duration) => "duration",
            _ => "performance_date",
        }
    }

    fn sort_dir_str(&self) -> &'static str {
        match &self.sort_dir {
            Some(SortDir::Asc) => "ASC",
            _ => "DESC",
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

/// Sort direction for list endpoints.
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SortDir {
    Asc,
    Desc,
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
                singers,
                songs,
            }
        })
        .collect();

    Ok(items)
}

/// Placeholder schema for multipart file upload bodies.
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
pub(crate) struct FileUpload {
    #[schema(format = Binary)]
    pub file: Vec<u8>,
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
            post(upload_audio).layer(DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route("/{id}/audio/{audio_id}", delete(delete_audio))
        .route(
            "/{id}/video",
            post(upload_video).layer(DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route("/{id}/video/{video_id}", delete(delete_video))
        .route(
            "/{id}/lyrics",
            get(lyrics::get_performance_lyrics)
                .put(lyrics::put_performance_lyrics)
                .delete(lyrics::delete_performance_lyrics),
        )
}

/// Loads all related entities for a performance row into a full [`PerformanceResponse`].
async fn hydrate(
    pool: &MySqlPool,
    perf: db::models::Performance,
) -> Result<PerformanceResponse, ApiError> {
    let (songs, singers, tags, audio, video) = tokio::try_join!(
        queries::performances::get_songs(pool, perf.id),
        queries::performances::get_singers(pool, perf.id),
        queries::performances::get_tags(pool, perf.id),
        queries::performance_audios::list_for_performance(pool, perf.id),
        queries::performance_videos::list_for_performance(pool, perf.id),
    )?;

    let songs = songs
        .into_iter()
        .map(|s| SongSummary {
            id: s.id,
            title: s.title,
            date_added: s.date_added,
            artists: vec![],
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

    let audio = audio
        .into_iter()
        .map(|a| MediaInfo {
            id: a.id,
            public_url: a.public_url,
        })
        .collect();

    let video = video
        .into_iter()
        .map(|v| MediaInfo {
            id: v.id,
            public_url: v.public_url,
        })
        .collect();

    Ok(PerformanceResponse {
        id: perf.id,
        title: perf.title,
        play_count: perf.play_count,
        duration: perf.duration,
        performance_date: perf.performance_date,
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

/// Reads the `file` field from a multipart body and returns its bytes, content type, and filename.
async fn read_file_field(
    multipart: &mut Multipart,
) -> Result<(Vec<u8>, String, Option<String>), ApiError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("multipart field error: {e:?}");
        ApiError::BadRequest(e.to_string())
    })? {
        if field.name() == Some("file") {
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let filename = field.file_name().map(str::to_string);
            let data = field
                .bytes()
                .await
                .map_err(|e| {
                    error!("multipart read error: {e:?}");
                    ApiError::BadRequest(e.to_string())
                })?
                .to_vec();
            return Ok((data, content_type, filename));
        }
    }
    Err(ApiError::BadRequest("missing 'file' field".into()))
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

    let (total, perfs) = tokio::try_join!(
        queries::performances::search_count(&state.pool, q),
        queries::performances::search(
            &state.pool,
            q,
            params.sort_col(),
            params.sort_dir_str(),
            limit,
            offset,
        ),
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
            performance_date: req.performance_date,
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
            performance_date: req.performance_date,
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

#[utoipa::path(
    post,
    path = "/api/performances/{id}/audio",
    params(("id" = Uuid, Path, description = "Performance ID")),
    request_body(content = FileUpload, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Audio uploaded", body = MediaInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn upload_audio(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<MediaInfo>), ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let (data, content_type, filename) = read_file_field(&mut multipart).await?;
    let ext = media::resolve_ext(media::MediaKind::Audio, &content_type, filename.as_deref())?;
    let saved = state.store.save("audio", ext, &data).await?;

    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let audio = queries::performance_audios::create(
        &mut conn,
        &NewPerformanceAudio {
            performance_id: id,
            public_url: saved.public_url,
            internal_path: Some(saved.internal_path),
        },
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(MediaInfo {
            id: audio.id,
            public_url: audio.public_url,
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/performances/{id}/audio/{audio_id}",
    params(
        ("id" = Uuid, Path, description = "Performance ID"),
        ("audio_id" = Uuid, Path, description = "Audio record ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn delete_audio(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, audio_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    let audio = queries::performance_audios::get_by_id(&state.pool, audio_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if audio.performance_id != id {
        return Err(ApiError::NotFound);
    }
    queries::performance_audios::delete(&state.pool, audio_id).await?;
    if let Some(path) = &audio.internal_path
        && let Err(e) = state.store.delete(path).await
    {
        error!("failed to delete audio file {path}: {e}");
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/performances/{id}/video",
    params(("id" = Uuid, Path, description = "Performance ID")),
    request_body(content = FileUpload, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Video uploaded", body = MediaInfo),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Performance not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn upload_video(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<MediaInfo>), ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    queries::performances::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let (data, content_type, filename) = read_file_field(&mut multipart).await?;
    let ext = media::resolve_ext(media::MediaKind::Video, &content_type, filename.as_deref())?;
    let saved = state.store.save("video", ext, &data).await?;

    let mut conn = state.pool.acquire().await.map_err(DbError::Sqlx)?;
    let video = queries::performance_videos::create(
        &mut conn,
        &NewPerformanceVideo {
            performance_id: id,
            public_url: saved.public_url,
            internal_path: Some(saved.internal_path),
        },
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(MediaInfo {
            id: video.id,
            public_url: video.public_url,
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/performances/{id}/video/{video_id}",
    params(
        ("id" = Uuid, Path, description = "Performance ID"),
        ("video_id" = Uuid, Path, description = "Video record ID"),
    ),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "performances",
    security(("session" = []))
)]
pub(crate) async fn delete_video(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, video_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::PERFORMANCES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    let video = queries::performance_videos::get_by_id(&state.pool, video_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if video.performance_id != id {
        return Err(ApiError::NotFound);
    }
    queries::performance_videos::delete(&state.pool, video_id).await?;
    if let Some(path) = &video.internal_path
        && let Err(e) = state.store.delete(path).await
    {
        error!("failed to delete video file {path}: {e}");
    }
    Ok(StatusCode::NO_CONTENT)
}
