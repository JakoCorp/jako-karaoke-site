//! Artist CRUD handlers and the `ArtistsApi` OpenAPI spec struct.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use api_types::{
    artists::{
        ArtistImageInfo, ArtistImageInput, ArtistImageKind, ArtistLinkInfo, ArtistLinkInput,
        ArtistLinkKind, ArtistResponse, ArtistSummary, CreateArtistRequest, UpdateArtistRequest,
    },
    common::ErrorResponse,
    pagination::PagedResponse,
};
use db::{
    MySqlPool,
    error::DbError,
    models::{NewArtist, NewArtistLink, UpdateArtist},
    queries,
};

use crate::{
    auth::middleware::AuthUser, capabilities, error::ApiError, pagination, state::AppState,
};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_artists, get_artist, create_artist, update_artist, delete_artist),
    components(schemas(
        ArtistSummary,
        ArtistResponse,
        ArtistImageInfo,
        ArtistImageInput,
        ArtistImageKind,
        ArtistLinkInfo,
        ArtistLinkInput,
        ArtistLinkKind,
        CreateArtistRequest,
        UpdateArtistRequest,
        ErrorResponse,
        PagedResponse<ArtistSummary>,
    ))
)]
pub(crate) struct ArtistsApi;

/// Query parameters for `GET /api/artists`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct ArtistListParams {
    /// Page number, 1-indexed. Defaults to 1.
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page. Defaults to 20. The server enforces a maximum.
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    /// Text search filter on artist name.
    pub q: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

impl ArtistListParams {
    fn limit_offset(&self) -> (u32, u32) {
        let per_page = self.per_page.min(pagination::MAX_PER_PAGE);
        let offset = self.page.saturating_sub(1).saturating_mul(per_page);
        (per_page, offset)
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_artists).post(create_artist))
        .route(
            "/{id}",
            get(get_artist).put(update_artist).delete(delete_artist),
        )
}

fn image_info(image: db::models::Image, kind: String) -> ArtistImageInfo {
    ArtistImageInfo {
        id: image.id,
        public_url: image.public_url,
        credits: image.credits,
        kind,
    }
}

fn link_info(link: db::models::ArtistLink) -> ArtistLinkInfo {
    ArtistLinkInfo {
        id: link.id,
        url: link.url,
        kind: link.kind,
        label: link.label,
    }
}

async fn hydrate(pool: &MySqlPool, artist: db::models::Artist) -> Result<ArtistResponse, ApiError> {
    let (raw_images, links) = tokio::try_join!(
        queries::artists::get_images(pool, artist.id),
        queries::artists::get_links(pool, artist.id),
    )?;
    Ok(ArtistResponse {
        id: artist.id,
        name: artist.name,
        description: artist.description,
        images: raw_images
            .into_iter()
            .map(|(i, k)| image_info(i, k))
            .collect(),
        links: links.into_iter().map(link_info).collect(),
    })
}

fn image_pairs(inputs: &[ArtistImageInput]) -> Vec<(Uuid, &str)> {
    inputs
        .iter()
        .map(|i| (i.image_id, i.kind.as_str()))
        .collect()
}

fn new_links(inputs: Vec<ArtistLinkInput>) -> Vec<NewArtistLink> {
    inputs
        .into_iter()
        .map(|l| NewArtistLink {
            url: l.url,
            kind: l.kind.as_str().to_string(),
            label: l.label,
        })
        .collect()
}

#[utoipa::path(
    get,
    path = "/api/artists",
    params(ArtistListParams),
    responses(
        (status = 200, description = "Paginated list of artists ordered by name", body = PagedResponse<ArtistSummary>),
    ),
    tag = "artists"
)]
pub(crate) async fn list_artists(
    State(state): State<AppState>,
    Query(params): Query<ArtistListParams>,
) -> Result<Json<PagedResponse<ArtistSummary>>, ApiError> {
    let (limit, offset) = params.limit_offset();
    let q = params.q.as_deref().filter(|s| !s.is_empty());
    let (artists, total) = tokio::try_join!(
        queries::artists::search(&state.pool, q, limit, offset),
        queries::artists::search_count(&state.pool, q),
    )?;

    let artist_ids: Vec<Uuid> = artists.iter().map(|a| a.id).collect();
    let mut artist_image_map = queries::artists::get_images_batch(&state.pool, &artist_ids).await?;

    let items = artists
        .into_iter()
        .map(|a| {
            let images = artist_image_map
                .remove(&a.id)
                .unwrap_or_default()
                .into_iter()
                .map(|(i, k)| image_info(i, k))
                .collect();
            ArtistSummary {
                id: a.id,
                name: a.name,
                description: a.description,
                images,
            }
        })
        .collect();

    Ok(Json(PagedResponse {
        items,
        total,
        page: params.page,
        per_page: limit,
    }))
}

#[utoipa::path(
    get,
    path = "/api/artists/{id}",
    params(("id" = Uuid, Path, description = "Artist ID")),
    responses(
        (status = 200, description = "Artist detail", body = ArtistResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "artists"
)]
pub(crate) async fn get_artist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ArtistResponse>, ApiError> {
    let artist = queries::artists::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(hydrate(&state.pool, artist).await?))
}

#[utoipa::path(
    post,
    path = "/api/artists",
    request_body = CreateArtistRequest,
    responses(
        (status = 201, description = "Created artist", body = ArtistResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    tag = "artists",
    security(("session" = []))
)]
pub(crate) async fn create_artist(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateArtistRequest>,
) -> Result<(StatusCode, Json<ArtistResponse>), ApiError> {
    if !auth.capabilities.contains(capabilities::ARTISTS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let image_pairs = image_pairs(&req.images);
    let new_links = new_links(req.links);

    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;
    let artist = queries::artists::create(
        &mut tx,
        &NewArtist {
            name: req.name,
            description: req.description,
        },
    )
    .await?;
    queries::artists::set_images(&mut tx, artist.id, &image_pairs).await?;
    queries::artists::set_links(&mut tx, artist.id, &new_links).await?;
    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok((
        StatusCode::CREATED,
        Json(hydrate(&state.pool, artist).await?),
    ))
}

#[utoipa::path(
    put,
    path = "/api/artists/{id}",
    params(("id" = Uuid, Path, description = "Artist ID")),
    request_body = UpdateArtistRequest,
    responses(
        (status = 200, description = "Updated artist", body = ArtistResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "artists",
    security(("session" = []))
)]
pub(crate) async fn update_artist(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateArtistRequest>,
) -> Result<Json<ArtistResponse>, ApiError> {
    if !auth.capabilities.contains(capabilities::ARTISTS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let image_pairs = image_pairs(&req.images);
    let new_links = new_links(req.links);

    let mut tx = state.pool.begin().await.map_err(DbError::Sqlx)?;
    let artist = queries::artists::update(
        &mut tx,
        id,
        &UpdateArtist {
            name: req.name,
            description: req.description,
        },
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    queries::artists::set_images(&mut tx, id, &image_pairs).await?;
    queries::artists::set_links(&mut tx, id, &new_links).await?;
    tx.commit().await.map_err(DbError::Sqlx)?;

    Ok(Json(hydrate(&state.pool, artist).await?))
}

#[utoipa::path(
    delete,
    path = "/api/artists/{id}",
    params(("id" = Uuid, Path, description = "Artist ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "artists",
    security(("session" = []))
)]
pub(crate) async fn delete_artist(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !auth.capabilities.contains(capabilities::ARTISTS_MANAGE_ANY) {
        return Err(ApiError::Forbidden);
    }
    let found = queries::artists::delete(&state.pool, id).await?;
    if found {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}
