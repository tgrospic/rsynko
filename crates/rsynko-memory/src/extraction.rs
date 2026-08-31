use crate::{Extraction, ExtractorKey};
use rsynko_media::{ExtractionApplyAlg, ExtractionCatalogAlg, MediaSorts};
use std::cell::RefCell;
use thiserror::Error;

/// Denotes the result a reference extractor assigns to an accepted URL.
#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceExtractionOutcome {
    /// Denotes successful extraction.
    Success(Extraction),
    /// Denotes an extractor failure with a stable message.
    Failure(String),
}

/// Interprets one extractor as inspectable prefix matching and a fixed outcome.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceExtractor {
    key: ExtractorKey,
    accepted_prefixes: Vec<String>,
    outcome: ReferenceExtractionOutcome,
}

impl ReferenceExtractor {
    /// Constructs a deterministic extractor interpretation.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        accepted_prefixes: impl IntoIterator<Item = impl Into<String>>,
        outcome: ReferenceExtractionOutcome,
    ) -> Self {
        Self {
            key: ExtractorKey::new(key),
            accepted_prefixes: accepted_prefixes.into_iter().map(Into::into).collect(),
            outcome,
        }
    }

    /// Constructs a successful deterministic extractor.
    #[must_use]
    pub fn succeeds(
        key: impl Into<String>,
        accepted_prefix: impl Into<String>,
        extraction: Extraction,
    ) -> Self {
        Self::new(
            key,
            [accepted_prefix],
            ReferenceExtractionOutcome::Success(extraction),
        )
    }

    /// Constructs a failing deterministic extractor.
    #[must_use]
    pub fn fails(
        key: impl Into<String>,
        accepted_prefix: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            key,
            [accepted_prefix],
            ReferenceExtractionOutcome::Failure(message.into()),
        )
    }
}

/// Denotes failures of the deterministic extraction interpreter.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReferenceExtractionError {
    /// Denotes application of a key absent from the catalog.
    #[error("unknown extractor {0:?}")]
    UnknownExtractor(ExtractorKey),
    /// Denotes a failure reported by a reference extractor.
    #[error("extractor {extractor:?} failed: {message}")]
    ExtractorFailed {
        /// Names the failing extractor.
        extractor: ExtractorKey,
        /// Preserves its failure message.
        message: String,
    },
}

/// Interprets an ordered extractor catalog in memory.
#[derive(Debug, Default)]
pub struct ReferenceExtractorRegistry {
    extractors: Vec<ReferenceExtractor>,
    applications: RefCell<Vec<ExtractorKey>>,
}

impl ReferenceExtractorRegistry {
    /// Appends an extractor at the next catalog selection position.
    pub fn push(&mut self, extractor: ReferenceExtractor) {
        self.extractors.push(extractor);
    }

    /// Observes extractor applications in application order.
    #[must_use]
    pub fn applications(&self) -> Vec<ExtractorKey> {
        self.applications.borrow().clone()
    }
}

impl MediaSorts for ReferenceExtractorRegistry {
    type Value = ();
    type Metadata = ();
    type Format = ();
    type Artifact = ();
    type Media = ();
    type Extraction = Extraction;
    type Extractor = ExtractorKey;
    type Predicate = ();
    type Selection = ();
    type Output = ();
}

impl ExtractionCatalogAlg for ReferenceExtractorRegistry {
    fn extractor_keys(&self) -> impl Iterator<Item = &ExtractorKey> {
        self.extractors.iter().map(|extractor| &extractor.key)
    }

    fn extractor_accepts(&self, extractor: &ExtractorKey, url: &str) -> bool {
        self.extractors
            .iter()
            .find(|candidate| candidate.key == *extractor)
            .is_some_and(|candidate| {
                candidate
                    .accepted_prefixes
                    .iter()
                    .any(|prefix| url.starts_with(prefix))
            })
    }
}

impl ExtractionApplyAlg for ReferenceExtractorRegistry {
    type Error = ReferenceExtractionError;

    fn extract_with(
        &self,
        extractor: &ExtractorKey,
        _url: &str,
    ) -> Result<Extraction, Self::Error> {
        let candidate = self
            .extractors
            .iter()
            .find(|candidate| candidate.key == *extractor)
            .ok_or_else(|| ReferenceExtractionError::UnknownExtractor(extractor.clone()))?;
        self.applications.borrow_mut().push(extractor.clone());
        match &candidate.outcome {
            ReferenceExtractionOutcome::Success(extraction) => Ok(extraction.clone()),
            ReferenceExtractionOutcome::Failure(message) => {
                Err(ReferenceExtractionError::ExtractorFailed {
                    extractor: extractor.clone(),
                    message: message.clone(),
                })
            }
        }
    }
}
