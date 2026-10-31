//! Soundbite CRUD handlers and the `SoundbitesApi` OpenAPI spec struct.

pub(crate) mod audio;
mod params;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::StatusCode,
    routing::{get, put},
};
use uuid::Uuid;

use api_types::{
    assets::AssetInfo,
    common::{ArtistInfo, ErrorResponse},
    pagination::PagedResponse,
    soundbites::{CreateSoundbiteRequest, SoundbiteResponse, UpdateSoundbiteRequest},
};
use db::{
    MySqlPool,
    error::DbError,
    models::{NewSoundbite, Soundbite, UpdateSoundbite},
    queries,
};

use crate::{
    assets,
    auth::middleware::AuthUser,
    capabilities, convert,
    error::ApiError,
    pagination,
    routes::common::{search_text, validate_stream_position},
    state::AppState,
};
use params::SoundbiteListParams;

/// Folder of the file store that holds soundbite audio.
const AUDIO_SUBFOLDER: &str = "soundbites";

const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;

const MAX_TITLE_CHARS: usize = 256;

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_soundbites,
        create_soundbite,
        update_soundbite,
        delete_soundbite,
        audio::upload_soundbite_audio,
    ),
    components(schemas(
        SoundbiteResponse,
        CreateSoundbiteRequest,
        UpdateSoundbiteRequest,
        ArtistInfo,
        AssetInfo,
        audio::SoundbiteAudioUpload,
        ErrorResponse,
        PagedResponse<SoundbiteResponse>,
    ))
)]
pub(crate) struct SoundbitesApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_soundbites).post(create_soundbite))
        .route("/{id}", put(update_soundbite).delete(delete_soundbite))
        .route(
            "/{id}/audio",
            put(audio::upload_soundbite_audio).layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES)),
        )
}

fn missing_audio() -> ApiError {
    ApiError::Internal("soundbite audio asset is missing".to_string())
}

/// Trims the title and rejects a blank or overlong one.
fn validate_title(title: &str) -> Result<String, ApiError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(ApiError::BadRequest("title must not be blank".to_string()));
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err(ApiError::BadRequest(format!(
            "title must be at most {MAX_TITLE_CHARS} characters"
        )));
    }
    Ok(title.to_string())
}

/// Hydrates a list of soundbites with their artists and audio for use in list responses.
async fn build_soundbite_responses(
    pool: &MySqlPool,
    soundbites: Vec<Soundbite>,
) -> Result<Vec<SoundbiteResponse>, ApiError> {
    let soundbite_ids: Vec<Uuid> = soundbites.iter().map(|s| s.id).collect();
    let asset_ids: Vec<Uuid> = soundbites.iter().filter_map(|s| s.asset_id).collect();

    let (mut artists_by_soundbite, assets_by_id) = tokio::try_join!(
        queries::soundbites::get_artists_batch(pool, &soundbite_ids),
        queries::assets::get_by_ids(pool, &asset_ids),
    )?;

    soundbites
        .into_iter()
        .map(|soundbite| {
            let asset = soundbite
                .asset_id
                .map(|asset_id| {
                    assets_by_id
                        .get(&asset_id)
                        .cloned()
                        .ok_or_else(missing_audio)
                })
                .transpose()?;
            let artists = artists_by_soundbite
                .remove(&soundbite.id)
                .unwrap_or_default();
            Ok(convert::soundbite_response(soundbite, artists, asset))
        })
        .collect()
}

async fn hydrate(pool: &MySqlPool, soundbite: Soundbite) -> Result<SoundbiteResponse, ApiError> {
    let (mut artists_by_soundbite, asset) = tokio::try_join!(
        queries::soundbites::get_artists_batch(pool, std::slice::from_ref(&soundbite.id)),
        async {
            match soundbite.asset_id {
                Some(asset_id) => queries::assets::get_by_id(pool, asset_id).await,
                None => Ok(None),
            }
        },
    )?;
    if soundbite.asset_id.is_some() && asset.is_none() {
        return Err(missing_audio());
    }
    let artists = artists_by_soundbite
        .remove(&soundbite.id)
        .unwrap_or_default();
    Ok(convert::soundbite_response(soundbite, artists, asset))
}

