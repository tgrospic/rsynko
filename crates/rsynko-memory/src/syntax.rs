//! Reference interpreters that tie the specification sorts to inspectable Rust syntax.

use crate::{
    Artifact, Collection, DownloadEvent, DownloadProgress, Extraction, ExtractorKey, Format, FormatPredicate,
    FormatSelection, InfoRecord, InfoValue, Media, ProcessingProgram, ProcessingStep, ProcessorId, UrlReference,
    interpret_selection, predicate_accepts,
};
use rsynko_download::DownloadObservationAlg;
use rsynko_media::*;
use std::path::{Path, PathBuf};

/// Reifies every media sort as inspectable Rust syntax.
///
/// One interpreter ties the whole knot: the sorts are shared, so the classes are only meaningful
/// together and a single reification serves all of them.
#[derive(Clone, Copy, Debug, Default)]
pub struct MediaSyntax;

impl MediaSorts for MediaSyntax {
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

impl FormatViewAlg for MediaSyntax {
    fn format_text<'a>(&self, format: &'a Format, key: &str) -> Option<&'a str> {
        format.text(key)
    }
}

impl MetadataViewAlg for MediaSyntax {
    fn metadata_text<'a>(&self, metadata: &'a InfoRecord, key: &str) -> Option<&'a str> {
        match metadata.get(key) {
            Some(InfoValue::String(value)) => Some(value),
            Some(
                InfoValue::Null
                | InfoValue::Bool(_)
                | InfoValue::Integer(_)
                | InfoValue::Float(_)
                | InfoValue::List(_)
                | InfoValue::Record(_),
            )
            | None => None,
        }
    }

    fn metadata_number(&self, metadata: &InfoRecord, key: &str) -> Option<i64> {
        match metadata.get(key) {
            Some(InfoValue::Integer(value)) => Some(*value),
            Some(
                InfoValue::Null
                | InfoValue::Bool(_)
                | InfoValue::Float(_)
                | InfoValue::String(_)
                | InfoValue::List(_)
                | InfoValue::Record(_),
            )
            | None => None,
        }
    }
}

impl MetadataAlg for MediaSyntax {
    fn empty_metadata(&self) -> Self::Metadata {
        InfoRecord::default()
    }

    fn null_metadata(&self) -> Self::Value {
        InfoValue::Null
    }
    fn boolean_metadata(&self, value: bool) -> Self::Value {
        InfoValue::Bool(value)
    }
    fn integer_metadata(&self, value: i64) -> Self::Value {
        InfoValue::Integer(value)
    }
    fn float_metadata(&self, value: f64) -> Self::Value {
        InfoValue::Float(value)
    }
    fn string_metadata(&self, value: impl Into<String>) -> Self::Value {
        InfoValue::String(value.into())
    }
    fn list_metadata(&self, values: impl IntoIterator<Item = Self::Value>) -> Self::Value {
        InfoValue::List(values.into_iter().collect())
    }
    fn record_metadata(&self, record: Self::Metadata) -> Self::Value {
        InfoValue::Record(record)
    }
    fn metadata(&self, fields: impl IntoIterator<Item = (String, Self::Value)>) -> Self::Metadata {
        let mut record = InfoRecord::default();
        for (key, value) in fields {
            record.insert(key, value);
        }
        record
    }
}

impl FormatAlg for MediaSyntax {
    fn format(&self, id: impl Into<String>, metadata: Self::Metadata) -> Self::Format {
        Format::new(id.into(), metadata)
    }
}

impl ArtifactAlg for MediaSyntax {
    fn artifact(&self, id: impl Into<String>, kind: ArtifactKind, metadata: Self::Metadata) -> Self::Artifact {
        Artifact::new(id.into(), kind, metadata)
    }
}

impl ExtractionAlg for MediaSyntax {
    fn media(
        &self,
        id: impl Into<String>,
        metadata: Self::Metadata,
        formats: impl IntoIterator<Item = Self::Format>,
    ) -> Self::Extraction {
        Extraction::Media(Media::new(id.into(), metadata, formats.into_iter().collect()))
    }

    fn url_reference(
        &self,
        url: impl Into<String>,
        extractor: Option<Self::Extractor>,
        transparent: bool,
    ) -> Self::Extraction {
        Extraction::Url(UrlReference::new(url.into(), extractor, transparent))
    }

    fn extraction_collection(
        &self,
        id: Option<String>,
        kind: CollectionKind,
        metadata: Self::Metadata,
        entries: impl IntoIterator<Item = Self::Extraction>,
    ) -> Self::Extraction {
        Extraction::Collection(Collection::new(id, kind, metadata, entries.into_iter().collect()))
    }
}

impl ExtractionViewAlg for MediaSyntax {
    fn as_media(&self, extraction: Extraction) -> Option<Media> {
        match extraction {
            Extraction::Media(media) => Some(media),
            Extraction::Url(_) | Extraction::Collection(_) => None,
        }
    }
}

