//! Performance video model.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Input for linking an asset to a performance as video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPerformanceVideo {
    pub asset_id: Uuid,
    pub kind: String,
}

/// Flat query result joining a performance video row with its asset fields.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PerformanceVideoRow {
    pub performance_id: Uuid,
    pub asset_id: Uuid,
    pub kind: String,
    pub title: Option<String>,
    pub credits: Option<String>,
    pub source_url: Option<String>,
    pub storage_url: Option<String>,
    pub internal_path: Option<String>,
    pub external_url: Option<String>,
    pub hash: Option<String>,
}
