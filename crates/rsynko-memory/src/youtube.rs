//! The reified Youtube request and the interpreters that tie the Youtube sorts to it.

use crate::{DownloadEvent, DownloadProgress, ReferenceDownloadEnv, ReferenceFetchError};
use ambassador::Delegate;
use rsynko_download::*;
use rsynko_yt::*;
use std::cell::RefCell;
use std::convert::Infallible;
use std::io::Cursor;
use std::path::Path;
use std::str::from_utf8;

/// Denotes one request required by Youtube interpretation without choosing HTTP representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum YoutubeRequest {
    /// Retrieves a watch page.
    Watch(String),
    /// Retrieves the player catalog for one video identity and API key.
    Player {
        /// Identifies the video.
        id: String,
        /// Authorizes the player endpoint.
        api_key: String,
        /// States what the client claims about itself.
        claim: PlayerClaim,
    },
    /// Retrieves the player program posing a video's challenges.
    PlayerProgram(String),
    /// Retrieves one direct media representation.
    Media(String),
}

/// Reifies Youtube requests as inspectable domain syntax.
#[derive(Clone, Copy, Debug, Default)]
pub struct YoutubeRequestSyntax;

impl YoutubeSorts for YoutubeRequestSyntax {
    type Request = YoutubeRequest;
    type Solutions = ();
}

impl YoutubeRequestAlg for YoutubeRequestSyntax {
    fn watch_request(&self, url: impl Into<String>) -> Self::Request {
        YoutubeRequest::Watch(url.into())
    }

    fn player_request(&self, id: impl Into<String>, api_key: impl Into<String>, claim: &PlayerClaim) -> Self::Request {
        YoutubeRequest::Player { id: id.into(), api_key: api_key.into(), claim: claim.clone() }
    }

    fn player_program_request(&self, url: impl Into<String>) -> Self::Request {
        YoutubeRequest::PlayerProgram(url.into())
    }

    fn media_request(&self, url: impl Into<String>) -> Self::Request {
        YoutubeRequest::Media(url.into())
    }
}

/// Names the watch URL the reference environment answers.
pub const REFERENCE_WATCH_URL: &str = "https://www.youtube.com/watch?v=law123";

/// Names the video identity that watch URL denotes.
pub const REFERENCE_VIDEO_ID: &str = "law123";

/// Locates the representation the reference player response describes.
pub const REFERENCE_MEDIA_URL: &str = "memory://law-media";

/// Names the container the reference player response states.
pub const REFERENCE_MEDIA_EXTENSION: &str = "mp4";

/// Locates the player program the reference watch page states.
pub const REFERENCE_PLAYER_PROGRAM_URL: &str = "memory://law-player.js";

/// States the player program the reference environment resolves challenges under.
pub const REFERENCE_PLAYER_PROGRAM: &str = "reference player program\ntimestamp=19000\n";

/// Interprets the Youtube query surface and resolves only the challenges it was seeded with.
///
/// Transport is a line-oriented encoding rather than HTML and JSON: decoding still reads the bytes
/// it is handed, which is all the response laws constrain.
#[derive(Debug, Default)]
pub struct ReferenceYoutubeEnv {
    solutions: Vec<(YoutubeChallenge, String)>,
    applications: RefCell<Vec<Vec<YoutubeChallenge>>>,
    executed: RefCell<Vec<YoutubeRequest>>,
}

impl ReferenceYoutubeEnv {
    /// Seeds one challenge with the value the player program would resolve it to.
    pub fn solve(&mut self, challenge: YoutubeChallenge, solution: impl Into<String>) {
        self.solutions.push((challenge, solution.into()));
    }

    /// Observes the requests executed so far, in execution order.
    #[must_use]
    pub fn executed(&self) -> Vec<YoutubeRequest> {
        self.executed.borrow().clone()
    }

    /// Observes the bulk challenge applications received, in application order.
    #[must_use]
    pub fn applications(&self) -> Vec<Vec<YoutubeChallenge>> {
        self.applications.borrow().clone()
    }

    /// States the watch-page bytes the reference watch URL answers with.
    #[must_use]
    pub fn watch_bytes() -> Vec<u8> {
        format!(
            "player=yes\napi_key=law-key\ntitle=Law Video\nid={REFERENCE_VIDEO_ID}\njs={REFERENCE_PLAYER_PROGRAM_URL}\nvisitor=law-session\n"
        )
        .into_bytes()
    }

    /// States the player bytes the reference player request answers with.
    #[must_use]
    pub fn player_bytes() -> Vec<u8> {
        format!("status=OK\ntitle=Law Video\nformat=18;{REFERENCE_MEDIA_EXTENSION};{REFERENCE_MEDIA_URL}\n")
            .into_bytes()
    }
}

