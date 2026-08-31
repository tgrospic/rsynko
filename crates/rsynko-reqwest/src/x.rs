use crate::RuntimeEnvironment;
use rsynko_media::{
    FORMAT_EXTENSION, FORMAT_HAS_AUDIO, FORMAT_HAS_VIDEO, FORMAT_SOURCE, FormatAlg, MetadataAlg,
};
use rsynko_memory::{Extraction, Format, InfoValue, Media, MediaSyntax};
use rsynko_x::*;
use serde_json::Value;

/// Names the observation stating which kind of file one format was taken from.
pub const X_KIND: &str = "x_kind";

/// Names what the answer calls the pieces it lists.
const MEDIA_DETAILS: &str = "mediaDetails";

/// Denotes failure to read what one tweet carries.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeXError {
    /// Denotes an address naming no tweet.
    #[error("{0} names no tweet")]
    Unaddressed(String),
    /// Denotes a tweet that could not be asked about.
    #[error("the tweet could not be asked about: {0}")]
    Unreachable(#[from] reqwest::Error),
    /// Denotes an answer that could not be read.
    #[error("what the tweet answered could not be read: {0}")]
    Unreadable(#[from] serde_json::Error),
    /// Denotes a tweet carrying nothing anybody can take.
    ///
    /// The address answers with nothing at all for a tweet that was removed, or one whose author
    /// is protected, so this is what being refused looks like as well as what an empty post is.
    #[error("the tweet carries no media, or is not public")]
    Empty,
}

impl XSorts for RuntimeEnvironment {
    type Request = String;
    type Attachment = Format;
}

impl XRequestAlg for RuntimeEnvironment {
    fn tweet_request(&self, address: impl Into<String>) -> String {
        address.into()
    }
}

impl XRequestViewAlg for RuntimeEnvironment {
    fn request_address<'a>(&self, request: &'a String) -> &'a str {
        request
    }
}

impl RuntimeEnvironment {
    /// Reads what one public tweet carries, as the formats a reader chooses between.
    ///
    /// # Errors
    ///
    /// Returns the failure to name, ask about, or read the tweet.
    pub(crate) fn extract_x(&self, url: &str) -> Result<Extraction, RuntimeXError> {
        let id = status_id(url).ok_or_else(|| RuntimeXError::Unaddressed(url.to_owned()))?;
        let answered = self.fetch_bytes(&self.status_request(&id))?;
        let answer: Value = serde_json::from_slice(&answered)?;
        let carried = answer
            .get(MEDIA_DETAILS)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let formats = carried
            .iter()
            .enumerate()
            .flat_map(|(place, piece)| read_piece(place, piece))
            .collect::<Vec<_>>();
        if formats.is_empty() {
            return Err(RuntimeXError::Empty);
        }
        let author = answer
            .pointer("/user/screen_name")
            .and_then(Value::as_str)
            .unwrap_or("tweet");
        Ok(Extraction::Media(Media::new(
            id.clone(),
            MediaSyntax.metadata([(
                "title".to_owned(),
                MediaSyntax.string_metadata(format!("{author}-{id}")),
            )]),
            formats,
        )))
    }
}

/// States every format one piece of a tweet offers.
fn read_piece(place: usize, piece: &Value) -> Vec<Format> {
    let Some(kind) = piece
        .get("type")
        .and_then(Value::as_str)
        .and_then(attachment_kind::from)
    else {
        return Vec::new();
    };
    let place = place + 1;
    match kind {
        // A video is offered at several sizes, and each is a whole file rather than a piece of
        // one, so every one of them is a format a reader may choose.
        AttachmentKind::Video | AttachmentKind::Animation => piece
            .pointer("/video_info/variants")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .filter(|variant| {
                variant.get("content_type").and_then(Value::as_str) == Some("video/mp4")
            })
            .filter_map(|variant| read_variant(place, kind, variant))
            .collect(),
        AttachmentKind::Photo => piece
            .get("media_url_https")
            .and_then(Value::as_str)
            .map(|address| read_photo(place, address))
            .into_iter()
            .collect(),
    }
}

/// States one size a tweet offers a video at.
fn read_variant(place: usize, kind: AttachmentKind, variant: &Value) -> Option<Format> {
    let address = variant.get("url").and_then(Value::as_str)?;
    let bitrate = variant.get("bitrate").and_then(Value::as_i64).unwrap_or(0);
    let measured = measurement(address);
    let identity = measured.as_ref().map_or_else(
        || format!("{}-{place}", attachment_kind::to(kind)),
        |size| format!("{}-{place}-{size}", attachment_kind::to(kind)),
    );
    Some(MediaSyntax.format(
        identity,
        MediaSyntax.metadata([
            (
                FORMAT_SOURCE.to_owned(),
                MediaSyntax.string_metadata(address),
            ),
            (
                FORMAT_EXTENSION.to_owned(),
                MediaSyntax.string_metadata("mp4"),
            ),
            (
                FORMAT_HAS_VIDEO.to_owned(),
                MediaSyntax.boolean_metadata(true),
            ),
            // An animation carries no sound, and is a video in every other way.
            (
                FORMAT_HAS_AUDIO.to_owned(),
                MediaSyntax.boolean_metadata(kind == AttachmentKind::Video),
            ),
            (
                X_KIND.to_owned(),
                MediaSyntax.string_metadata(attachment_kind::to(kind)),
            ),
            ("bitrate".to_owned(), MediaSyntax.integer_metadata(bitrate)),
            (
                "quality".to_owned(),
                MediaSyntax.string_metadata(measured.unwrap_or_default()),
            ),
        ]),
    ))
}

/// States the one file a picture is, at the size it was posted at.
fn read_photo(place: usize, address: &str) -> Format {
    let extension = address.rsplit('.').next().unwrap_or("jpg").to_owned();
    Format::new(
        format!("photo-{place}"),
        MediaSyntax.metadata([
            (
                FORMAT_SOURCE.to_owned(),
                // A picture is served at the size it is asked for, and the original is the one
                // that was posted rather than one the service made to show it with.
                MediaSyntax.string_metadata(format!("{address}?format={extension}&name=orig")),
            ),
            (
                FORMAT_EXTENSION.to_owned(),
                MediaSyntax.string_metadata(extension),
            ),
            (
                FORMAT_HAS_VIDEO.to_owned(),
                MediaSyntax.boolean_metadata(false),
            ),
            (
                FORMAT_HAS_AUDIO.to_owned(),
                MediaSyntax.boolean_metadata(false),
            ),
            (
                X_KIND.to_owned(),
                MediaSyntax.string_metadata(attachment_kind::to(AttachmentKind::Photo)),
            ),
        ]),
    )
}

/// Observes the size a video address states it was encoded at, when it states one.
fn measurement(address: &str) -> Option<String> {
    address
        .split('/')
        .find(|segment| {
            let mut sides = segment.split('x');
            let (across, down) = (sides.next(), sides.next());
            sides.next().is_none()
                && across.is_some_and(|side| {
                    !side.is_empty() && side.bytes().all(|b| b.is_ascii_digit())
                })
                && down.is_some_and(|side| {
                    !side.is_empty() && side.bytes().all(|b| b.is_ascii_digit())
                })
        })
        .map(str::to_owned)
}

/// Names the value one observation states, when it states text.
#[must_use]
pub fn kind_of(format: &Format) -> Option<AttachmentKind> {
    match format.metadata.get(X_KIND) {
        Some(InfoValue::String(stated)) => attachment_kind::from(stated),
        _ => None,
    }
}
