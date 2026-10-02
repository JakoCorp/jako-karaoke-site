//! Database model types: row structs and their `New`/`Update` input counterparts.

pub mod artist;
pub mod asset;
pub mod lyrics;
pub mod pending_oauth;
pub mod performance;
pub mod performance_audio;
pub mod performance_video;
pub mod playlist;
pub mod session;
pub mod song;
pub mod tag;
pub mod user;

pub use artist::{Artist, ArtistLink, NewArtist, NewArtistLink, UpdateArtist};
pub use asset::{Asset, NewExternalAsset, NewInternalAsset};
pub use lyrics::{Lyrics, NewLyrics};
pub use pending_oauth::PendingOAuth;
pub use performance::{NewPerformance, Performance, UpdatePerformance};
pub use performance_audio::{NewPerformanceAudio, PerformanceAudio, PerformanceAudioRow};
pub use performance_video::{NewPerformanceVideo, PerformanceVideo, PerformanceVideoRow};
pub use playlist::{NewPlaylist, Playlist, UpdatePlaylist};
pub use session::Session;
pub use song::{NewSong, Song, UpdateSong};
pub use tag::{NewTag, Tag, TagWithKind};
pub use user::{NewUser, UpdateUser, User, UserSummary};
