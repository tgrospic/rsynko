use crate::MediaSorts;
use alux_ext::ext;
use ambassador::delegatable_trait;
use thiserror::Error;

/// Provides the carriers and constructors for extraction results.
#[delegatable_trait]
pub trait ExtractionAlg: MediaSorts {
    /// Defines a fully described media extraction.
    fn media(
        &self,
        id: impl Into<String>,
        metadata: Self::Metadata,
        formats: impl IntoIterator<Item = Self::Format>,
    ) -> Self::Extraction;
    /// Defines a URL requiring another extraction step.
    fn url_reference(
        &self,
        url: impl Into<String>,
        extractor: Option<Self::Extractor>,
        transparent: bool,
    ) -> Self::Extraction;
    /// Defines an ordered extraction collection.
    fn extraction_collection(
        &self,
        id: Option<String>,
        kind: CollectionKind,
        metadata: Self::Metadata,
        entries: impl IntoIterator<Item = Self::Extraction>,
    ) -> Self::Extraction;
}

/// Specifies observation of what shape an extraction result denotes.
#[delegatable_trait]
pub trait ExtractionViewAlg: MediaSorts {
    /// Observes the single media item an extraction denotes, exactly when it denotes one.
    fn as_media(&self, extraction: Self::Extraction) -> Option<Self::Media>;
}

/// Specifies the observations one media item offers.
#[delegatable_trait]
pub trait MediaViewAlg: MediaSorts {
    /// Observes the media identity.
    fn media_id<'a>(&self, media: &'a Self::Media) -> &'a str;
    /// Observes the media title exactly when one was extracted.
    fn media_title<'a>(&self, media: &'a Self::Media) -> Option<&'a str>;
    /// Observes the formats in extractor order.
    fn media_formats<'a>(&self, media: &'a Self::Media) -> &'a [Self::Format];
}

/// Specifies an ordered catalog of extractor meanings.
#[delegatable_trait]
pub trait ExtractionCatalogAlg: MediaSorts {
    /// Observes extractor keys in selection order.
    fn extractor_keys(&self) -> impl Iterator<Item = &Self::Extractor>;

    /// Observes whether one extractor accepts an input URL.
    fn extractor_accepts(&self, extractor: &Self::Extractor, url: &str) -> bool;
}

/// Specifies application of one selected extractor.
#[delegatable_trait]
pub trait ExtractionApplyAlg: MediaSorts {
    /// Denotes an extractor-specific failure.
    type Error;

    /// Applies a selected extractor to an accepted URL.
    ///
    /// # Errors
    ///
    /// Returns the selected extractor's error when extraction fails.
    fn extract_with(
        &self,
        extractor: &Self::Extractor,
        url: &str,
    ) -> Result<Self::Extraction, Self::Error>;
}

/// Classifies an ordered extraction collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectionKind {
    /// Denotes independently selectable entries.
    Playlist,
    /// Denotes entries comprising one logical media item.
    MultiVideo,
}

/// Denotes failure to derive extraction from an input URL.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExtractUrlError<ExtractorError> {
    /// Denotes that no catalog entry accepts the URL.
    #[error("no extractor accepts {url}")]
    Unsupported {
        /// Names the unsupported input.
        url: String,
    },
    /// Denotes failure of the selected extractor.
    #[error("selected extractor failed: {0}")]
    Extractor(ExtractorError),
}

/// Derives ordered extractor choice and URL extraction from primitive capabilities.
#[ext(name = ExtractionExt)]
pub impl<This, Extractor, Extraction> This
where
    This: ExtractionCatalogAlg<Extractor = Extractor>
        + ExtractionApplyAlg<Extractor = Extractor, Extraction = Extraction>,
{
    /// Chooses the first extractor accepting the URL.
    fn choose_extractor(&self, url: &str) -> Option<&Extractor> {
        self.extractor_keys()
            .find(|extractor| self.extractor_accepts(extractor, url))
    }

    /// Extracts a URL through the first accepting extractor.
    ///
    /// # Errors
    ///
    /// Returns [`ExtractUrlError::Unsupported`] when no extractor accepts the URL, or wraps the
    /// selected extractor's failure in [`ExtractUrlError::Extractor`].
    fn extract_url(&self, url: &str) -> Result<Extraction, ExtractUrlError<This::Error>> {
        let extractor = self
            .choose_extractor(url)
            .ok_or_else(|| ExtractUrlError::Unsupported {
                url: url.to_owned(),
            })?;
        self.extract_with(extractor, url)
            .map_err(ExtractUrlError::Extractor)
    }
}
