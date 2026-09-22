//! Pending OAuth claim model.

/// A pending OAuth signup awaiting username selection.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PendingOAuth {
    /// Raw unhashed token stored in the `oauth_pending` HttpOnly cookie.
    pub token: String,
    /// OAuth provider name. One of: `twitch`, `discord`.
    pub provider: String,
    /// The user's numeric ID on the OAuth provider platform.
    pub provider_id: u64,
    /// Username derived from the provider profile, shown as the default in the claim UI.
    pub suggested_username: String,
}
