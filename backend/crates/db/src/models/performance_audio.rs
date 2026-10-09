//! Performance audio model.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Input for linking an asset to a performance as audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPerformanceAudio {
    pub asset_id: Uuid,
    pub kind: String,
}

/// Flat query result joining a performance audio row with its asset fields.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PerformanceAudioRow {
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
