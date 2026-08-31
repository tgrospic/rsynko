use crate::{RuntimeObservation, RuntimeObservationSender, RuntimePause, RuntimeXError};
use core::convert::Infallible;
use reqwest::blocking::{Client, RequestBuilder, Response};
use rsynko_download::*;
use rsynko_media::*;
use rsynko_memory::{
    Artifact, DownloadEvent, DownloadProgress, DownloadSyntax, Extraction, ExtractorKey, Format,
    FormatPredicate, FormatSelection, InfoRecord, InfoValue, Media, MediaSyntax, YoutubeRequest,
    interpret_selection, predicate_accepts,
};
use rsynko_x::status_id;
use rsynko_yt::{YoutubeError, YoutubeExtractionExt, YoutubeRequestBytesAlg, youtube_id};
use std::cell::RefCell;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Denotes the exact media bytes of `fixture://single-video`.
pub const FIXTURE_BYTES: &[u8] = b"rsynko fixture: single-video\n";

const FIXTURE_MEDIA_URL: &str = "fixture-media://single-video/best.mp4";

/// Names the handheld client, which states one muxed representation served without throttling.
pub const ANDROID_CLIENT: &str = "ANDROID";

/// Names the headset client, which states every adaptive representation.
pub const VISIONOS_CLIENT: &str = "VISIONOS";

/// States the client the headset catalog is requested as.
const VISIONOS_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_7_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15";

/// States the client the handheld catalog is requested as.
const ANDROID_USER_AGENT: &str = "com.google.android.youtube/21.26.364 (Linux; U; Android 11) gzip";

/// Denotes resource retrieval failure in the Reqwest interpreter.
#[derive(Debug, Error)]
pub enum RuntimeFetchError {
    /// Denotes an unknown fixture resource.
    #[error("unknown fixture resource {0}")]
    UnknownFixture(String),
    /// Denotes an HTTP request failure.
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    /// Denotes failure while incrementally reading a resource response.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Denotes extraction failure in the assembled runtime.
#[derive(Debug, Error)]
pub enum RuntimeExtractionError {
    /// Denotes Youtube specification failure.
    #[error("{0}")]
    Youtube(YoutubeError<RuntimeFetchError, serde_json::Error, Infallible>),
    /// Denotes failure to read what one tweet carries.
    #[error("{0}")]
    X(RuntimeXError),
    /// Denotes application of an unknown extractor key.
    #[error("unknown extractor {0:?}")]
    UnknownExtractor(ExtractorKey),
}

/// Interprets exact requests, fixture bytes, atomic files, and runtime observations.
#[derive(Debug)]
pub struct RuntimeEnvironment {
    client: Client,
    events: RefCell<Vec<DownloadEvent>>,
    progress: RefCell<Vec<DownloadProgress>>,
    extractors: [ExtractorKey; 3],
    observer: Option<RuntimeObservationSender>,
    pause: Option<RuntimePause>,
}

/// Names this retrieval the way a browser names itself.
///
/// What answers these addresses answers pages, and a request stating no agent at all is refused
/// rather than answered.
const BROWSER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/139 Safari/537.36";

impl RuntimeEnvironment {
    /// Constructs the production environment and its HTTP client.
    ///
    /// # Errors
    ///
    /// Returns failure to construct the Rustls-backed client.
    pub fn build() -> Result<Self, reqwest::Error> {
        // Rustls states the protocol and leaves the cryptography to a provider, which is a choice
        // this interpreter makes once, here, rather than one the library makes for it.
        let _installed = rustls::crypto::ring::default_provider().install_default();
        Ok(Self {
            client: Client::builder().build()?,
            events: RefCell::default(),
            progress: RefCell::default(),
            extractors: [
                ExtractorKey::new("fixture"),
                ExtractorKey::new("youtube"),
                ExtractorKey::new("x"),
            ],
            observer: None,
            pause: None,
        })
    }

    /// Constructs the production environment with a live observation channel.
    ///
    /// # Errors
    ///
    /// Returns failure to construct the Rustls-backed client.
    pub fn build_observed(observer: RuntimeObservationSender) -> Result<Self, reqwest::Error> {
        let mut environment = Self::build()?;
        environment.observer = Some(observer);
        Ok(environment)
    }

    /// Constructs the production environment with observation and cooperative pause control.
    ///
    /// # Errors
    ///
    /// Returns failure to construct the Rustls-backed client.
    pub fn build_pausable(
        observer: RuntimeObservationSender,
        pause: RuntimePause,
    ) -> Result<Self, reqwest::Error> {
        let mut environment = Self::build_observed(observer)?;
        environment.pause = Some(pause);
        Ok(environment)
    }

