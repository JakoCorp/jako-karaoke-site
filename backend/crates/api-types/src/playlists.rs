//! Playlist resource types.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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
    pub title: String,
    pub description: Option<String>,
    pub kind: PlaylistKind,
    pub is_public: bool,
}

/// Request body for `PUT /api/playlists/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePlaylistRequest {
    pub title: String,
    pub description: Option<String>,
    pub kind: PlaylistKind,
    pub is_public: bool,
}

/// Full playlist metadata returned by detail and list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlaylistResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub kind: PlaylistKind,
    pub is_public: bool,
    /// User who created this playlist. `None` for system generated playlists.
    pub created_by: Option<Uuid>,
}

/// Request body for `POST /api/playlists/{id}/performances`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AddPerformancesRequest {
    pub performance_ids: Vec<Uuid>,
}

/// Request body for `DELETE /api/playlists/{id}/performances`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RemovePerformancesRequest {
    pub performance_ids: Vec<Uuid>,
}
