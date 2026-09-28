//! Conversions from database models to API response types.

use chrono::{DateTime, Utc};

use api_types::{
    artists::ArtistImageInfo,
    assets::AssetInfo,
    performances::{AudioInfo, VideoInfo},
    playlists::{PlaylistEntry, PlaylistKind, PlaylistResponse},
    songs::SongImageInfo,
    tags::TagResponse,
};
use db::{
    models::Tag,
    models::playlist::Playlist,
    queries::{artists::ArtistImageRow, songs::SongImageRow},
};

use crate::error::ApiError;

/// Converts a [`Playlist`] model to a [`PlaylistResponse`].
///
/// Returns an error if the stored kind string is not a known variant.
pub(crate) fn playlist_response(playlist: Playlist) -> Result<PlaylistResponse, ApiError> {
    let kind = match playlist.kind.as_str() {
        "user" => PlaylistKind::User,
        "official" => PlaylistKind::Official,
        "favorites" => PlaylistKind::Favorites,
        other => {
            return Err(ApiError::Internal(format!(
                "unknown playlist kind in database: {other}"
            )));
        }
    };
    Ok(PlaylistResponse {
        id: playlist.id,
        title: playlist.title,
        description: playlist.description,
        kind,
        is_public: playlist.is_public,
        created_by: playlist.created_by,
        performance_count: playlist.performance_count as u64,
    })
}

/// Wraps a [`PerformanceSummary`] with its playlist metadata.
pub(crate) fn playlist_entry(
    performance: api_types::performances::PerformanceSummary,
    added_at: DateTime<Utc>,
) -> PlaylistEntry {
    PlaylistEntry {
        performance,
        added_at,
    }
}

/// Converts a [`Tag`] model to a [`TagResponse`].
pub(crate) fn tag_response(tag: Tag) -> TagResponse {
    TagResponse {
        id: tag.id,
        name: tag.name,
    }
}

pub(crate) fn asset_info_from_row(
    asset_id: uuid::Uuid,
    title: Option<String>,
    credits: Option<String>,
    source_url: Option<String>,
    storage_url: Option<String>,
    external_url: Option<String>,
) -> AssetInfo {
    AssetInfo {
        asset_id,
        title,
        credits,
        source_url,
        storage_url,
        external_url,
    }
}

/// Converts a [`PerformanceAudioRow`] to an [`AudioInfo`].
pub(crate) fn audio_info(row: db::models::PerformanceAudioRow) -> AudioInfo {
    AudioInfo {
        kind: row.kind,
        asset: asset_info_from_row(
            row.asset_id,
            row.title,
            row.credits,
            row.source_url,
            row.storage_url,
            row.external_url,
        ),
    }
}

/// Converts a [`PerformanceVideoRow`] to a [`VideoInfo`].
pub(crate) fn video_info(row: db::models::PerformanceVideoRow) -> VideoInfo {
    VideoInfo {
        kind: row.kind,
        asset: asset_info_from_row(
            row.asset_id,
            row.title,
            row.credits,
            row.source_url,
            row.storage_url,
            row.external_url,
        ),
    }
}

/// Converts a [`SongImageRow`] to a [`SongImageInfo`].
pub(crate) fn song_image_info(row: SongImageRow) -> SongImageInfo {
    SongImageInfo {
        kind: row.kind,
        asset: asset_info_from_row(
            row.asset_id,
            row.title,
            row.credits,
            row.source_url,
            row.storage_url,
            row.external_url,
        ),
    }
}

/// Converts an [`ArtistImageRow`] to an [`ArtistImageInfo`].
pub(crate) fn artist_image_info(row: ArtistImageRow) -> ArtistImageInfo {
    ArtistImageInfo {
        kind: row.kind,
        asset: asset_info_from_row(
            row.asset_id,
            row.title,
            row.credits,
            row.source_url,
            row.storage_url,
            row.external_url,
        ),
    }
}
