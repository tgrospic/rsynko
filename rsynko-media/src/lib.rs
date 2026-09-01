#![doc = include_str!("../README.md")]

mod application;
mod artifact;
mod extraction;
mod format;
mod laws;
mod observation;
mod output;
mod processing;
mod selection;
mod sorts;

pub use application::{
    ApplicationError, ApplicationExt, FormatSourceAlg, MediaDownloadError, MediaDownloadExt, MediaOutputExt,
    OutputTarget,
};
pub use artifact::{ARTIFACT_LOCATION, ArtifactAlg, ArtifactKind};
pub use extraction::{
    CollectionKind, ExtractUrlError, ExtractionAlg, ExtractionApplyAlg, ExtractionCatalogAlg, ExtractionExt,
    ExtractionViewAlg, MediaViewAlg,
};
pub use format::{FORMAT_EXTENSION, FORMAT_HAS_AUDIO, FORMAT_HAS_VIDEO, FORMAT_SOURCE, FormatAlg, FormatViewAlg};
pub use laws::{
    ArtifactLaws, ExtractionLawFixture, ExtractionLaws, FormatLaws, MediaProgramLawFixture, MediaProgramLaws,
    ObservationLaws, OutputNameLaws, ProcessingApplicationLaws, ProcessingLawFixture, ProcessingLaws, SelectionLaws,
};
pub use observation::{MetadataAlg, MetadataExt, MetadataViewAlg};
pub use output::{OutputNameAlg, OutputNameExt, portable_file_name, portable_file_stem, portable_user_file_name};
pub use processing::{
    ProcessingApplyAlg, ProcessingExt, ProcessingProgramAlg, ProcessingProgramExt, ProcessingProgramViewAlg,
    ProcessingStage,
};
pub use selection::{
    FormatComparison, FormatPredicateAlg, FormatPredicateExt, FormatPredicateMatchAlg, FormatSelectionAlg,
    FormatSelectionApplyAlg, FormatSelectionProgramExt,
};
pub use sorts::{MediaSorts, ProcessingSorts};

// Ambassador's generated delegation macros, re-exported so interpreters can compose.
pub use artifact::ambassador_impl_ArtifactAlg;
pub use extraction::ambassador_impl_ExtractionAlg;
pub use extraction::ambassador_impl_ExtractionApplyAlg;
pub use extraction::ambassador_impl_ExtractionCatalogAlg;
pub use extraction::ambassador_impl_ExtractionViewAlg;
pub use extraction::ambassador_impl_MediaViewAlg;
pub use format::ambassador_impl_FormatAlg;
pub use format::ambassador_impl_FormatViewAlg;
pub use observation::{ambassador_impl_MetadataAlg, ambassador_impl_MetadataViewAlg};
pub use output::ambassador_impl_OutputNameAlg;
pub use processing::ambassador_impl_ProcessingProgramAlg;
pub use selection::ambassador_impl_FormatPredicateAlg;
pub use selection::ambassador_impl_FormatPredicateMatchAlg;
pub use selection::ambassador_impl_FormatSelectionAlg;
pub use selection::ambassador_impl_FormatSelectionApplyAlg;
