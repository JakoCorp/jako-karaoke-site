//! Pending OAuth claim model.

/// A pending OAuth signup awaiting username selection.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PendingOAuth {
    pub token: String,
    pub provider: String,
    pub provider_id: u64,
    pub suggested_username: String,
}
