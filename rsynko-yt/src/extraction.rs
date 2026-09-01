use crate::*;
use alux_ext::ext;
use rsynko_media::*;
use std::fmt::{Display, Formatter, Result as FormatResult};
use thiserror::Error;

/// Derives Youtube extraction from request, response, and domain-data algebras.
#[ext(name = YoutubeExtractionExt)]
pub impl<This, RequestError, ResponseError, ChallengeError> This
where
    This: YoutubeRequestAlg
        + YoutubeClientAlg
        + YoutubeProgramAlg
        + YoutubeRequestBytesAlg<Error = RequestError>
        + YoutubeResponseAlg<Error = ResponseError>
        + YoutubeChallengeAlg<Error = ChallengeError>
        + YoutubeSolutionAlg
        + YoutubeUrlAlg
        + MetadataAlg
        + FormatAlg
        + ExtractionAlg,
{
    /// Extracts one Youtube watch URL into a preference-ordered media catalog.
    ///
    /// # Errors
    ///
    /// Returns URL, request, response, availability, challenge, or empty-catalog failure.
    fn extract_youtube(
        &self,
        url: &str,
    ) -> Result<This::Extraction, YoutubeError<RequestError, ResponseError, ChallengeError>> {
        let id = youtube_id(url).ok_or_else(|| YoutubeError::Url(url.to_owned()))?;
        let watch_request = self.watch_request(url);
        let page_bytes = self.youtube_request_bytes(&watch_request).map_err(YoutubeError::Request)?;
        let page = self.decode_youtube_watch(&page_bytes).map_err(YoutubeError::Response)?;
        if !page.has_player_response {
            return Err(YoutubeError::MissingPlayerResponse);
        }
        let api_key = page.api_key.ok_or(YoutubeError::MissingApiKey)?;
        // A challenge is posed by one player program, and Youtube grants a catalog matching the
        // program the client states it runs, so the program is retrieved before the catalog.
        let program = match page.player_url {
            Some(url) => {
                let request = self.player_program_request(url);
                let bytes = self.youtube_request_bytes(&request).map_err(YoutubeError::Request)?;
                String::from_utf8_lossy(&bytes).into_owned()
            }
            None => String::new(),
        };
        let player = self.granted_catalog(&id, &api_key, page.visitor_data.as_deref(), &program)?;
        if player.status.as_deref() != Some("OK") {
            return Err(YoutubeError::Unavailable(player.reason.unwrap_or_else(|| "video unavailable".to_owned())));
        }
        let unreadable = player.unreadable;
        let mut ranked = player.formats;
        ranked.sort_by_key(|format| (format.height.unwrap_or(0), format.bitrate.unwrap_or(0)));
        if ranked.is_empty() {
            return Err(YoutubeError::NoDirectFormat);
        }
        let grants = self.grant_formats(&program, &ranked).map_err(YoutubeError::Challenge)?;
        let mut formats = Vec::new();
        let mut withheld = None;
        let mut withheld_count = 0_usize;
        let mut throttled_count = 0_usize;
        for (format, grant) in ranked.iter().zip(grants) {
            match grant {
                // A throttled representation is retrievable, only slowly, so it stays in the
                // catalog rather than disappearing from it.
                YoutubeGrant::Granted { url, .. } => {
                    formats.push(self.youtube_format(format.clone(), url));
                }
                YoutubeGrant::Throttled { url, .. } => {
                    throttled_count += 1;
                    formats.push(self.youtube_format(format.clone(), url));
                }
                YoutubeGrant::Withheld { challenge, .. } => {
                    withheld_count += 1;
                    withheld = Some(withheld_reason(&challenge).max(withheld.unwrap_or(WithheldReason::Throttle)));
                }
            }
        }
        if formats.is_empty() {
            return Err(YoutubeError::Withheld(withheld.unwrap_or(WithheldReason::Throttle)));
        }
        let title = player.title.or(page.title);
        let mut fields = vec![
            (YOUTUBE_DESCRIBED.to_owned(), self.integer_metadata(i64::try_from(ranked.len()).unwrap_or(i64::MAX))),
            (YOUTUBE_WITHHELD.to_owned(), self.integer_metadata(i64::try_from(withheld_count).unwrap_or(i64::MAX))),
            (YOUTUBE_THROTTLED.to_owned(), self.integer_metadata(i64::try_from(throttled_count).unwrap_or(i64::MAX))),
            (YOUTUBE_UNREADABLE.to_owned(), self.integer_metadata(i64::try_from(unreadable).unwrap_or(i64::MAX))),
        ];
        if let Some(title) = title {
            fields.push(("title".to_owned(), self.string_metadata(title)));
        }
        Ok(self.media(id, self.metadata(fields), formats))
    }

    /// Observes the union of the catalogs this interpreter's clients are granted, in claim order.
    ///
    /// Each client is granted a different catalog, and neither is the whole truth: one states a
    /// muxed representation carrying both streams, another states every adaptive representation on
    /// its own. A client refused states nothing about the others.
    ///
    /// # Errors
    ///
    /// Returns request or response failure.
    fn granted_catalog(
        &self,
        id: &str,
        api_key: &str,
        visitor: Option<&str>,
        program: &str,
    ) -> Result<YoutubePlayer, YoutubeError<RequestError, ResponseError, ChallengeError>> {
        let mut catalog = YoutubePlayer { status: None, reason: None, title: None, formats: Vec::new(), unreadable: 0 };
        let clients: Vec<String> = self.player_clients().map(str::to_owned).collect();
        for client in clients {
            let claim =
                PlayerClaim { client, visitor: visitor.map(str::to_owned), timestamp: self.program_timestamp(program) };
            let request = self.player_request(id, api_key, &claim);
            let bytes = self.youtube_request_bytes(&request).map_err(YoutubeError::Request)?;
            let granted = self.decode_youtube_player(&bytes).map_err(YoutubeError::Response)?;
            if granted.status.as_deref() != Some("OK") {
                catalog.status = catalog.status.or(granted.status);
                catalog.reason = catalog.reason.or(granted.reason);
                continue;
            }
            catalog.status = Some("OK".to_owned());
            catalog.reason = None;
            catalog.title = catalog.title.or(granted.title);
            catalog.unreadable += granted.unreadable;
            for format in granted.formats {
                if !catalog.formats.iter().any(|held| held.id == format.id) {
                    catalog.formats.push(format);
                }
            }
        }
        Ok(catalog)
    }

    /// Defines one generic format from Youtube player observations and its granted URL.
    fn youtube_format(&self, format: YoutubeFormat, url: String) -> This::Format {
        let mut fields = vec![
            (FORMAT_SOURCE.to_owned(), self.string_metadata(url)),
            (FORMAT_HAS_AUDIO.to_owned(), self.boolean_metadata(format.has_audio)),
            (FORMAT_HAS_VIDEO.to_owned(), self.boolean_metadata(format.has_video)),
        ];
        if let Some(value) = format.extension {
            fields.push((FORMAT_EXTENSION.to_owned(), self.string_metadata(value)));
        }
        if let Some(value) = format.quality {
            fields.push(("quality".to_owned(), self.string_metadata(value)));
        }
        for (key, value) in [
            ("width", format.width),
            ("height", format.height),
            ("bitrate", format.bitrate),
            ("content_length", format.content_length),
        ] {
            if let Some(value) = value {
                fields.push((key.to_owned(), self.integer_metadata(value)));
            }
        }
        if let Some(value) = format.codecs {
            fields.push(("codecs".to_owned(), self.string_metadata(value)));
        }
        self.format(format.id, self.metadata(fields))
    }
}

