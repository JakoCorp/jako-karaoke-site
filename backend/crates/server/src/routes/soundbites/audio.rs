//! Audio handler for soundbites.

use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use tracing::error;
use uuid::Uuid;

use api_types::{common::ErrorResponse, soundbites::SoundbiteResponse};
use db::{models::Soundbite, queries};

use crate::{
    assets::{self, UploadedAudio},
    auth::middleware::AuthUser,
    capabilities,
    error::ApiError,
    state::AppState,
};

use super::{AUDIO_SUBFOLDER, hydrate};

/// Placeholder schema for the audio multipart upload body.
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
pub(crate) struct SoundbiteAudioUpload {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}

/// Reads the `file` field of a multipart body and ignores every other field.
async fn read_audio_upload(multipart: &mut Multipart) -> Result<UploadedAudio, ApiError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("multipart field error: {e:?}");
        ApiError::BadRequest(e.to_string())
    })? {
        if field.name() != Some("file") {
            continue;
        }
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let filename = field.file_name().map(str::to_string);
        let bytes = field.bytes().await.map_err(|e| {
            error!("multipart read error: {e:?}");
            ApiError::BadRequest(e.to_string())
        })?;
        return Ok(UploadedAudio {
            data: bytes.to_vec(),
            content_type,
            filename,
            title: None,
            credits: None,
            source_url: None,
        });
    }
    Err(ApiError::BadRequest("missing 'file' field".into()))
}

#[utoipa::path(
    put,
    path = "/api/soundbites/{id}/audio",
    params(("id" = Uuid, Path, description = "Soundbite ID")),
    request_body(content = SoundbiteAudioUpload, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Soundbite with the uploaded audio", body = SoundbiteResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Soundbite not found", body = ErrorResponse),
    ),
    tag = "soundbites",
    security(("session" = []))
)]
pub(crate) async fn upload_soundbite_audio(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<SoundbiteResponse>, ApiError> {
    if !auth
        .capabilities
        .contains(capabilities::SOUNDBITES_MANAGE_ANY)
    {
        return Err(ApiError::Forbidden);
    }
    let soundbite = queries::soundbites::get_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let upload = read_audio_upload(&mut multipart).await?;
    let asset = assets::save_audio(&state, AUDIO_SUBFOLDER, upload).await?;

    queries::soundbites::set_asset(&state.pool, id, asset.id).await?;
    if let Some(previous_asset_id) = soundbite.asset_id {
        assets::delete_if_unreferenced(&state, previous_asset_id).await?;
    }

    let soundbite = Soundbite {
        asset_id: Some(asset.id),
        ..soundbite
    };
    Ok(Json(hydrate(&state.pool, soundbite).await?))
}
