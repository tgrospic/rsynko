use crate::ManagerSorts;
use alux_ext::ext;
use ambassador::delegatable_trait;
use std::path::PathBuf;

/// Provides the carrier representing a downloads collection.
///
/// The specification does not require that carrier to be a mutable record, a vector, or even an
/// in-memory value. Each interpreter chooses its representation.
pub trait DownloadsAlg: ManagerSorts {
    /// Defines an empty downloads collection.
    fn empty_downloads(&self) -> Self::Downloads;
}

/// Provides the carrier and constructor for one source request.
#[delegatable_trait]
pub trait SourceRequestAlg: ManagerSorts {
    /// Defines one source request with explicit download options.
    fn source(&self, input: impl Into<String>, output: Self::Output, options: Self::Options) -> Self::Source;
}

/// Provides the carrier and constructors for initial media options.
#[delegatable_trait]
pub trait MediaOptionsAlg: ManagerSorts {
    /// Defines a progressive request containing both audio and video.
    fn progressive(&self) -> Self::Options;

    /// Defines an audio-only request.
    fn audio(&self) -> Self::Options;

    /// Defines a video-only request.
    fn video(&self) -> Self::Options;
}

/// Specifies which submitted lines a source retrieves rather than a transfer walks.
///
/// A line names a path unless some source recognizes it as its own. Recognizing is what a source
/// interpreter adds, and nothing here knows what any of them look like: this application transfers
/// paths, and every other reading of a line is something a source claimed.
#[delegatable_trait]
pub trait SourceRecognitionAlg {
    /// States whether some source retrieves what the line names.
    fn recognizes_source(&self, line: &str) -> bool;
}

/// Specifies what one submitted line names.
///
/// What a line names decides what transferring it means. A line naming two ends names a transfer
/// between them, a line no source recognizes names one path to transfer, and a line a source
/// recognizes names something that source retrieves. A path is not a media item with alternative
/// formats, and never acquires one by being submitted.
#[delegatable_trait]
pub trait SubmissionAlg: ManagerSorts {
    /// Defines the request one submitted line names.
    fn submitted(&self, line: &str) -> Self::Source;
}

/// Provides the carrier and constructors for output choices.
#[delegatable_trait]
pub trait OutputChoiceAlg: ManagerSorts {
    /// Defines output naming derived from extracted media meaning.
    fn suggested_output(&self) -> Self::Output;

    /// Defines an exact output path.
    fn exact_output(&self, path: impl Into<PathBuf>) -> Self::Output;
}

/// Describes downloads collections that can accumulate source requests.
pub trait DownloadCollectionAlg: ManagerSorts {
    /// Appends source requests in declaration order.
    #[must_use]
    fn add_sources(self, sources: impl IntoIterator<Item = Self::Source>) -> Self;
}

/// Derives downloads-collection construction by composing associated carriers.
#[ext(name = DownloadsExt)]
pub impl<This> This
where
    This: DownloadsAlg + SourceRequestAlg,
    This::Downloads: DownloadCollectionAlg<Source = This::Source>,
{
    /// Defines a downloads collection containing `sources` in declaration order.
    fn downloads(&self, sources: impl IntoIterator<Item = This::Source>) -> This::Downloads {
        self.empty_downloads().add_sources(sources)
    }
}

/// Derives progressive downloads with media-derived output names.
#[ext(name = ProgressiveDownloadsExt)]
pub impl<This> This
where
    This: DownloadsAlg + SourceRequestAlg + MediaOptionsAlg + OutputChoiceAlg,
    This::Downloads: DownloadCollectionAlg<Source = This::Source>,
{
    /// Defines a downloads collection from source names using progressive media and suggested
    /// output naming.
    fn progressive_downloads(&self, sources: impl IntoIterator<Item = impl Into<String>>) -> This::Downloads {
        self.downloads(
            sources.into_iter().map(|source| self.source(source, self.suggested_output(), self.progressive())),
        )
    }
}
