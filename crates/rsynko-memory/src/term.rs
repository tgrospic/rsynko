//! The reified media terms: the carrier one reference interpreter chooses for every sort.

use alux_sdk::IterTraversableExt;
use derive_more::From;
use derive_new::new;
use rsynko_media::*;
use std::collections::BTreeMap;

/// Denotes a metadata value without closing the set of extractor-defined fields.
#[derive(Clone, Debug, Default, PartialEq, From)]
pub enum InfoValue {
    /// Denotes unavailable metadata.
    #[default]
    Null,
    /// Denotes a boolean observation.
    Bool(bool),
    /// Denotes an integral numeric observation.
    Integer(i64),
    /// Denotes a non-integral numeric observation.
    Float(f64),
    /// Denotes textual metadata.
    String(String),
    /// Denotes an ordered metadata sequence.
    List(Vec<InfoValue>),
    /// Denotes a metadata record.
    Record(InfoRecord),
}

impl InfoValue {
    /// Observes the value as text exactly when it denotes text.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    /// Observes the value as a stated truth exactly when it denotes one.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Observes the value as a number exactly when it denotes one.
    ///
    /// Integral observations are widened, which is exact for every magnitude a format states and
    /// approximate beyond it. Comparison is the only thing this crate does with the result.
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "format observations are compared, not accumulated")]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Integer(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }
}

/// Denotes an open metadata record.
///
/// Keys are canonicalized by lexical order. Field order is intentionally not part of metadata
/// equality.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InfoRecord(BTreeMap<String, InfoValue>);

impl InfoRecord {
    /// Inserts a field and returns the previously denoted value, if any.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<InfoValue>) -> Option<InfoValue> {
        self.0.insert(key.into(), value.into())
    }

    /// Observes one field without exposing the record representation.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&InfoValue> {
        self.0.get(key)
    }

    /// Observes whether the record contains no fields.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Observes the number of fields in the record.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<&str> for InfoValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

/// Denotes one alternative representation of a media item.
///
/// A format is an identity together with an open record of what an extractor observed about it. It
/// fixes no schema: where its bytes rest, what streams it carries, and every quantity selection
/// compares are observations, not fields. An extractor states what it saw, and an interpreter that
/// locates bytes by other means simply states other observations.
#[derive(Clone, Debug, PartialEq, new)]
pub struct Format {
    /// Identifies the format within one media description.
    pub id: String,
    /// Preserves everything the extractor observed about the format.
    pub metadata: InfoRecord,
}

impl Format {
    /// Observes one named value.
    #[must_use]
    pub fn observe(&self, key: &str) -> Option<&InfoValue> {
        self.metadata.get(key)
    }

    /// Observes one named value as text.
    #[must_use]
    pub fn text(&self, key: &str) -> Option<&str> {
        self.observe(key).and_then(InfoValue::as_text)
    }

    /// Observes one named value as a number.
    #[must_use]
    pub fn number(&self, key: &str) -> Option<f64> {
        self.observe(key).and_then(InfoValue::as_number)
    }

    /// Observes one named truth, where an unstated observation denotes falsity.
    #[must_use]
    pub fn flag(&self, key: &str) -> bool {
        self.observe(key).and_then(InfoValue::as_bool).unwrap_or(false)
    }

    /// Observes the container or filename extension when stated.
    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.text(FORMAT_EXTENSION)
    }

    /// Observes whether the format carries audio.
    #[must_use]
    pub fn has_audio(&self) -> bool {
        self.flag(FORMAT_HAS_AUDIO)
    }

    /// Observes whether the format carries video.
    #[must_use]
    pub fn has_video(&self) -> bool {
        self.flag(FORMAT_HAS_VIDEO)
    }

    /// Observes where the format's bytes rest when the extractor stated it.
    #[must_use]
    pub fn source(&self) -> Option<&str> {
        self.text(FORMAT_SOURCE)
    }
}

/// Denotes one artifact owned by the processing pipeline.
#[derive(Clone, Debug, PartialEq, new)]
pub struct Artifact {
    /// Identifies the artifact independently of where it currently rests.
    pub id: String,
    /// Classifies the artifact's role.
    pub kind: ArtifactKind,
    /// Preserves artifact-specific observations.
    pub metadata: InfoRecord,
}

impl Artifact {
    /// Observes where the artifact currently rests when that has been stated.
    #[must_use]
    pub fn location(&self) -> Option<&str> {
        self.metadata.get(ARTIFACT_LOCATION).and_then(InfoValue::as_text)
    }
}

/// Identifies an extractor independently of its implementation type.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtractorKey(pub String);

impl ExtractorKey {
    /// Constructs an extractor key.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// Denotes one fully described media item.
#[derive(Clone, Debug, PartialEq, new)]
pub struct Media {
    /// Identifies the media within its extractor.
    pub id: String,
    /// Preserves all extracted metadata, including extractor-specific fields.
    pub metadata: InfoRecord,
    /// Lists directly retrievable formats in extractor order.
    pub formats: Vec<Format>,
}

/// Denotes a URL whose meaning requires another extraction.
#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct UrlReference {
    /// Names the referenced URL.
    pub url: String,
    /// Selects a specific extractor when present.
    pub extractor: Option<ExtractorKey>,
    /// Preserves whether outer metadata overlays the referenced result.
    pub transparent: bool,
}

/// Denotes an ordered collection of extraction results.
#[derive(Clone, Debug, PartialEq, new)]
pub struct Collection {
    /// Identifies the collection when known.
    pub id: Option<String>,
    /// Classifies collection composition.
    pub kind: CollectionKind,
    /// Preserves collection metadata.
    pub metadata: InfoRecord,
    /// Preserves entry order.
    pub entries: Vec<Extraction>,
}

