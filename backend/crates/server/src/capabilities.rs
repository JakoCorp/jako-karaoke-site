//! Capability title constants and authorization mappings.
//!
//! Handlers compare against [`AuthUser::capabilities`],
//! which is fetched fresh on every request.

use api_types::playlists::PlaylistKind;

/// Required to grant or revoke capabilities for any user.
pub const CAPABILITIES_MANAGE: &str = "capabilities:manage";

/// Required to create, update, or delete any artist record.
pub const ARTISTS_MANAGE_ANY: &str = "artists:manage_any";

/// Required to create, update, or delete any song record.
pub const SONGS_MANAGE_ANY: &str = "songs:manage_any";

/// Required to create, update, or delete any tag record.
pub const TAGS_MANAGE_ANY: &str = "tags:manage_any";

/// Required to create, update, or delete any performance record.
pub const PERFORMANCES_MANAGE_ANY: &str = "performances:manage_any";

/// Allows viewing private playlists owned by any user.
pub const PLAYLISTS_VIEW_PRIVATE: &str = "playlists:view_private";

/// Required to create playlists with `kind = official`.
pub const PLAYLISTS_CREATE_OFFICIAL: &str = "playlists:create_official";

/// Required to create playlists with `kind = favorites`.
/// Assigned automatically during new user registration.
pub const PLAYLISTS_CREATE_FAVORITES: &str = "playlists:create_favorites";

/// Required to update or delete playlists owned by other users.
pub const PLAYLISTS_MANAGE_ANY: &str = "playlists:manage_any";

/// Every capability string recognised by this server.
pub const ALL_CAPABILITIES: &[&str] = &[
    CAPABILITIES_MANAGE,
    ARTISTS_MANAGE_ANY,
    SONGS_MANAGE_ANY,
    TAGS_MANAGE_ANY,
    PERFORMANCES_MANAGE_ANY,
    PLAYLISTS_VIEW_PRIVATE,
    PLAYLISTS_CREATE_OFFICIAL,
    PLAYLISTS_CREATE_FAVORITES,
    PLAYLISTS_MANAGE_ANY,
];

/// Returns the capability required to create a playlist of the given kind, or
/// `None` if any authenticated user may create it.
pub fn required_playlist_create_capability(kind: &PlaylistKind) -> Option<&'static str> {
    match kind {
        PlaylistKind::User => None,
        PlaylistKind::Official => Some(PLAYLISTS_CREATE_OFFICIAL),
        PlaylistKind::Favorites => Some(PLAYLISTS_CREATE_FAVORITES),
    }
}
