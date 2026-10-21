//! Policy deciding when a listening session counts as a play.

/// Share of a very short performance that must be listened to before a play counts.
const BASE_SHARE_PERCENT: f64 = 50.0;

/// Extra share, on top of the base, that very long performances approach.
const SCALED_SHARE_PERCENT: f64 = 20.0;

/// Growth scale of the extra share in minutes, larger values make the share grow more slowly.
const SHARE_GROWTH_MINUTES: f64 = 10.0;

/// Fewest seconds of listening that can ever count as a play.
const MIN_REQUIRED_SECONDS: u32 = 1;

/// Duration assumed for performances without a usable one.
const FALLBACK_DURATION_SECONDS: u32 = 180;

/// Longest a session may stay open before it can no longer be completed.
pub(crate) const MAX_SESSION_AGE_SECONDS: i64 = 6 * 60 * 60;

/// Returns the share of a performance, in percent, that must be listened to.
///
/// It grows from [`BASE_SHARE_PERCENT`] towards the base plus [`SCALED_SHARE_PERCENT`]
/// as the duration increases, so longer performances require a larger share.
fn required_share_percent(duration_seconds: u32) -> f64 {
    let minutes = f64::from(duration_seconds) / 60.0;
    BASE_SHARE_PERCENT + SCALED_SHARE_PERCENT * (1.0 - (-minutes / SHARE_GROWTH_MINUTES).exp())
}

fn effective_duration(duration: Option<u32>) -> u32 {
    duration
        .filter(|seconds| *seconds > 0)
        .unwrap_or(FALLBACK_DURATION_SECONDS)
}

/// Returns the seconds of listening required before a play counts.
pub(crate) fn required_seconds(duration: Option<u32>) -> u32 {
    let duration = effective_duration(duration);
    let required = f64::from(duration) * required_share_percent(duration) / 100.0;
    (required.ceil() as u32).max(MIN_REQUIRED_SECONDS)
}

/// Returns the seconds a user must wait between two counted plays of the same performance.
pub(crate) fn cooldown_seconds(duration: Option<u32>) -> u32 {
    effective_duration(duration)
}