/// Denotes one step of extraction meaning.
#[derive(Clone, Debug, PartialEq)]
pub enum Extraction {
    /// Denotes a fully described media item.
    Media(Media),
    /// Denotes a URL requiring another extraction step.
    Url(UrlReference),
    /// Denotes an ordered collection of further results.
    Collection(Collection),
}

/// Denotes a predicate over what one format states about itself.
///
/// Every primitive except identity ranges over observations, so a predicate can speak about any
/// field an extractor recorded rather than a fixed set of columns.
#[derive(Clone, Debug, PartialEq)]
pub enum FormatPredicate {
    /// Accepts every format.
    Any,
    /// Accepts one format identifier.
    Id(String),
    /// Accepts formats stating the named observation.
    Observed(String),
    /// Accepts formats whose named observation is the stated text.
    Text {
        /// Names the observation.
        key: String,
        /// States the text the observation must equal.
        value: String,
    },
    /// Accepts formats whose named observation is the stated truth.
    Flag {
        /// Names the observation.
        key: String,
        /// States the truth the observation must equal.
        value: bool,
    },
    /// Accepts formats whose named observation compares as stated.
    Number {
        /// Names the observation.
        key: String,
        /// States how the observation is compared.
        comparison: FormatComparison,
        /// States the number compared against.
        value: f64,
    },
    /// Accepts formats satisfying both predicates.
    All(Box<FormatPredicate>, Box<FormatPredicate>),
    /// Accepts formats rejected by the child predicate.
    Not(Box<FormatPredicate>),
}

/// Denotes a first-order format-selection program.
#[derive(Clone, Debug, PartialEq)]
pub enum FormatSelection {
    /// Chooses the most preferred matching format.
    Best(FormatPredicate),
    /// Chooses the least preferred matching format.
    Worst(FormatPredicate),
    /// Combines every child selection in declaration order.
    Merge(Vec<FormatSelection>),
    /// Chooses the first successful child selection.
    Fallback(Vec<FormatSelection>),
}

/// Interprets one reified predicate against one format.
///
/// An unstated observation satisfies no predicate that speaks about it, so absence is rejection
/// rather than failure.
#[must_use]
pub fn predicate_accepts(predicate: &FormatPredicate, format: &Format) -> bool {
    match predicate {
        FormatPredicate::Any => true,
        FormatPredicate::Id(id) => format.id == *id,
        FormatPredicate::Observed(key) => format.observe(key).is_some(),
        FormatPredicate::Text { key, value } => format.text(key) == Some(value.as_str()),
        FormatPredicate::Flag { key, value } => format.flag(key) == *value,
        FormatPredicate::Number { key, comparison, value } => {
            format.number(key).is_some_and(|observed| match comparison {
                FormatComparison::AtMost => observed <= *value,
                FormatComparison::Exactly => (observed - *value).abs() < f64::EPSILON,
                FormatComparison::AtLeast => observed >= *value,
            })
        }
        FormatPredicate::All(left, right) => predicate_accepts(left, format) && predicate_accepts(right, format),
        FormatPredicate::Not(child) => !predicate_accepts(child, format),
    }
}

/// Interprets one reified selection program against a preference-ordered catalog.
///
/// This is the canonical meaning of the reified syntax; an interpreter that chose those sorts
/// states `FormatSelectionApplyAlg` by delegating here.
#[must_use]
pub fn interpret_selection<'a, This>(
    alg: &This,
    formats: &'a [Format],
    selection: &FormatSelection,
) -> Option<Vec<&'a Format>>
where
    This: FormatPredicateMatchAlg<Predicate = FormatPredicate, Format = Format>,
{
    match selection {
        FormatSelection::Best(predicate) => {
            formats.iter().rev().find(|format| alg.format_matches(predicate, format)).map(|format| vec![format])
        }
        FormatSelection::Worst(predicate) => {
            formats.iter().find(|format| alg.format_matches(predicate, format)).map(|format| vec![format])
        }
        FormatSelection::Merge(selections) => {
            selections.iter().traverse_iter(|child| interpret_selection(alg, formats, child).ok_or(())).ok()
        }
        FormatSelection::Fallback(selections) => {
            selections.iter().find_map(|child| interpret_selection(alg, formats, child))
        }
    }
}

/// Identifies a processor independently of its implementation type.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessorId(pub String);

impl ProcessorId {
    /// Constructs a processor identity.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// Denotes application of one processor at one stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessingStep {
    /// Names the stage selecting this step.
    pub stage: ProcessingStage,
    /// Names the processor to apply.
    pub processor: ProcessorId,
}

impl ProcessingStep {
    /// Constructs a processing step.
    #[must_use]
    pub fn new(stage: ProcessingStage, processor: impl Into<String>) -> Self {
        Self { stage, processor: ProcessorId::new(processor) }
    }
}

/// Denotes an ordered sequence of artifact transformations.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProcessingProgram(Vec<ProcessingStep>);

impl ProcessingProgram {
    /// Constructs a program from steps in declaration order.
    #[must_use]
    pub fn from_steps(steps: impl IntoIterator<Item = ProcessingStep>) -> Self {
        Self(steps.into_iter().collect())
    }

    /// Appends one step to the program.
    pub fn push(&mut self, step: ProcessingStep) {
        self.0.push(step);
    }

    /// Observes steps in declaration order.
    pub fn steps(&self) -> impl Iterator<Item = &ProcessingStep> {
        self.0.iter()
    }

    /// Denotes sequential composition of two programs.
    #[must_use]
    pub fn then(mut self, next: Self) -> Self {
        self.0.extend(next.0);
        self
    }
}