#[utoipa::path(
    get,
    path = "/api/soundbites",
    params(SoundbiteListParams),
    responses(
        (status = 200, description = "Paged list of soundbites, newest first", body = PagedResponse<SoundbiteResponse>),
    ),
    tag = "soundbites"
)]
pub(crate) async fn list_soundbites(
    State(state): State<AppState>,
    Query(params): Query<SoundbiteListParams>,
) -> Result<Json<PagedResponse<SoundbiteResponse>>, ApiError> {
    let (limit, offset) = pagination::limit_offset(params.page, params.per_page);
    let q = search_text(params.q.as_deref());

    let (total, soundbites) = tokio::try_join!(
        queries::soundbites::search_count(&state.pool, q),
        queries::soundbites::search(&state.pool, q, limit, offset),
    )?;

    let items = build_soundbite_responses(&state.pool, soundbites).await?;

    Ok(Json(PagedResponse {
        items,
        total,
        page: params.page,
        per_page: limit,
    }))
}

#[utoipa::path(
    post,
    path = "/api/soundbites",
    request_body = CreateSoundbiteRequest,
    responses(
        (status = 201, description = "Created soundbite without audio", body = SoundbiteResponse),
        (status = 400, description = "Invalid title or stream position", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 409, description = "A soundbite already has this date, stream number and soundbite number", body = ErrorResponse),
    ),
    tag = "soundbites",
    security(("session" = []))
)]
pub(crate) async fn create_soundbite(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateSoundbiteRequest>,
) -> Result<(StatusCode, Json<SoundbiteResponse>), ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::SOUNDBITES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    let title = validate_title(&req.title)?;
    validate_stream_position(req.stream_number, req.soundbite_number, "soundbite_number")?;
    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;

    let soundbite = queries::soundbites::create(
        &mut tx,
        &NewSoundbite {
            created_by: Some(auth.user_id),
            title,
            soundbite_date: req.soundbite_date,
            stream_number: req.stream_number,
            soundbite_number: req.soundbite_number,
        },
    )
    .await?;

    queries::soundbites::set_artists(&mut tx, soundbite.id, &req.artist_ids).await?;

    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok((
        StatusCode::CREATED,
        Json(hydrate(&state.pool, soundbite).await?),
    ))
}

#[utoipa::path(
    put,
    path = "/api/soundbites/{id}",
    params(("id" = Uuid, Path, description = "Soundbite ID")),
    request_body = UpdateSoundbiteRequest,
    responses(
        (status = 200, description = "Updated soundbite", body = SoundbiteResponse),
        (status = 400, description = "Invalid title or stream position", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 409, description = "A soundbite already has this date, stream number and soundbite number", body = ErrorResponse),
    ),
    tag = "soundbites",
    security(("session" = []))
)]
pub(crate) async fn update_soundbite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSoundbiteRequest>,
) -> Result<Json<SoundbiteResponse>, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::SOUNDBITES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    let title = validate_title(&req.title)?;
    validate_stream_position(req.stream_number, req.soundbite_number, "soundbite_number")?;
    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;

    let soundbite = queries::soundbites::update(
        &mut tx,
        id,
        &UpdateSoundbite {
            title,
            soundbite_date: req.soundbite_date,
            stream_number: req.stream_number,
            soundbite_number: req.soundbite_number,
        },
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    queries::soundbites::set_artists(&mut tx, id, &req.artist_ids).await?;

    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok(Json(hydrate(&state.pool, soundbite).await?))
}

#[utoipa::path(
    delete,
    path = "/api/soundbites/{id}",
    params(("id" = Uuid, Path, description = "Soundbite ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "soundbites",
    security(("session" = []))
)]
pub(crate) async fn delete_soundbite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::SOUNDBITES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    let soundbite = queries::soundbites::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    queries::soundbites::delete(&state.pool, id).await?;
    if let Some(asset_id) = soundbite.asset_id {
        assets::delete_if_unreferenced(&state, asset_id).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}
