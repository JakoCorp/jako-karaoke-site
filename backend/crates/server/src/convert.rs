//! Conversions from database models to API response types.

use chrono::{DateTime, Utc};

use api_types::{
    artists::ArtistImageInfo,
    assets::AssetInfo,
    common::ArtistInfo,
    performances::{AudioInfo, VideoInfo},
    playlists::{PlaylistEntry, PlaylistKind, PlaylistResponse},
    plays::UserPlayEntry,
    songs::SongImageInfo,
    soundbites::SoundbiteResponse,
    tags::TagResponse,
};
use db::{
    models::{Artist, Asset, Soundbite, Tag, playlist::Playlist},
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

/// Wraps a [`PerformanceSummary`] with one user's play metadata.
pub(crate) fn user_play_entry(
    performance: api_types::performances::PerformanceSummary,
    user_play_count: u64,
    last_played_at: DateTime<Utc>,
) -> UserPlayEntry {
    UserPlayEntry {
        performance,
        user_play_count,
        last_played_at,
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
    hash: Option<String>,
) -> AssetInfo {
    AssetInfo {
        asset_id,
        title,
        credits,
        source_url,
        storage_url,
        external_url,
        hash,
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
            row.hash,
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
            row.hash,
        ),
    }
}

/// Combines a [`Soundbite`] with its artists and audio [`Asset`] into a [`SoundbiteResponse`].
pub(crate) fn soundbite_response(
    soundbite: Soundbite,
    artists: Vec<Artist>,
    asset: Option<Asset>,
) -> SoundbiteResponse {
    SoundbiteResponse {
        id: soundbite.id,
        title: soundbite.title,
        soundbite_date: soundbite.soundbite_date,
        stream_number: soundbite.stream_number,
        soundbite_number: soundbite.soundbite_number,
        artists: artists
            .into_iter()
            .map(|artist| ArtistInfo {
                id: artist.id,
                name: artist.name,
                description: artist.description,
            })
            .collect(),
        duration: asset.as_ref().and_then(|asset| asset.length),
        audio: asset.map(|asset| {
            asset_info_from_row(
                asset.id,
                asset.title,
                asset.credits,
                asset.source_url,
                asset.storage_url,
                asset.external_url,
                asset.hash,
            )
        }),
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
            None,
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
            None,
        ),
    }
}
