//! Shared response fragments used across multiple resource types.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// A minimal artist record embedded in song and performance responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ArtistInfo {
    /// Unique identifier.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Optional freeform bio or description.
    pub description: Option<String>,
}

/// A tag embedded in song responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TagInfo {
    /// Unique identifier.
    pub id: Uuid,
    /// Display name of the tag.
    pub name: String,
    /// Freeform category string (e.g. `"genre"`, `"mood"`).
    pub kind: String,
}

/// Body returned for all error responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Human readable description of the error.
    pub error: String,
}