/// Denotes failure to interpret one Youtube watch URL.
#[derive(Debug, Error)]
pub enum YoutubeError<RequestError, ResponseError, ChallengeError> {
    /// Denotes a malformed or unsupported Youtube URL.
    #[error("malformed Youtube URL: {0}")]
    Url(String),
    /// Denotes request execution failure.
    #[error("Youtube request failed: {0}")]
    Request(RequestError),
    /// Denotes response decoding failure.
    #[error("Youtube response decoding failed: {0}")]
    Response(ResponseError),
    /// Denotes absence of a parseable player response.
    #[error("the Youtube page contains no parseable initial player response")]
    MissingPlayerResponse,
    /// Denotes absence of the player API key in the watch-page configuration.
    #[error("the Youtube page contains no InnerTube API key")]
    MissingApiKey,
    /// Denotes an unavailable video according to the player response.
    #[error("Youtube reports: {0}")]
    Unavailable(String),
    /// Denotes absence of any described format.
    #[error("the player response describes no audio or video format")]
    NoDirectFormat,
    /// Denotes failure to resolve the challenges the player response poses.
    #[error("Youtube challenge resolution failed: {0}")]
    Challenge(ChallengeError),
    /// Denotes that every described format stayed guarded by an unresolved challenge.
    #[error("every described format is withheld behind an unresolved {0} challenge")]
    Withheld(WithheldReason),
}

/// Names the kind of unresolved challenge withholding an entire format catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WithheldReason {
    /// Withholds because throttling parameters governing the formats stayed unresolved.
    Throttle,
    /// Withholds because signatures guarding the formats stayed unresolved.
    Signature,
}

