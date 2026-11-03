//! Policy deciding when a listening session on a soundbite counts as a play.

/// Share of a soundbite that must be listened to before a play counts.
const LISTEN_SHARE: f64 = 0.8;

/// Seconds of a soundbite that may stay unheard while a play still counts.
const MAX_UNHEARD_SECONDS: f64 = 1.0;

/// Fewest seconds a user must wait between two counted plays of the same soundbite.
const MIN_COOLDOWN_SECONDS: u32 = 60;

/// Duration assumed for soundbites without a usable one.
const FALLBACK_DURATION_SECONDS: u32 = 3;

/// Seconds a session may be short of the required time when it is completed.
///
/// Session timestamps have whole second precision and the start request takes time to arrive, so
/// an honest listener can look slightly early.
const COMPLETE_GRACE_SECONDS: i64 = 1;

fn effective_duration(duration: Option<u32>) -> u32 {
    duration
        .filter(|seconds| *seconds > 0)
        .unwrap_or(FALLBACK_DURATION_SECONDS)
}

/// Returns the seconds of listening required before a play counts.
///
/// That is the smaller of 80% of the soundbite and all of it but the last second, rounded up.
pub(crate) fn required_seconds(duration: Option<u32>) -> u32 {
    let duration = f64::from(effective_duration(duration));
    let required = (duration * LISTEN_SHARE).min(duration - MAX_UNHEARD_SECONDS);
    required.max(0.0).ceil() as u32
}

/// Returns the seconds a user must wait between two counted plays of the same soundbite.
pub(crate) fn cooldown_seconds(duration: Option<u32>) -> u32 {
    effective_duration(duration).max(MIN_COOLDOWN_SECONDS)
}

/// Returns whether a session open for `elapsed_seconds` has been open long enough to complete.
pub(crate) fn is_long_enough(elapsed_seconds: i64, duration: Option<u32>) -> bool {
    elapsed_seconds + COMPLETE_GRACE_SECONDS >= i64::from(required_seconds(duration))
}