    /// Observes terminal download events in emission order.
    #[must_use]
    pub fn events(&self) -> Vec<DownloadEvent> {
        self.events.borrow().clone()
    }

    /// Observes byte-progress reports in emission order.
    #[must_use]
    pub fn progress(&self) -> Vec<DownloadProgress> {
        self.progress.borrow().clone()
    }

    fn request(&self, request: &YoutubeRequest) -> RequestBuilder {
        match request {
            // A watch page and the player program it names are both ordinary browser documents.
            YoutubeRequest::Watch(url) | YoutubeRequest::PlayerProgram(url) => {
                self.client.get(url).header("user-agent", BROWSER_AGENT)
            }
            // The headset client states the full adaptive catalog without attestation and without
            // a player token, which is what an unauthenticated retrieval can ask for today. It
            // still states the program it runs, so a response that does guard its representations
            // is answered rather than refused.
            YoutubeRequest::Player { id, api_key, claim } => {
                let (context, agent, name, version) = if claim.client == ANDROID_CLIENT {
                    (
                        serde_json::json!({
                            "clientName": ANDROID_CLIENT,
                            "clientVersion": "21.26.364",
                            "androidSdkVersion": 30,
                            "userAgent": ANDROID_USER_AGENT,
                            "osName": "Android",
                            "osVersion": "11",
                            "visitorData": claim.visitor,
                            "hl": "en",
                            "gl": "US",
                            "timeZone": "UTC",
                            "utcOffsetMinutes": 0
                        }),
                        ANDROID_USER_AGENT,
                        "3",
                        "21.26.364",
                    )
                } else {
                    (
                        serde_json::json!({
                            "clientName": VISIONOS_CLIENT,
                            "clientVersion": "1.02",
                            "deviceMake": "Apple",
                            "deviceModel": "RealityDevice17,1",
                            "userAgent": VISIONOS_USER_AGENT,
                            "osName": "visionOS",
                            "osVersion": "26.5.23O471",
                            "visitorData": claim.visitor,
                            "hl": "en",
                            "gl": "US",
                            "timeZone": "UTC",
                            "utcOffsetMinutes": 0
                        }),
                        VISIONOS_USER_AGENT,
                        "101",
                        "1.02",
                    )
                };
                let mut request = self
                    .client
                    .post(format!(
                        "https://www.youtube.com/youtubei/v1/player?key={api_key}&prettyPrint=false"
                    ))
                    .header("content-type", "application/json")
                    .header("origin", "https://www.youtube.com")
                    .header("x-youtube-client-name", name)
                    .header("x-youtube-client-version", version)
                    .header("user-agent", agent);
                // The session the watch page issued is what an unauthenticated catalog request is
                // recognized by; without it the request reads as coming from nobody at all.
                if let Some(visitor) = &claim.visitor {
                    request = request.header("x-goog-visitor-id", visitor);
                }
                request.json(&serde_json::json!({
                    "context": { "client": context },
                    "playbackContext": { "contentPlaybackContext": {
                        "html5Preference": "HTML5_PREF_WANTS",
                        "signatureTimestamp": claim.timestamp
                    }},
                    "videoId": id,
                    "contentCheckOk": true,
                    "racyCheckOk": true
                }))
            }
            YoutubeRequest::Media(url) => self
                .client
                .get(url)
                .header("referer", "https://www.youtube.com/")
                .header(
                    "user-agent",
                    "com.google.android.youtube/21.26.364 (Linux; U; Android 11) gzip",
                ),
        }
    }

    /// Asks one address and reads the whole of what it answers.
    ///
    /// The agent is stated because a request without one is refused rather than answered: an
    /// address that answers a page states what it would state to a browser, or nothing.
    pub(crate) fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, reqwest::Error> {
        Ok(self
            .client
            .get(url)
            .header("user-agent", BROWSER_AGENT)
            .send()?
            .error_for_status()?
            .bytes()?
            .to_vec())
    }

    fn execute(&self, request: &YoutubeRequest) -> Result<Response, RuntimeFetchError> {
        Ok(self.request(request).send()?.error_for_status()?)
    }

    fn read_stream(
        &self,
        stream: &mut RuntimeFetchStream,
        buffer: &mut [u8],
    ) -> Result<usize, RuntimeFetchError> {
        if let Some(pause) = &self.pause
            && !pause.wait_until_running()
        {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "download cancelled").into());
        }
        Ok(stream.0.read(buffer)?)
    }
}

