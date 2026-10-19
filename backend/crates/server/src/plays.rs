//! Policy deciding when a listening session counts as a play.

/// Share of a performance that must be listened to before a play counts.
const REQUIRED_SHARE_PERCENT: u32 = 50;

/// Fewest seconds of listening that can ever count as a play.
const MIN_REQUIRED_SECONDS: u32 = 15;

/// Duration assumed for performances without one.
const FALLBACK_DURATION_SECONDS: u32 = 180;

/// Longest a session may stay open before it can no longer be completed.
pub(crate) const MAX_SESSION_AGE_SECONDS: i64 = 6 * 60 * 60;

/// Returns the seconds of listening required before a play counts.
pub(crate) fn required_seconds(duration: Option<u32>) -> u32 {
    let duration = duration.unwrap_or(FALLBACK_DURATION_SECONDS);
    (duration * REQUIRED_SHARE_PERCENT)
        .div_ceil(100)
        .max(MIN_REQUIRED_SECONDS)
}

/// Returns the seconds a user must wait between two counted plays of the same performance.
pub(crate) fn cooldown_seconds(duration: Option<u32>) -> u32 {
    duration.unwrap_or(FALLBACK_DURATION_SECONDS)
}