impl MediaViewAlg for MediaSyntax {
    fn media_id<'a>(&self, media: &'a Media) -> &'a str {
        &media.id
    }

    fn media_title<'a>(&self, media: &'a Media) -> Option<&'a str> {
        media.metadata.get("title").and_then(InfoValue::as_text)
    }

    fn media_formats<'a>(&self, media: &'a Media) -> &'a [Format] {
        &media.formats
    }
}

impl FormatPredicateAlg for MediaSyntax {
    fn any_format(&self) -> Self::Predicate {
        FormatPredicate::Any
    }
    fn format_id(&self, id: impl Into<String>) -> Self::Predicate {
        FormatPredicate::Id(id.into())
    }
    fn observed_format(&self, key: impl Into<String>) -> Self::Predicate {
        FormatPredicate::Observed(key.into())
    }
    fn text_format(&self, key: impl Into<String>, value: impl Into<String>) -> Self::Predicate {
        FormatPredicate::Text { key: key.into(), value: value.into() }
    }
    fn flag_format(&self, key: impl Into<String>, value: bool) -> Self::Predicate {
        FormatPredicate::Flag { key: key.into(), value }
    }
    fn number_format(&self, key: impl Into<String>, comparison: FormatComparison, value: f64) -> Self::Predicate {
        FormatPredicate::Number { key: key.into(), comparison, value }
    }
    fn all_formats(&self, left: Self::Predicate, right: Self::Predicate) -> Self::Predicate {
        FormatPredicate::All(Box::new(left), Box::new(right))
    }
    fn not_format(&self, predicate: Self::Predicate) -> Self::Predicate {
        FormatPredicate::Not(Box::new(predicate))
    }
}

impl FormatSelectionAlg for MediaSyntax {
    fn best_format(&self, predicate: Self::Predicate) -> Self::Selection {
        FormatSelection::Best(predicate)
    }
    fn worst_format(&self, predicate: Self::Predicate) -> Self::Selection {
        FormatSelection::Worst(predicate)
    }
    fn merge_formats(&self, selections: impl IntoIterator<Item = Self::Selection>) -> Self::Selection {
        FormatSelection::Merge(selections.into_iter().collect())
    }
    fn fallback_formats(&self, selections: impl IntoIterator<Item = Self::Selection>) -> Self::Selection {
        FormatSelection::Fallback(selections.into_iter().collect())
    }
}

impl OutputNameAlg for MediaSyntax {
    fn output_name(&self, component: String) -> Self::Output {
        PathBuf::from(component)
    }
}

impl ProcessingProgramAlg for ProcessingSyntax {
    fn processor(&self, id: impl Into<String>) -> Self::Processor {
        ProcessorId(id.into())
    }

    fn processing_step(&self, stage: ProcessingStage, processor: Self::Processor) -> Self::Step {
        ProcessingStep { stage, processor }
    }

    fn empty_processing(&self) -> Self::Program {
        ProcessingProgram::default()
    }

    fn processing(&self, steps: impl IntoIterator<Item = Self::Step>) -> Self::Program {
        ProcessingProgram::from_steps(steps)
    }

    fn then_processing(&self, first: Self::Program, next: Self::Program) -> Self::Program {
        first.then(next)
    }
}

impl ProcessingProgramViewAlg for ProcessingSyntax {
    fn processing_steps<'a>(program: &'a Self::Program) -> impl Iterator<Item = &'a Self::Step>
    where
        Self::Step: 'a,
    {
        program.steps()
    }

    fn processing_stage(step: &Self::Step) -> ProcessingStage {
        step.stage
    }
}

/// Reifies processing programs as ordered inspectable syntax.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessingSyntax;

impl ProcessingSorts for ProcessingSyntax {
    type Processor = ProcessorId;
    type Step = ProcessingStep;
    type Program = ProcessingProgram;
}

/// Reifies download observations as inspectable values.
#[derive(Clone, Copy, Debug, Default)]
pub struct DownloadSyntax;

impl DownloadObservationAlg for DownloadSyntax {
    type Event = DownloadEvent;
    type Progress = DownloadProgress;

    fn download_progress(&self, destination: &Path, downloaded: u64, total: Option<u64>) -> Self::Progress {
        DownloadProgress::new(destination.to_owned(), downloaded, total)
    }

    fn download_succeeded(&self, destination: &Path, bytes: u64) -> Self::Event {
        DownloadEvent::Succeeded { destination: destination.to_owned(), bytes }
    }

    fn download_failed(&self, destination: &Path, message: String) -> Self::Event {
        DownloadEvent::Failed { destination: destination.to_owned(), message }
    }
}

impl FormatPredicateMatchAlg for MediaSyntax {
    fn format_matches(&self, predicate: &FormatPredicate, format: &Format) -> bool {
        predicate_accepts(predicate, format)
    }
}

impl FormatSelectionApplyAlg for MediaSyntax {
    fn select_formats<'a>(&self, formats: &'a [Format], selection: &FormatSelection) -> Option<Vec<&'a Format>> {
        interpret_selection(self, formats, selection)
    }
}
