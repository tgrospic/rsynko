use crate::*;
use alux_ext::ext;
use rsynko_download::*;
use rsynko_media::*;
use std::fmt::Display;
use std::path::PathBuf;
use thiserror::Error;

/// Derives Youtube extraction and specialized media retrieval as generic download.
#[ext(name = YoutubeApplicationExt)]
pub impl<
    This,
    Request,
    RequestError,
    ResponseError,
    ChallengeError,
    FetchError,
    PublishError,
    Event,
    Progress,
> This
where
    This: YoutubeSorts<Request = Request>
        + YoutubeRequestAlg
        + YoutubeClientAlg
        + YoutubeProgramAlg
        + YoutubeRequestBytesAlg<Error = RequestError>
        + YoutubeResponseAlg<Error = ResponseError>
        + YoutubeChallengeAlg<Error = ChallengeError>
        + YoutubeSolutionAlg
        + YoutubeUrlAlg
        + MetadataAlg
        + FormatAlg
        + ExtractionAlg
        + ExtractionViewAlg
        + MediaViewAlg
        + FormatViewAlg
        + FormatSelectionApplyAlg
        + FormatSourceAlg<Request>
        + FetchStreamAlg<Request, Error = FetchError>
        + AtomicPublishAlg<Error = PublishError>
        + DownloadObservationAlg<Event = Event, Progress = Progress>
        + DownloadProgressAlg<Progress = Progress>
        + DownloadReportAlg<Event = Event>,
    FetchError: Display,
    PublishError: Display,
{
    /// Downloads one Youtube URL through exactly one selected direct format.
    ///
    /// # Errors
    ///
    /// Returns Youtube extraction, selection, retrieval, or publication failure.
    fn download_youtube(
        &self,
        url: &str,
        selection: &This::Selection,
        target: &OutputTarget,
    ) -> Result<
        PathBuf,
        YoutubeApplicationError<
            RequestError,
            ResponseError,
            ChallengeError,
            FetchError,
            PublishError,
        >,
    > {
        let extraction = self
            .extract_youtube(url)
            .map_err(YoutubeApplicationError::Extraction)?;
        self.download_extraction(extraction, selection, target)
            .map_err(YoutubeApplicationError::Media)
    }
}

/// Denotes failure of one Youtube-specialized media download.
#[derive(Debug, Error)]
pub enum YoutubeApplicationError<
    RequestError,
    ResponseError,
    ChallengeError,
    FetchError,
    PublishError,
> {
    /// Denotes Youtube extraction failure.
    #[error("{0}")]
    Extraction(YoutubeError<RequestError, ResponseError, ChallengeError>),
    /// Denotes failure of the shared direct-media program.
    #[error("{0}")]
    Media(MediaDownloadError<FetchError, PublishError>),
}