impl YoutubeSorts for ReferenceYoutubeEnv {
    type Request = YoutubeRequest;
    type Solutions = Vec<(YoutubeChallenge, String)>;
}

impl YoutubeRequestAlg for ReferenceYoutubeEnv {
    fn watch_request(&self, url: impl Into<String>) -> Self::Request {
        YoutubeRequestSyntax.watch_request(url)
    }

    fn player_request(&self, id: impl Into<String>, api_key: impl Into<String>, claim: &PlayerClaim) -> Self::Request {
        YoutubeRequestSyntax.player_request(id, api_key, claim)
    }

    fn player_program_request(&self, url: impl Into<String>) -> Self::Request {
        YoutubeRequestSyntax.player_program_request(url)
    }

    fn media_request(&self, url: impl Into<String>) -> Self::Request {
        YoutubeRequestSyntax.media_request(url)
    }
}

impl YoutubeClientAlg for ReferenceYoutubeEnv {
    fn player_clients(&self) -> impl Iterator<Item = &str> {
        ["REFERENCE"].into_iter()
    }
}

impl YoutubeProgramAlg for ReferenceYoutubeEnv {
    fn program_timestamp(&self, program: &str) -> Option<i64> {
        program.lines().find_map(|line| line.strip_prefix("timestamp="))?.parse().ok()
    }
}

impl YoutubeRequestBytesAlg for ReferenceYoutubeEnv {
    type Error = ReferenceFetchError;

    fn youtube_request_bytes(&self, request: &Self::Request) -> Result<Vec<u8>, Self::Error> {
        self.executed.borrow_mut().push(request.clone());
        match request {
            YoutubeRequest::Watch(url) if url == REFERENCE_WATCH_URL => Ok(Self::watch_bytes()),
            YoutubeRequest::PlayerProgram(url) if url == REFERENCE_PLAYER_PROGRAM_URL => {
                Ok(REFERENCE_PLAYER_PROGRAM.as_bytes().to_vec())
            }
            YoutubeRequest::Player { id, .. } if id == REFERENCE_VIDEO_ID => Ok(Self::player_bytes()),
            YoutubeRequest::Watch(url) | YoutubeRequest::PlayerProgram(url) | YoutubeRequest::Media(url) => {
                Err(ReferenceFetchError::UnknownResource(url.clone()))
            }
            YoutubeRequest::Player { id, .. } => Err(ReferenceFetchError::UnknownResource(id.clone())),
        }
    }
}

impl YoutubeResponseAlg for ReferenceYoutubeEnv {
    type Error = ReferenceFetchError;

    fn decode_youtube_watch(&self, bytes: &[u8]) -> Result<YoutubeWatchPage, Self::Error> {
        let fields = decode_fields(bytes)?;
        Ok(YoutubeWatchPage {
            has_player_response: field(&fields, "player") == Some("yes"),
            api_key: field(&fields, "api_key").map(str::to_owned),
            player_url: field(&fields, "js").map(str::to_owned),
            visitor_data: field(&fields, "visitor").map(str::to_owned),
            title: field(&fields, "title").map(str::to_owned),
        })
    }

    fn decode_youtube_player(&self, bytes: &[u8]) -> Result<YoutubePlayer, Self::Error> {
        let fields = decode_fields(bytes)?;
        let formats =
            fields.iter().filter(|(key, _)| key == "format").filter_map(|(_, value)| decode_format(value)).collect();
        Ok(YoutubePlayer {
            status: field(&fields, "status").map(str::to_owned),
            reason: field(&fields, "reason").map(str::to_owned),
            title: field(&fields, "title").map(str::to_owned),
            formats,
            unreadable: 0,
        })
    }
}

impl YoutubeUrlAlg for ReferenceYoutubeEnv {
    fn throttle_challenge(&self, url: &str) -> Option<String> {
        query_value(url, "n")
    }

    fn with_throttle(&self, url: &str, solution: &str) -> String {
        with_query(url, "n", solution)
    }

    fn with_signature(&self, url: &str, parameter: &str, signature: &str) -> String {
        with_query(url, parameter, signature)
    }
}

impl YoutubeChallengeAlg for ReferenceYoutubeEnv {
    type Error = Infallible;

    fn solve_challenges(
        &self,
        program: &str,
        challenges: impl IntoIterator<Item = YoutubeChallenge>,
    ) -> Result<Self::Solutions, Self::Error> {
        let posed: Vec<YoutubeChallenge> = challenges.into_iter().collect();
        self.applications.borrow_mut().push(posed.clone());
        // A challenge only has a solution under the program that poses it.
        if program.is_empty() {
            return Ok(Vec::new());
        }
        Ok(posed
            .into_iter()
            .filter_map(|challenge| {
                self.solutions
                    .iter()
                    .find(|(known, _)| *known == challenge)
                    .map(|(_, solution)| (challenge, solution.clone()))
            })
            .collect())
    }
}

