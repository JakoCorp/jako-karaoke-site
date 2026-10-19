//! YouTube Data API lookups for externally linked media.

use std::time::Duration;

use reqwest::Url;
use serde::Deserialize;
use tracing::warn;

use crate::state::AppState;

const VIDEOS_ENDPOINT: &str = "https://www.googleapis.com/youtube/v3/videos";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const VIDEO_ID_LENGTH: usize = 11;

#[derive(Deserialize)]
struct VideosResponse {
    items: Vec<VideoItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoItem {
    content_details: ContentDetails,
}

#[derive(Deserialize)]
struct ContentDetails {
    duration: String,
}

/// Looks up the length in seconds of the YouTube video at `external_url`.
///
/// Returns `None` when no Google API key is configured, the URL is not a YouTube video,
/// or the lookup fails. Failures are logged and never block the caller.
pub(crate) async fn video_length(state: &AppState, external_url: &str) -> Option<u32> {
    let api_key = state.config.google_api_key.as_deref()?;
    let video_id = extract_video_id(external_url)?;
    match fetch_length(&state.http_client, api_key, &video_id).await {
        Ok(length) => length,
        Err(error) => {
            warn!(
                "youtube length lookup failed for {video_id}: {}",
                error.without_url()
            );
            None
        }
    }
}

async fn fetch_length(
    client: &reqwest::Client,
    api_key: &str,
    video_id: &str,
) -> Result<Option<u32>, reqwest::Error> {
    let url = Url::parse_with_params(
        VIDEOS_ENDPOINT,
        &[
            ("part", "contentDetails"),
            ("id", video_id),
            ("key", api_key),
        ],
    )
    .expect("valid YouTube API URL");
    let response: VideosResponse = client
        .get(url)
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response
        .items
        .first()
        .and_then(|item| parse_iso8601_duration(&item.content_details.duration)))
}

/// Extracts the video ID from a watch URL.
fn extract_video_id(external_url: &str) -> Option<String> {
    let url = Url::parse(external_url).ok()?;
    let video_id = match url.host_str()? {
        "youtu.be" => url.path_segments()?.next()?.to_string(),
        "youtube.com" | "www.youtube.com" | "m.youtube.com" | "music.youtube.com" => url
            .query_pairs()
            .find(|(key, _)| key == "v")
            .map(|(_, value)| value.into_owned())?,
        _ => return None,
    };
    let is_valid = video_id.len() == VIDEO_ID_LENGTH
        && video_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    is_valid.then_some(video_id)
}

/// Parses an ISO 8601 duration such as `PT3M32S` into whole seconds.
///
/// Returns `None` for malformed values and for zero length durations, which YouTube
/// reports for live streams and premieres.
fn parse_iso8601_duration(value: &str) -> Option<u32> {
    let mut total_seconds: u32 = 0;
    let mut digits = String::new();
    let mut in_time_part = false;

    for character in value.strip_prefix('P')?.chars() {
        match character {
            'T' => in_time_part = true,
            '0'..='9' => digits.push(character),
            unit => {
                let amount: u32 = digits.parse().ok()?;
                digits.clear();
                let unit_seconds = match (in_time_part, unit) {
                    (false, 'D') => 86_400,
                    (true, 'H') => 3_600,
                    (true, 'M') => 60,
                    (true, 'S') => 1,
                    _ => return None,
                };
                total_seconds = total_seconds.checked_add(amount.checked_mul(unit_seconds)?)?;
            }
        }
    }

    (digits.is_empty() && total_seconds > 0).then_some(total_seconds)
}