/// Carries an incrementally readable fixture or HTTP resource.
pub struct RuntimeFetchStream(Box<dyn Read + Send>);

/// Carries a filesystem publication that is not yet visible at its final path.
pub struct RuntimePublication {
    file: File,
    partial: PathBuf,
    destination: PathBuf,
}

impl MediaSorts for RuntimeEnvironment {
    type Value = InfoValue;
    type Metadata = InfoRecord;
    type Format = Format;
    type Artifact = Artifact;
    type Media = Media;
    type Extraction = Extraction;
    type Extractor = ExtractorKey;
    type Predicate = FormatPredicate;
    type Selection = FormatSelection;
    type Output = PathBuf;
}

impl FormatViewAlg for RuntimeEnvironment {
    fn format_text<'a>(&self, format: &'a Format, key: &str) -> Option<&'a str> {
        format.text(key)
    }
}

impl YoutubeRequestBytesAlg for RuntimeEnvironment {
    type Error = RuntimeFetchError;

    fn youtube_request_bytes(&self, request: &YoutubeRequest) -> Result<Vec<u8>, Self::Error> {
        Ok(self.execute(request)?.bytes()?.to_vec())
    }
}

impl FetchStreamAlg for RuntimeEnvironment {
    type Error = RuntimeFetchError;
    type Stream = RuntimeFetchStream;

    fn open_fetch(&self, url: &str) -> Result<FetchStream<Self::Stream>, Self::Error> {
        if url.starts_with("fixture-media://") {
            if url != FIXTURE_MEDIA_URL {
                return Err(RuntimeFetchError::UnknownFixture(url.to_owned()));
            }
            return Ok(FetchStream::new(
                RuntimeFetchStream(Box::new(Cursor::new(FIXTURE_BYTES))),
                u64::try_from(FIXTURE_BYTES.len()).ok(),
            ));
        }
        let response = self.client.get(url).send()?.error_for_status()?;
        let total = response.content_length();
        Ok(FetchStream::new(
            RuntimeFetchStream(Box::new(response)),
            total,
        ))
    }

    fn read_fetch(
        &self,
        stream: &mut Self::Stream,
        buffer: &mut [u8],
    ) -> Result<usize, Self::Error> {
        self.read_stream(stream, buffer)
    }
}

impl FetchStreamAlg<String> for RuntimeEnvironment {
    type Error = RuntimeFetchError;
    type Stream = RuntimeFetchStream;

    fn open_fetch(&self, url: &String) -> Result<FetchStream<Self::Stream>, Self::Error> {
        <Self as FetchStreamAlg<str>>::open_fetch(self, url.as_str())
    }

    fn read_fetch(
        &self,
        stream: &mut Self::Stream,
        buffer: &mut [u8],
    ) -> Result<usize, Self::Error> {
        self.read_stream(stream, buffer)
    }
}

impl FormatSourceAlg<String> for RuntimeEnvironment {
    fn format_source(&self, format: &Format) -> Option<String> {
        format.source().map(str::to_owned)
    }
}

impl FetchStreamAlg<YoutubeRequest> for RuntimeEnvironment {
    type Error = RuntimeFetchError;
    type Stream = RuntimeFetchStream;

    fn open_fetch(
        &self,
        request: &YoutubeRequest,
    ) -> Result<FetchStream<Self::Stream>, Self::Error> {
        let response = self.execute(request)?;
        let total = response.content_length();
        Ok(FetchStream::new(
            RuntimeFetchStream(Box::new(response)),
            total,
        ))
    }

    fn read_fetch(
        &self,
        stream: &mut Self::Stream,
        buffer: &mut [u8],
    ) -> Result<usize, Self::Error> {
        self.read_stream(stream, buffer)
    }
}

impl AtomicPublishAlg for RuntimeEnvironment {
    type Error = io::Error;
    type Publication = RuntimePublication;

    fn begin_publication(&self, destination: &Path) -> Result<Self::Publication, Self::Error> {
        if let Some(parent) = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        let partial = partial_path(destination);
        if partial.exists() {
            fs::remove_file(&partial)?;
        }
        let file = File::create(&partial)?;
        Ok(RuntimePublication {
            file,
            partial,
            destination: destination.to_owned(),
        })
    }

    fn write_publication(
        &self,
        publication: &mut Self::Publication,
        bytes: &[u8],
    ) -> Result<(), Self::Error> {
        publication.file.write_all(bytes)
    }

