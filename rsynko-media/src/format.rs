use crate::MediaSorts;
use ambassador::delegatable_trait;

/// Specifies observation of what one format states about itself.
#[delegatable_trait]
pub trait FormatViewAlg: MediaSorts {
    /// Observes one named value of a format as text.
    fn format_text<'a>(&self, format: &'a Self::Format, key: &str) -> Option<&'a str>;
}

/// Provides the carriers and constructor for one alternative representation.
#[delegatable_trait]
pub trait FormatAlg: MediaSorts {
    /// Defines one representation from its identity and everything observed about it.
    fn format(&self, id: impl Into<String>, metadata: Self::Metadata) -> Self::Format;
}

/// Names the observation locating a format's bytes when an extractor states one.
pub const FORMAT_SOURCE: &str = "url";

/// Names the observation stating a format's container or filename extension.
pub const FORMAT_EXTENSION: &str = "ext";

/// Names the observation stating whether a format contains audio.
pub const FORMAT_HAS_AUDIO: &str = "has_audio";

/// Names the observation stating whether a format contains video.
pub const FORMAT_HAS_VIDEO: &str = "has_video";