impl Display for WithheldReason {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(match self {
            Self::Signature => "signature",
            Self::Throttle => "throttling",
        })
    }
}

/// Names why one challenge withheld a format.
fn withheld_reason(challenge: &YoutubeChallenge) -> WithheldReason {
    match challenge {
        YoutubeChallenge::Signature(_) => WithheldReason::Signature,
        YoutubeChallenge::Throttle(_) => WithheldReason::Throttle,
    }
}

/// Names the observation counting the formats the player response described.
pub const YOUTUBE_DESCRIBED: &str = "youtube_described";

/// Names the observation counting the formats an unresolved signature withheld.
pub const YOUTUBE_WITHHELD: &str = "youtube_withheld";

/// Names the observation counting the formats an unresolved throttling parameter governs.
pub const YOUTUBE_THROTTLED: &str = "youtube_throttled";

/// Names the observation counting the described formats the interpreter could not read.
pub const YOUTUBE_UNREADABLE: &str = "youtube_unreadable";

/// Derives what a reader is told about extraction and about the requests that follow it.
#[ext(name = YoutubeNotesExt)]
pub impl<This> This
where
    This: MetadataViewAlg,
{
    /// States what extraction observed about granting, in the words a reader reads.
    ///
    /// A player response describes formats this interpreter may or may not be able to read: some
    /// state no location, some are held behind an unanswered signature, and some are served
    /// slowly on purpose. What is said here is what was counted, and nothing is said about a
    /// count the extractor did not state.
    fn granting_notes(&self, metadata: &This::Metadata) -> Vec<String> {
        let counted = |key| self.metadata_number(metadata, key);
        let mut notes = Vec::new();
        if let Some(described) = counted(YOUTUBE_DESCRIBED) {
            notes.push(format!("the player response described {described} formats"));
        }
        if let Some(unreadable) = counted(YOUTUBE_UNREADABLE).filter(|count| *count > 0) {
            notes.push(format!("{unreadable} described formats state no location this interpreter can read"));
        }
        if let Some(withheld) = counted(YOUTUBE_WITHHELD) {
            notes.push(if withheld == 0 {
                "every signature was answered by the player program".to_owned()
            } else {
                format!("{withheld} formats withheld behind an unanswered signature")
            });
        }
        if let Some(throttled) = counted(YOUTUBE_THROTTLED).filter(|count| *count > 0) {
            notes.push(format!("{throttled} formats served at a throttled rate"));
        }
        notes
    }
}

/// Names what a media URL carries that a reader must never be shown.
const SIGNED_URL_MARK: &str = " for url (";

/// Names the rejection every reader of withheld media eventually meets.
const FORBIDDEN: &str = "403 Forbidden";

/// States what a reader is told about one failed media request.
///
/// A media URL is signed, which makes it a credential rather than an address: it is never
/// repeated, however the failure is worded. What is left is stated first in words a reader knows,
/// and the diagnostic follows for whoever goes looking.
#[must_use]
pub fn media_failure(detail: &str) -> String {
    let stated = detail
        .find(SIGNED_URL_MARK)
        .map_or_else(|| detail.to_owned(), |index| format!("{}; signed media URL redacted", &detail[..index]));
    if stated.contains(FORBIDDEN) {
        return format!("media request rejected: HTTP 403 Forbidden\n{stated}");
    }
    stated
}

/// Observes the video identity denoted by a supported Youtube URL.
#[must_use]
pub fn youtube_id(input: &str) -> Option<String> {
    let without_scheme = input.strip_prefix("https://").or_else(|| input.strip_prefix("http://"))?;
    let (authority, suffix) =
        without_scheme.split_once('/').map_or((without_scheme, ""), |(authority, suffix)| (authority, suffix));
    let host = authority.split('@').next_back()?.split(':').next()?;
    match host {
        "youtu.be" | "www.youtu.be" => {
            suffix.split(['?', '#', '/']).next().filter(|id| !id.is_empty()).map(str::to_owned)
        }
        "youtube.com" | "www.youtube.com" | "m.youtube.com" => {
            if let Some(query) = suffix.strip_prefix("watch?") {
                query.split('&').find_map(|pair| {
                    let (key, value) = pair.split_once('=')?;
                    (key == "v" && !value.is_empty()).then(|| value.to_owned())
                })
            } else {
                suffix.split(['?', '#']).next()?.split('/').nth(1).filter(|id| !id.is_empty()).map(str::to_owned)
            }
        }
        _ => None,
    }
}
