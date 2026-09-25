//! Playlist resource types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::performances::PerformanceSummary;

/// Valid kind values for a playlist.
///
/// `Favorites` playlists are created automatically on user registration and
/// cannot be created or deleted through normal playlist endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlaylistKind {
    User,
    Official,
    Favorites,
}

impl PlaylistKind {
    /// Returns the string stored in the database for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Official => "official",
            Self::Favorites => "favorites",
        }
    }
}

/// Request body for `POST /api/playlists`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePlaylistRequest {
    /// Display title.
    pub title: String,
    /// Optional freeform description.
    pub description: Option<String>,
    /// Determines visibility rules and creation permissions.
    pub kind: PlaylistKind,
    /// When false, only the creator and users with `playlists:view_private` can access this playlist.
    pub is_public: bool,
}

/// Request body for `PUT /api/playlists/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePlaylistRequest {
    /// Display title.
    pub title: String,
    /// Optional freeform description.
    pub description: Option<String>,
    /// Determines visibility rules and creation permissions.
    pub kind: PlaylistKind,
    /// When false, only the creator and users with `playlists:view_private` can access this playlist.
    pub is_public: bool,
}

/// Full playlist metadata returned by detail and list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlaylistResponse {
    /// Unique identifier.
    pub id: Uuid,
    /// Display title.
    pub title: String,
    /// Optional freeform description.
    pub description: Option<String>,
    /// Playlist type affecting visibility and permission rules.
    pub kind: PlaylistKind,
    /// When false, only the creator and users with `playlists:view_private` can access this playlist.
    pub is_public: bool,
    /// User who created this playlist. `None` for system generated playlists.
    pub created_by: Option<Uuid>,
    /// Number of performances in this playlist.
    pub performance_count: u64,
}

/// A performance within a playlist, including when it was added.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlaylistEntry {
    /// The performance in this slot.
    #[serde(flatten)]
    pub performance: PerformanceSummary,
    /// Timestamp when this performance was added to the playlist.
    pub added_at: DateTime<Utc>,
}

/// Request body for `POST /api/playlists/{id}/performances`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AddPerformancesRequest {
    /// IDs of performances to append. Duplicates are silently skipped. Returns 400 if the new total would exceed 1000.
    pub performance_ids: Vec<Uuid>,
}

/// Request body for `DELETE /api/playlists/{id}/performances`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RemovePerformancesRequest {
    /// IDs of performances to remove. IDs not in the playlist are silently ignored.
    pub performance_ids: Vec<Uuid>,
}