impl YoutubeSolutionAlg for ReferenceYoutubeEnv {
    fn solution_of(&self, solutions: &Self::Solutions, challenge: &YoutubeChallenge) -> Option<String> {
        solutions.iter().find(|(posed, _)| posed == challenge).map(|(_, solution)| solution.clone())
    }
}

/// Retrieves the representation one Youtube media request states.
///
/// Publication, progress, and reporting are the shared download interpreter's; only locating the
/// bytes is Youtube-specific, so only that is stated here.
#[allow(clippy::duplicated_attributes, reason = "one delegation per capability, not per target")]
#[derive(Debug, Default, Delegate)]
#[delegate(AtomicPublishAlg, target = "resources")]
#[delegate(DownloadObservationAlg, target = "resources")]
#[delegate(DownloadReportAlg, target = "resources")]
#[delegate(DownloadProgressAlg, target = "resources")]
pub struct ReferenceYoutubeDownloadEnv {
    resources: ReferenceDownloadEnv,
    opened: RefCell<Vec<YoutubeRequest>>,
}

impl ReferenceYoutubeDownloadEnv {
    /// Associates exact bytes with one media URL.
    pub fn register_resource(&mut self, url: impl Into<String>, bytes: impl Into<Vec<u8>>) {
        self.resources.register_resource(url, bytes);
    }

    /// Observes bytes published at one final path.
    #[must_use]
    pub fn file(&self, path: &Path) -> Option<Vec<u8>> {
        self.resources.file(path)
    }

    /// Observes the requests retrieval opened, in retrieval order.
    #[must_use]
    pub fn opened(&self) -> Vec<YoutubeRequest> {
        self.opened.borrow().clone()
    }

    /// Observes terminal events in emission order.
    #[must_use]
    pub fn events(&self) -> Vec<DownloadEvent> {
        self.resources.events()
    }

    /// Observes byte-progress reports in emission order.
    #[must_use]
    pub fn progress(&self) -> Vec<DownloadProgress> {
        self.resources.progress()
    }
}

impl FetchStreamAlg<YoutubeRequest> for ReferenceYoutubeDownloadEnv {
    type Error = ReferenceFetchError;
    type Stream = Cursor<Vec<u8>>;

    fn open_fetch(&self, request: &YoutubeRequest) -> Result<FetchStream<Self::Stream>, Self::Error> {
        self.opened.borrow_mut().push(request.clone());
        match request {
            YoutubeRequest::Media(url) => self.resources.open_fetch(url),
            YoutubeRequest::Watch(url) | YoutubeRequest::PlayerProgram(url) => {
                Err(ReferenceFetchError::UnknownResource(url.clone()))
            }
            YoutubeRequest::Player { id, .. } => Err(ReferenceFetchError::UnknownResource(id.clone())),
        }
    }

    fn read_fetch(&self, stream: &mut Self::Stream, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        self.resources.read_fetch(stream, buffer)
    }
}

/// Reads the `key=value` lines one reference response states.
fn decode_fields(bytes: &[u8]) -> Result<Vec<(String, String)>, ReferenceFetchError> {
    let text = from_utf8(bytes).map_err(|_| ReferenceFetchError::UnknownResource("undecodable response".to_owned()))?;
    Ok(text
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect())
}

/// Observes the first value stated under one field name.
fn field<'a>(fields: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fields.iter().find(|(name, _)| name == key).map(|(_, value)| value.as_str())
}

/// Reads one `itag;extension;url` format description.
fn decode_format(value: &str) -> Option<YoutubeFormat> {
    let mut parts = value.split(';');
    let id = parts.next()?.to_owned();
    let extension = parts.next()?.to_owned();
    let url = parts.next()?.to_owned();
    Some(YoutubeFormat {
        id,
        source: YoutubeFormatSource::Direct(url),
        extension: Some(extension),
        has_audio: true,
        has_video: true,
        quality: None,
        width: None,
        height: Some(360),
        bitrate: Some(500_000),
        content_length: None,
        codecs: None,
    })
}

/// Observes one query value without depending on a serialization library.
fn query_value(url: &str, key: &str) -> Option<String> {
    url.split_once('?')?.1.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        (name == key).then(|| value.to_owned())
    })
}

/// Replaces or appends one query value.
fn with_query(url: &str, key: &str, value: &str) -> String {
    let (base, query) = url.split_once('?').unwrap_or((url, ""));
    let pairs: Vec<String> = query
        .split('&')
        .filter(|pair| !pair.is_empty() && pair.split_once('=').is_none_or(|(name, _)| name != key))
        .map(str::to_owned)
        .chain([format!("{key}={value}")])
        .collect();
    format!("{base}?{}", pairs.join("&"))
}
