//! Policies deciding when a listening session counts as a play.

pub(crate) mod performance;
pub(crate) mod soundbite;

/// Longest a session may stay open before it can no longer be completed.
pub(crate) const MAX_SESSION_AGE_SECONDS: i64 = 6 * 60 * 60;
