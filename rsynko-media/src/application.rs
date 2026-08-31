use crate::*;
use alux_ext::ext;
use itertools::Itertools;
use rsynko_download::*;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Specifies how one generic format denotes a retrievable source carrier.
///
/// A format states where its bytes rest as an observation, so an interpreter that cannot locate a
/// format from what it observed says so rather than inventing a location.
pub trait FormatSourceAlg<Source>: MediaSorts {
    /// Defines the source consumed by resource retrieval for one format.
    fn format_source(&self, format: &Self::Format) -> Option<Source>;
}

/// Selects how a final output path is denoted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputTarget {
    /// Uses an explicit final path.
    Path(PathBuf),
    /// Derives `MEDIA_ID.EXTENSION` in the current directory.
    MediaId,
    /// Derives `PORTABLE_TITLE.EXTENSION`, falling back to the media identity.
    Title,
}

/// Denotes failure of an ordinary direct-resource media program.
#[derive(Debug, Error)]
pub enum ApplicationError<ExtractionError, FetchError, PublishError> {
    /// Denotes extraction failure.
    #[error("{0}")]
    Extraction(ExtractUrlError<ExtractionError>),
    /// Denotes failure of the shared direct-media program.
    #[error("{0}")]
    Media(MediaDownloadError<FetchError, PublishError>),
}

/// Denotes failure after extraction has produced a media description.
#[derive(Debug, Error)]
pub enum MediaDownloadError<FetchError, PublishError> {
    /// Denotes a result not supported by the single-media program.
    #[error("the extracted result is not a single media item")]
    NotSingleMedia,
    /// Denotes absence of a matching direct format.
    #[error("no directly retrievable format matches the download options")]
    NoMatchingFormat,
    /// Denotes a selected format the interpreter cannot locate.
    #[error("the selected format states no location this interpreter can retrieve")]
    UnlocatableFormat,
    /// Denotes a selection producing more than one resource.
    #[error("the initial downloader requires exactly one selected format")]
    MultipleFormats,
    /// Denotes resource download failure.
    #[error("{0}")]
    Download(DownloadError<FetchError, PublishError>),
}

/// Derives selection, naming, and download for an already extracted result.
#[ext(name = MediaDownloadExt)]
pub impl<This, Source, FetchError, PublishError, Event, Progress> This
where
    This: ExtractionViewAlg
        + MediaViewAlg
        + FormatViewAlg
        + FormatSelectionApplyAlg
        + FormatSourceAlg<Source>
        + FetchStreamAlg<Source, Error = FetchError>
        + AtomicPublishAlg<Error = PublishError>
        + DownloadObservationAlg<Event = Event, Progress = Progress>
        + DownloadProgressAlg<Progress = Progress>
        + DownloadReportAlg<Event = Event>,
    FetchError: Display,
    PublishError: Display,
{
    /// Downloads exactly one selected format from an extracted single-media result.
    ///
    /// # Errors
    ///
    /// Returns media-shape, selection, retrieval, or publication failure.
    fn download_extraction(
        &self,
        extraction: This::Extraction,
        selection: &This::Selection,
        target: &OutputTarget,
    ) -> Result<PathBuf, MediaDownloadError<FetchError, PublishError>> {
        let media = self
            .as_media(extraction)
            .ok_or(MediaDownloadError::NotSingleMedia)?;
        let formats = self
            .select_formats(self.media_formats(&media), selection)
            .ok_or(MediaDownloadError::NoMatchingFormat)?;
        let format = formats
            .into_iter()
            .exactly_one()
            .map_err(|_| MediaDownloadError::MultipleFormats)?;
        let destination = self.media_output_path(&media, format, target);
        let source = self
            .format_source(format)
            .ok_or(MediaDownloadError::UnlocatableFormat)?;
        self.download_resource(&source, &destination)
            .map_err(MediaDownloadError::Download)?;
        Ok(destination)
    }
}

/// Derives ordinary extraction, format selection, destination naming, and download.
#[ext(name = ApplicationExt)]
pub impl<This, ExtractionError, FetchError, PublishError, Event, Progress> This
where
    This: ExtractionCatalogAlg
        + ExtractionApplyAlg<Error = ExtractionError>
        + ExtractionViewAlg
        + MediaViewAlg
        + FormatViewAlg
        + FormatSelectionApplyAlg
        + FormatSourceAlg<String>
        + FetchStreamAlg<String, Error = FetchError>
        + AtomicPublishAlg<Error = PublishError>
        + DownloadObservationAlg<Event = Event, Progress = Progress>
        + DownloadProgressAlg<Progress = Progress>
        + DownloadReportAlg<Event = Event>,
    FetchError: Display,
    PublishError: Display,
{
    /// Downloads one URL through exactly one selected direct format.
    ///
    /// # Errors
    ///
    /// Returns extraction, selection, retrieval, or publication failure.
    fn download_url(
        &self,
        url: &str,
        selection: &This::Selection,
        target: &OutputTarget,
    ) -> Result<PathBuf, ApplicationError<ExtractionError, FetchError, PublishError>> {
        let extraction = self
            .extract_url(url)
            .map_err(ApplicationError::Extraction)?;
        self.download_extraction(extraction, selection, target)
            .map_err(ApplicationError::Media)
    }
}

/// Derives a final path from media observations, the selected format, and the output target.
#[ext(name = MediaOutputExt)]
pub impl<This> This
where
    This: MediaViewAlg + FormatViewAlg,
{
    /// Defines the final path one media item publishes its selected format at.
    fn media_output_path(
        &self,
        media: &This::Media,
        format: &This::Format,
        target: &OutputTarget,
    ) -> PathBuf {
        match target {
            OutputTarget::Path(path) => path.clone(),
            OutputTarget::MediaId => media_id_path(
                self.media_id(media),
                self.format_text(format, FORMAT_EXTENSION),
            ),
            OutputTarget::Title => crate::portable_file_name(
                self.media_title(media),
                self.media_id(media),
                self.format_text(format, FORMAT_EXTENSION),
            ),
        }
    }
}

fn media_id_path(id: &str, extension: Option<&str>) -> PathBuf {
    let safe_id: String = id
        .chars()
        .map(|character| match character {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect();
    let extension = extension.unwrap_or("bin");
    Path::new(&format!("{safe_id}.{extension}")).to_owned()
}
