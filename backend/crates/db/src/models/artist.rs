//! Artist model.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An artist record fetched from the database.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

/// Input for creating a new artist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewArtist {
    pub name: String,
    pub description: Option<String>,
}

/// Input for replacing an artist's mutable fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateArtist {
    pub name: String,
    pub description: Option<String>,
}

/// An external link associated with an artist.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct ArtistLink {
    pub id: Uuid,
    pub artist_id: Uuid,
    pub url: String,
    pub kind: String,
    pub label: Option<String>,
}

/// Input for creating a new artist link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewArtistLink {
    pub url: String,
    pub kind: String,
    pub label: Option<String>,
}