    fn commit_publication(&self, publication: Self::Publication) -> Result<(), Self::Error> {
        publication.file.sync_all()?;
        let RuntimePublication {
            file,
            partial,
            destination,
        } = publication;
        drop(file);
        let result = fs::rename(&partial, destination);
        if result.is_err() {
            let _ignored = fs::remove_file(partial);
        }
        result
    }

    fn abort_publication(&self, publication: Self::Publication) {
        let RuntimePublication { file, partial, .. } = publication;
        drop(file);
        let _ignored = fs::remove_file(partial);
    }
}

impl DownloadReportAlg for RuntimeEnvironment {
    type Event = DownloadEvent;

    fn report_download(&self, event: DownloadEvent) {
        self.events.borrow_mut().push(event.clone());
        if let Some(observer) = &self.observer {
            observer
                .send(RuntimeObservation::terminal(event))
                .expect("a running download is being read");
        }
    }
}

impl DownloadProgressAlg for RuntimeEnvironment {
    type Progress = DownloadProgress;

    fn report_progress(&self, progress: DownloadProgress) {
        self.progress.borrow_mut().push(progress.clone());
        if let Some(observer) = &self.observer {
            observer
                .send(RuntimeObservation::progress(progress))
                .expect("a running download is being read");
        }
    }
}

impl DownloadObservationAlg for RuntimeEnvironment {
    type Event = DownloadEvent;
    type Progress = DownloadProgress;

    fn download_progress(
        &self,
        destination: &Path,
        downloaded: u64,
        total: Option<u64>,
    ) -> Self::Progress {
        DownloadSyntax.download_progress(destination, downloaded, total)
    }

    fn download_succeeded(&self, destination: &Path, bytes: u64) -> Self::Event {
        DownloadSyntax.download_succeeded(destination, bytes)
    }

    fn download_failed(&self, destination: &Path, message: String) -> Self::Event {
        DownloadSyntax.download_failed(destination, message)
    }
}

impl ExtractionCatalogAlg for RuntimeEnvironment {
    fn extractor_keys(&self) -> impl Iterator<Item = &ExtractorKey> {
        self.extractors.iter()
    }

    fn extractor_accepts(&self, extractor: &ExtractorKey, url: &str) -> bool {
        match extractor.0.as_str() {
            "fixture" => url == "fixture://single-video",
            "youtube" => youtube_id(url).is_some(),
            "x" => status_id(url).is_some(),
            _ => false,
        }
    }
}

impl ExtractionApplyAlg for RuntimeEnvironment {
    type Error = RuntimeExtractionError;

    fn extract_with(&self, extractor: &ExtractorKey, url: &str) -> Result<Extraction, Self::Error> {
        match extractor.0.as_str() {
            "fixture" => Ok(fixture_extraction()),
            "youtube" => self
                .extract_youtube(url)
                .map_err(RuntimeExtractionError::Youtube),
            "x" => self.extract_x(url).map_err(RuntimeExtractionError::X),
            _ => Err(RuntimeExtractionError::UnknownExtractor(extractor.clone())),
        }
    }
}

impl FormatPredicateMatchAlg for RuntimeEnvironment {
    fn format_matches(&self, predicate: &FormatPredicate, format: &Format) -> bool {
        predicate_accepts(predicate, format)
    }
}

impl FormatSelectionApplyAlg for RuntimeEnvironment {
    fn select_formats<'a>(
        &self,
        formats: &'a [Format],
        selection: &FormatSelection,
    ) -> Option<Vec<&'a Format>> {
        interpret_selection(self, formats, selection)
    }
}

fn fixture_observations() -> InfoRecord {
    MediaSyntax.metadata([
        (
            FORMAT_SOURCE.to_owned(),
            MediaSyntax.string_metadata(FIXTURE_MEDIA_URL),
        ),
        (
            FORMAT_EXTENSION.to_owned(),
            MediaSyntax.string_metadata("mp4"),
        ),
        (
            FORMAT_HAS_AUDIO.to_owned(),
            MediaSyntax.boolean_metadata(true),
        ),
        (
            FORMAT_HAS_VIDEO.to_owned(),
            MediaSyntax.boolean_metadata(true),
        ),
    ])
}

fn fixture_extraction() -> Extraction {
    Extraction::Media(Media::new(
        "single-video".to_owned(),
        InfoRecord::default(),
        vec![Format::new(
            "fixture-best".to_owned(),
            fixture_observations(),
        )],
    ))
}

fn partial_path(destination: &Path) -> PathBuf {
    let mut name: OsString = destination.as_os_str().to_owned();
    name.push(".part");
    PathBuf::from(name)
}
