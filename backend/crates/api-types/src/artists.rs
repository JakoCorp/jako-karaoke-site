//! Artist request and response types.
//!
//! Updates use PUT semantics: all fields are required and missing optionals mean null.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Valid kind values for an image attached to an artist.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtistImageKind {
    Avatar,
}

impl ArtistImageKind {
    /// Returns the string stored in the database for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Avatar => "avatar",
        }
    }
}

/// Valid platform kinds for an artist external link.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtistLinkKind {
    Youtube,
    Website,
    X,
    Instagram,
    Twitch,
    Other,
}

impl ArtistLinkKind {
    /// Returns the string stored in the database for this kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Youtube => "youtube",
            Self::Website => "website",
            Self::X => "x",
            Self::Instagram => "instagram",
            Self::Twitch => "twitch",
            Self::Other => "other",
        }
    }
}

/// An image attached to an artist with its semantic role.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistImageInfo {
    pub id: Uuid,
    /// Publicly served URL for clients.
    pub public_url: String,
    pub credits: Option<String>,
    pub kind: String,
}

/// Input for attaching an existing image to an artist.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistImageInput {
    pub image_id: Uuid,
    pub kind: ArtistImageKind,
}

/// An external link associated with an artist.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistLinkInfo {
    pub id: Uuid,
    pub url: String,
    pub kind: String,
    pub label: Option<String>,
}

/// Input for creating an external link on an artist.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistLinkInput {
    pub url: String,
    pub kind: ArtistLinkKind,
    pub label: Option<String>,
}

/// A lean artist record returned by the list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub images: Vec<ArtistImageInfo>,
}

/// A full artist record returned by detail and mutation endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub images: Vec<ArtistImageInfo>,
    pub links: Vec<ArtistLinkInfo>,
}

/// Request body for `POST /api/artists`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateArtistRequest {
    pub name: String,
    pub description: Option<String>,
    pub images: Vec<ArtistImageInput>,
    pub links: Vec<ArtistLinkInput>,
}

/// Request body for `PUT /api/artists/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateArtistRequest {
    pub name: String,
    pub description: Option<String>,
    pub images: Vec<ArtistImageInput>,
    pub links: Vec<ArtistLinkInput>,
}
