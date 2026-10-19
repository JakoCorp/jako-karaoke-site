//! Media type resolution for uploaded audio, video, and image files.

use std::io::Cursor;

use lofty::{config::ParseOptions, file::AudioFile, probe::Probe};

use crate::error::ApiError;

/// The kind of media being uploaded.
pub enum MediaKind {
    Audio,
    Video,
    Image,
}

const AUDIO_EXTS: &[&str] = &["mp3", "ogg", "wav", "flac", "aac", "m4a"];
const VIDEO_EXTS: &[&str] = &["mp4", "webm", "ogv", "mov", "mkv"];
const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "avif", "gif"];

/// Returns the playback length in whole seconds of an audio file.
///
/// Returns `None` when the format is not recognised or the length is zero.
pub fn audio_length(data: &[u8]) -> Option<u32> {
    let probe = Probe::new(Cursor::new(data)).guess_file_type().ok()?;
    let file = probe
        .options(ParseOptions::new().read_tags(false))
        .read()
        .ok()?;
    let seconds = file.properties().duration().as_secs_f64().round() as u32;
    (seconds > 0).then_some(seconds)
}

/// Resolves the stored file extension for a media upload.
///
/// Tries the MIME type first via mime_guess's database, then the original
/// filename extension as a fallback, since browsers sometimes send
/// `application/octet-stream` for unknown types.
///
/// # Errors
///
/// Returns [`ApiError::BadRequest`] if neither the MIME type nor the filename
/// extension matches the allowlist for the given `kind`.
pub fn resolve_ext(
    kind: MediaKind,
    content_type: &str,
    filename: Option<&str>,
) -> Result<&'static str, ApiError> {
    let allowed = match kind {
        MediaKind::Audio => AUDIO_EXTS,
        MediaKind::Video => VIDEO_EXTS,
        MediaKind::Image => IMAGE_EXTS,
    };

    // Strip MIME parameters (e.g. "audio/mpeg; codecs=mp3")
    let ct = content_type.split(';').next().unwrap_or("").trim();

    if let Some(exts) = mime_guess::get_mime_extensions_str(ct) {
        for &ext in exts {
            if allowed.contains(&ext) {
                return Ok(ext);
            }
        }
    }

    // Fall back to the filename extension the browser reported
    if let Some(name) = filename {
        let file_ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
        if let Some(&ext) = allowed.iter().find(|&&a| a == file_ext.as_str()) {
            return Ok(ext);
        }
    }

    Err(ApiError::BadRequest(format!(
        "unsupported media type '{ct}'; accepted: {}",
        allowed.join(", ")
    )))
}
